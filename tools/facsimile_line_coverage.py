"""Independent source inventory and visible-reader coverage algorithms."""
import math
from pathlib import Path

import inner_audit_common as common
import audit_inner_tomi as historical


def component_observations(gray, threshold, connectivity=8, offset=(0, 0)):
    import cv2
    import numpy as np
    cv2.setNumThreads(1)
    count, labels, stats, centers = cv2.connectedComponentsWithStats(
        (gray < threshold).astype(np.uint8), connectivity=connectivity)
    ox, oy = offset
    records = []
    for index in range(1, count):
        x, y, width, height, area = map(int, stats[index])
        records.append({'component': index, 'threshold': threshold, 'area': area,
                        'box': [x + ox, y + oy, x + width + ox, y + height + oy],
                        'center': [float(centers[index, 0] + ox), float(centers[index, 1] + oy)]})
    return records, labels


def union_boxes(boxes):
    return [min(b[0] for b in boxes), min(b[1] for b in boxes),
            max(b[2] for b in boxes), max(b[3] for b in boxes)]


def separate_source_columns(gray, settings):
    """Measure a source-only gutter, preserving ink on both sides and at its edges."""
    import numpy as np
    width, height = gray.shape[1], gray.shape[0]
    roi = common.pixel_bounds(settings['inspection_roi'], [width, height])
    observations = []
    for threshold in settings['thresholds']:
        glyph = common.ink_line_height(gray, settings={'threshold': threshold,
                                                      'connectivity': settings['connectivity']})
        components, _ = component_observations(gray, threshold, settings['connectivity'])
        glyphs = [c for c in components if glyph and 0.5 * glyph <= c['box'][3] - c['box'][1] <= 1.8 * glyph
                  and c['box'][2] - c['box'][0] <= 4 * glyph]
        profile = (gray < threshold).sum(axis=0).astype(float)
        smoothed = np.convolve(profile, np.ones(settings['gutter']['smoothing_pixels']) / settings['gutter']['smoothing_pixels'], mode='same')
        left, right = [int(v * width) for v in settings['gutter']['search_x']]
        cutoff = float(np.percentile(smoothed[left:right], 90)) * settings['gutter']['profile_fraction']
        low = smoothed[left:right] <= cutoff
        boundaries = np.flatnonzero(np.diff(np.r_[False, low, False]))
        candidates = []
        for start, end in boundaries.reshape(-1, 2):
            x0, x1 = int(start + left), int(end + left)
            if not glyph or x1 - x0 < settings['gutter']['minimum_width_glyphs'] * glyph:
                continue
            before = [c for c in glyphs if roi[0] <= c['box'][0] and c['box'][2] <= x0]
            after = [c for c in glyphs if c['box'][0] >= x1]
            if len(before) >= 8 and len(after) >= 8:
                candidates.append({'box': [x0, 0, x1, height],
                                   'side_support': min(len(before), len(after)), 'width': x1 - x0, 'kind': 'low_ink_run'})
        if glyph:
            for component in components:
                bx = component['box']
                if (left <= bx[0] <= right and bx[3] - bx[1] > 8 * glyph
                        and bx[2] - bx[0] < 0.3 * glyph):
                    before = [c for c in glyphs if roi[0] <= c['box'][0] and c['box'][2] <= bx[0]]
                    after = [c for c in glyphs if c['box'][0] >= bx[2]]
                    if len(before) >= 8 and len(after) >= 8:
                        candidates.append({'box': [bx[0], 0, bx[2], height],
                                           'side_support': min(len(before), len(after)),
                                           'width': bx[2] - bx[0], 'kind': 'rule_hypothesis'})
        candidates.sort(key=lambda c: (-c['side_support'], -c['width'], c['box'][0]))
        observations.append({'threshold': threshold, 'glyph_height': glyph, 'candidates': candidates})
    first, second = observations
    valid = bool(first['candidates'] and second['candidates'] and first['glyph_height'] and
                 abs(first['candidates'][0]['box'][0] - second['candidates'][0]['box'][0]) <= first['glyph_height'])
    gutter = first['candidates'][0]['box'] if valid else None
    left_estimate = None
    if gutter and first['glyph_height']:
        glyph = first['glyph_height']
        components, _ = component_observations(gray, settings['thresholds'][0], settings['connectivity'])
        rows = {}
        for component in components:
            box = component['box']
            if box[0] >= gutter[2] and 0.5 * glyph <= box[3] - box[1] <= 1.8 * glyph:
                rows.setdefault(round(component['center'][1] / (2 * glyph)), []).append(box[0])
        starts = [min(items) for items in rows.values() if len(items) >= 6]
        clusters = {}
        for start in starts:
            clusters.setdefault(round(start / glyph), []).append(start)
        if clusters:
            repeated = sorted(clusters.values(), key=lambda values: (-len(values), min(values)))[0]
            if len(repeated) >= 3:
                left_estimate = int(np.median(repeated))
    safe_left = max(gutter[2], left_estimate - int(first['glyph_height'])) if gutter and left_estimate else gutter[2] if gutter else roi[0]
    return {'roi': list(roi), 'observations': observations, 'gutter': gutter,
            'status': 'measured' if valid else 'unresolved',
            'latin_left': safe_left, 'latin_left_estimate': left_estimate,
            'latin_edge_uncertainty': [max(roi[0], left_estimate - int(first['glyph_height'])),
                                       left_estimate + int(first['glyph_height'])] if left_estimate else None}


def assemble_source_lines(components, glyph_height, latin_left, settings):
    """Group against fitted baselines; preserve ambiguous marks as separate objects."""
    if not glyph_height:
        return [], components
    h = glyph_height
    seeds = [c for c in components if c['box'][0] >= latin_left
             and 0.5 * h <= c['box'][3] - c['box'][1] <= 1.8 * h
             and c['box'][2] - c['box'][0] <= 12 * h]
    groups = []
    used = set()
    for component in sorted(seeds, key=lambda c: (c['center'][1], c['center'][0], c['component'])):
        possible = [g for g in groups if abs(component['center'][1] - g['baseline']) <= settings['grouping']['vertical_glyphs'] * h
                    and all(abs(c['center'][1] - component['center'][1]) <= h for c in g['members'])
                    and min(max(c['box'][0] - component['box'][2], component['box'][0] - c['box'][2], 0)
                            for c in g['members']) <= settings['grouping']['horizontal_glyphs'] * h]
        if len(possible) == 1:
            group = possible[0]
            group['members'].append(component)
            group['baseline'] = sum(c['center'][1] for c in group['members']) / len(group['members'])
        elif not possible:
            groups.append({'baseline': component['center'][1], 'members': [component]})
        else:
            continue
        used.add(component['component'])
    merged = []
    for group in sorted(groups, key=lambda g: (g['baseline'], g['members'][0]['box'][0])):
        possible = [g for g in merged if abs(g['baseline'] - group['baseline']) <= settings['grouping']['baseline_glyphs'] * h
                    and not any(min(g['baseline'], group['baseline']) < other['baseline'] < max(g['baseline'], group['baseline'])
                                for other in groups if other is not group and abs(other['baseline'] - group['baseline']) > 0.25 * h)]
        if len(possible) == 1:
            target = possible[0]
            target['members'].extend(group['members'])
            target['baseline'] = sum(c['center'][1] for c in target['members']) / len(target['members'])
        else:
            merged.append(group)
    groups = merged
    # Marks attach only when both baseline and horizontal evidence have one parent.
    for component in components:
        if component['component'] in used or component['box'][0] < latin_left:
            continue
        if component['box'][3] - component['box'][1] > 1.8 * h:
            continue
        parents = []
        for group in groups:
            if abs(component['center'][1] - group['baseline']) > settings['grouping']['punctuation_vertical_glyphs'] * h:
                continue
            gap = min(max(c['box'][0] - component['box'][2], component['box'][0] - c['box'][2], 0) for c in group['members'])
            if gap <= settings['grouping']['punctuation_horizontal_glyphs'] * h:
                parents.append(group)
        if len(parents) == 1:
            parents[0]['members'].append(component)
            used.add(component['component'])
    lines = []
    for group in groups:
        members = sorted(group['members'], key=lambda c: (c['box'][0], c['box'][1]))
        lines.append({'box': union_boxes([c['box'] for c in members]), 'baseline': group['baseline'],
                      'components': [c['component'] for c in members]})
    return sorted(lines, key=lambda l: (l['baseline'], l['box'][0])), [c for c in components if c['component'] not in used]


def account_residual_ink(labels, assigned):
    """Every nonbackground component belongs to exactly one explicit object."""
    import numpy as np
    available = set(map(int, np.unique(labels))) - {0}
    seen = [c for ids in assigned for c in ids]
    if len(seen) != len(set(seen)) or set(seen) != available:
        raise common.AuditError('Inventory component accounting is incomplete or duplicated')
    return {'components': len(available), 'assigned_components': len(seen),
            'residual_pixels': 0, 'selected_pixels': int((labels > 0).sum())}


def save_crop(image, box, run, args, name):
    import facsimile_line_audit as cli
    repo, work = cli.paths(args)
    path = common.safe_write_path(run / 'crops' / f'{name}.png', work, repo,
                                  policy=common.AuditOutputPolicy('audit'))
    path.parent.mkdir(parents=True, exist_ok=True)
    image.crop(tuple(box)).save(path)
    return {'path': path.relative_to(run).as_posix(), 'sha256': common.hash_file(path), 'box_px': list(box)}


def inventory_source_page(image, record, page, owners, settings, run, args):
    import numpy as np
    from PIL import ImageDraw
    gray = np.asarray(image.convert('L'))
    columns = separate_source_columns(gray, settings)
    if settings.get('reviewed_gutter_override'):
        columns.update(settings['reviewed_gutter_override'], status='reviewed_source_override')
    x0, y0, x1, y1 = columns['roi']
    raw, labels = component_observations(gray[y0:y1, x0:x1], settings['thresholds'][0],
                                        settings['connectivity'], (x0, y0))
    scale = columns['observations'][0]['glyph_height']
    lines, residual = assemble_source_lines(raw, scale, columns['latin_left'], settings)
    detector = common.json_hash({'settings': settings, 'adapter': 'independent-source-components-v1'})
    objects = []
    assigned = []
    for line in lines:
        assigned.append(line['components'])
        objects.append(dict(line, kind='line', classification='body' if columns['status'] in {'measured', 'reviewed_source_override'} else 'uncertain',
                            segmentation_status='provisional', required_geometry_reviewed=False))
    residual_groups = {}
    for component in residual:
        box = component['box']
        kind = 'neighbor_column' if columns['gutter'] and box[2] <= columns['gutter'][0] else 'uncertainty'
        residual_groups.setdefault((kind, int(box[1] // (scale or 32))), []).append(component)
    for (kind, _), members in sorted(residual_groups.items()):
        box = union_boxes([c['box'] for c in members])
        ids = [c['component'] for c in members]
        assigned.append(ids)
        objects.append({'box': box, 'components': ids, 'kind': kind,
                        'classification': 'uncertain', 'segmentation_status': 'unresolved',
                        'required_geometry_reviewed': False})
    # Faint components have their own inventory objects when no dark parent exists.
    faint, faint_labels = component_observations(gray[y0:y1, x0:x1], settings['thresholds'][1],
                                               settings['connectivity'], (x0, y0))
    dark_owner = {c: i for i, obj in enumerate(objects) for c in obj['components']}
    relations = {}
    selected = labels > 0
    for faint_id, dark_id in np.unique(np.column_stack((faint_labels[selected], labels[selected])), axis=0):
        relations.setdefault(int(faint_id), set()).add(dark_owner[int(dark_id)])
    faint_assignment = []
    for component in faint:
        bx = component['box']
        parents = sorted(relations.get(component['component'], []))
        if len(parents) == 1:
            objects[parents[0]].setdefault('faint_components', []).append(component['component'])
            objects[parents[0]]['box'] = union_boxes([objects[parents[0]]['box'], bx])
        else:
            objects.append({'box': bx, 'components': [], 'faint_components': [component['component']],
                            'kind': 'threshold_disagreement', 'classification': 'uncertain',
                            'segmentation_status': 'unresolved', 'required_geometry_reviewed': False})
        faint_assignment.append([component['component']])
    overlay = image.convert('RGB')
    draw = ImageDraw.Draw(overlay)
    for obj in objects:
        box = obj['box']
        occurrence_id = common.json_hash([record['sha256'], page, box, obj['kind'], detector])
        obj.update(occurrence_id=occurrence_id, source_sha256=record['sha256'],
                   source_index=record['source_index'], page=page, dpi=settings['source_dpi'],
                   raster_dimensions=list(image.size), possible_owners=sorted(owners),
                   parent_ids=[], child_ids=[], ocr_observations=[])
        obj['touches_roi_edges'] = [name for name, yes in (
            ('left', box[0] <= x0), ('top', box[1] <= y0), ('right', box[2] >= x1), ('bottom', box[3] >= y1)) if yes]
        if scale and box[3] - box[1] > 8 * scale:
            obj['child_candidates'] = [[box[0], max(box[1], line['box'][1]), box[2], min(box[3], line['box'][3])]
                                       for line in lines if box_intersects_y(box, line['box'])]
        obj['crop'] = save_crop(image, box, run, args, occurrence_id)
        draw.rectangle(box, outline='green' if obj['classification'] == 'body' else 'red', width=2)
    full = save_crop(image, [0, 0, image.width, image.height], run, args, f'source_{record["source_index"]}_{page}')
    marked = save_crop(overlay, [0, 0, image.width, image.height], run, args, f'overlay_{record["source_index"]}_{page}')
    return {'source_index': record['source_index'], 'source_sha256': record['sha256'],
            'page': page, 'columns': columns, 'components': raw, 'faint_components': faint,
            'accounting': [account_residual_ink(labels, assigned), account_residual_ink(faint_labels, faint_assignment)],
            'segmentation_status': 'unresolved' if any(o['classification'] == 'uncertain' for o in objects) else 'provisional',
            'full_context': full, 'overlay': marked, 'occurrences': objects}


def box_intersects_y(first, second):
    return min(first[3], second[3]) > max(first[1], second[1])


def inventory_source_units(root, manifest, all_units, selected, settings, run, args, prerequisites, *, source_policy=None):
    from PIL import Image
    import facsimile_line_audit as cli
    repo, work = cli.paths(args)
    endpoints = manifest['reviewed_source_endpoints']
    ranges = []
    for index, unit in enumerate(all_units):
        following = next((u for u in all_units[index + 1:] if u['source'] == unit['source']), None)
        end = following['start_page'] if following else endpoints[str(unit['source'])]
        ranges.append(dict(unit_id=unit['id'], source_index=unit['source'], start=unit['start_page'], end=end))
    requested = {(r['source_index'], p) for r in ranges if r['unit_id'] in {u['id'] for u in selected}
                 for p in range(r['start'], r['end'] + 1)}
    for decision in (source_policy or {}).get('decisions', []):
        if decision.get('owner') in {u['id'] for u in selected}:
            requested.add((decision['source_index'], decision['page']))
    pages, errors = [], []
    for source_index, page in sorted(requested):
        record_path, record = cli.snapshot_input(root, manifest, 'source_pdf', source_index=source_index)
        owners = [r['unit_id'] for r in ranges if r['source_index'] == source_index and r['start'] <= page <= r['end']]
        try:
            metadata = {'snapshot_id': manifest['snapshot_id'], 'code': cli.executed_code(repo),
                        'source_sha256': record['sha256'], 'page': page, 'settings': common.json_hash(settings), 'owners': owners,
                        'policy': common.json_hash(source_policy)}
            entry_path = run / 'source-pages' / f'{source_index}_{page}.json'
            cached = common.reusable_entry(entry_path, metadata, work, repo, policy=common.AuditOutputPolicy('audit')) if args.resume else None
            if cached:
                for crop in [cached['full_context'], cached['overlay']] + [o['crop'] for o in cached['occurrences']]:
                    if common.hash_file(run / common.relative_path(crop['path'])) != crop['sha256']:
                        raise common.AuditError('Completed source-page crop identity mismatch')
                pages.append(cached)
                common.progress(f'Resumed source inventory {source_index}/{page}')
                continue
            raster, _ = common.render_page(record_path, record, page, settings['source_dpi'], work, repo,
                                           pdftoppm=prerequisites['tools']['pdftoppm']['path'],
                                           policy=common.AuditOutputPolicy('audit'))
            with Image.open(raster) as image:
                effective = measured_page_settings(settings, source_policy or {}, record, page, image)
                item = inventory_source_page(image, record, page, owners, effective, run, args)
            if 'ocr_unavailable' in prerequisites['tools']:
                item['ocr_status'] = 'ocr_unavailable'
            else:
                tesseract = prerequisites['tools']['tesseract']
                item['ocr'] = historical.ocr_page_region(record_path, record, page, settings['source_dpi'],
                    settings['ocr']['source_psm'], settings['ocr']['language'], work, repo,
                    box=settings['inspection_roi'], tesseract=tesseract['path'],
                    tessdata_dir=tesseract['tessdata_dir'], pdftoppm=prerequisites['tools']['pdftoppm']['path'],
                    policy=common.AuditOutputPolicy('audit'))
                item['ocr_status'] = 'observed'
            pages.append(item)
            cli.write(entry_path, {'schema_version': 1, 'metadata': metadata, 'complete': True, 'data': item}, args)
            cli.write(run / 'source-inventory-partial.json', {'schema_version': 1, 'snapshot_id': manifest['snapshot_id'],
                      'completed_pages': len(pages), 'requested_pages': len(requested),
                      'page_entries': [f'source-pages/{p["source_index"]}_{p["page"]}.json' for p in pages]}, args)
            common.progress(f'Inventory {source_index}/{page}: {len(item["occurrences"])} explicit objects')
        except (common.AuditError, OSError, ValueError) as exc:
            errors.append({'source_index': source_index, 'page': page, 'error': str(exc)})
    result = {'schema_version': 1, 'snapshot_id': manifest['snapshot_id'], 'pages': pages,
              'requested_pages': len(requested), 'completed_pages': len(pages), 'ranges': ranges,
              'raw_inventory_id': common.json_hash(pages)}
    return result, errors


def apply_source_policy(inventory, policy, *, run=None, args=None, render_children=True):
    import copy
    result = copy.deepcopy(inventory)
    lookup = {o['occurrence_id']: o for p in result['pages'] for o in p['occurrences']}
    for decision in policy.get('decisions', []):
        occurrence = lookup.get(decision['occurrence_id'])
        if occurrence is None:
            continue
        if occurrence['crop']['sha256'] != decision['crop_sha256'] or occurrence['source_sha256'] != decision['source_sha256']:
            raise common.AuditError('Source policy crop/source identity mismatch')
        occurrence.update(classification=decision['classification'], owner=decision.get('owner'),
                          required_geometry_reviewed=decision.get('required_geometry_reviewed', False),
                          source_partial=decision.get('source_partial', False),
                          review_provenance=decision.get('provenance', 'source_review'),
                          review_reason=decision['reason'], reviewer=decision['reviewer'],
                          unit_start=decision.get('unit_start', False), unit_end=decision.get('unit_end', False))
        occurrence['reviewed_scope_extension'] = bool(decision.get('owner') and decision['owner'] not in occurrence['possible_owners'])
        children = decision.get('children', [])
        if children and render_children and (run is None or args is None):
            raise common.AuditError('Measured source children require an owned run for their untouched crops')
        for child in children:
            validate_pixel_box(child['box_px'], occurrence['raster_dimensions'])
            box, parent = child['box_px'], occurrence['box']
            if not (parent[0] <= box[0] < box[2] <= parent[2] and parent[1] <= box[1] < box[3] <= parent[3]):
                raise common.AuditError('Reviewed child escapes its original source object')
            if child.get('classification') not in {'body', 'heading', 'annotation', 'furniture', 'uncertain'}:
                raise common.AuditError('Unknown source child classification')
            owner_sources = {r['unit_id']: r['source_index'] for r in inventory['ranges']}
            if child.get('owner') and owner_sources.get(child['owner']) != occurrence['source_index']:
                raise common.AuditError('Source child has conflicting source/owner identity')
            if child['classification'] == 'body' and not child.get('owner'):
                raise common.AuditError('Reviewed body child requires one owner')
            from PIL import Image
            child_id = common.json_hash([occurrence['occurrence_id'], box, 'reviewed_source_child_v1'])
            if render_children:
                with Image.open(Path(run) / occurrence['crop']['path']) as image:
                    crop = save_crop(image, [box[0] - parent[0], box[1] - parent[1], box[2] - parent[0], box[3] - parent[1]],
                                     Path(run), args, child_id)
            else:
                crop = dict(occurrence['crop'])
            crop['box_px'] = box
            item = dict(occurrence, occurrence_id=child_id, box=box, crop=crop,
                        classification=child['classification'], owner=child.get('owner'),
                        required_geometry_reviewed=True, kind='reviewed_child', components=[], faint_components=[],
                        parent_ids=[occurrence['occurrence_id']], child_ids=[])
            for page in result['pages']:
                if page['source_index'] == item['source_index'] and page['page'] == item['page']:
                    page['occurrences'].append(item)
                    break
            occurrence['child_ids'].append(child_id)
        if children:
            # The parent retains all original member components and any unreviewed ink.
            occurrence['required_geometry_reviewed'] = False
            occurrence['classification'] = 'uncertain'
    result['policy_sha256'] = common.json_hash(policy)
    validate_source_inventory(result)
    return result


def measured_page_settings(settings, policy, record, page, image):
    import copy
    import hashlib
    selected = [r for r in policy.get('page_overrides', [])
                if r['source_index'] == record['source_index'] and r['page'] == page]
    if not selected:
        return settings
    if len(selected) != 1:
        raise common.AuditError('Conflicting reviewed source-page overrides')
    decision = selected[0]
    if (record['sha256'] != decision['source_sha256'] or
            hashlib.sha256(image.convert('L').tobytes()).hexdigest() != decision['gray_sha256']):
        raise common.AuditError('Reviewed source-page override identity mismatch')
    validate_pixel_box(decision['gutter_box_px'], list(image.size))
    revised = copy.deepcopy(settings)
    revised['reviewed_gutter_override'] = {'gutter': decision['gutter_box_px'],
                                         'latin_left': decision['latin_left_px']}
    return revised


def validate_source_inventory(inventory):
    seen = set()
    owner_sources = {r['unit_id']: r['source_index'] for r in inventory['ranges']}
    for page in inventory['pages']:
        for occurrence in page['occurrences']:
            if occurrence['occurrence_id'] in seen:
                raise common.AuditError('Repeated source occurrence identity')
            seen.add(occurrence['occurrence_id'])
            validate_pixel_box(occurrence['box'], occurrence['raster_dimensions'])
            if occurrence['classification'] not in {'body', 'heading', 'annotation', 'furniture', 'uncertain'}:
                raise common.AuditError('Unknown source classification')
            if occurrence.get('owner') and owner_sources.get(occurrence['owner']) != occurrence['source_index']:
                raise common.AuditError('Reviewed owner conflicts with captured unit/source identity')
            if occurrence.get('required_geometry_reviewed') and occurrence['classification'] == 'body' and not occurrence.get('owner'):
                raise common.AuditError('Reviewed body geometry requires one logical owner')
        if any(a['residual_pixels'] or a['components'] != a['assigned_components'] for a in page['accounting']):
            raise common.AuditError('Unaccounted source ink')
    return inventory


def validate_pixel_box(box, dimensions):
    if (not isinstance(box, list) or len(box) != 4 or
            any(isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(v) for v in box) or
            not 0 <= box[0] < box[2] <= dimensions[0] or not 0 <= box[1] < box[3] <= dimensions[1]):
        raise common.AuditError(f'Invalid full-pixel box: {box}')
    return box


def write_source_review_sheet(inventory, run, args):
    import html
    import facsimile_line_audit as cli
    sections = ['<!doctype html><meta charset="utf-8"><title>Source-only inventory review</title>',
                '<style>body{font:18px system-ui;max-width:1200px;margin:auto}img{max-width:100%}article{border-top:1px solid #ccc;padding:1em}code{word-break:break-all}</style>',
                '<h1>Source-only inventory review</h1><p>Classification and ownership hypotheses require source review. No reader verdict is shown.</p>']
    for page in inventory['pages']:
        sections.append(f'<article><h2>Source {page["source_index"] + 1}, physical page {page["page"]}</h2>')
        sections.append(f'<a href="{html.escape(page["full_context"]["path"], quote=True)}">Untouched source</a>')
        sections.append(f'<img loading="lazy" src="{html.escape(page["overlay"]["path"], quote=True)}">')
        for obj in page['occurrences']:
            sections.append(f'<p><code>{obj["occurrence_id"]}</code>: {html.escape(obj["kind"])} / {html.escape(obj["classification"])}; possible owners {html.escape(", ".join(obj["possible_owners"]))}; <a href="{obj["crop"]["path"]}">crop</a></p>')
        sections.append('</article>')
    repo, work = cli.paths(args)
    path = common.safe_write_path(run / 'source-review.html', work, repo, policy=common.AuditOutputPolicy('audit'))
    path.write_text('\n'.join(sections), encoding='utf-8')


def adjudicate_source(args):
    import facsimile_line_audit as cli
    run = Path(args.inventory_dir).resolve()
    inventory = cli.read_json(run / 'source-inventory.json')
    reviews = cli.read_json(args.source_reviews)
    if reviews.get('schema_version') != 1:
        raise common.AuditError('Source reviews require schema_version 1')
    if reviews.get('snapshot_id') != inventory['snapshot_id'] or reviews.get('raw_inventory_id') != inventory['raw_inventory_id']:
        raise common.AuditError('Stale source review snapshot/inventory identity')
    repo, work = cli.paths(args)
    target = common.safe_write_path(args.policy_out, work, repo, policy=common.AuditOutputPolicy('audit'))
    if target.exists():
        raise common.AuditError('Policy destination must be fresh')
    manifest = cli.read_json(run / 'run-manifest.json')
    if common.hash_file(run / 'source-inventory.json') != manifest['record_sha256']['source-inventory.json']:
        raise common.AuditError('Source inventory record identity mismatch')
    root, captured = cli.resolve_run_snapshot(manifest, args)
    policy_path, _ = cli.snapshot_input(root, captured, 'source_policy')
    policy = cli.read_json(policy_path)
    lookup = {o['occurrence_id']: o for p in inventory['pages'] for o in p['occurrences']}
    imported = []
    for decision in reviews.get('decisions', []):
        for flag in ('required_geometry_reviewed', 'source_partial', 'unit_start', 'unit_end'):
            if flag in decision and not isinstance(decision[flag], bool):
                raise common.AuditError(f'Source review {flag} requires a boolean')
        obj = lookup.get(decision.get('occurrence_id'))
        if not obj or obj['crop']['sha256'] != decision.get('crop_sha256'):
            raise common.AuditError('Unknown occurrence or stale source crop hash')
        if common.hash_file(run / common.relative_path(obj['crop']['path'])) != obj['crop']['sha256']:
            raise common.AuditError('Reviewed source crop bytes changed')
        if not decision.get('reason') or not decision.get('reviewer'):
            raise common.AuditError('Source decision requires reason and reviewer')
        if decision.get('classification') not in {'body', 'heading', 'annotation', 'furniture', 'uncertain'}:
            raise common.AuditError('Invalid reviewed classification')
        owner_sources = {r['unit_id']: r['source_index'] for r in inventory['ranges']}
        if decision.get('owner') and owner_sources.get(decision['owner']) != obj['source_index']:
            raise common.AuditError('Reviewed owner lies outside captured source/owner identity')
        if decision.get('required_geometry_reviewed') and decision['classification'] == 'body' and not decision.get('owner'):
            raise common.AuditError('Body review requires one owner')
        record = dict(decision, source_sha256=obj['source_sha256'], source_index=obj['source_index'], page=obj['page'],
                      evidence_sha256=obj['crop']['sha256'],
                      snapshot_id=inventory['snapshot_id'], raw_inventory_id=inventory['raw_inventory_id'])
        if record.get('provenance') == 'synthetic_ground_truth' and captured['target_kind'] != 'verification':
            raise common.AuditError('Synthetic review cannot approve corpus source')
        imported.append(record)
    page_imports = []
    for decision in reviews.get('page_overrides', []):
        pages = [p for p in inventory['pages'] if p['source_index'] == decision.get('source_index')
                 and p['page'] == decision.get('page')]
        if len(pages) != 1 or pages[0]['full_context']['sha256'] != decision.get('full_crop_sha256'):
            raise common.AuditError('Stale reviewed page-context identity')
        page = pages[0]
        if common.hash_file(run / common.relative_path(page['full_context']['path'])) != page['full_context']['sha256']:
            raise common.AuditError('Reviewed source page bytes changed')
        from PIL import Image
        import hashlib
        with Image.open(run / page['full_context']['path']) as image:
            validate_pixel_box(decision['gutter_box_px'], list(image.size))
            if not decision['gutter_box_px'][2] <= decision['latin_left_px'] < image.width:
                raise common.AuditError('Reviewed Latin edge must follow the gutter')
            digest = hashlib.sha256(image.convert('L').tobytes()).hexdigest()
        if not decision.get('reason') or not decision.get('reviewer'):
            raise common.AuditError('Page override requires source-only reason and reviewer')
        page_imports.append(dict(decision, gray_sha256=digest, source_sha256=page['source_sha256'],
                                 evidence_sha256=page['full_context']['sha256']))
    if len({(p['source_index'], p['page']) for p in page_imports}) != len(page_imports):
        raise common.AuditError('Duplicate source page overrides')
    if len({d['occurrence_id'] for d in imported}) != len(imported):
        raise common.AuditError('Duplicate source review decisions')
    revised = {d['occurrence_id']: d for d in policy['decisions']}
    revised.update({d['occurrence_id']: d for d in imported})
    policy['decisions'] = [revised[k] for k in sorted(revised)]
    previous_pages = {(p['source_index'], p['page']): p for p in policy.get('page_overrides', [])}
    previous_pages.update({(p['source_index'], p['page']): p for p in page_imports})
    policy['page_overrides'] = [previous_pages[k] for k in sorted(previous_pages)]
    policy['version'] += 1
    apply_source_policy(inventory, policy, run=run, args=args, render_children=False)
    cli.validate_source_policy(policy, target_kind=captured['target_kind'])
    cli.write(target, policy, args)
    return policy
