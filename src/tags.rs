//! The committed sidecar data model: per-page transcription records (both
//! candidate passes, the arbitrated result, verification flags), the page-role
//! map, the Life/pair-group boundary table, derived bibliographic metadata, and
//! source hashes, plus (de)serialization and resumable lookups.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::render::{file_hash, TRANSCRIBE_DPI};

/// The sidecar schema version written by the current code. Bumped whenever the
/// serialized shape changes.
pub const SCHEMA_VERSION: u32 = 2;

/// The committed cache for the whole work: everything the offline build needs so
/// it makes zero network calls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sidecar {
    pub schema_version: u32,
    /// The source PDFs, in order (index 0 = v1, index 1 = v2).
    pub sources: Vec<SourcePdf>,
    /// Derived bibliographic strings for the re-typeset title pages. The
    /// front-matter prose (Monitum, Praefatio) is not duplicated here: it is the
    /// `result` of the corresponding `FrontProse` page records.
    pub front_matter: FrontMatter,
    /// Every processed source page, in order.
    pub pages: Vec<PageRecord>,
    /// Detected Life/Comparatio boundaries.
    pub lives: Vec<LifeBoundary>,
    /// In-flight Batch API submissions, one per stage and chunk, so an interrupted
    /// run re-attaches to a batch already in progress instead of resubmitting and
    /// re-paying. Empty between stages and on a complete sidecar.
    #[serde(default)]
    pub batches: Vec<BatchState>,
}

/// A submitted but not-yet-collected batch: which stage and chunk it covers and
/// the batch id to re-attach to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchState {
    pub stage: String,
    pub chunk: usize,
    pub batch_id: String,
}

/// One source PDF: its path, page count, and file hash (for cache keying and
/// staleness checks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePdf {
    pub path: String,
    pub page_count: u32,
    pub file_sha256: String,
}

/// Derived bibliographic metadata used on the re-typeset title pages.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub editor: String,
    pub date: String,
}

/// One processed source page: its role, both candidate transcriptions, the
/// agreement status, verification flags, and the authoritative result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRecord {
    /// Index into `Sidecar::sources`.
    pub source: usize,
    /// 1-based page in that PDF.
    pub source_page: u32,
    /// SHA-256 of the rendered PNG, kept as an integrity field (not the key).
    pub image_sha256: String,
    pub role: PageRole,
    /// Sonnet transcription (`None` if the page is skipped or unprocessed).
    pub pass_a: Option<PageDoc>,
    /// Haiku transcription.
    pub pass_b: Option<PageDoc>,
    pub status: PageStatus,
    pub flags: Vec<VerifyFlag>,
    /// Authoritative transcription (`None` until reconciled).
    pub result: Option<PageDoc>,
    /// Which model produced `result`: `Some("claude-sonnet-5")` on an accepted
    /// page, `Some("claude-opus-4-8")` on an escalated one, `None` until reconciled.
    #[serde(default)]
    pub result_model: Option<String>,
}

/// A structured per-page transcription: an ordered sequence of blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageDoc {
    pub blocks: Vec<Block>,
}

/// One block of a transcribed page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub kind: BlockKind,
    /// Chapter numeral, as printed (e.g. `"XI."`), when `kind` is `Chapter`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roman: Option<String>,
    /// Parenthetical section number, when `kind` is `Section`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section_no: Option<u32>,
    /// Whether this block's body text continues the previous page/paragraph.
    #[serde(default)]
    pub continues_previous: bool,
    /// Verbatim Latin (plus Greek for front-matter prose). A `Verse` block keeps
    /// its line breaks as `\n`.
    pub text: String,
}

/// What a source page is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageRole {
    /// Ignored (blanks, marbling, covers, library slips, barcodes).
    Skip,
    /// Front matter reproduced as an embedded cropped scan (Google/QR, vignette).
    FrontImage,
    /// Front-matter prose re-typeset (Monitum, Praefatio, title page).
    FrontProse,
    /// A body page with the Greek/Latin columns.
    Body,
}

/// The reconciliation status of a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageStatus {
    /// Not a transcribed page (role `Skip` or `FrontImage`).
    NotBody,
    /// Both passes agreed and no cheap check fired; `result` is pass A.
    Agreed,
    /// The passes disagreed; Opus produced `result`.
    Arbitrated,
    /// The passes agreed but a cheap check fired; Opus produced `result`.
    Escalated,
}

/// The kind of a transcription block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    Heading,
    Chapter,
    Section,
    Paragraph,
    Verse,
}

/// One detected Life or Comparatio and the body pages it spans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeBoundary {
    pub id: String,
    pub title: String,
    pub kind: LifeKind,
    /// Pair-group number binding two Lives and their Comparatio.
    pub group: u32,
    /// Inclusive range of indices into `Sidecar::pages`.
    pub first_page: usize,
    pub last_page: usize,
}

/// Whether a unit is a Life or a Comparatio (synkrisis).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifeKind {
    Life,
    Comparatio,
}

/// A verification finding on a page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyFlag {
    pub kind: FlagKind,
    pub detail: String,
}

/// The kind of a verification finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagKind {
    /// A gap or reset in section/chapter numbering across the page boundary.
    SectionContinuity,
    /// The Tesseract-`lat` char count diverged coarsely from the Sonnet pass.
    OcrDivergence,
    /// The Sonnet and Haiku passes substantively disagreed under orthography
    /// folding, so the page was escalated to Opus.
    ModelDivergence,
    /// The Opus reply was implausibly short, so the Sonnet pass was kept.
    ArbiterFallback,
}

impl Sidecar {
    /// Read a sidecar from JSON.
    pub fn load(path: &Path) -> Result<Sidecar> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("reading sidecar {}", path.display()))?;
        let sidecar =
            serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        Ok(sidecar)
    }

    /// Write the sidecar as pretty JSON.
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = serde_json::to_string_pretty(self).context("serializing sidecar")?;
        fs::write(path, text).with_context(|| format!("writing sidecar {}", path.display()))?;
        Ok(())
    }

    /// Look up a page record by source index and 1-based source page, for
    /// resumable runs that must recognize an already-tagged page without
    /// re-rendering it.
    pub fn page(&self, source: usize, source_page: u32) -> Option<&PageRecord> {
        self.pages
            .iter()
            .find(|p| p.source == source && p.source_page == source_page)
    }

    /// Mutable page lookup, for the pipeline to update a record in place.
    pub fn page_mut(&mut self, source: usize, source_page: u32) -> Option<&mut PageRecord> {
        self.pages
            .iter_mut()
            .find(|p| p.source == source && p.source_page == source_page)
    }

    /// Whether any current source file's hash differs from the stored hash (or a
    /// source is missing), meaning the cache is stale.
    pub fn is_source_stale(&self, sources: &[&Path]) -> bool {
        if self.sources.len() != sources.len() {
            return true;
        }
        for (stored, path) in self.sources.iter().zip(sources) {
            match file_hash(path) {
                Ok(h) if h == stored.file_sha256 => {}
                _ => return true,
            }
        }
        false
    }
}

/// The cache key for a page: `(source file hash, source page, DPI)`. The key is
/// the source-page identity at a fixed DPI, so a resumed run recognizes an
/// already-tagged page without re-rasterizing it.
pub fn cache_key(sidecar: &Sidecar, record: &PageRecord) -> (String, u32, u32) {
    let file_sha256 = sidecar
        .sources
        .get(record.source)
        .map(|s| s.file_sha256.clone())
        .unwrap_or_default();
    (file_sha256, record.source_page, TRANSCRIBE_DPI)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> PageDoc {
        PageDoc {
            blocks: vec![
                Block {
                    kind: BlockKind::Chapter,
                    roman: Some("XI.".to_string()),
                    section_no: None,
                    continues_previous: false,
                    text: String::new(),
                },
                Block {
                    kind: BlockKind::Paragraph,
                    roman: None,
                    section_no: None,
                    continues_previous: true,
                    text: "Romulus igitur…".to_string(),
                },
            ],
        }
    }

    fn sample_sidecar() -> Sidecar {
        Sidecar {
            schema_version: SCHEMA_VERSION,
            sources: vec![
                SourcePdf {
                    path: "Ploutarchou_Bioi_v1.pdf".to_string(),
                    page_count: 643,
                    file_sha256: "aaa".to_string(),
                },
                SourcePdf {
                    path: "Ploutarchou_Bioi_v2.pdf".to_string(),
                    page_count: 722,
                    file_sha256: "bbb".to_string(),
                },
            ],
            front_matter: FrontMatter {
                title: "Plutarchi Vitae".to_string(),
                editor: "Theod. Dœhner".to_string(),
                date: "M DCCC LVII".to_string(),
            },
            pages: vec![
                PageRecord {
                    source: 0,
                    source_page: 14,
                    image_sha256: "img14".to_string(),
                    role: PageRole::Body,
                    pass_a: Some(sample_doc()),
                    pass_b: Some(sample_doc()),
                    status: PageStatus::Agreed,
                    flags: Vec::new(),
                    result: Some(sample_doc()),
                    result_model: Some(crate::llm::SONNET.to_string()),
                },
                PageRecord {
                    source: 1,
                    source_page: 1,
                    image_sha256: "img-v2-1".to_string(),
                    role: PageRole::Skip,
                    pass_a: None,
                    pass_b: None,
                    status: PageStatus::NotBody,
                    flags: Vec::new(),
                    result: None,
                    result_model: None,
                },
            ],
            lives: vec![LifeBoundary {
                id: "romulus".to_string(),
                title: "ROMULUS".to_string(),
                kind: LifeKind::Life,
                group: 1,
                first_page: 0,
                last_page: 0,
            }],
            batches: vec![BatchState {
                stage: "transcribe".to_string(),
                chunk: 0,
                batch_id: "msgbatch_abc".to_string(),
            }],
        }
    }

    #[test]
    fn schema_version_bumped() {
        // The compact-schema / batch-state shape is schema version 2.
        assert_eq!(SCHEMA_VERSION, 2);
    }

    #[test]
    fn sidecar_roundtrips_batches_and_result_model() {
        let sidecar = sample_sidecar();
        let json = serde_json::to_string_pretty(&sidecar).unwrap();
        let back: Sidecar = serde_json::from_str(&json).unwrap();
        assert_eq!(back.batches, sidecar.batches);
        assert_eq!(
            back.pages[0].result_model.as_deref(),
            Some(crate::llm::SONNET)
        );
        assert_eq!(back.pages[1].result_model, None);

        // An old sidecar without the new fields still deserializes, defaulting them.
        let legacy = r#"{
            "schema_version": 1,
            "sources": [],
            "front_matter": { "title": "", "editor": "", "date": "" },
            "pages": [],
            "lives": []
        }"#;
        let parsed: Sidecar = serde_json::from_str(legacy).unwrap();
        assert!(parsed.batches.is_empty());
    }

    #[test]
    fn sidecar_roundtrips_through_json() {
        let sidecar = sample_sidecar();
        let json = serde_json::to_string_pretty(&sidecar).unwrap();
        let back: Sidecar = serde_json::from_str(&json).unwrap();
        assert_eq!(sidecar, back);
    }

    #[test]
    fn page_lookup_by_source_and_page() {
        let sidecar = sample_sidecar();
        assert_eq!(sidecar.page(0, 14).unwrap().role, PageRole::Body);
        assert_eq!(sidecar.page(1, 1).unwrap().role, PageRole::Skip);
        assert!(sidecar.page(0, 999).is_none());
    }

    #[test]
    fn is_source_stale_detects_file_change() {
        let dir = tempfile::tempdir().unwrap();
        let v1 = dir.path().join("v1.pdf");
        let v2 = dir.path().join("v2.pdf");
        fs::write(&v1, b"one").unwrap();
        fs::write(&v2, b"two").unwrap();

        let mut sidecar = sample_sidecar();
        sidecar.sources[0].file_sha256 = file_hash(&v1).unwrap();
        sidecar.sources[1].file_sha256 = file_hash(&v2).unwrap();

        let paths: Vec<&Path> = vec![&v1, &v2];
        assert!(!sidecar.is_source_stale(&paths));

        fs::write(&v1, b"one changed").unwrap();
        assert!(sidecar.is_source_stale(&paths));
    }

    #[test]
    fn cache_key_uses_file_hash_page_and_dpi() {
        let sidecar = sample_sidecar();
        let key = cache_key(&sidecar, &sidecar.pages[0]);
        assert_eq!(key, ("aaa".to_string(), 14, TRANSCRIBE_DPI));
    }
}
