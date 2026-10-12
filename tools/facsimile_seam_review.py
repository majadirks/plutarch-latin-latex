"""Optional, explicitly capped model observations for immutable facsimile tasks."""
import argparse
import base64
import importlib.metadata
import json
import re
import sys
from pathlib import Path
from collections import Counter
import inner_audit_common as common
import facsimile_line_audit as audit

SDK_VERSION = '0.84.0'
MODEL = 'claude-haiku-4-5'
PROMPT_VERSION = 'neutral-source-visible-reader-v1'
SCHEMA_VERSION = 1


def validate_observation(value,task):
    if not isinstance(value,dict) or set(value)!= {'task_id','lines'} or value['task_id']!=task['task_id'] or not isinstance(value['lines'],list):
        raise common.AuditError('Model response requires the exact task ID and lines array')
    allowed = {crop['crop_id']:crop for crop in task['reader_crops']}
    seen = set()
    for line in value['lines']:
        if not isinstance(line,dict) or set(line)!= {'line_id','verdict','object_type','reader_crop_ids','reason'}:
            raise common.AuditError('Model response line has invalid fields')
        identifier = line['line_id']
        if identifier not in task['line_ids'] or identifier in seen:
            raise common.AuditError('Model response has unknown or duplicate line IDs')
        if line['verdict'] not in ('complete','partial','absent','unclear') or line['object_type'] not in ('body','annotation','furniture','unclear'):
            raise common.AuditError('Model response has an unsupported verdict or object type')
        supports = line['reader_crop_ids']
        if (not isinstance(supports,list) or any(not isinstance(crop,str) or crop not in allowed for crop in supports)
            or len(supports)!=len(set(supports)) or any(identifier not in allowed[crop]['logical_line_ids'] for crop in supports)):
            raise common.AuditError('Model response has unknown or unrelated reader support')
        if not isinstance(line['reason'],str) or not line['reason'].strip():
            raise common.AuditError('Model response needs a reason for every line')
        if identifier in task['unlocalized_line_ids'] and line['verdict']!='unclear':
            raise common.AuditError('Unlocalized reader extent requires an unclear observation')
        if line['verdict']!='unclear' and not supports:
            raise common.AuditError('A decisive observation requires supporting visible-reader crops')
        seen.add(identifier)
    if seen!=set(task['line_ids']):
        raise common.AuditError('Model response is missing requested line IDs')
    return value


def task_prompt(task):
    labels = [{'crop_id':crop['crop_id'],'role':('source_reference' if crop.get('occurrence_id') in task['source_reference_occurrence_ids'].values() else 'source_alternate') if role=='source' else role,
               'line_ids':crop['logical_line_ids'],
               'label':crop['label'],'dimensions':crop['dimensions']} for role,items in
              (('source',task['source_crops']),('visible_reader',task['reader_crops'])) for crop in items]
    return ('Compare the labeled source pixels with actual visible reader pixels. Preserve the printed edition; '
        'do not infer missing Latin, reconstruct text, normalize spelling, or use a guessed transcription. '
        'Decide whether all required visible source ink is retained, partially retained, absent, or unclear. '
        'Alternative source scans may show the same passage; one complete visible copy suffices. '
        'Judge the complete source reference, not merely a damaged alternate. '
        'A region without localized reader extent must be unclear. Return JSON only, with task_id and lines. '
        'Include exactly one result for each listed immutable line ID, with fields line_id, '
        'verdict (complete/partial/absent/unclear), object_type (body/annotation/furniture/unclear), '
        'reader_crop_ids (supporting visible-reader crop IDs), and a concise reason. '
        'No additional IDs or fields.\n'+json.dumps({'task_id':task['task_id'],'line_ids':task['line_ids'],
        'unlocalized_line_ids':task['unlocalized_line_ids'],'images':labels},sort_keys=True))


def load_review_tasks(args):
    repo,work = audit.paths(args)
    path = Path(args.tasks).resolve()
    if path==(repo/'secrets.json').resolve():
        raise common.AuditError('Credentials cannot be task input')
    record = audit.read_json(path)
    if record.get('schema_version')!=1 or record.get('task_set_id')!=common.json_hash(record.get('tasks')):
        raise common.AuditError('Invalid or changed task set identity')
    parent = audit.read_json(path.parent/'report-manifest.json')
    machine = audit.read_json(path.parent/'run-manifest.json')
    if (parent['record_sha256']['review-tasks.json']!=common.hash_file(path) or parent['snapshot_id']!=record['snapshot_id'] or
        parent['runtime_id']!=record['runtime_id'] or machine['snapshot_id']!=record['snapshot_id'] or machine['runtime_id']!=record['runtime_id']):
        raise common.AuditError('Task file differs from its report manifest')
    for name in ('logical-lines.json','coverage-ledger.json'):
        if common.hash_file(path.parent/name)!=machine['record_sha256'][name]:
            raise common.AuditError('Task machine/source geometry identity changed')
    logical = {line['logical_line_id']:line for line in audit.read_json(path.parent/'logical-lines.json')['lines']}
    identifiers = set()
    from PIL import Image
    for task in record['tasks']:
        if (task['task_id'] in identifiers or task['snapshot_id']!=record['snapshot_id'] or task['runtime_id']!=record['runtime_id'] or
            len(task['line_ids'])>6 or len(task['line_ids'])!=len(set(task['line_ids']))):
            raise common.AuditError('Invalid task IDs or scope')
        identifiers.add(task['task_id'])
        if any(identifier not in logical for identifier in task['line_ids']):
            raise common.AuditError('Task contains unknown source region IDs')
        for crop in task['source_crops']+task['reader_crops']:
            actual = (path.parent/common.relative_path(crop['path'])).resolve()
            if not common.contained(actual,path.parent) or common.hash_file(actual)!=crop['sha256']:
                raise common.AuditError('Task image is stale or escapes its evidence root')
            with Image.open(actual) as image:
                if image.format!='PNG' or list(image.size)!=crop['dimensions'] or max(image.size)>1400:
                    raise common.AuditError('Task images require bounded neutral PNG tiles')
                image.verify()
    selected = args.task_ids.split(',') if args.task_ids else sorted(identifiers)
    if len(selected)!=len(set(selected)) or any(identifier not in identifiers for identifier in selected):
        raise common.AuditError('Unknown or repeated --task-ids')
    tasks = {task['task_id']:dict(task,source_reference_occurrence_ids={identifier:logical[identifier]['canonical_occurrence_id'] for identifier in task['line_ids']}) for task in record['tasks']}
    return path,record,[tasks[identifier] for identifier in selected]


def load_live_client(repo):
    try:
        version = importlib.metadata.version('anthropic')
    except importlib.metadata.PackageNotFoundError as exc:
        raise common.AuditError('Install the separately pinned Anthropic SDK for live review') from exc
    if version!=SDK_VERSION:
        raise common.AuditError('Live review requires pinned Anthropic SDK '+SDK_VERSION)
    from anthropic import Anthropic
    try:
        credentials = json.loads((repo/'secrets.json').read_text(encoding='utf-8'))
        key = credentials['anthropic_api_key']
    except (OSError,ValueError,KeyError) as exc:
        raise common.AuditError('Live review requires repository-root anthropic_api_key configuration') from exc
    if not isinstance(key,str) or not key.strip():
        raise common.AuditError('Live review requires a nonempty API key')
    return Anthropic(api_key=key,max_retries=0,timeout=120.0),version


def review_cache_metadata(task,model):
    prompt = task_prompt(task)
    return {'task':task,'model':model,'prompt_version':PROMPT_VERSION,'prompt_sha256':common.json_hash(prompt),
            'schema_version':SCHEMA_VERSION,'reviewer_sha256':common.hash_file(__file__),
            'helper_sha256':{Path(module.__file__).name:common.hash_file(module.__file__) for module in (common,audit)},
            'sdk_required_version':SDK_VERSION,'max_tokens':2000,'timeout_seconds':120,'max_retries':0}


def response_observation(raw,task,model,response_path):
    body = ''.join(block['text'] for block in raw['content'] if block.get('type')=='text')
    value = validate_observation(json.loads(body),task)
    return {'schema_version':1,'kind':'model_observation','transport':'live','task_id':task['task_id'],
            'observations':value,'response_sha256':common.hash_file(response_path),
            'model_requested':model,'model_returned':raw['model'],'usage':raw.get('usage'),
            'sdk_version':SDK_VERSION,'machine_ledger_modified':False}


def review_seam_tasks(args):
    repo,work = audit.paths(args)
    inspection = args.offline or args.dry_run
    triple = [bool(args.task_ids),bool(args.retry_unknown),bool(args.attempt_id)]
    if (args.retry_unknown or args.attempt_id) and not all(triple):
        raise common.AuditError('Unknown-delivery retry requires --task-ids, --retry-unknown and a new --attempt-id together')
    if args.attempt_id and not re.fullmatch(r'[A-Za-z0-9_-]{1,64}',args.attempt_id):
        raise common.AuditError('Attempt IDs require 1-64 letters, digits, underscores or hyphens')
    if args.max_calls<0 or (not inspection and args.max_calls<=0):
        raise common.AuditError('Live review requires positive --max-calls')
    if args.replay_response and (not args.offline or args.retry_unknown or not args.task_ids or ',' in args.task_ids):
        raise common.AuditError('Replay requires --offline and one explicit --task-ids selection')
    task_path,record,tasks = load_review_tasks(args)
    out = common.safe_write_path(args.out,work,repo,policy=common.AuditOutputPolicy('audit'))
    if out==task_path.parent or common.contained(task_path,out):
        raise common.AuditError('Model output requires a separate evidence directory')
    def write(path,value):
        common.write_json(path,value,work,repo,policy=common.AuditOutputPolicy('audit'))
    if args.replay_response:
        replay_path = Path(args.replay_response).resolve()
        if replay_path==(repo/'secrets.json').resolve():
            raise common.AuditError('Credentials cannot be replay input')
        value = validate_observation(audit.read_json(replay_path),tasks[0])
        result = {'schema_version':1,'kind':'replay','transport':'offline','task_id':tasks[0]['task_id'],
                  'response_sha256':common.hash_file(replay_path),'observations':value,
                  'model_requested':args.model,'model_returned':None,'usage':None,'machine_ledger_modified':False}
        write(out/'replays'/(common.json_hash(result)+'.json'),result)
        return 0
    if args.retry_unknown:
        for task in tasks:
            metadata = review_cache_metadata(task,args.model)
            folder = out/common.json_hash(metadata)
            state = audit.read_json(folder/'state.json')
            if (state.get('metadata')!=metadata or args.attempt_id in state.get('attempts',[]) or
                state.get('status') not in ('delivery_unknown','in_flight')):
                raise common.AuditError('Every retry selection requires unknown delivery and an unused attempt ID')
            active = state.get('active_attempt')
            if not isinstance(active,str) or not re.fullmatch(r'[A-Za-z0-9_-]{1,64}',active):
                raise common.AuditError('Invalid previous attempt identity')
            if (folder/'attempts'/active/'response.json').exists():
                raise common.AuditError('A preserved response requires local recovery rather than unknown-delivery resubmission')
    submitted,client,sdk = 0,None,None
    states = []
    failed = False
    with common.run_directory(out,work,repo,policy=common.AuditOutputPolicy('audit')):
        for task in tasks:
            prompt = task_prompt(task)
            metadata = review_cache_metadata(task,args.model)
            key = common.json_hash(metadata)
            folder = out/key
            state_path = folder/'state.json'
            if state_path.exists():
                state = audit.read_json(state_path)
                if state.get('metadata')!=metadata:
                    raise common.AuditError('Review cache identity differs')
                if not inspection and not args.resume:
                    raise common.AuditError('Existing model evidence requires --resume')
            else:
                state = {'schema_version':1,'task_id':task['task_id'],'cache_key':key,'metadata':metadata,
                         'status':'pending','attempts':[],'active_attempt':None}
            if (state.get('status') not in ('pending','in_flight','completed','delivery_unknown','response_invalid') or
                not isinstance(state.get('attempts'),list) or len(state['attempts'])!=len(set(state['attempts'])) or
                any(not isinstance(attempt,str) or not re.fullmatch(r'[A-Za-z0-9_-]{1,64}',attempt) for attempt in state['attempts']) or
                (state['status']!='pending' and state.get('active_attempt') not in state['attempts'])):
                raise common.AuditError('Invalid review delivery state or attempt identity')
            if state['status']=='in_flight':
                active = folder/'attempts'/state['active_attempt']
                if (active/'response.json').exists():
                    try:
                        recovered = response_observation(audit.read_json(active/'response.json'),task,args.model,active/'response.json')
                        write(active/'observation.json',recovered)
                        state['status']='completed'
                    except (ValueError,KeyError,TypeError,common.AuditError):
                        state['status']='response_invalid'
                else:
                    state['status']='delivery_unknown'
                    write(active/'delivery-unknown.json',{'status':'delivery_unknown','reason':'Previous request has no durable completed response'})
                write(state_path,state)
            if state['status']=='completed':
                active = folder/'attempts'/state['active_attempt']
                observed = audit.read_json(active/'observation.json')
                if common.hash_file(active/'response.json')!=observed['response_sha256']:
                    raise common.AuditError('Completed raw model response changed')
                original = response_observation(audit.read_json(active/'response.json'),task,args.model,active/'response.json')
                if original!=observed:
                    raise common.AuditError('Completed model observation differs from the preserved response')
            if args.retry_unknown:
                if state['status']!='delivery_unknown' or args.attempt_id in state['attempts']:
                    raise common.AuditError('Retry requires unknown delivery and an unused attempt ID')
            can_submit = (state['status']=='pending' or args.retry_unknown) and bool(task['line_ids']) and len(task['unlocalized_line_ids'])<len(task['line_ids'])
            preview = {'task_id':task['task_id'],'cache_key':key,'status':state['status'],'eligible':can_submit,
                       'line_ids':task['line_ids'],'images':[{'crop_id':crop['crop_id'],'dimensions':crop['dimensions']}
                        for crop in task['source_crops']+task['reader_crops']], 'prompt':prompt,
                       'retry_attempt_id':args.attempt_id if args.retry_unknown else None}
            if can_submit and not inspection and submitted<args.max_calls:
                attempt = args.attempt_id or 'initial'
                if attempt in state['attempts']:
                    raise common.AuditError('Attempt ID has already been used')
                attempt_dir = folder/'attempts'/attempt
                content = []
                for crop in task['source_crops']+task['reader_crops']:
                    content.append({'type':'text','text':crop['label']+' / crop_id '+crop['crop_id']})
                    content.append({'type':'image','source':{'type':'base64','media_type':'image/png',
                        'data':base64.b64encode((task_path.parent/crop['path']).read_bytes()).decode('ascii')}})
                content.append({'type':'text','text':prompt})
                if len(task['source_crops'])+len(task['reader_crops'])>100 or len(json.dumps(content).encode('utf-8'))>32*1024*1024:
                    raise common.AuditError('Image request exceeds the supported API image/request bounds')
                if client is None:
                    client,sdk = load_live_client(repo)
                if args.retry_unknown:
                    write(folder/'attempts'/state['active_attempt']/'archived-state.json',state)
                state.update(status='in_flight',active_attempt=attempt,attempts=state['attempts']+[attempt])
                write(state_path,state)
                submitted+=1
                try:
                    response = client.messages.create(model=args.model,max_tokens=2000,messages=[{'role':'user','content':content}])
                    raw = response.model_dump(mode='json')
                    write(attempt_dir/'response.json',raw)
                    try:
                        observation = response_observation(raw,task,args.model,attempt_dir/'response.json')
                        write(attempt_dir/'observation.json',observation)
                        state['status']='completed'
                    except (ValueError,common.AuditError):
                        state['status']='response_invalid'
                        write(attempt_dir/'validation-error.json',{'status':'response_invalid','reason':'Response JSON failed the local schema/identity checks'})
                        failed=True
                except BaseException as exc:
                    state['status']='delivery_unknown'
                    write(attempt_dir/'delivery-unknown.json',{'status':'delivery_unknown','exception_type':type(exc).__name__,
                                                            'http_status':getattr(exc,'status_code',None)})
                    failed=True
                    write(state_path,state)
                    if isinstance(exc,(KeyboardInterrupt,SystemExit)):
                        raise
                write(state_path,state)
                preview['status']=state['status']
            elif not can_submit and state['status']=='pending':
                preview['status']='inconclusive_not_submitted'
            if inspection:
                write(folder/'preview.json',preview)
            states.append(preview)
        summary = {'schema_version':1,'task_set_id':record['task_set_id'],'model_requested':args.model,
                   'offline':args.offline,'dry_run':args.dry_run,'max_calls':args.max_calls,'submitted':submitted,
                   'states':dict(Counter(state['status'] for state in states)),'selected_tasks':len(states),
                   'sdk_version':sdk,'machine_ledger_modified':False}
        write(out/'review-summary.json',summary)
        common.progress(json.dumps(summary,indent=2))
    if client is not None:
        client.close()
    return 3 if failed else 0


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tasks',required=True)
    parser.add_argument('--out',required=True)
    parser.add_argument('--repo-root',default=str(Path(__file__).resolve().parent.parent))
    parser.add_argument('--work-dir',default=audit.DEFAULT_WORK)
    parser.add_argument('--max-calls',type=int,default=0)
    parser.add_argument('--model',default=MODEL)
    for name in ('offline','resume','dry-run','retry-unknown'):
        parser.add_argument('--'+name,action='store_true')
    for name in ('task-ids','attempt-id','replay-response'):
        parser.add_argument('--'+name)
    args=parser.parse_args()
    try:
        return review_seam_tasks(args)
    except (common.AuditError,OSError,ValueError,KeyError,TypeError) as exc:
        print(str(exc),file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        return 3


if __name__=='__main__':
    sys.exit(main())
