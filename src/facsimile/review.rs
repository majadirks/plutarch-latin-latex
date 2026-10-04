//! The optional, opt-in Claude Vision line review (`--review-lines`).
//!
//! The normal build is deterministic and network-free. This pass exists only to
//! correct the boundary decisions the offline heuristics are least sure about: on
//! each unit's first page it submits the outlier (tall / narrow) leading bands to
//! Claude Vision and, from the verdicts, pins the unit's `skip_top_bands` and
//! records provenance in `review_notes`. Those conclusions are written back into the
//! committed `facsimile_units.json`, so every subsequent normal build reuses them
//! with zero network calls — mirroring how the front-matter sidecar caches
//! transcription.

use std::io::Cursor;

use anyhow::Result;
use image::{GrayImage, ImageFormat};

use crate::facsimile::lines::Band;
use crate::facsimile::reflow;
use crate::facsimile::units::{Unit, UnitRange};
use crate::llm::instrument::UsageTracker;
use crate::llm::{LineVerdict, Vision, SONNET};

/// Run the line review when `enabled`, building the Vision client lazily so a
/// disabled run (the normal build) constructs no client and makes no calls. Returns
/// `true` when the review ran (the caller then re-saves the unit table).
pub fn review_lines_pass<F>(
    enabled: bool,
    ranges: &[UnitRange],
    render: &impl Fn(usize, u32) -> Result<GrayImage>,
    tracker: &UsageTracker,
    units: &mut [Unit],
    vision_factory: F,
) -> Result<bool>
where
    F: FnOnce() -> Result<Box<dyn Vision>>,
{
    if !enabled {
        return Ok(false);
    }
    let vision = vision_factory()?;
    review_units(ranges, render, vision.as_ref(), tracker, units)?;
    Ok(true)
}

/// Review each unit's first-page outlier leading bands and fold the verdicts into
/// its `skip_top_bands` / `review_notes`. A leading `Title` pins the top-trim through
/// that band; a leading `Furniture` folds into the pinned count; the first `Body`
/// verdict ends the scan (the body has begun).
fn review_units(
    ranges: &[UnitRange],
    render: &impl Fn(usize, u32) -> Result<GrayImage>,
    vision: &dyn Vision,
    tracker: &UsageTracker,
    units: &mut [Unit],
) -> Result<()> {
    for range in ranges {
        let (pages, metrics) = reflow::unit_body_strips(range, render)?;
        let Some((col, bands)) = pages.first() else {
            continue;
        };
        let mut pin: Option<u32> = None;
        let mut notes: Vec<String> = Vec::new();
        for i in reflow::review_candidates(col, bands, &metrics) {
            let png = crop_band_png(col, &bands[i])?;
            let (verdict, usage) = vision.review_line(&crate::b64(&png))?;
            tracker.record(SONNET, (range.unit.source, range.unit.start_page), usage);
            match verdict {
                LineVerdict::Title => {
                    pin = Some(i as u32 + 1);
                    notes.push(format!("vision: title at band {i}"));
                }
                LineVerdict::Furniture => {
                    let p = i as u32 + 1;
                    pin = Some(pin.map_or(p, |cur| cur.max(p)));
                    notes.push(format!("vision: furniture at band {i}"));
                }
                LineVerdict::Body => break,
            }
        }
        if let Some(p) = pin {
            if let Some(u) = units.iter_mut().find(|u| u.id == range.unit.id) {
                u.skip_top_bands = Some(p);
                u.review_notes = Some(notes.join("; "));
            }
        }
    }
    Ok(())
}

/// Encode one line-band crop of a column image as PNG bytes for a Vision request.
fn crop_band_png(col: &GrayImage, band: &Band) -> Result<Vec<u8>> {
    let h = band.bottom.saturating_sub(band.top).max(1);
    let crop = image::imageops::crop_imm(col, 0, band.top, col.width(), h).to_image();
    let mut bytes = Vec::new();
    image::DynamicImage::ImageLuma8(crop)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::instrument::Usage;
    use crate::tags::{LifeKind, PageRole};
    use std::cell::RefCell;

    /// A Vision stub counting `review_line` calls and returning a canned verdict.
    struct CountingVision {
        calls: RefCell<usize>,
        verdict: LineVerdict,
    }

    impl Vision for CountingVision {
        fn classify(&self, _png: &str) -> Result<(PageRole, Usage)> {
            panic!("classify must not be called by the line review");
        }
        fn review_line(&self, _png: &str) -> Result<(LineVerdict, Usage)> {
            *self.calls.borrow_mut() += 1;
            Ok((self.verdict, Usage::default()))
        }
    }

    fn unit(id: &str) -> Unit {
        Unit {
            id: id.to_string(),
            latin_title: id.to_uppercase(),
            greek_title: "Τ".to_string(),
            kind: LifeKind::Life,
            group: 1,
            source: 0,
            start_page: 1,
            skip_top_bands: None,
            title_top_frac: None,
            body_top_frac: None,
            last_page: None,
            pinned: false,
            review_notes: None,
        }
    }

    #[test]
    fn review_offline_without_flag() {
        // Disabled: the factory is never called, so no client is built and no API
        // call is made.
        let built = RefCell::new(false);
        let factory = || {
            *built.borrow_mut() = true;
            let v: Box<dyn Vision> = Box::new(CountingVision {
                calls: RefCell::new(0),
                verdict: LineVerdict::Body,
            });
            Ok(v)
        };
        let tracker = UsageTracker::new();
        let mut units = vec![unit("theseus")];
        let range = UnitRange {
            unit: units[0].clone(),
            end_page: 3,
            shared_tail: false,
        };
        let ran = review_lines_pass(
            false,
            &[range],
            &|_, _| panic!("render must not be called when disabled"),
            &tracker,
            &mut units,
            factory,
        )
        .unwrap();
        assert!(!ran);
        assert!(
            !*built.borrow(),
            "a Vision client was built with the flag off"
        );
    }
}
