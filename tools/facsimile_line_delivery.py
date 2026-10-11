"""Authenticate audit scope, panel observations and immutable evidence archives."""
import argparse
import copy
import json
import shutil
import sys
from collections import Counter
from html.parser import HTMLParser
from pathlib import Path

import facsimile_line_audit as cli
import facsimile_line_coverage as coverage
import inner_audit_common as common


def array_records(path, member):
    """Read one top-level array incrementally without retaining sibling registries."""
    decoder = json.JSONDecoder()
    with Path(path).open(encoding='utf-8') as stream:
        buffer,position,eof = '',0,False
        def more():
            nonlocal buffer,position,eof
            buffer = buffer[position:]
            position = 0
            chunk = stream.read(1024*1024)
            buffer += chunk
            eof = not chunk
        def whitespace():
            nonlocal position
            while True:
                while position<len(buffer) and buffer[position].isspace(): position += 1
                if position<len(buffer) or eof: return
                more()
        def punctuation(expected):
            nonlocal position
            whitespace()
            if position>=len(buffer) or buffer[position]!=expected:
                raise common.AuditError(f'Invalid JSON structure in {path}')
            position += 1
        def value():
            nonlocal position
            whitespace()
            while True:
                try:
                    result,end = decoder.raw_decode(buffer,position)
                    if end==len(buffer) and not eof:
                        more()
                        continue
                    position = end
                    return result
                except json.JSONDecodeError as exc:
                    if eof: raise common.AuditError(f'Invalid JSON in {path}') from exc
                    more()
        punctuation('{')
        while True:
            whitespace()
            if position<len(buffer) and buffer[position]=='}':
                raise common.AuditError(f'Missing {member} array in {path}')
            key = value()
            punctuation(':')
            if key==member:
                punctuation('[')
                whitespace()
                if position<len(buffer) and buffer[position]==']': return
                while True:
                    yield value()
                    whitespace()
                    if position<len(buffer) and buffer[position]==']': return
                    punctuation(',')
            value()
            punctuation(',')


def authenticated_run(args):
    run = Path(args.run_dir).resolve()
    record = cli.read_json(run/'run-manifest.json')
    if not record.get('processing_complete') or record.get('completed_stage') not in ('coverage','seams'):
        raise common.AuditError('Delivery requires a completed coverage or seam run')
    root,manifest = cli.resolve_run_snapshot(record,args)
    for name,digest in record['record_sha256'].items():
        if common.hash_file(run/common.relative_path(name))!=digest:
            raise common.AuditError('Run record digest changed: '+name)
    if record['snapshot_id']!=manifest['snapshot_id'] or record['code']!=manifest['runtime']['code']:
        raise common.AuditError('Run and captured comparison-code identities disagree')
    return run,record,root,manifest


class EvidenceLinks(HTMLParser):
    def __init__(self):
        super().__init__()
        self.paths,self.fragments,self.anchors = [],[],set()

    def handle_starttag(self,tag,attrs):
        fields = dict(attrs)
        if 'id' in fields: self.anchors.add(fields['id'])
        for name in ('href','src'):
            if name in fields:
                value = fields[name]
                (self.fragments if value.startswith('#') else self.paths).append(value)


def evidence_checks(run,record,ledger):
    checked = {}
    def crop(item):
        path = (run/common.relative_path(item['path'])).resolve()
        if not common.contained(path,run): raise common.AuditError('Evidence path escapes its run')
        if path not in checked: checked[path] = common.hash_file(path)
        if checked[path]!=item['sha256']: raise common.AuditError('Evidence digest mismatch: '+item['path'])
    for page in array_records(run/'source-inventory.json','pages'):
        for item in [page['full_context'],page['overlay']]+[obj['crop'] for obj in page['occurrences']]: crop(item)
    for measurement in ledger['measurements'].values():
        for item in measurement['crops'].values(): crop(item)
    links = 0
    if record['completed_stage']=='seams':
        for task in array_records(run/'review-tasks.json','tasks'):
            for item in task['source_crops']+task['reader_crops']:
                crop(item['original'])
                crop(item)
                crop({'path':item['display_path'],'sha256':item['display_sha256']})
        parser = EvidenceLinks()
        with (run/'coverage-report.html').open(encoding='utf8') as stream:
            for chunk in iter(lambda:stream.read(1024*1024),''): parser.feed(chunk)
        parser.close()
        for relative in parser.paths:
            path = (run/common.relative_path(relative)).resolve()
            if not common.contained(path,run) or not path.is_file(): raise common.AuditError('Missing report evidence link: '+relative)
            links += 1
        if any(fragment[1:] not in parser.anchors for fragment in parser.fragments):
            raise common.AuditError('Missing report task anchor')
    return {'authenticated_crop_files':len(checked),'report_links':links}


def audit_scope(args):
    import pymupdf
    run,record,root,manifest = authenticated_run(args)
    pages,source_counts,ink = set(),Counter(),Counter()
    for page in array_records(run/'source-inventory.json','pages'):
        key = (page['source_index'],page['page'])
        if key in pages: raise common.AuditError('Repeated physical source page')
        pages.add(key)
        source_counts['physical_objects'] += len(page['occurrences'])
        source_counts['source_pages_with_unreviewed_geometry'] += any(not o['required_geometry_reviewed'] for o in page['occurrences'])
        for item in page['accounting']:
            for key,value in item.items():
                if isinstance(value,int) and not isinstance(value,bool): ink[key] += value
    source_scans = []
    for source in (r for r in manifest['inputs'] if r['role']=='source_pdf'):
        pdf,_ = cli.snapshot_input(root,manifest,'source_pdf',source_index=source['source_index'])
        with pymupdf.open(pdf) as document: count = len(document)
        processed = sorted(page for index,page in pages if index==source['source_index'])
        excluded = sorted(set(range(1,count+1))-set(processed))
        source_scans.append({'source_index':source['source_index'],'physical_pages':count,'processed_pages':processed,
                             'excluded_physical_pages':excluded,'exclusions_are_not_coverage_verdicts':True})
    reader_map = cli.read_json(run/'reader-map.json')
    ledger = cli.read_json(run/'coverage-ledger.json')
    units = sorted({row['unit_id'] for row in reader_map['body_pages']})
    requested = record['scope']['units'] if isinstance(record['scope'],dict) else record['scope']
    selected = set(requested)
    table_path,_ = cli.snapshot_input(root,manifest,'unit_table')
    table = cli.read_json(table_path)
    expected_pages = set()
    for index,unit in enumerate(table):
        if unit['id'] not in selected: continue
        following = next((other for other in table[index+1:] if other['source']==unit['source']),None)
        ending = following['start_page'] if following else manifest['reviewed_source_endpoints'][str(unit['source'])]
        expected_pages.update((unit['source'],page) for page in range(unit['start_page'],ending+1))
    policy_path,_ = cli.snapshot_input(root,manifest,'source_policy')
    policy = cli.read_json(policy_path)
    expected_pages.update((d['source_index'],d['page']) for d in policy['decisions']
                          if d.get('owner') in selected or any(child.get('owner') in selected for child in d.get('children',[])))
    for group in policy['duplicate_groups']:
        if group['owner'] in selected:
            expected_pages.update((mapping['source_index'],mapping['page']) for mapping in group['region_maps'])
    explicit_pages = record['scope'].get('source_pages') if isinstance(record['scope'],dict) else None
    if explicit_pages:
        explicit_pairs = {tuple(page) for page in explicit_pages}
        if not explicit_pairs<=expected_pages: raise common.AuditError('Explicit source scope escapes captured unit ranges')
        expected_pages = explicit_pairs
    if pages!=expected_pages or record['requested_pages']!=len(expected_pages):
        raise common.AuditError('Processed source pages disagree with independently counted requested ranges')
    expected_readers = [row for row in manifest['reader_map']['body_pages'] if row['unit_id'] in selected]
    if reader_map['body_pages']!=expected_readers:
        raise common.AuditError('Run reader occurrences disagree with captured requested units')
    counts = Counter(line['status'] for line in ledger['lines'])
    if dict(counts)!=ledger['counts'] or len(pages)!=record['completed_pages']:
        raise common.AuditError('Actual record counts disagree with declared processing counts')
    per_tomus = []
    observed = {(p['tomus'],p['page']) for p in array_records(run/'reader-observations.json','pages')}
    if len(observed)!=record['processed_reader_pages'] or len(expected_readers)!=record['requested_reader_occurrences']:
        raise common.AuditError('Actual reader observations disagree with run counts')
    for tomus in range(1,9):
        rows = [row for row in reader_map['body_pages'] if row['tomus']==tomus]
        physical = {row['page'] for row in rows}
        per_tomus.append({'tomus':tomus,'requested_body_occurrences':len(rows),'requested_body_pages':len(physical),
            'processed_body_pages':sum((tomus,page) in observed for page in physical),
            'failed_body_pages':sum((tomus,page) not in observed for page in physical)})
    captured_units = {unit['unit_id'] for unit in manifest['reader_map']['units']}
    explicit = isinstance(record['scope'],dict) and bool(record['scope'].get('source_pages'))
    result = {'schema_version':1,'snapshot_id':manifest['snapshot_id'],'runtime_id':record['runtime_id'],
        'delivery_code_sha256':common.hash_file(__file__),'comparison_code':record['code'],
        'processing_complete':True,'completed_stage':record['completed_stage'],
        'capture_counts':manifest['observed_counts'],'requested_units':requested,'processed_units':units,
        'unprocessed_captured_units':sorted(captured_units-set(units)),
        'requested_source_pages':record['requested_pages'],'processed_source_pages':len(pages),
        'failed_source_pages':record['requested_pages']-len(pages),'source_scans':source_scans,
        'source_counts':dict(source_counts),'ink_accounting':dict(ink),'reader_tomi':per_tomus,
        'reader_nonbody_pages':manifest['observed_counts']['physical_pages']-manifest['observed_counts']['body_pages'],
        'coverage':dict(counts),'source_denominators':ledger['source_denominators'],'ocr_denominators':ledger['ocr_denominators'],
        'reviewed_required_regions':sum(line['required_geometry_reviewed'] for line in ledger['lines']),
        'unreviewed_required_regions':sum(not line['required_geometry_reviewed'] for line in ledger['lines']),
        'processing_errors':cli.read_json(run/'errors.json')['errors'],
        'full_captured_unit_ranges':not explicit and set(units)==captured_units,
        'calibration_freeze_approved':False,'complete_latin_coverage_certified':False}
    result['evidence_checks'] = evidence_checks(run,record,ledger)
    if record['completed_stage']=='seams':
        seams = cli.read_json(run/'seam-inventory.json')
        tasks = cli.read_json(run/'review-tasks.json')
        result.update(seams=seams['counts'],requested_seams=seams['requested_seams'],processed_seams=seams['processed_seams'],
                      unresolved_seams=seams['unresolved_seams'],neutral_tasks=tasks['task_count'],inconclusive_tasks=tasks['inconclusive_count'])
    if args.panel:
        if not Path(args.panel).is_file() or Path(args.panel).name!='panel.json':
            raise common.AuditError('Scope reporting requires a previously fixed panel.json')
        panel_args = copy.copy(args)
        panel_args.snapshot = str(root)
        panel_args.run_dir = str(Path(args.panel).resolve().parent)
        panel = coverage.write_calibration_panel(panel_args)
        if cli.read_json(args.panel)!=panel: raise common.AuditError('Fixed panel identity mismatch')
        associated = {}
        results = {line['logical_line_id']:line for line in ledger['lines']}
        for item in array_records(run/'correspondences.json','lines'):
            for tomus,page in item['reader_pages']:
                associated.setdefault((tomus,page),set()).add(item['logical_line_id'])
        observations = []
        for page in panel['pages']:
            key = (page['tomus'],page['reader_page'])
            identifiers = associated.get(key,set())
            page_counts = Counter(results[i]['status'] for i in identifiers)
            observations.append(dict(page,reader_processed=key in observed,associated_source_regions=len(identifiers),
                associated_coverage=dict(page_counts),reviewed_required_regions=sum(results[i]['required_geometry_reviewed'] for i in identifiers),
                interpretation='Bounded observations; unlocalized source regions are not assigned to a particular reader page'))
        result['panel'] = {'reader_input_identity':panel['reader_input_identity'],'denominator':200,'pages':observations,
            'processed_reader_pages':sum(page['reader_processed'] for page in observations),
            'source_scope_complete':result['full_captured_unit_ranges'],'misses':None,'false_alarms':None,
            'ground_truth_review_pending':True,'unlocalized_source_regions':sum(line['status']=='unresolved' for line in ledger['lines'])}
    return result


def verify_archive(root):
    root = Path(root).resolve()
    manifest = cli.read_json(root/'manifest.json')
    expected = {item['path'] for item in manifest['files']}
    actual = {path.relative_to(root).as_posix() for path in root.rglob('*') if path.is_file() and path!=root/'manifest.json'}
    if actual!=expected: raise common.AuditError('Archive file inventory differs from manifest')
    for item in manifest['files']:
        path = (root/common.relative_path(item['path'])).resolve()
        if not common.contained(path,root) or common.hash_file(path)!=item['sha256'] or path.stat().st_size!=item['bytes']:
            raise common.AuditError('Archive file identity mismatch: '+item['path'])
    common.progress(f'Authenticated {len(expected)} archived files; manifest {common.hash_file(root/"manifest.json")}')
    return manifest


def archive(args):
    run,record,snapshot,manifest = authenticated_run(args)
    repo,work = cli.paths(args)
    destination = Path(args.archive_dir).resolve()
    common.safe_write_path(destination/'manifest.json',work,repo,capture_root=destination)
    if destination.exists(): raise common.AuditError('Archive destination must be fresh')
    roots = [(snapshot,'snapshot'),(run,'run')]
    for name,label in (('case_root','verification'),('calibration_dir','calibration')):
        supplied = getattr(args,name,None)
        if supplied:
            source = Path(supplied).resolve()
            if not source.is_dir() or not common.contained(source,work/'runs') or common.contained(destination,source):
                raise common.AuditError('Supplementary archive evidence requires an existing audit-run directory')
            roots.append((source,label))
    destination.mkdir(parents=True)
    files = []
    for source,label in roots:
        for path in sorted(source.rglob('*')):
            if not path.is_file(): continue
            resolved = path.resolve()
            if not common.contained(resolved,source) or path.name.lower()=='secrets.json' or path.name.endswith('.lock'):
                raise common.AuditError('Unsafe or active archive source: '+str(path))
            relative = Path(label)/path.relative_to(source)
            target = common.safe_write_path(destination/relative,work,repo,capture_root=destination)
            target.parent.mkdir(parents=True,exist_ok=True)
            digest = common.hash_file(path)
            with path.open('rb') as original,target.open('xb') as copied:
                shutil.copyfileobj(original,copied,1024*1024)
            if common.hash_file(target)!=digest or common.hash_file(path)!=digest:
                raise common.AuditError('Archive source or copy changed: '+str(relative))
            files.append({'path':relative.as_posix(),'bytes':target.stat().st_size,'sha256':digest})
    if common.hash_file(__file__)!=args.delivery_code_sha256:
        raise common.AuditError('Executed delivery code changed during archive creation')
    archive_manifest = {'schema_version':1,'snapshot_id':manifest['snapshot_id'],'runtime_id':record['runtime_id'],
        'delivery_code_sha256':args.delivery_code_sha256,'comparison_code':record['code'],'files':files,
        'source_roots':[{ 'path':str(path),'label':label} for path,label in roots],
        'calibration_freeze_approved':False,'complete_latin_coverage_certified':False}
    cli.write(destination/'manifest.json',archive_manifest,args,capture_root=destination)
    verify_archive(destination)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=('scope','archive','verify-archive','controls'))
    parser.add_argument('--repo-root',default=str(Path(__file__).resolve().parent.parent))
    parser.add_argument('--work-dir',default=cli.DEFAULT_WORK)
    for name in ('run-dir','snapshot','out','panel','archive-dir','case-root','calibration-dir','controls','pdftoppm','tesseract','tessdata-dir'):
        parser.add_argument('--'+name)
    args = parser.parse_args()
    args.delivery_code_sha256 = common.hash_file(__file__)
    try:
        if args.command=='controls':
            if not args.run_dir or not args.out: raise common.AuditError('Controls require --run-dir and --out')
            import facsimile_glyph_splits
            return facsimile_glyph_splits.score_controls(args)
        elif args.command=='verify-archive':
            if not args.archive_dir: raise common.AuditError('Archive verification requires --archive-dir')
            verify_archive(args.archive_dir)
        elif args.command=='archive':
            if not args.run_dir or not args.archive_dir: raise common.AuditError('Archive requires --run-dir and --archive-dir')
            archive(args)
        else:
            if not args.run_dir or not args.out: raise common.AuditError('Scope reporting requires --run-dir and --out')
            if Path(args.out).exists(): raise common.AuditError('Scope report output must be fresh')
            result = audit_scope(args)
            if common.hash_file(__file__)!=args.delivery_code_sha256:
                raise common.AuditError('Executed delivery code changed during scope reporting')
            cli.write(args.out,result,args)
        return 0
    except (common.AuditError,OSError,ValueError,KeyError,TypeError) as exc:
        print(str(exc),file=sys.stderr)
        return 2


if __name__=='__main__':
    sys.exit(main())
