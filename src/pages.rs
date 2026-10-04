//! Page-role classification: the contiguous middle of each source PDF is body
//! and needs no API call; only the front/back boundary regions are examined, with
//! near-uniform thumbnails skipped deterministically and the rest labelled by a
//! cheap Haiku vision call. The resulting map is cached and hand-editable in the
//! sidecar.

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

use crate::llm::instrument::{progress_bar, PageId, Usage, UsageTracker};
use crate::llm::{Client, ContentBlock, LineVerdict, Vision, HAIKU, SONNET};
use crate::tags::{PageRecord, PageRole, PageStatus, Sidecar};

/// How many pages at each end of a source are treated as a boundary region to be
/// examined; everything between is unambiguously body.
pub const BOUNDARY_SCAN: u32 = 25;

/// Whether a 1-based page falls in a source's front/back boundary region.
pub fn is_boundary(page: u32, page_count: u32) -> bool {
    page <= BOUNDARY_SCAN || page > page_count.saturating_sub(BOUNDARY_SCAN)
}

/// Classify every source page into the sidecar, rendering thumbnails only for
/// boundary-region pages. `render_thumb(source, page)` yields a thumbnail PNG's
/// bytes; production renders it with poppler, tests supply synthetic bytes so the
/// classifier runs without the binary. An already-classified page is left as is,
/// so a resumed run makes no redundant calls.
pub fn classify(
    vision: &dyn Vision,
    sidecar: &mut Sidecar,
    tracker: &UsageTracker,
    render_thumb: impl Fn(usize, u32) -> Result<Vec<u8>>,
) -> Result<()> {
    let boundary_pages: Vec<(usize, u32)> = sidecar
        .sources
        .iter()
        .enumerate()
        .flat_map(|(si, src)| {
            (1..=src.page_count)
                .filter(move |&p| is_boundary(p, src.page_count))
                .map(move |p| (si, p))
        })
        .filter(|&(si, p)| sidecar.page(si, p).is_none())
        .collect();

    let pb = progress_bar(boundary_pages.len() as u64, "classifying");
    for (si, page) in boundary_pages {
        let bytes = render_thumb(si, page)?;
        let (role, usage) = if looks_like_skip(&bytes) {
            (PageRole::Skip, None)
        } else {
            let base64 = crate::b64(&bytes);
            let (role, usage) = vision.classify(&base64)?;
            (role, Some(usage))
        };
        if let Some(usage) = usage {
            let id: PageId = (si, page);
            tracker.record(HAIKU, id, usage);
        }
        push_record(sidecar, si, page, role);
        pb.inc(1);
    }
    pb.finish_and_clear();

    // The contiguous middle of each source is body without any API call.
    for si in 0..sidecar.sources.len() {
        let count = sidecar.sources[si].page_count;
        for page in 1..=count {
            if !is_boundary(page, count) && sidecar.page(si, page).is_none() {
                push_record(sidecar, si, page, PageRole::Body);
            }
        }
    }

    sidecar.pages.sort_by_key(|p| (p.source, p.source_page));
    Ok(())
}

/// Append a page record carrying a freshly determined role. A body/front-prose
/// page is left unprocessed (its transcription comes later); a skipped or
/// image-only page is already terminal.
fn push_record(sidecar: &mut Sidecar, source: usize, source_page: u32, role: PageRole) {
    let status = match role {
        PageRole::Body | PageRole::FrontProse => PageStatus::NotBody,
        PageRole::Skip | PageRole::FrontImage => PageStatus::NotBody,
    };
    sidecar.pages.push(PageRecord {
        source,
        source_page,
        image_sha256: String::new(),
        role,
        pass_a: None,
        pass_b: None,
        status,
        flags: Vec::new(),
        result: None,
        result_model: None,
    });
}

/// Classify one page from a low-DPI thumbnail via Haiku. Backs
/// [`crate::llm::ApiVision::classify`].
pub fn classify_page(client: &Client, png_base64: &str) -> Result<(PageRole, Usage)> {
    let content = vec![
        ContentBlock::Text(CLASSIFY_PROMPT.to_string()),
        ContentBlock::Image {
            png_base64: png_base64.to_string(),
        },
    ];
    let completion = client.message(HAIKU, None, &content, 256, Some(classify_schema()))?;
    let role = parse_role(&completion.text)?;
    Ok((role, completion.usage))
}

const CLASSIFY_PROMPT: &str = "This is a low-resolution thumbnail of one page from \
a scanned book. Classify the page as exactly one of: \"skip\" (blank, marbling, a \
cover, a library slip, or a barcode), \"front_image\" (a scanned notice or an \
engraved vignette to be reproduced as an image), \"front_prose\" (front-matter \
prose such as a title page, preface, or editorial note to be re-typeset), or \
\"body\" (a two-column page of running text). Return only the JSON object with a \
one-line reason.";

fn classify_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "role": {
                "type": "string",
                "enum": ["skip", "front_image", "front_prose", "body"]
            },
            "reason": { "type": "string" }
        },
        "required": ["role", "reason"],
        "additionalProperties": false
    })
}

/// Judge one line-band crop via Sonnet as a title, furniture, or body line. Backs
/// [`crate::llm::ApiVision::review_line`].
pub fn review_line_page(client: &Client, png_base64: &str) -> Result<(LineVerdict, Usage)> {
    let content = vec![
        ContentBlock::Text(REVIEW_PROMPT.to_string()),
        ContentBlock::Image {
            png_base64: png_base64.to_string(),
        },
    ];
    let completion = client.message(SONNET, None, &content, 256, Some(review_schema()))?;
    let verdict = parse_verdict(&completion.text)?;
    Ok((verdict, completion.usage))
}

const REVIEW_PROMPT: &str = "This is a cropped horizontal strip from one page of a \
scanned nineteenth-century edition of Plutarch's Lives. Classify the strip as \
exactly one of: \"title\" (a printed display title of a Life or Comparatio, e.g. \
ROMULUS or COMPARATIO THESEI ET ROMULI, usually centered and set apart), \
\"furniture\" (a running head, a page number or folio, a printer's signature or \
catchword, or an ornament — not editorial text), or \"body\" (a line of the running \
Latin text). Return only the JSON object with a one-line reason.";

fn review_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "verdict": {
                "type": "string",
                "enum": ["title", "furniture", "body"]
            },
            "reason": { "type": "string" }
        },
        "required": ["verdict", "reason"],
        "additionalProperties": false
    })
}

fn parse_verdict(reply: &str) -> Result<LineVerdict> {
    let value: Value = serde_json::from_str(reply).context("parsing line-review reply")?;
    let verdict = value
        .get("verdict")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("line-review reply has no verdict"))?;
    match verdict {
        "title" => Ok(LineVerdict::Title),
        "furniture" => Ok(LineVerdict::Furniture),
        "body" => Ok(LineVerdict::Body),
        other => Err(anyhow!("unknown verdict {other:?}")),
    }
}

fn parse_role(reply: &str) -> Result<PageRole> {
    let value: Value = serde_json::from_str(reply).context("parsing classification reply")?;
    let role = value
        .get("role")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("classification reply has no role"))?;
    match role {
        "skip" => Ok(PageRole::Skip),
        "front_image" => Ok(PageRole::FrontImage),
        "front_prose" => Ok(PageRole::FrontProse),
        "body" => Ok(PageRole::Body),
        other => Err(anyhow!("unknown role {other:?}")),
    }
}

/// Whether a thumbnail is near-uniform in luminance — a blank leaf (almost all
/// light) or a dark cover (almost all dark) — and so can be skipped without an
/// API call. Returns false when the bytes cannot be decoded, deferring the
/// decision to the vision classifier.
pub fn looks_like_skip(png_bytes: &[u8]) -> bool {
    let Ok(img) = image::load_from_memory(png_bytes) else {
        return false;
    };
    let luma = img.to_luma8();
    let total = (luma.width() as u64) * (luma.height() as u64);
    if total == 0 {
        return false;
    }
    let mut dark = 0u64;
    let mut light = 0u64;
    for px in luma.pixels() {
        let l = px.0[0];
        if l < 40 {
            dark += 1;
        } else if l > 215 {
            light += 1;
        }
    }
    let dark_fraction = dark as f64 / total as f64;
    let light_fraction = light as f64 / total as f64;
    // A blank page is almost entirely light; a dark cover almost entirely dark. A
    // text page keeps a non-trivial light background too, so the blank threshold
    // is set high enough to leave text pages for the classifier.
    light_fraction > 0.995 || dark_fraction > 0.95
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Luma};

    struct PanicVision;
    impl Vision for PanicVision {
        fn classify(&self, _png: &str) -> Result<(PageRole, Usage)> {
            panic!("vision.classify must not be called");
        }
        fn review_line(&self, _png: &str) -> Result<(LineVerdict, Usage)> {
            panic!("vision.review_line must not be called");
        }
    }

    struct CannedVision {
        role: PageRole,
        usage: Usage,
    }
    impl Vision for CannedVision {
        fn classify(&self, _png: &str) -> Result<(PageRole, Usage)> {
            Ok((self.role, self.usage))
        }
        fn review_line(&self, _png: &str) -> Result<(LineVerdict, Usage)> {
            Ok((LineVerdict::Body, Usage::default()))
        }
    }

    fn solid_png(value: u8) -> Vec<u8> {
        let img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_pixel(20, 30, Luma([value]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageLuma8(img)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        bytes
    }

    fn sidecar_with_sources(v1_pages: u32, v2_pages: u32) -> Sidecar {
        Sidecar {
            schema_version: crate::tags::SCHEMA_VERSION,
            sources: vec![
                crate::tags::SourcePdf {
                    path: "v1.pdf".to_string(),
                    page_count: v1_pages,
                    file_sha256: "a".to_string(),
                },
                crate::tags::SourcePdf {
                    path: "v2.pdf".to_string(),
                    page_count: v2_pages,
                    file_sha256: "b".to_string(),
                },
            ],
            front_matter: Default::default(),
            pages: Vec::new(),
            lives: Vec::new(),
            batches: Vec::new(),
        }
    }

    #[test]
    fn looks_like_skip_detects_uniform() {
        assert!(looks_like_skip(&solid_png(255))); // blank
        assert!(looks_like_skip(&solid_png(0))); // dark cover
        assert!(!looks_like_skip(&solid_png(128))); // midtone — defer to API
        assert!(!looks_like_skip(b"not a png")); // undecodable — defer to API
    }

    #[test]
    fn uniform_thumbnail_classified_skip_without_api() {
        // A single-page source so the one page is in the boundary region. A blank
        // thumbnail must be skipped deterministically, never reaching the client.
        let mut sidecar = sidecar_with_sources(1, 1);
        let tracker = UsageTracker::new();
        classify(&PanicVision, &mut sidecar, &tracker, |_, _| {
            Ok(solid_png(255))
        })
        .unwrap();
        for page in &sidecar.pages {
            assert_eq!(page.role, PageRole::Skip);
        }
        assert_eq!(tracker.call_count(), 0);
    }

    #[test]
    fn middle_pages_marked_body_without_api() {
        // Pages past the boundary region on both ends are body with no render and
        // no API call.
        let mut sidecar = sidecar_with_sources(100, 100);
        let tracker = UsageTracker::new();
        classify(&PanicVision, &mut sidecar, &tracker, |_, page| {
            assert!(is_boundary(page, 100), "middle page {page} must not render");
            Ok(solid_png(255))
        })
        .unwrap();
        let mid = sidecar.page(0, 50).unwrap();
        assert_eq!(mid.role, PageRole::Body);
        assert_eq!(tracker.call_count(), 0);
    }

    #[test]
    fn classify_is_cached() {
        let mut sidecar = sidecar_with_sources(1, 1);
        // Pre-populate the single page; a second run must not render or call out.
        push_record(&mut sidecar, 0, 1, PageRole::FrontProse);
        push_record(&mut sidecar, 1, 1, PageRole::Body);
        let tracker = UsageTracker::new();
        classify(&PanicVision, &mut sidecar, &tracker, |_, _| {
            panic!("cached pages must not render")
        })
        .unwrap();
        assert_eq!(sidecar.page(0, 1).unwrap().role, PageRole::FrontProse);
    }

    #[test]
    fn classify_round_trips_roles_in_sidecar() {
        let mut sidecar = sidecar_with_sources(1, 1);
        let vision = CannedVision {
            role: PageRole::FrontProse,
            usage: Usage::default(),
        };
        let tracker = UsageTracker::new();
        classify(&vision, &mut sidecar, &tracker, |_, _| Ok(solid_png(128))).unwrap();
        let json = serde_json::to_string(&sidecar).unwrap();
        let back: Sidecar = serde_json::from_str(&json).unwrap();
        assert_eq!(back.page(0, 1).unwrap().role, PageRole::FrontProse);
    }

    #[test]
    fn classify_records_usage_per_classified_page() {
        // Two single-page sources, both non-uniform, so each hits the API exactly
        // once; a blank page would be skipped deterministically and not recorded.
        let mut sidecar = sidecar_with_sources(1, 1);
        let vision = CannedVision {
            role: PageRole::Body,
            usage: Usage {
                input_tokens: 50,
                output_tokens: 5,
            },
        };
        let tracker = UsageTracker::new();
        classify(&vision, &mut sidecar, &tracker, |_, _| Ok(solid_png(128))).unwrap();
        assert_eq!(tracker.call_count(), 2);
        let report = tracker.report();
        assert_eq!(report.models.len(), 1);
        assert_eq!(report.models[0].model, HAIKU);
        assert_eq!(report.models[0].calls, 2);
    }

    #[test]
    fn parse_role_maps_all_values() {
        assert_eq!(
            parse_role(r#"{"role":"skip","reason":"blank"}"#).unwrap(),
            PageRole::Skip
        );
        assert_eq!(
            parse_role(r#"{"role":"body","reason":"text"}"#).unwrap(),
            PageRole::Body
        );
        assert!(parse_role(r#"{"role":"other"}"#).is_err());
    }
}
