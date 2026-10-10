"""Shared offline audit infrastructure; optional imaging dependencies are lazy."""
import hashlib
import json
import math
import os
import re
import shutil
import subprocess
import tempfile
import threading
import time
from pathlib import Path
from contextlib import contextmanager
from dataclasses import dataclass

class AuditError(Exception):
    """An actionable configuration or tool failure."""


class CalibrationError(AuditError):
    """An explicitly requested calibration gate failed."""


def json_hash(data):
    return hashlib.sha256(json.dumps(data, ensure_ascii=False, sort_keys=True,
                                     separators=(",", ":"), allow_nan=False).encode()).hexdigest()


def validate_box(box):
    if (not isinstance(box, list) or len(box) != 4
            or any(isinstance(n, bool) or not isinstance(n, (int, float))
                   or not math.isfinite(n) for n in box)
            or not 0 <= box[0] < box[2] <= 1 or not 0 <= box[1] < box[3] <= 1):
        raise AuditError(f"Invalid normalized box: {box}")
    return box


def page_geometry(page):
    return {"mediabox": list(page.mediabox), "cropbox": list(page.cropbox),
            "rotation": page.rotation, "render_box_policy": "mediabox"}


def normalized_pdf_box(page, box):
    """Map unrotated PyMuPDF points to displayed MediaBox coordinates."""
    import pymupdf
    mx0, my0, mx1, my1 = page.mediabox
    inverse = ~page.transformation_matrix
    corners = []
    for x, y in ((box[0], box[1]), (box[2], box[1]), (box[2], box[3]), (box[0], box[3])):
        point = pymupdf.Point(x, y) * inverse
        u, v = (point.x - mx0) / (mx1 - mx0), (my1 - point.y) / (my1 - my0)
        rotated = {0: (u, v), 90: (1 - v, u), 180: (1 - u, 1 - v), 270: (v, 1 - u)}
        if page.rotation not in rotated:
            raise AuditError("Unsupported page rotation")
        corners.append(rotated[page.rotation])
    return [min(p[0] for p in corners), min(p[1] for p in corners),
            max(p[0] for p in corners), max(p[1] for p in corners)]


def pixel_bounds(box, size):
    validate_box(box)
    width, height = size
    return (math.floor(box[0] * width), math.floor(box[1] * height),
            math.ceil(box[2] * width), math.ceil(box[3] * height))


def parse_tex_refs(path):
    text = re.sub(r"(?<!\\)%[^\n]*", "", Path(path).read_text(encoding="utf-8"))
    text = text.split(r"\begin{document}", 1)[1].split(r"\end{document}", 1)[0]
    return re.findall(r"\\includegraphics(?:\s*\[[^\]]*\])?\s*\{([^{}]+\.png)\}", text)


def enumerate_image_occurrences(page):
    """Enumerate displayed image occurrences, including repeated uses of one object."""
    return [{"xref": image["xref"], "box": normalized_pdf_box(page, image["bbox"])}
            for image in page.get_image_info(xrefs=True)]


def decoded_image_identity(pixmap):
    import pymupdf
    if pixmap.alpha:
        pixmap = pymupdf.Pixmap(pixmap, 0)
    if pixmap.colorspace != pymupdf.csRGB:
        pixmap = pymupdf.Pixmap(pymupdf.csRGB, pixmap)
    return {"pixel_sha256": hashlib.sha256(pixmap.samples).hexdigest(),
            "dimensions": [pixmap.width, pixmap.height]}


BODY_ASSET = re.compile(r"facsimile_(.+)_(\d+)\.png")


def read_footer_folio(page):
    """Return every generated Arabic folio printed in the bottom margin of a page.

    The reader's `folio` page style prints `\\thepage` (body folio = physical page -
    body-first page + 1) in the bottom margin; unit openings suppress it with an empty
    page style, so a body opening carries no footer word and its folio is inferred."""
    found = []
    for x0, y0, x1, y1, text, *_ in page.get_text("words"):
        label = text.strip()
        if label.isdigit():
            box = normalized_pdf_box(page, [x0, y0, x1, y1])
            if box[1] >= 0.9:
                found.append({"folio": int(label), "box": box})
    return found


def image_views(gray):
    """Keep faint grayscale ink and a local-contrast view as separately scored observations."""
    import cv2
    return {"grayscale": gray, "contrast": cv2.createCLAHE(clipLimit=2.0, tileGridSize=(8, 8)).apply(gray)}


def ink_line_height(gray, *, settings=None):
    """Estimate character height from the same dark-ink components in source and reader.

    The upper quartile excludes small punctuation and interlinear scraps; long rules
    and page-spanning components do not supply the typography scale.
    """
    import cv2
    import numpy as np
    settings = settings or {}
    _, _, stats, _ = cv2.connectedComponentsWithStats(
        (gray < settings.get("threshold", 120)).astype(np.uint8),
        connectivity=settings.get("connectivity", 8))
    heights = sorted(int(s[3]) for s in stats[1:] if
                     max(4, 0.002 * gray.shape[0]) <= s[3] <= 0.06 * gray.shape[0]
                     and 2 <= s[2] <= 3 * s[3] and s[4] >= 5)
    return float(heights[3 * len(heights) // 4]) if len(heights) >= 8 else None


def render_page(pdf, record, page_number, dpi, work, repo, *, pdftoppm=None, policy=None):
    """Render the full MediaBox with Poppler; cache only verified completed rasters."""
    import pymupdf
    from PIL import Image
    executable = pdftoppm or shutil.which("pdftoppm")
    if not executable or not isinstance(dpi, int) or not 45 <= dpi <= 600:
        raise AuditError("Poppler must be available and DPI must be between 45 and 600")
    version = run_external([executable, "-v"]).splitlines()[0]
    with pymupdf.open(pdf) as document:
        if not 1 <= page_number <= len(document):
            raise AuditError("Render page outside input PDF bounds")
        geometry = page_geometry(document[page_number - 1])
    metadata = {"input_sha256": record["sha256"], "page": page_number, "dpi": dpi,
                "poppler_version": version, **geometry}
    key = hashlib.sha256(json.dumps(metadata, sort_keys=True).encode()).hexdigest()
    folder = safe_write_path(Path(work) / "cache" / "renders" / key, work, repo, policy=policy)
    raster, sidecar = folder / "page.png", folder / "metadata.json"
    cached = reusable_entry(sidecar, metadata, work, repo, policy=policy)
    if cached and raster.exists() and hash_file(raster) == cached.get("raster_sha256"):
        return raster, cached
    folder.mkdir(parents=True, exist_ok=True)
    lock = folder / "render.lock"
    waited = time.monotonic()
    while True:
        try:
            descriptor = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
            break
        except FileExistsError as exc:
            cached = reusable_entry(sidecar, metadata, work, repo, policy=policy)
            if cached and raster.exists() and hash_file(raster) == cached.get('raster_sha256'):
                return raster, cached
            if time.monotonic() - waited >= 180:
                raise AuditError(f"Render lock did not complete within 180 seconds; inspect the owning invocation: {lock}") from exc
            time.sleep(.25)
    os.close(descriptor)
    try:
        with tempfile.TemporaryDirectory(dir=folder) as temporary:
            prefix = Path(temporary) / "page"
            with progress_heartbeat(f"Rendering {record['path']} page {page_number}"):
                run_external([executable, "-png", "-f", str(page_number), "-l", str(page_number),
                              "-r", str(dpi), "-singlefile", str(pdf), str(prefix)], timeout=180)
            rendered = prefix.with_suffix(".png")
            with Image.open(rendered) as image:
                image.load()
                dimensions = list(image.size)
            os.replace(rendered, safe_write_path(raster, work, repo, policy=policy))
        data = {**metadata, "raster_dimensions": dimensions, "raster_sha256": hash_file(raster),
                "unusual_coordinate_status": "requires independently inspected overlay" if
                geometry["rotation"] or geometry["cropbox"] != geometry["mediabox"] else "standard corpus geometry"}
        write_json(sidecar, {"schema_version": 1, "complete": True, "metadata": metadata, "data": data}, work, repo, policy=policy)
        progress(f"Rendered {record['path']} page {page_number} at {dpi} DPI")
        return raster, data
    finally:
        lock.unlink()


@contextmanager
def run_directory(path, work, repo, *, policy=None):
    run = safe_write_path(path, work, repo, policy=policy)
    run.mkdir(parents=True, exist_ok=True)
    lock = safe_write_path(run / "run.lock", work, repo)
    try:
        descriptor = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
    except FileExistsError as exc:
        raise AuditError(f"Another invocation owns {lock}; inspect before removing a stale lock") from exc
    os.close(descriptor)
    try:
        yield run
    finally:
        lock.unlink()


def hash_file(path):
    digest = hashlib.sha256()
    last = time.monotonic()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
            if time.monotonic() - last >= 30:
                progress(f"Hashing {path}")
                last = time.monotonic()
    return digest.hexdigest()


def relative_path(value):
    path = Path(value)
    if path.is_absolute() or path.drive or ".." in path.parts or not path.parts:
        raise AuditError(f"Expected a relative path without traversal: {value}")
    return path


def contained(path, root):
    return path == root or root in path.parents


@dataclass(frozen=True)
class AuditOutputPolicy:
    """Bind exceptional fixture PDF creation to an owned root and exact allowlist."""
    profile: str = "historical"
    fixture_root: Path | None = None
    fixture_files: frozenset = frozenset()

    def __post_init__(self):
        if self.profile not in {"historical", "audit", "fixture"}:
            raise AuditError("Unknown output policy profile")
        if self.profile == "fixture" and self.fixture_root is None:
            raise AuditError("Fixture policy requires an exact owned root")


def safe_write_path(destination, work_dir, repo_root, *, capture_root=None, policy=None):
    """Authorize actual resolved destinations, including junction/symlink targets."""
    destination = Path(destination).resolve()
    work = Path(work_dir).resolve()
    repo = Path(repo_root).resolve()
    protected = [repo / name for name in (
        "src", "build_facsimile", "cache", "transcription_cache", "transcriptions", "archive", ".git",
    )]
    protected.extend([work / "spec"])
    protected.extend(path.resolve() for path in repo.glob("*.pdf"))
    protected.append((repo / "facsimile_units.json").resolve())
    protected.append((repo / "secrets.json").resolve())
    if any(contained(destination, path.resolve()) for path in protected):
        raise AuditError(f"Protected input/spec destination: {destination}")
    if capture_root is None:
        roots = [(work / "cache").resolve(), (work / "runs").resolve()]
        snapshot = (work / "snapshots").resolve()
        if contained(destination, snapshot):
            raise AuditError(f"Snapshot destination is read-only: {destination}")
    else:
        root = Path(capture_root).resolve()
        snapshots = work / "snapshots"
        if root.parent != snapshots or not contained(root, work):
            raise AuditError("Capture root must be a direct, non-redirected snapshot child.")
        if (root / "manifest.json").exists() or (root / "snapshot-manifest.json").exists():
            raise AuditError(f"Completed snapshot cannot be overwritten: {root}")
        roots = [root]
    if not any(contained(destination, root) and contained(root, work) for root in roots):
        raise AuditError(f"Destination is outside the authorized output directories: {destination}")
    if destination.suffix.lower() == ".pdf" and capture_root is None:
        if policy is None or policy.profile != "fixture":
            raise AuditError("Ordinary processing cannot write PDF inputs.")
        fixture = Path(policy.fixture_root).resolve()
        if (not contained(fixture, (work / "runs" / "verification").resolve())
                or fixture == (work / "runs" / "verification").resolve()
                or not contained(destination, fixture)
                or destination.relative_to(fixture).as_posix() not in policy.fixture_files
                or destination.exists()):
            raise AuditError("Fixture PDF must be fresh, allowlisted and inside its owned verification root")
    return destination


def write_json(path, data, work, repo, *, capture_root=None, policy=None):
    path = safe_write_path(path, work, repo, capture_root=capture_root, policy=policy)
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent,
                                     suffix=".tmp", delete=False) as stream:
        temporary = Path(stream.name)
        json.dump(data, stream, ensure_ascii=False, indent=2, sort_keys=bool(policy), allow_nan=False)
        stream.write("\n")
    try:
        safe_write_path(path, work, repo, capture_root=capture_root, policy=policy)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def progress(message):
    print(message, flush=True)


@contextmanager
def progress_heartbeat(message):
    stopped = threading.Event()

    def report():
        while not stopped.wait(30):
            progress(message)

    worker = threading.Thread(target=report, daemon=True)
    worker.start()
    try:
        yield
    finally:
        stopped.set()
        worker.join()


def reusable_entry(path, metadata, work_dir, repo_root, *, policy=None):
    """Return completed JSON diagnostic data only when all identity/settings match."""
    path = safe_write_path(path, work_dir, repo_root, policy=policy)
    try:
        entry = json.loads(path.read_text(encoding="utf-8"))
        if (entry.get("schema_version") == 1 and entry.get("complete") is True
                and entry.get("metadata") == metadata and "data" in entry):
            return entry["data"]
    except (OSError, ValueError, AttributeError):
        pass
    return None


def run_external(arguments, timeout=30):
    """Execute an argument list without a shell or a visible Windows window."""
    try:
        result = subprocess.run(
            arguments, shell=False, capture_output=True, text=True,
            encoding="utf-8", errors="replace", timeout=timeout,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise AuditError(f"Cannot run {arguments[0]}: {exc}") from exc
    if result.returncode:
        raise AuditError(f"{arguments[0]} exited {result.returncode}: {result.stderr.strip()}")
    return (result.stdout + result.stderr).strip()


def map_reader_occurrences(reader_inputs, expected_counts=None):
    """Map ordered TeX image references to displayed reader pages, folios and units.

    Only actual ordered `\\includegraphics` occurrences are mapped, not unique image
    objects; front-matter scans, blank recto separators and signature padding are kept
    distinct from numbered body images. Printed folios come from the generated footer;
    suppressed unit openings are inferred from the tomus's single page/folio offset."""
    import pymupdf
    expected = expected_counts
    tomi, body_rows, unit_summaries = [], [], []
    for reader in sorted(reader_inputs, key=lambda r: r["tomus"]):
        tomus = reader["tomus"]
        tex, pdf = Path(reader["tex_path"]), Path(reader["pdf_path"])
        record = {"sha256": reader["pdf_sha256"]}
        refs = parse_tex_refs(tex)
        with pymupdf.open(pdf) as document:
            scanned, occurrences = [], []
            for number, page in enumerate(document, 1):
                occ = enumerate_image_occurrences(page)
                scanned.append({"page": number, "occurrences": occ,
                                "folios": read_footer_folio(page),
                                "word_count": len(page.get_text("words"))})
                occurrences.extend((number, o) for o in occ)
            if len(refs) != len(occurrences):
                raise AuditError(f"Ambiguous tomus {tomus} image inventory: {len(refs)} TeX refs / {len(occurrences)} occurrences")
            placements = {number: [] for number in range(1, len(document) + 1)}
            for asset, (number, occ) in zip(refs, occurrences):
                relative_path(asset)
                identity = decoded_image_identity(pymupdf.Pixmap(document, occ["xref"])) if occ["xref"] else None
                available = reader.get("body_assets", {}).get(asset)
                if available and identity != available:
                    raise AuditError(f"Decoded asset identity mismatch: tomus {tomus}, {asset}")
                occ = dict(occ, decoded_identity=identity,
                           identity_status="authenticated_asset" if available else "embedded_only" if identity else "ambiguous")
                match = BODY_ASSET.fullmatch(Path(asset).name)
                placements[number].append({"asset": asset, "box": occ["box"],
                                           "unit_id": match[1] if match else None,
                                           "ordinal": int(match[2]) if match else None,
                                           "image_occurrence": occ})
            body_numbers = [n for n, items in placements.items() if any(i["unit_id"] for i in items)]
            if any(sum(1 for i in placements[n] if i["unit_id"]) > 1 for n in body_numbers):
                raise AuditError(f"Multiple body occurrences on one tomus {tomus} page need manual mapping")
            first_body, last_body = (min(body_numbers), max(body_numbers)) if body_numbers else (None, None)
            offsets = {scan["page"] - scan["folios"][0]["folio"]
                       for scan in scanned
                       if len(scan["folios"]) == 1 and any(i["unit_id"] for i in placements[scan["page"]])}
            offset = next(iter(offsets)) if len(offsets) == 1 else None
            pages, anomalies = [], []
            for scan in scanned:
                number = scan["page"]
                items = placements[number]
                body = next((i for i in items if i["unit_id"]), None)
                front_assets = [Path(i["asset"]).name for i in items if not i["unit_id"]]
                if body:
                    kind = "body"
                elif number < (first_body or 0):
                    kind = "frontmatter"
                elif last_body is not None and number > last_body:
                    kind = "padding"
                else:
                    kind = "separator"
                entry = {"page": number, "kind": kind, "has_image": bool(items),
                         "image_assets": front_assets, "word_count": scan["word_count"]}
                if body:
                    validate_box(body["box"])
                    is_opening = body["ordinal"] == 1
                    printed = scan["folios"]
                    if len(printed) == 1:
                        folio, provenance = printed[0]["folio"], "printed"
                        if offset is not None and number - offset != folio:
                            anomalies.append({"page": number, "printed": folio, "offset": offset})
                    elif offset is not None:
                        folio, provenance = number - offset, "inferred"
                    else:
                        folio, provenance = None, "unknown"
                    entry.update({"unit_id": body["unit_id"], "ordinal": body["ordinal"],
                                  "asset": body["asset"], "body_box": body["box"],
                                  "is_opening": is_opening, "folio": folio,
                                  "folio_provenance": provenance})
                    body_rows.append({"tomus": tomus, "page": number, "unit_id": body["unit_id"],
                                      "ordinal": body["ordinal"], "asset": body["asset"],
                                      "input_sha256": record["sha256"], "body_box": body["box"],
                                      "folio": folio, "folio_provenance": provenance,
                                      "is_opening": is_opening,
                                      "image_occurrence": body["image_occurrence"]})
                pages.append(entry)
            tomus_body = [p for p in pages if p["kind"] == "body"]
            if expected and (len(document) != expected["physical_by_tomus"][tomus - 1] or len(tomus_body) != expected["body_by_tomus"][tomus - 1]):
                raise AuditError(f"Pinned page count mismatch in tomus {tomus}")
            order, grouped = [], {}
            for row in tomus_body:
                grouped.setdefault(row["unit_id"], []).append(row)
                if row["unit_id"] not in order:
                    order.append(row["unit_id"])
            units = []
            for unit_id in order:
                rows = grouped[unit_id]
                ordinals = [r["ordinal"] for r in rows]
                if sorted(ordinals) != list(range(1, len(rows) + 1)):
                    raise AuditError(f"Non-contiguous body ordinals for {unit_id} in tomus {tomus}")
                opening, closing = rows[0], rows[-1]
                units.append({"unit_id": unit_id, "tomus": tomus, "page_count": len(rows),
                              "opening_page": opening["page"], "opening_folio": opening["folio"],
                              "opening_folio_provenance": opening["folio_provenance"],
                              "closing_page": closing["page"], "closing_folio": closing["folio"]})
            unit_summaries.extend(units)
            tomi.append({"tomus": tomus, "input_sha256": record["sha256"],
                         "physical_pages": len(document), "body_pages": len(tomus_body),
                         "first_body_page": first_body, "last_body_page": last_body,
                         "folio_offset": offset, "offset_candidates": sorted(offsets),
                         "folio_anomalies": anomalies, "units": units, "pages": pages})
            progress(f"Reader map: tomus {tomus}, {len(document)} physical / {len(tomus_body)} body / "
                     f"{len(units)} units, folio offset {offset}")
    if expected and (len(body_rows) != expected["body_pages"] or len(unit_summaries) != expected["units"]):
        raise AuditError(f"Reader map totals {len(body_rows)} body / {len(unit_summaries)} units differ from pinned corpus counts")
    return {"schema_version": 1, "adapter": "reader-map-folios-v1", "tomi": tomi,
            "body_pages": body_rows, "units": unit_summaries}

