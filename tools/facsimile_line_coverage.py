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
        objects.append(dict(line, kind='line', classification='body' if columns['status'] == 'measured' else 'uncertain',
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


def inventory_source_units(root, manifest, all_units, selected, settings, run, args, prerequisites):
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
    pages, errors = [], []
    for source_index, page in sorted(requested):
        record_path, record = cli.snapshot_input(root, manifest, 'source_pdf', source_index=source_index)
        owners = [r['unit_id'] for r in ranges if r['source_index'] == source_index and r['start'] <= page <= r['end']]
        try:
            metadata = {'snapshot_id': manifest['snapshot_id'], 'code': cli.executed_code(repo),
                        'source_sha256': record['sha256'], 'page': page, 'settings': common.json_hash(settings), 'owners': owners}
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
                item = inventory_source_page(image, record, page, owners, settings, run, args)
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
