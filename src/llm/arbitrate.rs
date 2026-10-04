//! The Opus escalation prompt pieces: the authoritative-arbiter system prompt and
//! a user prompt that presents both candidate transcriptions. The batched Opus
//! escalation in [`crate::verify`] reuses these to re-decide a flagged page's Latin
//! from the image.

use anyhow::{Context, Result};

use crate::tags::PageDoc;

/// Output ceiling for an Opus escalation: it must cover a full-page transcription
/// with headroom.
pub(crate) const MAX_TOKENS: u32 = 24000;

pub(crate) const SYSTEM_PROMPT: &str = "You are the authoritative arbiter for a faithful \
transcription of a printed nineteenth-century Plutarch (Didot, Paris 1857). Two \
transcribers produced candidate transcriptions of the Latin (right) column of the \
page image; they disagree or a consistency check flagged the page. Decide the \
correct text from the IMAGE, exactly as printed — never normalize spelling, u/v, \
i/j, ligatures, the long s, or punctuation, and never translate the Greek. The \
Greek column is available in the image only as a disambiguation aid. Return only \
the requested JSON object.";

/// The arbitration user prompt embedding both candidates. Exposed so tests can
/// assert both candidates are presented.
pub fn user_prompt(pass_a: &PageDoc, pass_b: &PageDoc) -> Result<String> {
    let a = serde_json::to_string_pretty(pass_a).context("serializing candidate A")?;
    let b = serde_json::to_string_pretty(pass_b).context("serializing candidate B")?;
    Ok(format!(
        "Candidate A (first transcriber):\n{a}\n\n\
         Candidate B (second transcriber):\n{b}\n\n\
         Produce the single correct, faithful transcription of the Latin column as a \
         PageDoc, resolving every disagreement against the printed image."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags::{Block, BlockKind};

    fn doc(text: &str) -> PageDoc {
        PageDoc {
            blocks: vec![Block {
                kind: BlockKind::Paragraph,
                roman: None,
                section_no: None,
                continues_previous: false,
                text: text.to_string(),
            }],
        }
    }

    #[test]
    fn user_prompt_presents_both_candidates() {
        let p = user_prompt(&doc("alpha text"), &doc("beta text")).unwrap();
        assert!(p.contains("Candidate A"));
        assert!(p.contains("Candidate B"));
        assert!(p.contains("alpha text"));
        assert!(p.contains("beta text"));
    }
}
