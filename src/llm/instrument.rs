//! Token-usage accounting and the shared progress-bar style. The tracker threads
//! through the orchestration as `&UsageTracker` so it never conflicts with the
//! `&mut Sidecar` the same loops hold, and the end-of-run report prints per-model
//! call counts and token totals.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::AddAssign;

use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

/// Input/output token counts from one Messages-API reply.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl Usage {
    /// Total tokens, input plus output.
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

impl AddAssign for Usage {
    fn add_assign(&mut self, rhs: Usage) {
        self.input_tokens += rhs.input_tokens;
        self.output_tokens += rhs.output_tokens;
    }
}

/// Which page a call was spent on: source index plus 1-based source page,
/// matching the `PageRecord` key.
pub type PageId = (usize, u32);

/// One recorded API call: the model, the page it was spent on, and its usage.
#[derive(Debug, Clone)]
struct CallRecord {
    model: String,
    page: PageId,
    usage: Usage,
}

/// Accumulates one `CallRecord` per API call, behind a `RefCell` so it can be
/// shared as `&UsageTracker` across loops that mutate the sidecar.
#[derive(Debug, Default)]
pub struct UsageTracker {
    calls: RefCell<Vec<CallRecord>>,
}

impl UsageTracker {
    /// A tracker with no recorded calls.
    pub fn new() -> Self {
        UsageTracker::default()
    }

    /// Record one API call against a model and page.
    pub fn record(&self, model: &str, page: PageId, usage: Usage) {
        self.calls.borrow_mut().push(CallRecord {
            model: model.to_string(),
            page,
            usage,
        });
    }

    /// Total number of recorded calls across all models.
    pub fn call_count(&self) -> usize {
        self.calls.borrow().len()
    }

    /// Group the recorded calls by model and summarize each.
    pub fn report(&self) -> UsageReport {
        let calls = self.calls.borrow();

        // Model -> (call count, total usage, per-page summed usage).
        let mut by_model: BTreeMap<String, ModelAccum> = BTreeMap::new();
        for call in calls.iter() {
            let accum = by_model.entry(call.model.clone()).or_default();
            accum.calls += 1;
            accum.total += call.usage;
            *accum.per_page.entry(call.page).or_default() += call.usage;
        }

        let models = by_model
            .into_iter()
            .map(|(model, accum)| {
                let mut page_totals: Vec<u64> = accum.per_page.values().map(Usage::total).collect();
                page_totals.sort_unstable();
                ModelUsage {
                    model,
                    calls: accum.calls,
                    input_tokens: accum.total.input_tokens,
                    output_tokens: accum.total.output_tokens,
                    total_tokens: accum.total.total(),
                    pages: page_totals.len(),
                    median_per_page: median(&page_totals),
                    max_per_page: page_totals.last().copied().unwrap_or(0),
                    min_per_page: page_totals.first().copied().unwrap_or(0),
                }
            })
            .collect();

        UsageReport { models }
    }
}

#[derive(Default)]
struct ModelAccum {
    calls: u64,
    total: Usage,
    per_page: BTreeMap<PageId, Usage>,
}

/// Median of a sorted, non-empty slice; 0 for an empty slice.
fn median(sorted: &[u64]) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) / 2
    }
}

/// Per-model usage summary over a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelUsage {
    pub model: String,
    pub calls: u64,
    /// Summed input and output tokens, kept separate so the cost gate can apply the
    /// differing input/output reference prices by hand.
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    /// Distinct pages this model was called on.
    pub pages: usize,
    pub median_per_page: u64,
    pub max_per_page: u64,
    pub min_per_page: u64,
}

/// The end-of-run usage report: one row per model that was called. Models never
/// called are omitted; a fully-cached offline run yields an empty report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageReport {
    pub models: Vec<ModelUsage>,
}

impl fmt::Display for UsageReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{:<18} {:>6} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
            "model",
            "calls",
            "input tok",
            "output tok",
            "total tok",
            "median/pg",
            "max/pg",
            "min/pg"
        )?;
        if self.models.is_empty() {
            write!(f, "(no API calls this run)")?;
            return Ok(());
        }
        for (i, m) in self.models.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(
                f,
                "{:<18} {:>6} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
                m.model,
                m.calls,
                m.input_tokens,
                m.output_tokens,
                m.total_tokens,
                m.median_per_page,
                m.max_per_page,
                m.min_per_page
            )?;
        }
        Ok(())
    }
}

/// Build a tqdm-style progress bar with one shared style, so classification,
/// transcription, and arbitration all look alike. Draws to stderr and hides
/// itself when stderr is not a terminal (piped output, or under `cargo test`).
pub fn progress_bar(len: u64, label: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "{prefix} [{bar:40.cyan/blue}] {pos}/{len} [{elapsed_precise}<{eta_precise}] {msg}",
        )
        .expect("static progress template is valid")
        .progress_chars("=>-"),
    );
    pb.set_prefix(label.to_string());
    pb
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(input: u64, output: u64) -> Usage {
        Usage {
            input_tokens: input,
            output_tokens: output,
        }
    }

    #[test]
    fn usage_report_counts_calls_per_model() {
        let t = UsageTracker::new();
        t.record("haiku", (0, 1), usage(10, 2));
        t.record("haiku", (0, 2), usage(10, 2));
        t.record("sonnet", (0, 1), usage(100, 20));
        let report = t.report();
        let haiku = report.models.iter().find(|m| m.model == "haiku").unwrap();
        let sonnet = report.models.iter().find(|m| m.model == "sonnet").unwrap();
        assert_eq!(haiku.calls, 2);
        assert_eq!(sonnet.calls, 1);
        assert_eq!(haiku.total_tokens, 24);
        assert_eq!(sonnet.total_tokens, 120);
        // Input and output are tracked separately for the cost gate.
        assert_eq!(haiku.input_tokens, 20);
        assert_eq!(haiku.output_tokens, 4);
        assert_eq!(sonnet.input_tokens, 100);
        assert_eq!(sonnet.output_tokens, 20);
    }

    #[test]
    fn usage_report_computes_per_page_median_max_min() {
        let t = UsageTracker::new();
        // Three pages with per-page totals 12, 112, 1012.
        t.record("sonnet", (0, 1), usage(10, 2));
        t.record("sonnet", (0, 2), usage(100, 12));
        t.record("sonnet", (0, 3), usage(1000, 12));
        let report = t.report();
        let m = &report.models[0];
        assert_eq!(m.pages, 3);
        assert_eq!(m.median_per_page, 112);
        assert_eq!(m.max_per_page, 1012);
        assert_eq!(m.min_per_page, 12);
    }

    #[test]
    fn usage_report_sums_multiple_calls_on_one_page() {
        let t = UsageTracker::new();
        t.record("opus", (0, 5), usage(10, 2));
        t.record("opus", (0, 5), usage(20, 3));
        let report = t.report();
        let m = &report.models[0];
        // Two calls collapse to a single per-page total of 35.
        assert_eq!(m.calls, 2);
        assert_eq!(m.pages, 1);
        assert_eq!(m.median_per_page, 35);
        assert_eq!(m.max_per_page, 35);
        assert_eq!(m.min_per_page, 35);
    }

    #[test]
    fn usage_report_omits_uncalled_models() {
        let t = UsageTracker::new();
        t.record("haiku", (0, 1), usage(10, 2));
        let report = t.report();
        assert_eq!(report.models.len(), 1);
        assert_eq!(report.models[0].model, "haiku");
    }

    #[test]
    fn empty_report_lists_no_models() {
        let t = UsageTracker::new();
        let report = t.report();
        assert!(report.models.is_empty());
        assert!(report.to_string().contains("no API calls"));
    }

    #[test]
    fn progress_bar_has_expected_length() {
        let pb = progress_bar(128, "transcribing");
        assert_eq!(pb.length(), Some(128));
    }

    #[test]
    fn usage_add_assign_sums_fields() {
        let mut a = usage(1, 2);
        a += usage(3, 4);
        assert_eq!(a, usage(4, 6));
        assert_eq!(a.total(), 10);
    }
}
