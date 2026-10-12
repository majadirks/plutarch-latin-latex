import json
from pathlib import Path
from PIL import Image

fixture=json.loads(Path('src/facsimile/fixtures/backlog_artifact_source_trace.json').read_text(encoding='utf-8'))
base=Path('tmp/pdfs/inner_tomi_audit_repairs/backlog_artifacts')
for record in fixture['repairs']:
    if record['backlog_id'] not in {'II2','II3','II4','II5','II6','II7','II8'}:
        continue
    print(record['backlog_id'])
    for line in record['reviewed_lines']:
        page=line['source_page']
        path=base/record['unit_id']/f's0-p{page}-raw.png'
        if not path.exists():
            continue
        im=Image.open(path).convert('L')
        left,top,right,bottom=line['region']
        limit=im.width-15 if page==record['source_page'] else right
        ink=[x for y in range(top,bottom) for x in range(left,min(right,limit)) if im.getpixel((x,y))<170]
        crop_left=next((entry['crop_edges'][0] for entry in record['source_evidence'] if entry['source_page']==page),left)
        left_margin=sum(im.getpixel((x,y))<170 for y in range(top,bottom) for x in range(crop_left,left))
        print(page,[left,top,right,bottom],(min(ink),max(ink)) if ink else None,
              'left-margin-ink',left_margin)
