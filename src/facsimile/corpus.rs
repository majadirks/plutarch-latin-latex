//! The built-in canonical corpus of Plutarch's Parallel Lives in the Didot
//! edition's order. The detector reads only Latin from the page to recognize
//! *which* known unit a page belongs to; the Greek title, `kind`, pair-`group`, and
//! ordering come from here, turning an unreliable polytonic-Greek-OCR problem into a
//! reliable Latin-recognition one.
//!
//! Each unit lists the folded Latin tokens a page must carry to match it
//! ([`CanonUnit::match_tokens`]). A Life matches on its name; a Comparatio matches
//! on both paired names plus a `COMPARATIO` token, so a Comparatio page resolves to
//! the Comparatio rather than to one of its component Lives.

use crate::assemble::token_matches;
use crate::tags::LifeKind;
use crate::verify::fold;

/// One canonical Life or Comparatio: the metadata detection cannot read from the
/// page (Greek title, kind, group, source) plus the folded Latin `match_tokens` a
/// page must contain to be recognized as this unit.
#[derive(Debug, Clone, Copy)]
pub struct CanonUnit {
    pub id: &'static str,
    pub latin_title: &'static str,
    pub greek_title: &'static str,
    pub kind: LifeKind,
    /// Pair-group number binding two (or more) Lives and their Comparatio.
    pub group: u32,
    /// Index into the source PDFs (0 = v1, 1 = v2).
    pub source: usize,
    /// Upper-case Latin tokens a page must all contain (matched via [`token_matches`]
    /// after folding) to resolve to this unit.
    pub match_tokens: &'static [&'static str],
}

use LifeKind::{Comparatio, Life};

/// The Parallel Lives in Didot order: v1 (source 0) then v2 (source 1). Each
/// Life/Comparatio appears exactly once; a Comparatio shares its `group` with the
/// Lives it pairs.
pub fn corpus() -> &'static [CanonUnit] {
    &CORPUS
}

#[rustfmt::skip]
static CORPUS: [CanonUnit; 68] = [
    // ---- Volume 1 (source 0) ----
    CanonUnit { id: "theseus", latin_title: "THESEUS", greek_title: "ΘΗΣΕΥΣ", kind: Life, group: 1, source: 0, match_tokens: &["THESEUS"] },
    CanonUnit { id: "romulus", latin_title: "ROMULUS", greek_title: "ΡΩΜΥΛΟΣ", kind: Life, group: 1, source: 0, match_tokens: &["ROMULUS"] },
    CanonUnit { id: "comparatio_thesei_et_romuli", latin_title: "COMPARATIO THESEI ET ROMULI", greek_title: "ΣΥΓΚΡΙΣΙΣ ΘΗΣΕΩΣ ΚΑΙ ΡΩΜΥΛΟΥ", kind: Comparatio, group: 1, source: 0, match_tokens: &["THESEUS", "ROMULUS", "COMPARATIO"] },

    CanonUnit { id: "lycurgus", latin_title: "LYCURGUS", greek_title: "ΛΥΚΟΥΡΓΟΣ", kind: Life, group: 2, source: 0, match_tokens: &["LYCURGUS"] },
    CanonUnit { id: "numa", latin_title: "NUMA", greek_title: "ΝΟΥΜΑΣ", kind: Life, group: 2, source: 0, match_tokens: &["NUMA"] },
    CanonUnit { id: "comparatio_lycurgi_et_numae", latin_title: "LYCURGI ET NUMAE COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΛΥΚΟΥΡΓΟΥ ΚΑΙ ΝΟΥΜΑ", kind: Comparatio, group: 2, source: 0, match_tokens: &["LYCURGUS", "NUMA", "COMPARATIO"] },

    CanonUnit { id: "solon", latin_title: "SOLON", greek_title: "ΣΟΛΩΝ", kind: Life, group: 3, source: 0, match_tokens: &["SOLON"] },
    CanonUnit { id: "poplicola", latin_title: "POPLICOLA", greek_title: "ΠΟΠΛΙΚΟΛΑΣ", kind: Life, group: 3, source: 0, match_tokens: &["POPLICOLA"] },
    CanonUnit { id: "comparatio_solonis_et_poplicolae", latin_title: "SOLONIS ET POPLICOLAE COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΣΟΛΩΝΟΣ ΚΑΙ ΠΟΠΛΙΚΟΛΑ", kind: Comparatio, group: 3, source: 0, match_tokens: &["SOLON", "POPLICOLA", "COMPARATIO"] },

    CanonUnit { id: "themistocles", latin_title: "THEMISTOCLES", greek_title: "ΘΕΜΙΣΤΟΚΛΗΣ", kind: Life, group: 4, source: 0, match_tokens: &["THEMISTOCLES"] },
    CanonUnit { id: "camillus", latin_title: "CAMILLUS", greek_title: "ΚΑΜΙΛΛΟΣ", kind: Life, group: 4, source: 0, match_tokens: &["CAMILLUS"] },

    CanonUnit { id: "pericles", latin_title: "PERICLES", greek_title: "ΠΕΡΙΚΛΗΣ", kind: Life, group: 5, source: 0, match_tokens: &["PERICLES"] },
    CanonUnit { id: "fabius_maximus", latin_title: "FABIUS MAXIMUS", greek_title: "ΦΑΒΙΟΣ ΜΑΞΙΜΟΣ", kind: Life, group: 5, source: 0, match_tokens: &["FABIUS"] },
    CanonUnit { id: "comparatio_periclis_et_fabii", latin_title: "PERICLIS CUM FABIO COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΠΕΡΙΚΛΕΟΥΣ ΚΑΙ ΦΑΒΙΟΥ ΜΑΞΙΜΟΥ", kind: Comparatio, group: 5, source: 0, match_tokens: &["PERICLES", "FABIUS", "COMPARATIO"] },

    CanonUnit { id: "alcibiades", latin_title: "ALCIBIADES", greek_title: "ΑΛΚΙΒΙΑΔΗΣ", kind: Life, group: 6, source: 0, match_tokens: &["ALCIBIADES"] },
    CanonUnit { id: "marcius_coriolanus", latin_title: "MARCIUS CORIOLANUS", greek_title: "ΜΑΡΚΙΟΣ ΚΟΡΙΟΛΑΝΟΣ", kind: Life, group: 6, source: 0, match_tokens: &["MARCIUS", "CORIOLANUS"] },
    CanonUnit { id: "comparatio_alcibiadis_et_coriolani", latin_title: "ALCIBIADIS ET CORIOLANI COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΑΛΚΙΒΙΑΔΟΥ ΚΑΙ ΜΑΡΚΙΟΥ ΚΟΡΙΟΛΑΝΟΥ", kind: Comparatio, group: 6, source: 0, match_tokens: &["ALCIBIADES", "CORIOLANUS", "COMPARATIO"] },

    CanonUnit { id: "timoleon", latin_title: "TIMOLEON", greek_title: "ΤΙΜΟΛΕΩΝ", kind: Life, group: 7, source: 0, match_tokens: &["TIMOLEON"] },
    CanonUnit { id: "aemilius_paulus", latin_title: "AEMILIUS PAULUS", greek_title: "ΑΙΜΙΛΙΟΣ ΠΑΥΛΟΣ", kind: Life, group: 7, source: 0, match_tokens: &["PAULUS"] },
    CanonUnit { id: "comparatio_timoleontis_et_aemilii", latin_title: "TIMOLEONTIS CUM AEMILIO PAULO COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΤΙΜΟΛΕΟΝΤΟΣ ΚΑΙ ΑΙΜΙΛΙΟΥ ΠΑΥΛΟΥ", kind: Comparatio, group: 7, source: 0, match_tokens: &["TIMOLEON", "PAULUS", "COMPARATIO"] },

    CanonUnit { id: "pelopidas", latin_title: "PELOPIDAS", greek_title: "ΠΕΛΟΠΙΔΑΣ", kind: Life, group: 8, source: 0, match_tokens: &["PELOPIDAS"] },
    CanonUnit { id: "marcellus", latin_title: "MARCELLUS", greek_title: "ΜΑΡΚΕΛΛΟΣ", kind: Life, group: 8, source: 0, match_tokens: &["MARCELLUS"] },
    CanonUnit { id: "comparatio_pelopidae_et_marcelli", latin_title: "COMPARATIO PELOPIDAE CUM MARCELLO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΠΕΛΟΠΙΔΟΥ ΚΑΙ ΜΑΡΚΕΛΛΟΥ", kind: Comparatio, group: 8, source: 0, match_tokens: &["PELOPIDAS", "MARCELLUS", "COMPARATIO"] },

    CanonUnit { id: "aristides", latin_title: "ARISTIDES", greek_title: "ΑΡΙΣΤΕΙΔΗΣ", kind: Life, group: 9, source: 0, match_tokens: &["ARISTIDES"] },
    CanonUnit { id: "cato_major", latin_title: "CATO MAJOR", greek_title: "ΜΑΡΚΟΣ ΚΑΤΩΝ", kind: Life, group: 9, source: 0, match_tokens: &["CATO", "MAJOR"] },
    CanonUnit { id: "comparatio_aristidis_et_catonis", latin_title: "ARISTIDIS ET CATONIS MAJORIS COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΑΡΙΣΤΕΙΔΟΥ ΚΑΙ ΚΑΤΩΝΟΣ", kind: Comparatio, group: 9, source: 0, match_tokens: &["ARISTIDES", "CATO", "COMPARATIO"] },

    CanonUnit { id: "philopoemen", latin_title: "PHILOPOEMEN", greek_title: "ΦΙΛΟΠΟΙΜΗΝ", kind: Life, group: 10, source: 0, match_tokens: &["PHILOPOEMEN"] },
    CanonUnit { id: "flamininus", latin_title: "T. QUINCTIUS FLAMININUS", greek_title: "ΤΙΤΟΣ ΦΛΑΜΙΝΙΝΟΣ", kind: Life, group: 10, source: 0, match_tokens: &["FLAMININUS"] },
    CanonUnit { id: "comparatio_philopoemenis_et_flaminini", latin_title: "PHILOPOEMENIS ET T. FLAMININI COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΦΙΛΟΠΟΙΜΕΝΟΣ ΚΑΙ ΤΙΤΟΥ ΦΛΑΜΙΝΙΝΟΥ", kind: Comparatio, group: 10, source: 0, match_tokens: &["PHILOPOEMEN", "FLAMININUS", "COMPARATIO"] },

    CanonUnit { id: "pyrrhus", latin_title: "PYRRHUS", greek_title: "ΠΥΡΡΟΣ", kind: Life, group: 11, source: 0, match_tokens: &["PYRRHUS"] },
    CanonUnit { id: "caius_marius", latin_title: "C. MARIUS", greek_title: "ΓΑΙΟΣ ΜΑΡΙΟΣ", kind: Life, group: 11, source: 0, match_tokens: &["MARIUS"] },

    CanonUnit { id: "lysander", latin_title: "LYSANDER", greek_title: "ΛΥΣΑΝΔΡΟΣ", kind: Life, group: 12, source: 0, match_tokens: &["LYSANDER"] },
    CanonUnit { id: "sylla", latin_title: "SYLLA", greek_title: "ΣΥΛΛΑΣ", kind: Life, group: 12, source: 0, match_tokens: &["SYLLA"] },
    CanonUnit { id: "comparatio_lysandri_et_syllae", latin_title: "COMPARATIO LYSANDRI CUM SYLLA", greek_title: "ΣΥΓΚΡΙΣΙΣ ΛΥΣΑΝΔΡΟΥ ΚΑΙ ΣΥΛΛΑ", kind: Comparatio, group: 12, source: 0, match_tokens: &["LYSANDER", "SYLLA", "COMPARATIO"] },

    CanonUnit { id: "cimon", latin_title: "CIMON", greek_title: "ΚΙΜΩΝ", kind: Life, group: 13, source: 0, match_tokens: &["CIMON"] },
    CanonUnit { id: "lucullus", latin_title: "LUCULLUS", greek_title: "ΛΟΥΚΟΥΛΛΟΣ", kind: Life, group: 13, source: 0, match_tokens: &["LUCULLUS"] },
    CanonUnit { id: "comparatio_cimonis_et_luculli", latin_title: "COMPARATIO CIMONIS ET LUCULLI", greek_title: "ΣΥΓΚΡΙΣΙΣ ΚΙΜΩΝΟΣ ΚΑΙ ΛΟΥΚΟΥΛΛΟΥ", kind: Comparatio, group: 13, source: 0, match_tokens: &["CIMON", "LUCULLUS", "COMPARATIO"] },

    // ---- Volume 2 (source 1) ----
    CanonUnit { id: "nicias", latin_title: "NICIAS", greek_title: "ΝΙΚΙΑΣ", kind: Life, group: 14, source: 1, match_tokens: &["NICIAS"] },
    CanonUnit { id: "crassus", latin_title: "CRASSUS", greek_title: "ΚΡΑΣΣΟΣ", kind: Life, group: 14, source: 1, match_tokens: &["CRASSUS"] },
    CanonUnit { id: "comparatio_niciae_et_crassi", latin_title: "NICIAE CUM CRASSO COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΝΙΚΙΟΥ ΚΑΙ ΚΡΑΣΣΟΥ", kind: Comparatio, group: 14, source: 1, match_tokens: &["NICIAS", "CRASSUS", "COMPARATIO"] },

    CanonUnit { id: "sertorius", latin_title: "SERTORIUS", greek_title: "ΣΕΡΤΩΡΙΟΣ", kind: Life, group: 15, source: 1, match_tokens: &["SERTORIUS"] },
    CanonUnit { id: "eumenes", latin_title: "EUMENES", greek_title: "ΕΥΜΕΝΗΣ", kind: Life, group: 15, source: 1, match_tokens: &["EUMENES"] },
    CanonUnit { id: "comparatio_sertorii_et_eumenis", latin_title: "SERTORII CUM EUMENE COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΣΕΡΤΩΡΙΟΥ ΚΑΙ ΕΥΜΕΝΟΥΣ", kind: Comparatio, group: 15, source: 1, match_tokens: &["SERTORIUS", "EUMENES", "COMPARATIO"] },

    CanonUnit { id: "agesilaus", latin_title: "AGESILAUS", greek_title: "ΑΓΗΣΙΛΑΟΣ", kind: Life, group: 16, source: 1, match_tokens: &["AGESILAUS"] },
    CanonUnit { id: "pompeius", latin_title: "POMPEIUS", greek_title: "ΠΟΜΠΗΙΟΣ", kind: Life, group: 16, source: 1, match_tokens: &["POMPEIUS"] },
    CanonUnit { id: "comparatio_agesilai_et_pompeii", latin_title: "AGESILAI ET POMPEI COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΑΓΗΣΙΛΑΟΥ ΚΑΙ ΠΟΜΠΗΙΟΥ", kind: Comparatio, group: 16, source: 1, match_tokens: &["AGESILAUS", "POMPEIUS", "COMPARATIO"] },

    CanonUnit { id: "alexander", latin_title: "ALEXANDER", greek_title: "ΑΛΕΞΑΝΔΡΟΣ", kind: Life, group: 17, source: 1, match_tokens: &["ALEXANDER"] },
    CanonUnit { id: "iulius_caesar", latin_title: "C. JULIUS CAESAR", greek_title: "ΓΑΙΟΣ ΙΟΥΛΙΟΣ ΚΑΙΣΑΡ", kind: Life, group: 17, source: 1, match_tokens: &["CAESAR"] },

    CanonUnit { id: "phocion", latin_title: "PHOCION", greek_title: "ΦΩΚΙΩΝ", kind: Life, group: 18, source: 1, match_tokens: &["PHOCION"] },
    CanonUnit { id: "cato_minor", latin_title: "CATO MINOR", greek_title: "ΚΑΤΩΝ Ο ΝΕΩΤΕΡΟΣ", kind: Life, group: 18, source: 1, match_tokens: &["CATO", "MINOR"] },

    CanonUnit { id: "agis", latin_title: "AGIS", greek_title: "ΑΓΙΣ", kind: Life, group: 19, source: 1, match_tokens: &["AGIS"] },
    CanonUnit { id: "cleomenes", latin_title: "CLEOMENES", greek_title: "ΚΛΕΟΜΕΝΗΣ", kind: Life, group: 19, source: 1, match_tokens: &["CLEOMENES"] },
    CanonUnit { id: "tiberius_gracchus", latin_title: "TIB. GRACCHUS", greek_title: "ΤΙΒΕΡΙΟΣ ΓΡΑΚΧΟΣ", kind: Life, group: 19, source: 1, match_tokens: &["TIB", "GRACCHUS"] },
    CanonUnit { id: "caius_gracchus", latin_title: "C. GRACCHUS", greek_title: "ΓΑΙΟΣ ΓΡΑΚΧΟΣ", kind: Life, group: 19, source: 1, match_tokens: &["GRACCHUS"] },
    CanonUnit { id: "comparatio_agidis_cleomenis_et_gracchorum", latin_title: "AGIDIS ET CLEOMENIS CUM GRACCHIS COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΑΓΙΔΟΣ ΚΑΙ ΚΛΕΟΜΕΝΟΥΣ ΚΑΙ ΓΡΑΚΧΩΝ", kind: Comparatio, group: 19, source: 1, match_tokens: &["CLEOMENES", "GRACCHUS", "COMPARATIO"] },

    CanonUnit { id: "demosthenes", latin_title: "DEMOSTHENES", greek_title: "ΔΗΜΟΣΘΕΝΗΣ", kind: Life, group: 20, source: 1, match_tokens: &["DEMOSTHENES"] },
    CanonUnit { id: "cicero", latin_title: "CICERO", greek_title: "ΚΙΚΕΡΩΝ", kind: Life, group: 20, source: 1, match_tokens: &["CICERO"] },
    CanonUnit { id: "comparatio_demosthenis_et_ciceronis", latin_title: "COMPARATIO DEMOSTHENIS CUM CICERONE", greek_title: "ΣΥΓΚΡΙΣΙΣ ΔΗΜΟΣΘΕΝΟΥΣ ΚΑΙ ΚΙΚΕΡΩΝΟΣ", kind: Comparatio, group: 20, source: 1, match_tokens: &["DEMOSTHENES", "CICERO", "COMPARATIO"] },

    CanonUnit { id: "demetrius", latin_title: "DEMETRIUS", greek_title: "ΔΗΜΗΤΡΙΟΣ", kind: Life, group: 21, source: 1, match_tokens: &["DEMETRIUS"] },
    CanonUnit { id: "antonius", latin_title: "ANTONIUS", greek_title: "ΑΝΤΩΝΙΟΣ", kind: Life, group: 21, source: 1, match_tokens: &["ANTONIUS"] },
    CanonUnit { id: "comparatio_demetrii_et_antonii", latin_title: "DEMETRII CUM ANTONIO COMPARATIO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΔΗΜΗΤΡΙΟΥ ΚΑΙ ΑΝΤΩΝΙΟΥ", kind: Comparatio, group: 21, source: 1, match_tokens: &["DEMETRIUS", "ANTONIUS", "COMPARATIO"] },

    CanonUnit { id: "dio", latin_title: "DIO", greek_title: "ΔΙΩΝ", kind: Life, group: 22, source: 1, match_tokens: &["DIO"] },
    CanonUnit { id: "brutus", latin_title: "BRUTUS", greek_title: "ΒΡΟΥΤΟΣ", kind: Life, group: 22, source: 1, match_tokens: &["BRUTUS"] },
    CanonUnit { id: "comparatio_dionis_et_bruti", latin_title: "COMPARATIO DIONIS CUM BRUTO", greek_title: "ΣΥΓΚΡΙΣΙΣ ΔΙΩΝΟΣ ΚΑΙ ΒΡΟΥΤΟΥ", kind: Comparatio, group: 22, source: 1, match_tokens: &["DIONIS", "BRUTUS", "COMPARATIO"] },

    CanonUnit { id: "artoxerxes", latin_title: "ARTOXERXES", greek_title: "ΑΡΤΟΞΕΡΞΗΣ", kind: Life, group: 23, source: 1, match_tokens: &["ARTOXERXES"] },
    CanonUnit { id: "aratus", latin_title: "ARATUS", greek_title: "ΑΡΑΤΟΣ", kind: Life, group: 24, source: 1, match_tokens: &["ARATUS"] },
    CanonUnit { id: "galba", latin_title: "GALBA", greek_title: "ΓΑΛΒΑΣ", kind: Life, group: 25, source: 1, match_tokens: &["GALBA"] },
    CanonUnit { id: "otho", latin_title: "OTHO", greek_title: "ΟΘΩΝ", kind: Life, group: 26, source: 1, match_tokens: &["OTHO"] },
];

/// Resolve a page's folded title tokens to the unique corpus unit all of whose
/// `match_tokens` are present. When several units match (a Comparatio page also
/// names its component Lives), the one requiring the most tokens wins — the
/// Comparatio — so it never resolves to a component Life. Returns `None` when no
/// unit matches or the best match is not unique.
pub fn match_page(page_tokens: &[String]) -> Option<usize> {
    let matches: Vec<(usize, usize)> = corpus()
        .iter()
        .enumerate()
        .filter_map(|(i, unit)| {
            if unit.match_tokens.is_empty() {
                return None;
            }
            let all = unit.match_tokens.iter().all(|t| {
                let folded = fold(t);
                page_tokens.iter().any(|p| token_matches(p, &folded))
            });
            all.then_some((i, unit.match_tokens.len()))
        })
        .collect();
    let max = matches.iter().map(|(_, count)| *count).max()?;
    let top: Vec<usize> = matches
        .iter()
        .filter(|(_, count)| *count == max)
        .map(|(i, _)| *i)
        .collect();
    (top.len() == 1).then(|| top[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| fold(w)).collect()
    }

    fn find(id: &str) -> usize {
        corpus().iter().position(|u| u.id == id).unwrap()
    }

    #[test]
    fn corpus_is_ordered_and_grouped() {
        let c = corpus();
        // Sources are non-decreasing (all v1 before all v2).
        for pair in c.windows(2) {
            assert!(
                pair[1].source >= pair[0].source,
                "source order breaks at {:?}",
                pair[1].id
            );
        }
        // Ids are unique.
        let mut ids: Vec<&str> = c.iter().map(|u| u.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), c.len(), "duplicate id in corpus");
        // Each group has at least one Life and at most one Comparatio.
        let groups: Vec<u32> = {
            let mut g: Vec<u32> = c.iter().map(|u| u.group).collect();
            g.sort_unstable();
            g.dedup();
            g
        };
        for g in groups {
            let lives = c.iter().filter(|u| u.group == g && u.kind == Life).count();
            let comps = c
                .iter()
                .filter(|u| u.group == g && u.kind == Comparatio)
                .count();
            assert!(lives >= 1, "group {g} has no Life");
            assert!(comps <= 1, "group {g} has more than one Comparatio");
        }
        // v1 ends on a Comparatio; v2 opens with a Life.
        let last_v1 = c.iter().rfind(|u| u.source == 0).unwrap();
        assert_eq!(last_v1.kind, Comparatio, "v1 must end on a Comparatio");
        let first_v2 = c.iter().find(|u| u.source == 1).unwrap();
        assert_eq!(first_v2.kind, Life, "v2 must open with a Life");
    }

    #[test]
    fn match_page_resolves_life() {
        assert_eq!(match_page(&toks(&["ROMULUS"])), Some(find("romulus")));
    }

    #[test]
    fn match_page_resolves_comparatio() {
        // A Comparatio page names both Lives plus COMPARATIO; it must resolve to the
        // Comparatio, not Lycurgus or Numa.
        let got = match_page(&toks(&["LYCURGI", "NUMAE", "COMPARATIO"]));
        assert_eq!(got, Some(find("comparatio_lycurgi_et_numae")));
    }

    #[test]
    fn match_page_none_on_unknown() {
        assert_eq!(match_page(&toks(&["XYZZY", "QVID"])), None);
    }

    #[test]
    fn match_page_resolves_short_name_dio() {
        // DIO is too short to reach a four-character prefix; exact-token matching
        // still resolves it, and the Comparatio page resolves to the Comparatio.
        assert_eq!(match_page(&toks(&["DIO"])), Some(find("dio")));
        assert_eq!(
            match_page(&toks(&["DIONIS", "BRUTO", "COMPARATIO"])),
            Some(find("comparatio_dionis_et_bruti"))
        );
    }
}
