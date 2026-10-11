"""Source-backed glyph-cut observations at visible reader-page joins."""
import math
import re
from collections import Counter
from pathlib import Path

import audit_inner_tomi as historical
import facsimile_line_audit as cli
import facsimile_line_coverage as coverage
import inner_audit_common as common


def page_tokens(observation, box):
    result = []
    for word in observation.get('words', []):
        pixels = [word['box'][i] * observation['raster_dimensions'][i % 2] for i in range(4)]
        if box[0] <= (pixels[0]+pixels[2])/2 <= box[2] and box[1] <= (pixels[1]+pixels[3])/2 <= box[3]:
            result.append((historical.comparison_views([word['text']])[0]['strict'], pixels))
    return result


def register_edge(source, reader, source_ocr, reader_ocr, source_box, reader_box, settings):
    """Fit intact neighboring words with the shared strict registration primitive."""
    import numpy as np
    source_tokens, reader_tokens = page_tokens(source_ocr, source_box), page_tokens(reader_ocr, reader_box)
    source_counts, reader_counts = Counter(t for t, _ in source_tokens), Counter(t for t, _ in reader_tokens)
    unique_reader = {token: box for token, box in reader_tokens if reader_counts[token] == 1}
    pairs = [(box, unique_reader[token]) for token, box in source_tokens
             if len(token) >= 4 and source_counts[token] == 1 and token in unique_reader]
    sh = coverage.anchor_glyph_height(source, [s for s, _ in pairs])
    rh = coverage.anchor_glyph_height(reader, [r for _, r in pairs])
    if len(pairs) < 2 or sh is None or rh is None:
        return {'matched': False, 'validity_reasons': ['paired_anchor_scale_unresolved']}
    scale = rh/sh
    translation = np.median([np.asarray([(r[0]+r[2])/2, (r[1]+r[3])/2]) -
        scale*np.asarray([(s[0]+s[2])/2, (s[1]+s[3])/2]) for s, r in pairs], axis=0).tolist()
    measured = dict(settings['registration'], reader_glyph_height=rh,
        reprojection_px=settings['registration']['residual_glyphs']*rh, expected_translation=translation)
    fit = common.register_pixel_regions(source, reader, source_box, reader_box, measured, scale)
    word_scale = float(np.median([(r[3]-r[1])/max(1, s[3]-s[1]) for s, r in pairs]))
    if abs(word_scale/scale-1) > settings['registration']['scale_tolerance']:
        fit['matched'] = False
        fit['validity_reasons'].append('word_box_and_glyph_scales_disagree')
    fit.update(source_anchor_box_px=source_box, reader_anchor_box_px=reader_box,
               source_glyph_height=sh, reader_glyph_height=rh, word_box_scale=word_scale,
               anchor_pairs=[{'source_box_px': s, 'reader_box_px': r} for s, r in pairs])
    return fit


def measure_glyph_cut(source_gray, pieces, settings):
    """Require supported upper/lower portions of the same source glyph on two pages."""
    import cv2
    import numpy as np
    if len(pieces) != 2 or not all(p['registration']['matched'] for p in pieces):
        return {'detected': False, 'reason': 'two_valid_edge_registrations_required', 'observations': []}
    first, second = pieces
    if first['tomus'] != second['tomus'] or second['reader_page'] != first['reader_page']+1:
        return {'detected': False, 'reason': 'adjacent_reader_pages_required', 'observations': []}
    minimum = settings['ink']['minimum_deficit_pixels']
    required = settings['source_index']['thresholds']['clipping']['minimum_slice_components']
    observations = []
    for index, threshold in enumerate(settings['thresholds']):
        count, labels, stats, _ = cv2.connectedComponentsWithStats((source_gray < threshold).astype(np.uint8),
            connectivity=settings['connectivity'])
        crossing = []
        for identifier in range(1, count):
            yy, xx = np.where(labels == identifier)
            supported, cut_sides, details = [], [], []
            for number, piece in enumerate(pieces):
                matrix = np.asarray(piece['matrix'])
                projected = np.column_stack((xx, yy, np.ones(len(xx)))) @ matrix.T
                body = piece['visible_box_px']
                edge = body[3] if number == 0 else body[1]
                sides = projected[:, 1] < edge
                area_scale = abs(float(np.linalg.det(matrix[:, :2])))
                inside_x = (projected[:, 0] >= body[0]) & (projected[:, 0] < body[2])
                upper, lower = float((sides & inside_x).sum()*area_scale), float((~sides & inside_x).sum()*area_scale)
                support = piece['measurement']['canonical_support_masks'][index][yy, xx]
                support_area = float(support.sum()*area_scale)
                cut_sides.append(min(upper, lower) >= minimum)
                supported.append(support_area >= minimum)
                details.append({'reader_page': piece['reader_page'], 'upper_area': upper, 'lower_area': lower,
                                'supported_area': support_area, 'edge_y_px': edge})
            if all(cut_sides) and all(supported):
                # Canonical support must run from upper pixels on the first page to lower pixels on the second.
                a = first['measurement']['canonical_support_masks'][index][yy, xx]
                b = second['measurement']['canonical_support_masks'][index][yy, xx]
                if float(np.median(yy[a])) >= float(np.median(yy[b])):
                    continue
                crossing.append({'source_box_px': [int(stats[identifier, 0]), int(stats[identifier, 1]),
                    int(stats[identifier, 0]+stats[identifier, 2]), int(stats[identifier, 1]+stats[identifier, 3])],
                    'pieces': details})
        observations.append({'threshold': threshold, 'crossing_components': crossing})
    consistent = [component for component in observations[0]['crossing_components'] if
        all(any(coverage.box_intersection(component['source_box_px'], other['source_box_px']) > 0
                for other in observation['crossing_components']) for observation in observations[1:])]
    detected = len(consistent) >= required
    return {'detected': detected, 'reason': 'glyph_cut_across_reader_pages' if detected else 'no_persistent_supported_glyph_cut',
            'consistent_component_count': len(consistent), 'minimum_crossing_components': required,
            'observations': observations, 'ink_completeness_is_separate': True}


def observe_control(control, root, manifest, settings, args, prerequisites, evidence_dir):
    """Measure fixed source geometry against independently registered visible page edges."""
    import numpy as np
    from PIL import Image
    repo, work = cli.paths(args)
    source_pdf, source_record = cli.snapshot_input(root, manifest, 'source_pdf', source_index=control['source_index'])
    reader_pdf, reader_record = cli.snapshot_input(root, manifest, 'reader_pdf', tomus=control['tomus'])
    for name, record in (('source', source_record), ('reader', reader_record)):
        if record['path'] != control[name+'_pdf'] or record['sha256'] != control[name+'_sha256']:
            raise common.AuditError('Acceptance control input identity mismatch: '+control['control_id'])
    if control['source_dpi'] != settings['source_dpi']:
        raise common.AuditError('Acceptance control source DPI differs from capture')
    pages = control['reader_pages']
    if len(pages) != 2 or pages[1] != pages[0]+1 or control['reader_edge_regions'] != ['bottom', 'top']:
        raise common.AuditError('Acceptance control requires bottom/top of adjacent pages')
    raster, source_meta = common.render_page(source_pdf, source_record, control['source_page'], control['source_dpi'],
        work, repo, pdftoppm=prerequisites['tools']['pdftoppm']['path'], policy=common.AuditOutputPolicy('audit'))
    with Image.open(raster) as image:
        if list(image.size) != control['source_raster_dimensions']:
            raise common.AuditError('Acceptance source raster dimensions differ')
        source = np.asarray(image.convert('L')).copy()
    target = control['source_box_px']
    coverage.validate_pixel_box(target, control['source_raster_dimensions'])
    height = coverage.anchor_glyph_height(source, [target])
    if height is None:
        return {'control_id': control['control_id'], 'detected': False, 'reason': 'source_glyph_scale_unresolved'}

    def ocr(pdf, record, page, dpi, psm, box):
        return historical.ocr_page_region(pdf, record, page, dpi, psm, settings['ocr']['language'], work, repo,
            box=box, tesseract=prerequisites['tools']['tesseract']['path'],
            tessdata_dir=prerequisites['tools']['tesseract']['tessdata_dir'],
            pdftoppm=prerequisites['tools']['pdftoppm']['path'], policy=common.AuditOutputPolicy('audit'))

    def crop(array, label):
        path = common.safe_write_path(evidence_dir/(control['control_id']+'_'+label+'.png'), work, repo,
                                     policy=common.AuditOutputPolicy('audit'))
        path.parent.mkdir(parents=True, exist_ok=True)
        Image.fromarray(array).save(path)
        return {'path': path.relative_to(evidence_dir.parent).as_posix(), 'sha256': common.hash_file(path)}

    source_ocr = ocr(source_pdf, source_record, control['source_page'], settings['source_dpi'],
                     settings['ocr']['source_psm'], settings['inspection_roi'])
    required_source = source[target[1]:target[3], target[0]:target[2]]
    crops = {'source': crop(required_source, 'source')}
    pieces = []
    for index, page in enumerate(pages):
        rows = [row for row in manifest['reader_map']['body_pages'] if
                (row['tomus'], row['page'], row['unit_id']) == (control['tomus'], page, control['unit_id'])]
        if len(rows) != 1 or rows[0]['folio'] != control['reader_folios'][index]:
            raise common.AuditError('Acceptance control reader page/folio/unit mismatch')
        row = rows[0]
        reader, reader_meta, visible = coverage.load_visible_reader_page(root, manifest, settings, args,
            prerequisites, control['tomus'], page)
        body = [round(v*reader.shape[1-i % 2]) for i, v in enumerate(row['body_box'])]
        visible = [max(body[0], visible[0]), max(body[1], visible[1]), min(body[2], visible[2]), min(body[3], visible[3])]
        reader_height = coverage.anchor_glyph_height(reader, [body])
        if reader_height is None:
            return {'control_id': control['control_id'], 'detected': False, 'reason': 'reader_glyph_scale_unresolved'}
        sy0, sy1 = (max(0, target[1]-6*height), target[1]-4) if index == 0 else (target[3]+4, min(source.shape[0], target[3]+6*height))
        sb = [target[0], math.floor(sy0), target[2], math.ceil(sy1)]
        rb = [body[0], max(body[1], math.floor(body[3]-8*reader_height)), body[2], body[3]] if index == 0 else [body[0], body[1], body[2], min(body[3], math.ceil(body[1]+8*reader_height))]
        reader_ocr = ocr(reader_pdf, reader_record, page, settings['reader_dpi'], settings['ocr']['reader_psm'], row['body_box'])
        fit = register_edge(source, reader, source_ocr, reader_ocr, sb, rb, settings)
        crops['reader_'+str(page)] = crop(reader[rb[1]:rb[3], rb[0]:rb[2]], 'reader_'+str(page))
        piece = {'tomus': control['tomus'], 'reader_page': page, 'registration': fit,
                 'visible_box_px': visible, 'reader_render': reader_meta}
        if fit['matched']:
            matrix = np.asarray(fit['matrix']).copy()
            matrix[:, 2] += matrix[:, :2] @ np.asarray(target[:2])
            measurement = coverage.measure_required_ink(required_source, reader, matrix, settings, visible_box=visible)
            crops['observed_'+str(page)] = crop(measurement['observed_gray'], 'observed_'+str(page))
            piece.update(matrix=matrix, measurement=measurement)
        pieces.append(piece)
    observation = measure_glyph_cut(required_source, pieces, settings)
    return {'control_id': control['control_id'], 'unit_id': control['unit_id'], 'tomus': control['tomus'],
            'user_excerpt': control['user_excerpt'], 'component_coordinate_frame': 'required_source_crop',
            'source_index': control['source_index'], 'source_page': control['source_page'],
            'source_box_px': target, 'reader_pages': pages, 'reader_folios': control['reader_folios'],
            'source_render': source_meta, 'crops': crops, **observation,
            'registrations': [{k: v for k, v in p.items() if k not in ('matrix', 'measurement')} for p in pieces]}


def score_controls(args):
    """Write independently identified observations without changing the machine ledger."""
    import html
    import json
    repo, work = cli.paths(args)
    output = common.safe_write_path(args.out, work, repo, policy=common.AuditOutputPolicy('audit'))
    if output.suffix.lower() != '.json':
        raise common.AuditError('Acceptance control output requires a .json filename')
    evidence = output.with_suffix('').with_name(output.stem+'_evidence')
    report = output.with_suffix('.html')
    if any(path.exists() for path in (output, evidence, report)):
        raise common.AuditError('Acceptance control outputs must be fresh')
    run = Path(args.run_dir).resolve()
    run_record = cli.read_json(run/'run-manifest.json')
    root, manifest = cli.resolve_run_snapshot(run_record, args)
    for name, digest in manifest['runtime']['code'].items():
        if common.hash_file(repo/common.relative_path(name)) != digest:
            raise common.AuditError('Captured comparison code changed')
    code = cli.executed_code(repo)
    control_path = Path(args.controls) if args.controls else repo/'tools/facsimile-real-acceptance-controls.json'
    control_hash = common.hash_file(control_path)
    controls = cli.read_json(control_path)
    if controls.get('schema_version') != 1 or controls['control_denominator'] != len(controls['controls']):
        raise common.AuditError('Invalid acceptance control denominator')
    if len({c['control_id'] for c in controls['controls']}) != len(controls['controls']):
        raise common.AuditError('Repeated acceptance control identifier')
    if any(not re.fullmatch(r'[A-Za-z0-9_]+', c['control_id']) for c in controls['controls']):
        raise common.AuditError('Invalid acceptance control identifier')
    settings_path, _ = cli.snapshot_input(root, manifest, 'settings')
    settings = cli.read_json(settings_path)
    prerequisites = cli.check_audit_prerequisites(args)
    stable = dict(prerequisites, tools={name: {k: v for k, v in item.items() if k not in ('path', 'tessdata_dir')}
        if isinstance(item, dict) else item for name, item in prerequisites['tools'].items()})
    if stable != manifest['runtime']['prerequisites']:
        raise common.AuditError('Control tool/package/language identity differs from capture')
    results = [observe_control(c, root, manifest, settings, args, prerequisites, evidence) for c in controls['controls']]
    if cli.executed_code(repo) != code or common.hash_file(control_path) != control_hash:
        raise common.AuditError('Control code or fixed geometry changed during measurement')
    result = {'schema_version': 1, 'snapshot_id': manifest['snapshot_id'], 'comparison_runtime_id': run_record['runtime_id'],
        'comparison_processing_complete': run_record.get('processing_complete', False),
        'observation_code': code, 'controls_sha256': control_hash, 'control_set': controls['control_set'],
        'denominator': len(results), 'detected': sum(c['detected'] for c in results), 'controls': results,
        'passed': bool(results) and all(c['detected'] for c in results),
        'independent_control_measurement': True, 'machine_ledger_changed': False,
        'random_panel_denominator': controls['random_panel_denominator'], 'counted_separately_from_random_panel': True}
    cli.write(output, result, args)
    sections = []
    for item in results:
        status = 'Glyphs cut across pages' if item['detected'] else 'Detection not established'
        sections.append('<article><h2>'+html.escape(item['control_id'])+'</h2><p><strong>'+status+'</strong></p>')
        if 'user_excerpt' in item:
            sections.append('<p>User locator: '+html.escape(item['user_excerpt'])+'</p>')
        for label, crop in item.get('crops', {}).items():
            sections.append('<p>'+html.escape(label)+'</p><img src="'+html.escape(crop['path'], quote=True)+'">')
        sections.append('<details><summary>Measurements and transforms</summary><pre>'+html.escape(json.dumps(item, indent=2, ensure_ascii=False))+'</pre></details>')
        sections.append('</article>')
    common.safe_write_path(report, work, repo, policy=common.AuditOutputPolicy('audit')).write_text(
        '<!doctype html><meta charset="utf-8"><title>Real glyph-cut controls</title><style>body{font-family:system-ui;max-width:1100px;margin:2em auto}img{max-width:100%}pre{white-space:pre-wrap}</style><h1>Real glyph-cut controls</h1><p>Independent source-backed observations. Combined ink completeness does not certify readable glyphs.</p>'+''.join(sections), encoding='utf8')
    common.progress(f'Real glyph-cut controls: {result["detected"]}/{len(results)}; {output}')
    return 0 if result['passed'] else 4
