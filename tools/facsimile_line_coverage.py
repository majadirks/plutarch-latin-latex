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
    source_settings = {key: settings[key] for key in ('source_dpi', 'inspection_roi', 'thresholds',
                                                    'connectivity', 'grouping', 'gutter')}
    if settings.get('reviewed_gutter_override'):
        source_settings['reviewed_gutter_override'] = settings['reviewed_gutter_override']
    detector = common.json_hash({'settings': source_settings, 'adapter': 'independent-source-components-v1'})
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
    for group in (source_policy or {}).get('duplicate_groups', []):
        if group.get('owner') in {u['id'] for u in selected}:
            for mapping in group['region_maps']:
                requested.add((mapping['source_index'], mapping['page']))
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
            if any(p['source_index'] == decision['source_index'] and p['page'] == decision['page'] for p in inventory['pages']):
                raise common.AuditError('Reviewed source occurrence is stale on an inventoried page')
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
            measured_box = child['box_px']
            box = [math.floor(measured_box[0]), math.floor(measured_box[1]),
                   math.ceil(measured_box[2]), math.ceil(measured_box[3])]
            parent = occurrence['box']
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
                        measured_box_px=measured_box,
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


def seed_duplicate_groups(policy):
    """Preserve numeric provenance without granting geometric equivalence."""
    return sorted([{'seed_id': common.json_hash(seed['numeric_provenance']),
                    'status': 'requires_region_review', 'numeric_provenance': seed['numeric_provenance']}
                   for seed in policy.get('duplicate_seeds', [])], key=lambda seed: seed['seed_id'])


def polygon_mask(polygon, dimensions):
    import cv2
    import numpy as np
    width, height = map(int, dimensions)
    mask = np.zeros((height, width), dtype=np.uint8)
    cv2.fillPoly(mask, [np.rint(polygon).astype(np.int32)], 1)
    return mask.astype(bool)


def resolve_duplicate_groups(inventory, policy):
    """Authenticate reviewed source fragments in a fixed canonical pixel frame."""
    import cv2
    import numpy as np
    lookup = {o['occurrence_id']: o for p in inventory['pages'] for o in p['occurrences']}
    owner_sources = {r['unit_id']: r['source_index'] for r in inventory['ranges']}
    groups, assigned, positions = [], {}, set()
    for group in policy.get('duplicate_groups', []):
        canonical = lookup.get(group.get('canonical_occurrence_id'))
        maps = group.get('region_maps', [])
        present = [m for m in maps if m.get('occurrence_id') in lookup]
        if canonical is None and not present:
            if any(m.get('source_index') == p['source_index'] and m.get('page') == p['page']
                   for m in maps for p in inventory['pages']):
                raise common.AuditError('Reviewed duplicate occurrence is stale on an inventoried page')
            continue
        if canonical is None or len(present) != len(maps):
            raise common.AuditError('Duplicate group is only partially inventoried')
        owner = group.get('owner')
        if canonical.get('owner') != owner or owner_sources.get(owner) != canonical['source_index']:
            raise common.AuditError('Canonical duplicate owner is unreviewed or conflicting')
        if (canonical['crop']['sha256'] != group.get('canonical_crop_sha256') or
                canonical['source_sha256'] != group.get('source_sha256') or
                group.get('evidence_sha256') != canonical['crop']['sha256']):
            raise common.AuditError('Stale canonical duplicate evidence')
        sequence = group.get('logical_sequence')
        if isinstance(sequence, bool) or not isinstance(sequence, int) or sequence < 0 or (owner, sequence) in positions:
            raise common.AuditError('Duplicate passage requires a unique nonnegative logical sequence')
        positions.add((owner, sequence))
        width, height = canonical['crop']['box_px'][2] - canonical['crop']['box_px'][0], canonical['crop']['box_px'][3] - canonical['crop']['box_px'][1]
        if width != int(width) or height != int(height):
            raise common.AuditError('Canonical crop requires integer pixel dimensions')
        canonical_union = np.zeros((int(height), int(width)), dtype=bool)
        reference_complete = canonical.get('required_geometry_reviewed', False) and not canonical.get('source_partial', False)
        for mapping in maps:
            obj = lookup[mapping['occurrence_id']]
            if (obj['crop']['sha256'] != mapping.get('crop_sha256') or
                    mapping.get('evidence_sha256') != obj['crop']['sha256'] or
                    obj['source_sha256'] != canonical['source_sha256'] or obj.get('owner') != owner):
                raise common.AuditError('Duplicate fragment evidence/source/owner conflict')
            validate_pixel_box(mapping['source_box_px'], obj['raster_dimensions'])
            b, parent = mapping['source_box_px'], obj['box']
            if not (parent[0] <= b[0] < b[2] <= parent[2] and parent[1] <= b[1] < b[3] <= parent[3]):
                raise common.AuditError('Duplicate fragment escapes its original occurrence')
            matrix = np.asarray(mapping['to_canonical'], dtype=float)
            if matrix.shape != (2, 3) or not np.isfinite(matrix).all() or abs(np.linalg.det(matrix[:, :2])) < 1e-8:
                raise common.AuditError('Invalid canonical affine transform')
            if obj['occurrence_id'] == canonical['occurrence_id']:
                origin = canonical['crop']['box_px'][:2]
                if not np.allclose(matrix, [[1,0,-origin[0]], [0,1,-origin[1]]], atol=1e-9, rtol=0):
                    raise common.AuditError('Canonical self-map must preserve its actual crop pixel frame')
            polygon = np.asarray(mapping['covered_polygon'], dtype=float)
            if (polygon.ndim != 2 or polygon.shape[1] != 2 or len(polygon) < 3 or
                    not np.isfinite(polygon).all() or (polygon[:, 0] < 0).any() or
                    (polygon[:, 0] > width).any() or (polygon[:, 1] < 0).any() or (polygon[:, 1] > height).any() or
                    abs(cv2.contourArea(polygon.astype(np.float32))) < 1):
                raise common.AuditError('Invalid canonical coverage polygon')
            residual = mapping.get('max_residual_px')
            if isinstance(residual, bool) or not isinstance(residual, (int, float)) or not math.isfinite(residual) or residual < 0:
                raise common.AuditError('Duplicate fragment requires a finite measured residual')
            corners = np.asarray([[b[0], b[1], 1], [b[2], b[1], 1], [b[2], b[3], 1], [b[0], b[3], 1]]) @ matrix.T
            contour = corners.astype(np.float32)
            if any(cv2.pointPolygonTest(contour, tuple(map(float, point)), True) < -0.01 for point in polygon):
                raise common.AuditError('Claimed coverage exceeds its mapped physical fragment')
            mask = polygon_mask(polygon, [width, height])
            if mapping.get('complete') and (not mask.all() or obj.get('source_partial', False)):
                raise common.AuditError('Partial source fragment cannot claim complete canonical coverage')
            if obj['occurrence_id'] == canonical['occurrence_id']:
                canonical_union |= mask
            previous = assigned.get(obj['occurrence_id'])
            if previous is not None and previous != group['canonical_occurrence_id']:
                raise common.AuditError('Source occurrence belongs to conflicting duplicate groups')
        if not maps or not canonical_union.all():
            raise common.AuditError('Canonical reference maps leave required spans uncovered')
        if group.get('complete') and not reference_complete:
            raise common.AuditError('Unreviewed or partial canonical reference cannot claim completeness')
        assigned.update({m['occurrence_id']: group['canonical_occurrence_id'] for m in maps})
        groups.append(dict(group, status='reviewed' if reference_complete else 'unresolved_reference',
                           canonical_dimensions=[int(width), int(height)], reference_complete=reference_complete,
                           group_id=common.json_hash([owner, sequence, canonical['occurrence_id'], maps])))
    return {'schema_version': 1, 'snapshot_id': inventory['snapshot_id'],
            'runtime_id': inventory.get('runtime_id'), 'seeds': seed_duplicate_groups(policy),
            'groups': sorted(groups, key=lambda g: (g['owner'], g['logical_sequence'], g['group_id']))}


def write_duplicate_review_sheet(inventory, duplicates, run, args):
    import html
    from PIL import Image, ImageDraw
    import facsimile_line_audit as cli
    repo, work = cli.paths(args)
    lookup = {o['occurrence_id']: o for p in inventory['pages'] for o in p['occurrences']}
    sections = ['<!doctype html><meta charset="utf-8"><title>Source duplicate review</title>',
                '<style>body{font:18px system-ui;max-width:1200px;margin:auto}img{max-width:48%}code{word-break:break-all}</style>',
                '<h1>Source-only duplicate review</h1><p>Numeric seeds require measured region review. No reader outcome grants equivalence.</p>']
    for seed in duplicates['seeds']:
        provenance = seed['numeric_provenance']
        sections.append('<h2>' + html.escape(provenance['unit_id']) + '</h2><pre>' +
                        html.escape(__import__('json').dumps(provenance, ensure_ascii=False, indent=2)) + '</pre>')
        pages = set(provenance.get('skip_pages', []) + provenance.get('retain_sequence', []))
        for page in inventory['pages']:
            if page['source_index'] == provenance['source'] and page['page'] in pages:
                sections.append(f'<p>Physical source page {page["page"]}</p><img loading="lazy" src="{html.escape(page["full_context"]["path"], quote=True)}">')
    for group in duplicates['groups']:
        sections.append('<h2>' + html.escape(group['owner']) + ' canonical ' + group['canonical_occurrence_id'] + '</h2>')
        canonical = lookup[group['canonical_occurrence_id']]
        with Image.open(Path(run) / canonical['crop']['path']) as original:
            overlay = original.convert('RGB')
            drawing = ImageDraw.Draw(overlay)
            for mapping in group['region_maps']:
                drawing.polygon([tuple(point) for point in mapping['covered_polygon']], outline='magenta', width=2)
            crop = save_crop(overlay, [0, 0, overlay.width, overlay.height], Path(run), args, 'duplicate_' + group['group_id'])
        sections.append('<p>Canonical coverage polygons</p><img src="' + crop['path'] + '">')
        for mapping in group['region_maps']:
            obj = lookup[mapping['occurrence_id']]
            sections.append('<p><code>' + obj['occurrence_id'] + '</code> ' + html.escape(str(mapping['covered_polygon'])) +
                            '</p><img loading="lazy" src="' + html.escape(obj['crop']['path'], quote=True) + '">')
    target = common.safe_write_path(Path(run) / 'duplicate-review.html', work, repo, policy=common.AuditOutputPolicy('audit'))
    target.write_text('\n'.join(sections), encoding='utf-8')


def adjudicate_duplicates(args):
    import facsimile_line_audit as cli
    run = Path(args.inventory_dir).resolve()
    inventory = cli.read_json(run / 'source-inventory.json')
    reviews = cli.read_json(args.duplicate_reviews)
    if (reviews.get('schema_version') != 1 or reviews.get('snapshot_id') != inventory['snapshot_id'] or
            reviews.get('raw_inventory_id') != inventory['raw_inventory_id']):
        raise common.AuditError('Stale duplicate review schema/inventory/snapshot identity')
    repo, work = cli.paths(args)
    target = common.safe_write_path(args.policy_out, work, repo, policy=common.AuditOutputPolicy('audit'))
    if target.exists():
        raise common.AuditError('Duplicate policy destination must be fresh')
    run_record = cli.read_json(run / 'run-manifest.json')
    if common.hash_file(run / 'source-inventory.json') != run_record['record_sha256']['source-inventory.json']:
        raise common.AuditError('Source inventory record identity mismatch')
    root, captured = cli.resolve_run_snapshot(run_record, args)
    policy_path, _ = cli.snapshot_input(root, captured, 'source_policy')
    policy = cli.read_json(policy_path)
    groups = reviews.get('duplicate_groups', [])
    lookup = {o['occurrence_id']: o for p in inventory['pages'] for o in p['occurrences']}
    for group in groups:
        if not group.get('reviewer') or not group.get('reason'):
            raise common.AuditError('Duplicate decision requires reviewer and source-only reason')
        if group.get('canonical_occurrence_id') not in lookup or not group.get('region_maps'):
            raise common.AuditError('Duplicate review requires an inventoried canonical reference and maps')
        if group.get('provenance') == 'synthetic_ground_truth' and captured['target_kind'] != 'verification':
            raise common.AuditError('Synthetic review cannot approve corpus duplicates')
        for mapping in group.get('region_maps', []):
            obj = lookup.get(mapping.get('occurrence_id'))
            if obj is None or common.hash_file(run / common.relative_path(obj['crop']['path'])) != mapping.get('crop_sha256'):
                raise common.AuditError('Duplicate crop bytes changed or occurrence is unknown')
            mapping.update(source_index=obj['source_index'], page=obj['page'])
    policy['duplicate_groups'] = policy.get('duplicate_groups', []) + groups
    policy['version'] += 1
    resolved_inventory = apply_source_policy(inventory, policy, render_children=False)
    resolve_duplicate_groups(resolved_inventory, policy)
    cli.validate_source_policy(policy, target_kind=captured['target_kind'])
    cli.write(target, policy, args)
    return policy


def build_logical_lines(inventory, duplicate_groups):
    """Retain every physical object and give reviewed equivalents one reference."""
    objects = [o for p in inventory['pages'] for o in p['occurrences']]
    lookup = {o['occurrence_id']: o for o in objects}
    pages = {(p['source_index'], p['page']): p for p in inventory['pages']}
    component_lookup = {(p['source_index'], p['page'], field, c['component']): c
                        for p in inventory['pages'] for field in ('components', 'faint_components') for c in p[field]}
    groups = duplicate_groups['groups']
    grouped = {m['occurrence_id']: g for g in groups for m in g['region_maps']}
    canonical_objects = {grouped[o['occurrence_id']]['canonical_occurrence_id'] if o['occurrence_id'] in grouped
                         else o['occurrence_id'] for o in objects}
    def source_order(obj):
        group = grouped.get(obj['occurrence_id'])
        if group:
            return min((lookup[m['occurrence_id']]['source_index'], lookup[m['occurrence_id']]['page'],
                        m['source_box_px'][1], m['source_box_px'][0], m['occurrence_id']) for m in group['region_maps'])
        return obj['source_index'], obj['page'], obj['box'][1], obj['box'][0], obj['occurrence_id']

    ordered = sorted((lookup[i] for i in canonical_objects), key=source_order)
    positions, group_order, logical = {}, {}, []
    for obj in ordered:
        group = grouped.get(obj['occurrence_id'])
        owner = group['owner'] if group else obj.get('owner')
        possible_owners = obj['possible_owners']
        owner_hint = owner or (possible_owners[0] if len(possible_owners) == 1 else None)
        x0, y0, x1, y1 = obj['box']
        width, height = x1 - x0, y1 - y0
        maps = group['region_maps'] if group else [{
            'occurrence_id': obj['occurrence_id'], 'source_box_px': obj['box'],
            'source_index': obj['source_index'], 'page': obj['page'],
            'to_canonical': [[1, 0, -x0], [0, 1, -y0]],
            'covered_polygon': [[0, 0], [width, 0], [width, height], [0, height]],
            'crop_sha256': obj['crop']['sha256'], 'evidence_sha256': obj['crop']['sha256'], 'max_residual_px': 0}]
        if group:
            if group['logical_sequence'] <= group_order.get(owner, -1):
                raise common.AuditError('Reviewed duplicate sequence contradicts canonical source order')
            group_order[owner] = group['logical_sequence']
        geometry_maps = [{key: mapping[key] for key in ('occurrence_id', 'source_box_px', 'to_canonical',
                                                       'covered_polygon', 'crop_sha256')} for mapping in maps]
        line_id = common.json_hash([owner, obj['occurrence_id'], geometry_maps])
        position = positions.get(owner_hint, 0)
        positions[owner_hint] = position + 1
        reviewed = bool(owner) and obj.get('required_geometry_reviewed', False) and not obj.get('source_partial', False)
        if group and not group['reference_complete']:
            reviewed = False
        components = []
        for field in ('components', 'faint_components'):
            for identifier in obj.get(field, []):
                member = component_lookup[(obj['source_index'], obj['page'], field, identifier)]
                box = member['box']
                components.append({'component': identifier, 'threshold': member['threshold'], 'area': member['area'],
                                   'box_px': [box[0]-x0, box[1]-y0, box[2]-x0, box[3]-y0]})
        spans = []
        for identifier in sorted({m['occurrence_id'] for m in maps}):
            members = [m for m in maps if m['occurrence_id'] == identifier]
            if group:
                import numpy as np
                union = np.zeros((int(height), int(width)), dtype=bool)
                for mapping in members:
                    union |= polygon_mask(mapping['covered_polygon'], [width, height])
                fraction = float(union.mean())
            else:
                fraction = 1.0
            spans.append({'occurrence_id': identifier, 'canonical_box_fraction': fraction,
                          'full_reference_span': fraction == 1.0 and not lookup[identifier].get('source_partial', False),
                          'source_partial': lookup[identifier].get('source_partial', False)})
        uncertainty = [] if reviewed else ['unreviewed_source_geometry_or_owner']
        if obj.get('source_partial', False):
            uncertainty.append('source_partial_reference')
        if obj.get('child_ids'):
            uncertainty.append('parent_retains_unreviewed_ink')
        logical.append({'logical_line_id': line_id, 'owner': owner_hint, 'owner_reviewed': bool(owner),
                        'sequence_position': position, 'canonical_occurrence_id': obj['occurrence_id'],
                        'canonical_dimensions': [width, height], 'canonical_crop': obj['crop'],
                        'required_box_px': [0, 0, width, height], 'required_components': components,
                        'required_ink_source': 'untouched_canonical_crop', 'classification': obj['classification'],
                        'geometry_reviewed': reviewed, 'source_partial': obj.get('source_partial', False),
                        'reviewed_unit_start': obj.get('unit_start', False), 'reviewed_unit_end': obj.get('unit_end', False),
                        'source_uncertainty': uncertainty, 'physical_maps': maps,
                        'source_region_coverage': spans,
                        'duplicate_group_id': group.get('group_id') if group else None,
                        'source_index': obj['source_index'], 'page': obj['page'], 'source_box_px': obj['box'],
                        'sequence_origin': list(source_order(obj)), 'ocr_assignment_status': 'pending',
                        'raw_ocr_available': bool(pages[(obj['source_index'],obj['page'])].get('ocr',{}).get('words')),
                        'source_segmentation_status': pages[(obj['source_index'], obj['page'])]['segmentation_status']})
    validate_logical_sequence(logical)
    redundant = sum(len({m['occurrence_id'] for m in g['region_maps']}) - 1 for g in groups)
    counts = {'physical_occurrences': len(objects), 'logical_regions': len(logical),
              'redundant_occurrences': redundant, 'body_lines': sum(l['classification'] == 'body' for l in logical),
              'annotations': sum(l['classification'] == 'annotation' for l in logical),
              'furniture': sum(l['classification'] == 'furniture' for l in logical),
              'source_partial': sum(l['source_partial'] for l in logical),
              'physical_source_partial': sum(o.get('source_partial', False) for o in objects),
              'unresolved_geometry': sum(not l['geometry_reviewed'] for l in logical),
              'unreviewed_owner': sum(not l['owner_reviewed'] for l in logical),
              'ocr_assignment_pending': len(logical),
              'raw_ocr_unavailable': sum(not l['raw_ocr_available'] for l in logical)}
    if len(logical) + redundant != len(objects):
        raise common.AuditError('Physical/logical duplicate accounting disagrees')
    return {'schema_version': 1, 'snapshot_id': inventory['snapshot_id'], 'runtime_id': inventory.get('runtime_id'),
            'lines': logical, 'counts': counts, 'logical_map_id': common.json_hash(logical),
            'index_contract': {'canonical_key': 'logical_line_id', 'physical_key': 'occurrence_id',
                               'tokenless_objects_remain_expected': True, 'physical_geometry': 'physical_maps'}}


def validate_logical_sequence(lines):
    seen, physical, positions = set(), {}, {}
    for line in lines:
        if line['logical_line_id'] in seen:
            raise common.AuditError('Repeated logical line identity')
        seen.add(line['logical_line_id'])
        owner = line['owner']
        if line['sequence_position'] != positions.get(owner, 0):
            raise common.AuditError('Contradictory logical order')
        positions[owner] = positions.get(owner, 0) + 1
        if line['geometry_reviewed'] and (not line['owner_reviewed'] or not owner or line['source_partial']):
            raise common.AuditError('Reviewed logical geometry requires one reviewed owner and a complete reference')
        for mapping in line['physical_maps']:
            previous = physical.setdefault(mapping['occurrence_id'], line['logical_line_id'])
            if previous != line['logical_line_id']:
                raise common.AuditError('Physical occurrence has contradictory logical membership')
        validate_pixel_box(line['required_box_px'], line['canonical_dimensions'])
    return lines


def write_logical_review_sheet(logical, run, args):
    import html
    import facsimile_line_audit as cli
    sections = ['<!doctype html><meta charset="utf-8"><title>Logical source sequence</title>',
                '<style>body{font:18px system-ui;max-width:1200px;margin:auto}code{word-break:break-all}img{max-width:100%}</style>',
                '<h1>Logical source sequence</h1><p>Required reference spans are independent of reader coverage. Unreviewed ownership stays explicit.</p>']
    for line in logical['lines']:
        sections.append('<article><h2>' + html.escape(line['owner'] or 'unresolved owner') +
                        f' / position {line["sequence_position"]}</h2><code>' + line['logical_line_id'] + '</code>' +
                        f'<p>Source {line["source_index"]+1}, physical page {line["page"]}; ' +
                        html.escape(line['classification']) + '; ' + html.escape(', '.join(line['source_uncertainty'])) +
                        '</p><img loading="lazy" src="' + html.escape(line['canonical_crop']['path'], quote=True) + '">' +
                        '<pre>' + html.escape(str(line['source_region_coverage'])) + '</pre></article>')
    repo, work = cli.paths(args)
    target = common.safe_write_path(Path(run) / 'logical-review.html', work, repo, policy=common.AuditOutputPolicy('audit'))
    target.write_text('\n'.join(sections), encoding='utf-8')


def box_intersection(first, second):
    return max(0, min(first[2], second[2]) - max(first[0], second[0])) * max(0, min(first[3], second[3]) - max(first[1], second[1]))


def source_inventory_to_tokens(inventory, logical_lines, ocr_observations=None):
    physical = {}
    ambiguous = []
    logical_lookup = {m['occurrence_id']: line for line in logical_lines['lines'] for m in line['physical_maps']}
    for page in inventory['pages']:
        words = (ocr_observations or {}).get((page['source_index'], page['page']), page.get('ocr', {})).get('words', [])
        objects = page['occurrences']
        for word in words:
            dimensions = page.get('ocr', {}).get('raster_dimensions') or objects[0]['raster_dimensions']
            box = [word['box'][i] * dimensions[i % 2] for i in range(4)]
            parents = [o for o in objects if box_intersection(box, o['box']) >= 0.5 * (box[2] - box[0]) * (box[3] - box[1])]
            child_parents = [o for o in parents if o.get('kind') == 'reviewed_child']
            if child_parents:
                ancestor_ids = {identifier for child in child_parents for identifier in child['parent_ids']}
                parents = [o for o in parents if o['occurrence_id'] not in ancestor_ids]
            if len(parents) != 1:
                ambiguous.append(dict(word, source_index=page['source_index'], page=page['page'], reason='ambiguous_inventory_assignment'))
                continue
            obj = parents[0]
            line = logical_lookup.get(obj['occurrence_id'])
            views = historical.comparison_views([word['text']])[0]
            token = dict(views, provider='tesseract_raster', occurrence_id=obj['occurrence_id'],
                         logical_line_id=line['logical_line_id'] if line else None,
                         source_index=page['source_index'], page=page['page'], source_box_px=box,
                         box=word['box'], physical_line_id=word['line_id'],
                         logical_position=line['sequence_position'] if line else None,
                         line=word['line_id'], confidence=word.get('conf'),
                         hyphen_join_hypothesis=word['text'].endswith(tuple(historical.HYPHENS)))
            physical.setdefault(obj['occurrence_id'], []).append(token)
    canonical, reverse, alternate_alignments = {}, {}, []
    for line in logical_lines['lines']:
        tokens = sorted(physical.get(line['canonical_occurrence_id'], []), key=lambda t: (tuple(t['physical_line_id']), t['source_box_px'][0]))
        canonical[line['logical_line_id']] = tokens
        for index, token in enumerate(tokens):
            reverse[f'{line["logical_line_id"]}:{index}'] = [dict(occurrence_id=token['occurrence_id'], source_box_px=token['source_box_px'])]
        for mapping in line['physical_maps']:
            other = physical.get(mapping['occurrence_id'], [])
            if mapping['occurrence_id'] == line['canonical_occurrence_id'] or not other or not tokens:
                continue
            refinement = historical.align_tokens(tokens, other, mode='local', settings=historical.detector_settings())
            # Geometric maps remain authoritative even when alternate OCR is uncertain.
            alternate_alignments.append({'occurrence_id': mapping['occurrence_id'],
                                          'logical_line_id': line['logical_line_id'],
                                          'ordered_similarity': refinement['ordered_similarity'],
                                          'inconclusive': refinement.get('inconclusive', False),
                                          'operations': refinement.get('operations', [])})
            if not refinement.get('inconclusive'):
                for operation in refinement.get('operations', []):
                    if not operation['covered']:
                        continue
                    for index in operation['source']:
                        for other_index in operation['output']:
                            token = other[other_index]
                            reverse[f'{line["logical_line_id"]}:{index}'].append({
                                'occurrence_id': token['occurrence_id'], 'source_box_px': token['source_box_px'],
                                'alignment_hypothesis': True})
    return {'schema_version': 1, 'physical_tokens': physical, 'canonical_tokens': canonical,
            'tokenless_objects': [{'occurrence_id': o['occurrence_id'], 'ocr_missing': True,
                                  'logical_line_id': logical_lookup[o['occurrence_id']]['logical_line_id'],
                                  'source_index': o['source_index'], 'page': o['page']}
                                 for p in inventory['pages'] for o in p['occurrences'] if not physical.get(o['occurrence_id'])],
            'ambiguous_words': ambiguous, 'reverse_lookup': reverse, 'alternate_alignments': alternate_alignments,
            'logical_map_id': logical_lines['logical_map_id']}


def source_bundle_to_index_unit(bundle, lines, unit_id, source_index):
    selected = [l for l in lines['lines'] if l['owner'] == unit_id and l['classification'] in {'body', 'uncertain'}]
    tokens = [t for line in selected for t in bundle['canonical_tokens'][line['logical_line_id']]]
    selected_ids = {line['logical_line_id'] for line in selected}
    return {'unit_id': unit_id, 'source_index': source_index, 'tokens': tokens,
            'strict': [t['strict'] for t in tokens], 'tolerant': [t['tolerant'] for t in tokens],
            'anchors': historical.anchor_information([t['strict'] for t in tokens]),
            'canonical_reverse_lookup': {key: value for key, value in bundle['reverse_lookup'].items()
                                        if key.split(':')[0] in selected_ids}}


def build_source_index(bundle, logical_lines, units, settings):
    index_units = [source_bundle_to_index_unit(bundle, logical_lines, u['id'], u['source']) for u in units]
    strict_docs, tolerant_docs = [], []
    size, stride = settings['source_index']['windows']['local']
    for unit in index_units:
        for start, end in historical.passage_windows(len(unit['tokens']), size, stride):
            if any(unit['strict'][start:end]):
                strict_docs.append(' '.join(unit['strict'][start:end]))
                tolerant_docs.append(' '.join(unit['tolerant'][start:end]))
    if not strict_docs or not any(len(doc.replace(' ', '')) >= 3 for doc in strict_docs):
        return {'schema_version': 1, 'status': 'no_source_anchors', 'units': index_units}
    fitted = historical.fit_source_index(strict_docs, tolerant_docs)
    strict_vocab, strict_idf = historical.vectorizer_vocab_idf(fitted['strict'][0])
    tolerant_vocab, tolerant_idf = historical.vectorizer_vocab_idf(fitted['tolerant'][0])
    return {'schema_version': 1, 'status': 'fitted_source_only', 'units': index_units,
            'strict_vocabulary': strict_vocab, 'strict_idf': strict_idf,
            'tolerant_vocabulary': tolerant_vocab, 'tolerant_idf': tolerant_idf}


def reader_observations_to_tokens(reader_map, ocr_observations):
    observations = []
    for row in reader_map['body_pages']:
        observation = ocr_observations.get((row['tomus'], row['page']), {})
        for word in observation.get('words', []):
            if box_intersection(word['box'], row['body_box']) <= 0:
                continue
            views = historical.comparison_views([word['text']])[0]
            dimensions = observation['raster_dimensions']
            observations.append(dict(views, unit_id=row['unit_id'], tomus=row['tomus'], page=row['page'],
                occurrence_id=f'{row["tomus"]}:{row["page"]}:{row["ordinal"]}',
                body_box=row['body_box'], box=word['box'],
                pixel_box=[word['box'][i] * dimensions[i % 2] for i in range(4)],
                line_id=word['line_id'], folio=row['folio'], folio_provenance=row['folio_provenance'],
                provider='tesseract_raster', hyphen_join_hypothesis=word['text'].endswith(tuple(historical.HYPHENS))))
    return sorted(observations, key=lambda t: (t['tomus'], t['page'], tuple(t['line_id']), t['pixel_box'][0]))


def locate_reader_neighborhoods(source_index, reader_tokens, settings):
    if source_index['status'] == 'no_source_anchors':
        return {'status': 'no_source_anchors', 'units': []}
    vectorizers = historical.load_source_index(source_index)
    located = []
    for unit in source_index['units']:
        reader = [t for t in reader_tokens if t['unit_id'] == unit['unit_id']]
        anchors = historical.find_reader_occurrences(reader, unit, vectorizers['strict'], settings['source_index'])
        path = historical.chain_matches(anchors, [], settings['source_index'])
        correspondences = []
        previous = -1
        restarts = []
        candidates, exact, refinements = {}, {}, []
        for anchor in path:
            margin = historical.context_placement_margin(anchor, anchors, [])
            if anchor['source_start'] < previous:
                restarts.append(anchor)
                continue
            previous = anchor['source_end']
            if margin < settings['registration']['placement_margin']:
                continue
            for index in range(anchor['length']):
                source_position, reader_position = anchor['source_start']+index, anchor['output_start']+index
                exact[source_position] = reader_position
                candidates[(source_position,reader_position)] = (anchor,margin)
            source_start, source_end = max(0,anchor['source_start']-16), min(len(unit['tokens']),anchor['source_end']+16)
            reader_start, reader_end = max(0,anchor['output_start']-16), min(len(reader),anchor['output_end']+16)
            refinement = historical.align_tokens(unit['tokens'][source_start:source_end], reader[reader_start:reader_end],
                                                  mode='local', settings=settings['source_index'])
            refinements.append({'source_interval':[source_start,source_end], 'reader_interval':[reader_start,reader_end],
                                'alignment':refinement})
            for operation in refinement.get('operations', []):
                if operation['covered']:
                    for si in operation['source']:
                        for ri in operation['output']:
                            candidates.setdefault((source_start+si,reader_start+ri),(anchor,margin))
        by_source = {}
        for si,ri in candidates:
            by_source.setdefault(si,set()).add(ri)
        ambiguities, last_reader = [], -1
        for si in sorted(by_source):
            choices = {exact[si]} if si in exact else by_source[si]
            if len(choices) != 1:
                ambiguities.append({'source_token_index':si,'reader_token_indices':sorted(choices),'reason':'ambiguous_local_refinement'})
                continue
            ri = next(iter(choices))
            if ri < last_reader:
                ambiguities.append({'source_token_index':si,'reader_token_indices':[ri],'reason':'nonmonotonic_local_refinement'})
                continue
            last_reader = ri
            anchor,margin = candidates[(si,ri)]
            correspondences.append({'source':unit['tokens'][si], 'reader':reader[ri], 'anchor':anchor,
                                    'placement_margin':margin, 'source_token_index':si,'reader_token_index':ri,
                                    'refinement_hypothesis':si not in exact})
        common.progress(f'Localization {unit["unit_id"]}: {len(anchors)} anchors, {len(correspondences)} ordered token pairs')
        located.append({'unit_id': unit['unit_id'], 'anchors': anchors, 'path': path,
                        'path_restarts': restarts, 'token_correspondences': correspondences,
                        'refinements':refinements,'refinement_ambiguities':ambiguities})
    return {'schema_version': 1, 'status': 'localized_observations', 'units': located}


def build_line_correspondences(logical_lines, locations, reader_map, settings):
    import bisect
    all_pairs = [pair for unit in locations.get('units', []) for pair in unit['token_correspondences']]
    direct = {}
    for pair in all_pairs:
        direct.setdefault(pair['source']['logical_line_id'], []).append(pair)
    lines = logical_lines['lines']
    by_owner = {}
    for line in lines:
        if direct.get(line['logical_line_id']):
            by_owner.setdefault(line['owner'], []).append(line)
    rows_by_owner = {}
    for row in reader_map['body_pages']:
        rows_by_owner.setdefault(row['unit_id'], []).append((row['tomus'],row['page']))
    positions_by_owner = {owner: [line['sequence_position'] for line in members] for owner,members in by_owner.items()}
    output = []
    for line in lines:
        own = direct.get(line['logical_line_id'], [])
        candidates = by_owner.get(line['owner'], [])
        positions = positions_by_owner.get(line['owner'], [])
        low = bisect.bisect_left(positions, line['sequence_position'])
        high = bisect.bisect_right(positions, line['sequence_position'])
        before = candidates[max(0,low-3):low]
        after = candidates[high:high+3]
        context = own + [p for l in before + after for p in direct[l['logical_line_id']]]
        reader_pages = sorted({(p['reader']['tomus'], p['reader']['page']) for p in context})
        unit_rows = rows_by_owner.get(line['owner'], [])
        if len(reader_pages) > 2 or (len(reader_pages) == 2 and
                (reader_pages[0][0] != reader_pages[1][0] or unit_rows.index(reader_pages[1]) - unit_rows.index(reader_pages[0]) != 1)):
            reader_pages = []
            reason = 'neighborhood_exceeds_two_adjacent_reader_pages'
        else:
            reason = None if context else 'no_ordered_anchors'
        if not own and not (before and after) and not line.get('reviewed_unit_start') and not line.get('reviewed_unit_end'):
            reason = 'interior_requires_two_sided_ordered_anchors'
            reader_pages = []
        if context:
            ordered_pairs = sorted(context, key=lambda p: (p['source']['logical_position'], p['source']['source_box_px'][0]))
            reader_positions = [(p['reader']['tomus'],p['reader']['page'],tuple(p['reader']['line_id']),p['reader']['pixel_box'][0]) for p in ordered_pairs]
            if reader_positions != sorted(reader_positions):
                reason = 'nonmonotonic_local_reader_path'
                reader_pages = []
        endpoint = ('opening' if line.get('reviewed_unit_start') else
                    'ending' if line.get('reviewed_unit_end') else 'interior')
        if endpoint != 'interior' and unit_rows:
            expected_edge = unit_rows[0] if endpoint == 'opening' else unit_rows[-1]
            if expected_edge not in reader_pages:
                reason = 'unit_endpoint_reader_edge_unbounded'
                reader_pages = []
        output.append({'logical_line_id': line['logical_line_id'], 'direct_tokens': own,
                       'before_lines': [l['logical_line_id'] for l in before], 'after_lines': [l['logical_line_id'] for l in after],
                       'context_tokens': context, 'reader_pages': [list(p) for p in reader_pages],
                       'context_kind': endpoint, 'localization_reason': reason,
                       'localization_status': 'bounded' if reader_pages else 'unresolved'})
    return {'schema_version': 1, 'lines': output}


def register_local_strips(correspondences, logical_lines, root, manifest, settings, run, args, prerequisites):
    """Fit each consistent local anchor strip independently in original pixels."""
    import cv2
    import numpy as np
    from collections import OrderedDict
    from PIL import Image
    import facsimile_line_audit as cli
    repo, work = cli.paths(args)
    lines = {line['logical_line_id']: line for line in logical_lines['lines']}
    images = OrderedDict()
    fit_cache = OrderedDict()

    def rendered(role, number, selector):
        key = (role, number, tuple(sorted(selector.items())))
        if key not in images:
            pdf, record = cli.snapshot_input(root, manifest, role, **selector)
            dpi = settings['source_dpi'] if role == 'source_pdf' else settings['reader_dpi']
            raster, metadata = common.render_page(pdf, record, number, dpi, work, repo,
                pdftoppm=prerequisites['tools']['pdftoppm']['path'], policy=common.AuditOutputPolicy('audit'))
            with Image.open(raster) as image:
                gray = np.asarray(image.convert('L'))
            images[key] = (gray, metadata, common.ink_line_height(gray))
            if len(images) > 4:
                images.popitem(last=False)
        images.move_to_end(key)
        return images[key]

    completed = []
    for count, correspondence in enumerate(correspondences['lines']):
        line = lines[correspondence['logical_line_id']]
        fits = []
        if correspondence['localization_status'] == 'bounded':
            grouped = {}
            for pair in correspondence['context_tokens']:
                source, reader = pair['source'], pair['reader']
                if source['source_index'] != line['source_index'] or source['page'] != line['page']:
                    continue
                key = (reader['tomus'], reader['page'], source['logical_line_id'], tuple(reader['line_id']))
                grouped.setdefault(key, []).append(pair)
            anchor_lines = []
            for key, pairs in grouped.items():
                anchor_lines.append({'key': key, 'pairs': pairs,
                    'source_box': union_boxes([p['source']['source_box_px'] for p in pairs]),
                    'reader_box': union_boxes([p['reader']['pixel_box'] for p in pairs]),
                    'word_scale':float(np.median([(p['reader']['pixel_box'][3]-p['reader']['pixel_box'][1]) /
                        max(1,p['source']['source_box_px'][3]-p['source']['source_box_px'][1]) for p in pairs]))})
            anchor_lines.sort(key=lambda a: (a['key'][0],a['key'][1],a['source_box'][1],a['source_box'][0]))
            strips = []
            for anchor in anchor_lines:
                if strips:
                    previous = strips[-1][-1]
                    scale = (anchor['reader_box'][3]-anchor['reader_box'][1])/max(1,anchor['source_box'][3]-anchor['source_box'][1])
                    source_gap = anchor['source_box'][1]-previous['source_box'][3]
                    reader_gap = anchor['reader_box'][1]-previous['reader_box'][3]
                    tolerance = max(1,anchor['reader_box'][3]-anchor['reader_box'][1]) * settings['registration']['gap_tolerance_glyphs']
                    same = (anchor['key'][:2] == previous['key'][:2] and abs(reader_gap-scale*source_gap) <= tolerance and
                            abs(anchor['word_scale']/previous['word_scale']-1) <= settings['registration']['scale_tolerance'])
                else:
                    same = False
                if same:
                    strips[-1].append(anchor)
                else:
                    strips.append([anchor])
            for strip in strips:
                tomus, page = strip[0]['key'][:2]
                source, source_meta, source_height = rendered('source_pdf', line['page'], {'source_index': line['source_index']})
                reader, reader_meta, reader_height = rendered('reader_pdf', page, {'tomus': tomus})
                if source_height is None or reader_height is None:
                    fits.append({'matched': False, 'validity_reasons': ['glyph_scale_unresolved'], 'reader_page': page, 'tomus': tomus})
                    continue
                scale = reader_height/source_height
                pairs = [p for anchor in strip for p in anchor['pairs']]
                word_scale = float(np.median([(p['reader']['pixel_box'][3]-p['reader']['pixel_box'][1]) /
                                             max(1,p['source']['source_box_px'][3]-p['source']['source_box_px'][1]) for p in pairs]))
                raw_source = union_boxes([a['source_box'] for a in strip])
                raw_reader = union_boxes([a['reader_box'] for a in strip])

                def padded(box, gray, padding):
                    return [max(0,math.floor(box[0]-padding)),max(0,math.floor(box[1]-padding)),
                            min(gray.shape[1],math.ceil(box[2]+padding)),min(gray.shape[0],math.ceil(box[3]+padding))]

                source_box = padded(raw_source, source, source_height * settings['registration']['source_padding_glyphs'])
                reader_box = padded(raw_reader, reader, reader_height * settings['registration']['reader_padding_glyphs'])
                measured = dict(settings['registration'], reader_glyph_height=reader_height,
                                reprojection_px=settings['registration']['residual_glyphs']*reader_height,
                                expected_translation=np.median([np.asarray([(p['reader']['pixel_box'][0]+p['reader']['pixel_box'][2])/2,
                                     (p['reader']['pixel_box'][1]+p['reader']['pixel_box'][3])/2]) -
                                     scale*np.asarray([(p['source']['source_box_px'][0]+p['source']['source_box_px'][2])/2,
                                     (p['source']['source_box_px'][1]+p['source']['source_box_px'][3])/2]) for p in pairs],axis=0).tolist())
                metadata = {'runtime_id': logical_lines['runtime_id'], 'caller': 'local_anchor_strip',
                            'source_render': source_meta, 'reader_render': reader_meta,
                            'source_box_px': source_box, 'reader_box_px': reader_box, 'settings': measured, 'initial_scale': scale,
                            'anchor_line_ids':sorted({p['source']['logical_line_id'] for p in pairs})}
                registration_id = common.json_hash(metadata)
                cache = work / 'cache/line_registration' / (registration_id+'.json')
                fit = fit_cache.get(registration_id)
                if fit is None:
                    fit = common.reusable_entry(cache, metadata, work, repo, policy=common.AuditOutputPolicy('audit'))
                if fit is None:
                    fit = common.register_pixel_regions(source, reader, source_box, reader_box, measured, scale)
                    if abs(word_scale/scale-1) > settings['registration']['scale_tolerance']:
                        fit['matched'] = False
                        fit['validity_reasons'].append('word_box_and_glyph_scales_disagree')
                    fit.update(source_index=line['source_index'], source_page=line['page'], reader_page=page, tomus=tomus,
                               source_dimensions=[source.shape[1],source.shape[0]], reader_dimensions=[reader.shape[1],reader.shape[0]],
                               source_box_px=source_box, reader_box_px=reader_box, source_glyph_height=source_height,
                               reader_glyph_height=reader_height, word_box_scale=word_scale,
                               anchor_line_ids=sorted({p['source']['logical_line_id'] for p in pairs}),
                               evidence_type='direct_anchor_strip' if correspondence['direct_tokens'] else 'inherited_bounded_gap',
                               registration_id=common.json_hash(metadata))
                    common.write_json(cache, {'schema_version':1,'complete':True,'metadata':metadata,'data':fit}, work, repo,
                                      policy=common.AuditOutputPolicy('audit'))
                fit_cache[registration_id] = fit
                fit_cache.move_to_end(registration_id)
                if len(fit_cache) > 1024:
                    fit_cache.popitem(last=False)
                source_range = fit['source_box_px']
                within = source_range[1] <= line['source_box_px'][1] and line['source_box_px'][3] <= source_range[3]
                distance = max(0,source_range[1]-line['source_box_px'][3],line['source_box_px'][1]-source_range[3])
                endpoint_near = correspondence['context_kind'] != 'interior' and distance <= 3*source_height
                fits.append(dict(fit, projection_valid_for_line=within or endpoint_near,
                                 evidence_type='direct_anchor_strip' if line['logical_line_id'] in fit['anchor_line_ids'] else
                                 'inherited_bounded_gap' if within or endpoint_near else 'changed_gap_context_only'))
        completed.append(dict(correspondence, registrations=fits,
                              image_status='registered' if any(f.get('matched') for f in fits) else 'unresolved'))
        if count % 100 == 0:
            common.progress(f'Local strip observations: {count+1}/{len(correspondences["lines"])}')
    return dict(correspondences, snapshot_id=logical_lines['snapshot_id'], runtime_id=logical_lines['runtime_id'], lines=completed)


def compact_correspondences(record):
    """Store shared token contexts and image fits once instead of per region."""
    pairs, registrations, lines = {}, {}, []
    for line in record['lines']:
        item = {key:value for key,value in line.items() if key not in ('direct_tokens','context_tokens','registrations')}
        for field in ('direct_tokens','context_tokens'):
            identifiers = []
            for pair in line[field]:
                identifier = common.json_hash(pair)
                pairs.setdefault(identifier,pair)
                identifiers.append(identifier)
            item[field+'_ids'] = identifiers
        refs = []
        for fit in line['registrations']:
            base = {key:value for key,value in fit.items() if key not in ('projection_valid_for_line','evidence_type')}
            identifier = fit.get('registration_id') or common.json_hash(base)
            previous = registrations.setdefault(identifier,base)
            if previous != base:
                raise common.AuditError('Conflicting shared registration identities')
            refs.append({'registration_id':identifier,'projection_valid_for_line':fit.get('projection_valid_for_line',False),
                         'evidence_type':fit.get('evidence_type','unresolved')})
        item['registration_refs'] = refs
        lines.append(item)
    return dict(record, storage_profile='shared_context_v1',lines=lines,token_pairs=pairs,registrations=registrations)


def expand_correspondence_line(record, line):
    if record.get('storage_profile') != 'shared_context_v1':
        return line
    return dict(line,direct_tokens=[record['token_pairs'][identifier] for identifier in line['direct_tokens_ids']],
                context_tokens=[record['token_pairs'][identifier] for identifier in line['context_tokens_ids']],
                registrations=[dict(record['registrations'][ref['registration_id']],**ref) for ref in line['registration_refs']])
CASE_IDS = ('retained_control', 'head_and_foot_loss', 'interior_block_loss',
            'redundant_copy_removed', 'sole_complete_copy_removed',
            'shared_page_unique_tail', 'visible_pdf_clipping')
FIXTURE_WORDS = '''amber baker cedar delta ember fable garden harbor island jacket kettle ladder
magnet nectar olive pencil quartz ribbon silver timber velvet willow yellow zephyr
anchor beacon castle dragon engine forest gravel hammer insect jungle kernel lantern
marble nickel ocean planet rabbit saddle temple violet walnut zodiac copper bridge
almond basket cherry dancer eagle feather garlic honey ivory jasper kitten lemon
meadow nutmeg orange pepper quiver rocket sailor turtle vapor winter xenon zebra
acorn barrel candle daisy elbow falcon golden handle iron jumper kayak lizard
mirror noodle oyster pocket quiet rainbow salmon thunder urban village wagon yogurt
artist bottle cabin desert elm fiddle ginger helmet iris jewel keeper lily
market nature orbit pebble queen river stone tower union valley wheat young
apricot bamboo cobalt drift earth flower globe health index journey knot lotus
mint north orchid palm quest radius shell tide upper vine world youth
arrow birch cloud dream east flame goose hill idea judge king lake
moss night owl pearl quail rose star tree uncle voice wind yoke
atlas brook crane dune ebony field glass hazel inlet jet kite leaf
maple nest opal pine quarry reed shore trail unity vista wave yawn
arch branch coral dawn echo fern grain hearth ink jade keel linen
mango navy opera pond quilt ridge seed torch usher verse wood yard'''.split()


def create_verification_inputs(args):
    import copy
    import io
    import json
    import numpy as np
    import pymupdf
    from PIL import Image, ImageDraw, ImageFont
    import facsimile_line_audit as cli
    repo, work = cli.paths(args)
    root = Path(args.out).resolve()
    verification_root = (work / 'runs/verification').resolve()
    if root.exists() or root == verification_root or not common.contained(root, verification_root):
        raise common.AuditError('Fixture root must be a fresh owned directory beneath runs/verification')
    allowlist = frozenset(f'{case}/snapshot/inputs/{filename}' for case in CASE_IDS
                         for filename in ('source.pdf', 'reader.pdf'))
    permission = common.AuditOutputPolicy('fixture', root, allowlist)
    common.safe_write_path(root / 'builder-manifest.json', work, repo)
    root.mkdir(parents=True)
    base_settings = cli.read_json(args.settings or repo / 'tools/facsimile-line-settings.json')
    source_size, reader_size = (4000, 2200), (1940, 2200)
    right_text, crop_left, crop_right, reader_left = 2080, 2060, 3940, 30
    source_top, reader_top, line_pitch, row_height = 120, 70, 104, 80
    font = ImageFont.load_default(size=44)
    text_rows = [' '.join(FIXTURE_WORDS[i * 12:(i + 1) * 12]) + '.' for i in range(16)]
    while max(font.getlength(text) for text in text_rows) > crop_right - right_text:
        font = ImageFont.load_default(size=font.size - 1)
    if font.size < 40:
        raise common.AuditError('Fixture text needs a readable scalable font')
    expected_cases = []
    for case in CASE_IDS:
        folder = root / case
        snapshot = folder / 'snapshot'
        inputs = snapshot / 'inputs'
        inputs.mkdir(parents=True)
        page_count = 2 if case in {'head_and_foot_loss', 'retained_control'} else 1
        sources, line_regions = [], []
        for page in range(1, page_count + 1):
            image = Image.new('L', source_size, 255)
            draw = ImageDraw.Draw(image)
            for index, text in enumerate(text_rows if not (case == 'retained_control' and page == 2) else ['Quiet cedar verse.']):
                text = ' '.join(text_rows[index].split()[:6] + text_rows[(index + 7) % 16].split()[6:]) if page == 2 and case != 'retained_control' else text
                if case == 'retained_control' and index == 8:
                    text = 'Quiet cedar verse.'
                y = source_top + index * line_pitch
                draw.text((120, y), text, font=font, fill=0)
                draw.text((right_text + (50 if case == 'retained_control' and index == 8 else 0), y), text, font=font, fill=0)
                a = np.asarray(image)
                yy, xx = np.where(a[y:y + row_height, crop_left:crop_right] < 255)
                box = [int(xx.min() + crop_left), int(yy.min() + y), int(xx.max() + crop_left + 1), int(yy.max() + y + 1)]
                line_regions.append({'source_page': page, 'line': index, 'source_box_px': box,
                                     'owner': 'sparse_fixture' if case == 'retained_control' and page == 2 else 'fixture',
                                     'text_label': f'synthetic_row_{page}_{index}',
                                     'unit_start': index == 0 and (page == 1 or case == 'retained_control'),
                                     'unit_end': (index == 15 and page == (2 if case == 'head_and_foot_loss' else 1)) or
                                                 (case == 'retained_control' and page == 2 and index == 0)})
            sources.append(image)
        if case in {'redundant_copy_removed', 'sole_complete_copy_removed', 'shared_page_unique_tail'}:
            duplicate = sources[0].copy()
            if case == 'shared_page_unique_tail':
                duplicate.paste(255, (crop_left, source_top + 12 * line_pitch, crop_right, source_size[1]))
            else:
                duplicate.paste(255, (crop_left, source_top, right_text + 80, source_size[1]))
            sources.append(duplicate)
            for row in list(line_regions):
                if case == 'shared_page_unique_tail' and row['line'] >= 12:
                    continue
                sy = source_top + row['line'] * line_pitch
                a = np.asarray(duplicate)
                yy, xx = np.where(a[sy:sy + row_height, crop_left:crop_right] < 255)
                box = [int(xx.min() + crop_left), int(yy.min() + sy), int(xx.max() + crop_left + 1), int(yy.max() + sy + 1)]
                line_regions.append(dict(row, source_page=2, source_box_px=box,
                                         source_partial=case != 'shared_page_unique_tail'))
        source_doc = pymupdf.open()
        for image in sources:
            data = io.BytesIO()
            image.save(data, format='PNG')
            page = source_doc.new_page(width=image.width * 72 / 300, height=image.height * 72 / 300)
            page.insert_image(page.rect, stream=data.getvalue())
        source_path = common.safe_write_path(inputs / 'source.pdf', work, repo, policy=permission)
        source_doc.save(source_path, deflate=True)
        source_doc.close()
        omissions = []
        if case == 'head_and_foot_loss':
            omissions = [(page, index) for page in (1, 2) for index in (0, 15)]
        elif case == 'interior_block_loss':
            omissions = [(1, index) for index in range(5, 9)]
        elif case == 'shared_page_unique_tail':
            omissions = [(1, index) for index in range(12, 16)]
        reader_sources = list(enumerate(sources[:page_count], 1))
        if case == 'sole_complete_copy_removed':
            reader_sources = [(2, sources[1])]
        reader_doc = pymupdf.open()
        placements, assets = [], []
        painted_regions = []
        for page_number, (source_page, source_image) in enumerate(reader_sources, 1):
            canvas = Image.new('L', reader_size, 255)
            page_rows = [r for r in line_regions if r['source_page'] == source_page]
            for row in page_rows:
                index = row['line']
                if (source_page, index) in omissions:
                    continue
                sy = source_top + index * line_pitch
                y = reader_top + (index - sum(p == source_page and i < index for p, i in omissions)) * line_pitch
                canvas.paste(source_image.crop((crop_left, sy, crop_right, sy + row_height)), (reader_left, y))
                box = row['source_box_px']
                placements.append({'source_page': source_page, 'source_line': index, 'reader_page': page_number,
                                   'reader_box_px': [box[0] - crop_left + reader_left, box[1] - sy + y,
                                                     box[2] - crop_left + reader_left, box[3] - sy + y],
                                   'builder_transform': [[1, 0, reader_left-crop_left], [0, 1, y - sy]]})
            owner = 'sparse_fixture' if case == 'retained_control' and page_number == 2 else 'fixture'
            ordinal = 1 if owner == 'sparse_fixture' else page_number
            asset_name = f'facsimile_{owner}_{ordinal}.png'
            canvas.save(inputs / asset_name)
            assets.append(asset_name)
            pdf_page = reader_doc.new_page(width=canvas.width * 72 / 300, height=canvas.height * 72 / 300)
            pdf_page.insert_image(pdf_page.rect, filename=str(inputs / asset_name))
            if case == 'visible_pdf_clipping':
                hidden = next(p for p in placements if p['source_line'] == 5)
                box = hidden['reader_box_px']
                pdf_page.draw_rect(pymupdf.Rect(box[0] * 72 / 300, box[1] * 72 / 300,
                                               box[2] * 72 / 300, box[3] * 72 / 300), color=None, fill=(1, 1, 1), overlay=True)
                painted_regions.append({'reader_page':page_number,'source_page':source_page,'source_line':5,
                                        'reader_box_px':box,'kind':'whole_line_overpaint'})
                terminal = next(p for p in placements if p['source_line'] == 10)
                bx = terminal['reader_box_px']
                pdf_page.draw_rect(pymupdf.Rect((bx[2] - 25) * 72 / 300, bx[1] * 72 / 300,
                                               bx[2] * 72 / 300, bx[3] * 72 / 300), color=None, fill=(1, 1, 1), overlay=True)
                painted_regions.append({'reader_page':page_number,'source_page':source_page,'source_line':10,
                                        'reader_box_px':[bx[2]-25,bx[1],bx[2],bx[3]],'kind':'terminal_glyph_overpaint'})
                omissions.append((1, 5))
        reader_path = common.safe_write_path(inputs / 'reader.pdf', work, repo, policy=permission)
        reader_doc.save(reader_path, deflate=True)
        reader_doc.close()
        tex = inputs / 'reader.tex'
        tex.write_text('\\begin{document}\n' + '\n'.join(f'\\includegraphics{{{asset}}}' for asset in assets) + '\n\\end{document}\n', encoding='utf-8')
        source_digest = common.hash_file(source_path)
        policy = {'schema_version': 1, 'version': 1, 'source_sha256': {'0': source_digest},
                  'decisions': [], 'duplicate_groups': [], 'duplicate_seeds': [],
                  'synthetic_regions': [dict(r, source_index=0, source_sha256=source_digest,
                                              classification='body', provenance='synthetic_ground_truth') for r in line_regions]}
        policy['synthetic_duplicate_regions'] = []
        if case in {'redundant_copy_removed', 'sole_complete_copy_removed', 'shared_page_unique_tail'}:
            for reference in [r for r in line_regions if r['source_page'] == 1]:
                others = [r for r in line_regions if r['source_page'] == 2 and r['line'] == reference['line']]
                if others:
                    policy['synthetic_duplicate_regions'].append({'canonical': reference, 'other': others[0],
                        'source_sha256': source_digest, 'provenance': 'synthetic_ground_truth',
                        'reason': 'Builder copied these source pixels before applying the recorded source-side clip'})
        settings = copy.deepcopy(base_settings)
        table = [{'id': 'fixture', 'source': 0, 'start_page': 1, 'group': 1, 'kind': 'life'}]
        if case == 'retained_control':
            table.append({'id': 'sparse_fixture', 'source': 0, 'start_page': 2, 'group': 1, 'kind': 'life'})
        for name, value in (('settings.json', settings), ('policy.json', policy), ('units.json', table)):
            cli.write(inputs / name, value, args)
        records = []
        roles = {'source.pdf': 'source_pdf', 'reader.pdf': 'reader_pdf', 'reader.tex': 'reader_tex',
                 'settings.json': 'settings', 'policy.json': 'source_policy', 'units.json': 'unit_table'}
        for name, role in roles.items():
            metadata = {'source_index': 0, 'page_count': len(sources)} if role == 'source_pdf' else {'tomus': 1} if role in {'reader_pdf', 'reader_tex'} else {}
            records.append(dict(path=name, snapshot_path=f'inputs/{name}', role=role,
                                sha256=common.hash_file(inputs / name), **metadata))
        for name in assets:
            records.append(dict(path=name, snapshot_path=f'inputs/{name}', role='reader_asset',
                                sha256=common.hash_file(inputs / name),
                                decoded_identity=common.decoded_image_identity(pymupdf.Pixmap(str(inputs / name)))))
        runtime, prerequisites = cli.compute_runtime_identity(args, settings=settings, policy=policy)
        for relative, digest in runtime['code'].items():
            destination = inputs / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes((repo / relative).read_bytes())
            records.append(dict(path=relative, snapshot_path=f'inputs/{relative}', role='code', sha256=digest))
        reader_map = common.map_reader_occurrences([dict(tomus=1, pdf_path=reader_path, tex_path=tex,
            pdf_sha256=common.hash_file(reader_path), tex_sha256=common.hash_file(tex),
            body_assets={name: common.decoded_image_identity(pymupdf.Pixmap(str(inputs / name))) for name in assets})])
        manifest = {'schema_version': 1, 'target_kind': 'verification', 'capture_complete': True,
                    'inputs': sorted(records, key=lambda r: r['path']), 'runtime': runtime,
                    'observed_counts': {'sources': 1, 'readers': 1, 'units': len(table), 'body_pages': len(assets), 'physical_pages': len(assets)},
                    'reviewed_source_endpoints': {'0': len(sources)}, 'reader_map': reader_map,
                    'case_id': case}
        manifest['snapshot_id'] = common.json_hash(manifest)
        cli.write(snapshot / 'snapshot-manifest.json', manifest, args)
        expectation = {'schema_version': 1, 'case_id': case, 'snapshot_id': manifest['snapshot_id'],
                       'source_regions': line_regions, 'reader_placements': placements,
                       'painted_reader_regions':painted_regions,
                       'source_raster_dimensions':list(source_size),'reader_raster_dimensions':list(reader_size),
                       'omitted_source_lines': [{'source_page': p, 'source_line': i} for p, i in omissions],
                       'partial_source_lines': [{'source_page': 1, 'source_line': 10}] if case == 'visible_pdf_clipping' else
                                               [{'source_page': 1, 'source_line': i} for i in range(16)] if case == 'sole_complete_copy_removed' else [],
                       'expected_no_unique_loss': case in {'retained_control', 'redundant_copy_removed'},
                       'builder_geometry_is_independent': True}
        cli.write(folder / 'case-expectations.json', expectation, args)
        expected_cases.append({'case_id': case, 'snapshot': f'{case}/snapshot',
                               'expectations_sha256': common.hash_file(folder / 'case-expectations.json')})
    cli.write(root / 'builder-manifest.json', {'schema_version': 1, 'target_kind': 'verification',
                                              'cases': expected_cases, 'font_size': font.size}, args)
    return root
