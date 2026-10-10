"""Independent source-pixel audit CLI; never changes accepted reader inputs."""
import argparse
import importlib
import importlib.metadata
import json
import os
import re
import shutil
import sys
from pathlib import Path

import inner_audit_common as common

AuditError = common.AuditError
DEFAULT_WORK = 'tmp/pdfs/facsimile_line_audit'
MODULES = ('facsimile_line_audit.py', 'facsimile_line_coverage.py',
           'facsimile_seam_review.py', 'inner_audit_common.py', 'audit_inner_tomi.py')
PACKAGES = {'numpy': 'numpy', 'Pillow': 'PIL', 'opencv-python-headless': 'cv2',
            'scikit-learn': 'sklearn', 'RapidFuzz': 'rapidfuzz', 'PyMuPDF': 'pymupdf'}


def read_json(path):
    try:
        return json.loads(Path(path).read_text(encoding='utf-8'))
    except (OSError, ValueError) as exc:
        raise AuditError(f'Cannot read JSON {path}: {exc}') from exc


def paths(args):
    repo = Path(args.repo_root).resolve()
    work = Path(args.work_dir)
    work = (repo / work).resolve() if not work.is_absolute() else work.resolve()
    return repo, work


def write(path, data, args, *, capture_root=None):
    repo, work = paths(args)
    common.write_json(path, data, work, repo, capture_root=capture_root,
                      policy=common.AuditOutputPolicy('audit'))


def check_audit_prerequisites(args):
    if sys.version_info < (3, 10):
        raise AuditError('Python 3.10+ is required')
    packages = {}
    pins = {}
    for line in (Path(__file__).parent / 'requirements-inner-audit.txt').read_text().splitlines():
        if '==' in line:
            name, version = line.split('==')
            pins[name] = version
    for name, module in PACKAGES.items():
        try:
            importlib.import_module(module)
            packages[name] = importlib.metadata.version(name)
        except Exception as exc:
            raise AuditError(f'Missing core prerequisite {name}: {exc}') from exc
        if packages[name] != pins[name]:
            raise AuditError(f'{name} requires pinned {pins[name]}, found {packages[name]}')
    tools = {}
    for name in ('pdftoppm', 'tesseract'):
        executable = getattr(args, name, None) or os.environ.get(f'PLUTARCH_{name.upper()}') or shutil.which(name)
        if not executable:
            if name == 'tesseract':
                tools['ocr_unavailable'] = 'Tesseract not resolved'
                continue
            raise AuditError('Poppler pdftoppm is required')
        executable = str(Path(executable).resolve())
        version = common.run_external([executable, '-v' if name == 'pdftoppm' else '--version']).splitlines()[0]
        tools[name] = {'path': executable, 'version': version, 'sha256': common.hash_file(executable)}
        if name == 'tesseract':
            directory = getattr(args, 'tessdata_dir', None) or os.environ.get('PLUTARCH_TESSDATA_DIR')
            command = [executable, '--list-langs']
            if directory:
                command.extend(['--tessdata-dir', str(directory)])
            languages = common.run_external(command)
            if not directory:
                match = re.search(r'"([^"]+)"', languages)
                directory = match[1] if match else None
            model = Path(directory) / 'eng.traineddata' if directory else None
            if model is None or not model.is_file() or 'eng' not in languages.splitlines()[1:]:
                tools['ocr_unavailable'] = 'English language bytes cannot be authenticated'
            else:
                tools[name].update(tessdata_dir=str(Path(directory).resolve()),
                                   eng_sha256=common.hash_file(model))
    return {'python': sys.version.split()[0], 'packages': packages, 'tools': tools}


def executed_code(repo):
    """Authenticate imported repository audit code, including future local helpers."""
    files = {repo / 'tools' / name for name in MODULES}
    for module in tuple(sys.modules.values()):
        filename = getattr(module, '__file__', None)
        if filename:
            path = Path(filename).resolve()
            if common.contained(path, repo / 'tools') and path.suffix == '.py':
                files.add(path)
    return {p.relative_to(repo).as_posix(): common.hash_file(p)
            for p in sorted(files) if p.is_file()}


def compute_runtime_identity(args, *, snapshot=None, settings=None, policy=None, scope=None):
    repo, _ = paths(args)
    prerequisites = check_audit_prerequisites(args)
    stable_tools = {name: {k: v for k, v in record.items() if k not in ('path', 'tessdata_dir')}
                    if isinstance(record, dict) else record
                    for name, record in prerequisites['tools'].items()}
    record = {'schema_version': 1, 'code': executed_code(repo),
              'settings_sha256': common.json_hash(settings), 'policy_sha256': common.json_hash(policy),
              'prerequisites': dict(prerequisites, tools=stable_tools), 'seed': settings.get('registration', {}).get('seed', 0),
              'scope': scope, 'snapshot_id': snapshot.get('snapshot_id') if snapshot else None,
              'stop_after': getattr(args, 'stop_after', None)}
    return dict(record, runtime_id=common.json_hash(record)), prerequisites


def validate_source_policy(policy, source_hashes=None, *, target_kind='corpus'):
    if policy.get('schema_version') != 1 or not isinstance(policy.get('version'), int):
        raise AuditError('Source policy requires schema_version 1 and integer version')
    hashes = policy.get('source_sha256', {})
    if target_kind != 'verification' and (policy.get('synthetic_regions') or policy.get('synthetic_duplicate_regions')):
        raise AuditError('Synthetic geometry is forbidden for corpus inputs')
    if source_hashes is not None and hashes != source_hashes:
        raise AuditError('Policy must bind exactly the captured source digests')
    for record in policy.get('decisions', []) + policy.get('duplicate_groups', []) + policy.get('page_overrides', []):
        if record.get('provenance') == 'synthetic_ground_truth' and target_kind != 'verification':
            raise AuditError('Synthetic policy is forbidden for corpus inputs')
        if not record.get('reason') or not re.fullmatch('[0-9a-f]{64}', record.get('evidence_sha256', '')):
            raise AuditError('Reviewed policy requires reason and evidence identity')
    return policy


def select_units(units, requested):
    identifiers = [u['id'] for u in units]
    if len(identifiers) != len(set(identifiers)):
        raise AuditError('Duplicate table unit IDs')
    selected = requested.split(',') if requested else identifiers
    if len(selected) != len(set(selected)) or any(i not in identifiers for i in selected):
        raise AuditError('Unknown or duplicate --units IDs')
    return [u for u in units if u['id'] in selected]


def capture_snapshot(args):
    import pymupdf
    repo, work = paths(args)
    root = Path(args.snapshot_dir).resolve()
    common.safe_write_path(root / 'snapshot-manifest.json', work, repo, capture_root=root)
    if root.exists():
        raise AuditError('Capture destination must be fresh')
    readers = Path(args.reader_dir)
    readers = (repo / readers).resolve() if not readers.is_absolute() else readers.resolve()
    settings_path = Path(args.settings) if args.settings else repo / 'tools/facsimile-line-settings.json'
    policy_path = Path(args.source_policy) if args.source_policy else repo / 'tools/facsimile-source-policy.json'
    if any(p.resolve() == (repo / 'secrets.json').resolve() for p in (settings_path, policy_path)):
        raise AuditError('Credentials cannot be audit inputs')
    settings, policy = read_json(settings_path), read_json(policy_path)
    runtime, prerequisites = compute_runtime_identity(args, settings=settings, policy=policy)
    reuse = None
    if getattr(args, 'reuse_snapshot', None):
        _, reuse = load_snapshot(args.reuse_snapshot)
        if reuse['runtime']['code'].get('tools/inner_audit_common.py') != runtime['code']['tools/inner_audit_common.py']:
            raise AuditError('Raw reader-map reuse requires identical executed mapper code')
    reused_inputs = {r['path']: r for r in reuse['inputs']} if reuse else {}
    records = []

    def add(path, role, **metadata):
        path = Path(path).resolve()
        if not path.is_file() or not common.contained(path, repo) or path.name == 'secrets.json':
            raise AuditError(f'Missing or escaping required capture input: {path}')
        relative = path.relative_to(repo).as_posix()
        records.append({'path': relative, 'snapshot_path': f'inputs/{relative}',
                        'role': role, 'sha256': common.hash_file(path), **metadata})
        return records[-1]

    source_hashes = {}
    for index in range(2):
        record = add(repo / f'Ploutarchou_Bioi_v{index + 1}.pdf', 'source_pdf', source_index=index)
        with pymupdf.open(repo / record['path']) as doc:
            record['page_count'] = len(doc)
        source_hashes[str(index)] = record['sha256']
    validate_source_policy(policy, source_hashes)
    add(repo / 'facsimile_units.json', 'unit_table')
    units = read_json(repo / 'facsimile_units.json')
    if len(units) != 68:
        raise AuditError('Corpus capture requires exactly 68 units')
    for name in ('inner_tomi_repair_source_trace.json', 'inner_tomi_audit_repair_source_trace.json'):
        add(repo / 'src/facsimile/fixtures' / name, 'repair_ledger')
    add(settings_path, 'settings')
    add(policy_path, 'source_policy')
    for filename in runtime['code']:
        add(repo / filename, 'code')
    reader_inputs = []
    assets_seen = set()
    for tomus in range(1, 9):
        pdf = readers / f'tomus_{tomus}_inner.pdf'
        tex = pdf.with_suffix('.tex')
        pdf_record = add(pdf, 'reader_pdf', tomus=tomus)
        tex_record = add(tex, 'reader_tex', tomus=tomus)
        assets = {}
        for reference in common.parse_tex_refs(tex):
            relative = common.relative_path(reference)
            asset = (readers / relative).resolve()
            if not common.contained(asset, readers):
                raise AuditError('Escaping image reference')
            if asset.exists():
                cached = reused_inputs.get(asset.relative_to(repo).as_posix())
                assets[reference] = (cached['decoded_identity'] if cached and cached.get('decoded_identity')
                                     and common.hash_file(asset) == cached['sha256'] else
                                     common.decoded_image_identity(pymupdf.Pixmap(str(asset))))
                if asset not in assets_seen:
                    add(asset, 'reader_asset', decoded_identity=assets[reference])
                    assets_seen.add(asset)
        reader_inputs.append(dict(tomus=tomus, pdf_path=pdf, tex_path=tex,
                                  pdf_sha256=pdf_record['sha256'], tex_sha256=tex_record['sha256'], body_assets=assets))
    mapping_inputs = [r for r in records if r['role'] in {'reader_pdf', 'reader_tex', 'reader_asset'}]
    can_reuse = reuse and all(r['path'] in reused_inputs and r['sha256'] == reused_inputs[r['path']]['sha256'] for r in mapping_inputs)
    if can_reuse:
        reader_map = reuse['reader_map']
        common.progress('Reused authenticated raw reader geometry and decoded pixels; no logical maps or verdicts reused')
    else:
        reader_map = common.map_reader_occurrences(reader_inputs)
    if len(reader_map['units']) != 68 or {u['unit_id'] for u in reader_map['units']} != {u['id'] for u in units}:
        raise AuditError('Reader/table unit identities differ')
    root.parent.mkdir(parents=True, exist_ok=True)
    root.mkdir()
    for record in sorted(records, key=lambda r: r['path']):
        destination = common.safe_write_path(root / record['snapshot_path'], work, repo, capture_root=root)
        destination.parent.mkdir(parents=True, exist_ok=True)
        with (repo / record['path']).open('rb') as source, destination.open('xb') as output:
            shutil.copyfileobj(source, output, 1024 * 1024)
        if common.hash_file(destination) != record['sha256'] or common.hash_file(repo / record['path']) != record['sha256']:
            raise AuditError(f'Input changed during capture: {record["path"]}')
    for record in records:
        if common.hash_file(repo / record['path']) != record['sha256']:
            raise AuditError(f'Original changed before capture completion: {record["path"]}')
    manifest = {'schema_version': 1, 'target_kind': 'corpus', 'capture_complete': True,
                'inputs': sorted(records, key=lambda r: r['path']), 'runtime': runtime,
                'observed_counts': {'sources': 2, 'readers': 8, 'units': 68,
                                    'physical_pages': sum(t['physical_pages'] for t in reader_map['tomi']),
                                    'body_pages': len(reader_map['body_pages'])},
                'reviewed_source_endpoints': {'0': 637, '1': 708},
                'reader_map': reader_map, 'operational_tools': prerequisites['tools']}
    stable = dict(manifest)
    stable.pop('operational_tools')
    manifest['snapshot_id'] = common.json_hash(stable)
    write(root / 'snapshot-manifest.json', manifest, args, capture_root=root)
    common.progress(f'Captured {manifest["snapshot_id"]}: {manifest["observed_counts"]}')
    return manifest


def load_snapshot(path):
    root = Path(path).resolve()
    manifest = read_json(root / 'snapshot-manifest.json')
    stable = {k: v for k, v in manifest.items() if k not in ('snapshot_id', 'operational_tools')}
    if manifest.get('schema_version') != 1 or manifest.get('capture_complete') is not True or common.json_hash(stable) != manifest.get('snapshot_id'):
        raise AuditError('Incomplete or changed snapshot manifest')
    seen = set()
    for record in manifest['inputs']:
        relative = common.relative_path(record['snapshot_path'])
        captured = (root / relative).resolve()
        if relative.as_posix() in seen or not common.contained(captured, root) or not captured.is_file() or common.hash_file(captured) != record['sha256']:
            raise AuditError(f'Snapshot input identity mismatch: {relative}')
        seen.add(relative.as_posix())
    return root, manifest


def snapshot_input(root, manifest, role, **selector):
    records = [r for r in manifest['inputs'] if r['role'] == role and all(r.get(k) == v for k, v in selector.items())]
    if len(records) != 1:
        raise AuditError(f'Expected one {role} input for {selector}')
    return root / records[0]['snapshot_path'], records[0]


def run_configuration(args):
    root, manifest = load_snapshot(args.snapshot)
    settings_path, _ = snapshot_input(root, manifest, 'settings')
    policy_path, _ = snapshot_input(root, manifest, 'source_policy')
    settings, policy = read_json(settings_path), read_json(policy_path)
    for supplied, captured in ((args.settings, settings), (args.source_policy, policy)):
        if supplied and read_json(supplied) != captured:
            raise AuditError('Changed settings/policy requires a new capture')
    repo, _ = paths(args)
    if executed_code(repo) != manifest['runtime']['code']:
        raise AuditError('Executed code differs from capture; create a new snapshot/run')
    unit_path, _ = snapshot_input(root, manifest, 'unit_table')
    units = select_units(read_json(unit_path), args.units)
    runtime, prerequisites = compute_runtime_identity(args, snapshot=manifest, settings=settings,
                                                     policy=policy, scope=[u['id'] for u in units])
    if runtime['prerequisites'] != manifest['runtime']['prerequisites']:
        raise AuditError('Tool/package/language identity differs from capture')
    return root, manifest, settings, policy, units, runtime, prerequisites


def inventory_run(args):
    import facsimile_line_coverage as coverage
    if not args.snapshot or not args.run_dir:
        raise AuditError('inventory requires --snapshot and --run-dir')
    root, manifest, settings, policy, units, runtime, prerequisites = run_configuration(args)
    repo, work = paths(args)
    run = common.safe_write_path(args.run_dir, work, repo, policy=common.AuditOutputPolicy('audit'))
    existing = run / 'run-manifest.json'
    if existing.exists():
        old = read_json(existing)
        if not args.resume or old['runtime_id'] != runtime['runtime_id']:
            raise AuditError('Existing run requires --resume with exact identity')
        if old.get('processing_complete'):
            for filename, digest in old['record_sha256'].items():
                if common.hash_file(run / common.relative_path(filename)) != digest:
                    raise AuditError('Completed run record digest changed')
            return 0
    elif run.exists() and not args.resume:
        raise AuditError('New run directory must be fresh')
    with common.run_directory(run, work, repo, policy=common.AuditOutputPolicy('audit')):
        run_record = dict(runtime, processing_complete=False, completed_stage='inventory', record_sha256={},
                          operational_snapshot=str(root), trace_diagnostics='not_requested')
        write(existing, run_record, args)
        unit_path, _ = snapshot_input(root, manifest, 'unit_table')
        inventory, errors = coverage.inventory_source_units(root, manifest, read_json(unit_path), units,
                                                            settings, run, args, prerequisites, source_policy=policy)
        inventory['runtime_id'] = runtime['runtime_id']
        inventory = coverage.apply_source_policy(inventory, policy, run=run, args=args)
        duplicates = coverage.resolve_duplicate_groups(inventory, policy)
        logical = coverage.build_logical_lines(inventory, duplicates)
        write(run / 'source-inventory.json', inventory, args)
        write(run / 'duplicate-groups.json', duplicates, args)
        write(run / 'logical-lines.json', logical, args)
        coverage.write_source_review_sheet(inventory, run, args)
        coverage.write_duplicate_review_sheet(inventory, duplicates, run, args)
        coverage.write_logical_review_sheet(logical, run, args)
        write(run / 'errors.json', {'schema_version': 1, 'snapshot_id': manifest['snapshot_id'],
                                  'runtime_id': runtime['runtime_id'], 'errors': errors}, args)
        run_record.update(processing_complete=not errors, requested_pages=inventory['requested_pages'],
                          completed_pages=inventory['completed_pages'],
                          record_sha256={name: common.hash_file(run / name)
                                         for name in ('source-inventory.json', 'duplicate-groups.json', 'logical-lines.json', 'errors.json')})
        write(existing, run_record, args)
    return 3 if errors else 0


def resolve_run_snapshot(run_record, args):
    _, work = paths(args)
    candidates = [Path(args.snapshot)] if args.snapshot else []
    if run_record.get('operational_snapshot'):
        candidates.append(Path(run_record['operational_snapshot']))
    candidates.extend(p.parent for p in (work / 'snapshots').glob('*/snapshot-manifest.json'))
    for candidate in candidates:
        if candidate.is_dir():
            manifest = read_json(candidate / 'snapshot-manifest.json')
            if manifest.get('snapshot_id') == run_record['snapshot_id']:
                return load_snapshot(candidate)
    raise AuditError('Cannot locate authenticated run snapshot; supply --snapshot')


def make_parser():
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest='command', required=True)
    for command in ('check', 'capture', 'inventory', 'run', 'report', 'adjudicate', 'verify', 'inspect-trace'):
        sub = subs.add_parser(command)
        sub.add_argument('--repo-root', default=str(Path(__file__).resolve().parent.parent))
        sub.add_argument('--work-dir', default=DEFAULT_WORK)
        for name in ('snapshot', 'run-dir', 'units', 'settings', 'source-policy', 'pdftoppm', 'tesseract', 'tessdata-dir', 'trace-dir'):
            sub.add_argument(f'--{name}')
        sub.add_argument('--resume', action='store_true')
        if command == 'capture':
            sub.add_argument('--reader-dir', default='build_facsimile')
            sub.add_argument('--snapshot-dir', required=True)
            sub.add_argument('--reuse-snapshot', help='Reuse authenticated raw reader geometry when mapper and input bytes agree')
        if command in ('run', 'verify'):
            sub.add_argument('--stop-after', choices=('locate', 'coverage'))
        if command == 'verify':
            sub.add_argument('--prepare', action='store_true')
            sub.add_argument('--out', required=True)
        if command == 'adjudicate':
            for name in ('inventory-dir', 'source-reviews', 'duplicate-reviews', 'policy-out', 'loss-reviews', 'reviews-out'):
                sub.add_argument(f'--{name}')
    return parser


def main():
    args = make_parser().parse_args()
    try:
        if args.command == 'check':
            common.progress(json.dumps(check_audit_prerequisites(args), indent=2))
        elif args.command == 'capture':
            capture_snapshot(args)
        elif args.command == 'inventory':
            return inventory_run(args)
        elif args.command == 'adjudicate':
            import facsimile_line_coverage as coverage
            if args.source_reviews and args.inventory_dir and args.policy_out and not args.duplicate_reviews and not args.loss_reviews:
                coverage.adjudicate_source(args)
            elif args.duplicate_reviews and args.inventory_dir and args.policy_out and not args.source_reviews and not args.loss_reviews:
                coverage.adjudicate_duplicates(args)
            else:
                raise AuditError('Source/duplicate mode requires --inventory-dir, exactly one review file and --policy-out')
        else:
            raise AuditError(f'{args.command}: this stage is not implemented yet')
        return 0
    except common.CalibrationError as exc:
        print(str(exc), file=sys.stderr)
        return 4
    except (AuditError, OSError, ValueError, KeyError) as exc:
        print(str(exc), file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        print('Interrupted processing', file=sys.stderr)
        return 3


if __name__ == '__main__':
    sys.exit(main())
