//! Volume-budgeting arithmetic: a volume's estimated page count is rounded up to
//! a 32-page unit when deciding which Lives land in which tomus. This 32 is a
//! *budgeting* granularity, held fixed so volume boundaries stay stable; it is
//! deliberately decoupled from the physical binding signature, which is 16 pages
//! (see `PAGES_PER_SIGNATURE` in `src/latex.rs`) and whose final signature may be
//! truncated. Keeping this rounding at 32 means the estimate here slightly
//! over-counts the true physical page total, which is harmless: it only feeds the
//! volume-fullness cap, never the real page count (that comes from the compiled
//! PDF).

/// Book pages in one volume-budgeting unit. This is the planning granularity used
/// to size volumes, not the physical binding signature (16 pages, in `src/latex.rs`).
pub const PAGES_PER_SIGNATURE: u32 = 32;

/// Round an estimated page count up to the next whole 32-page budgeting unit.
pub fn pad_to_multiple_of_32(pages: u32) -> u32 {
    pages.div_ceil(PAGES_PER_SIGNATURE) * PAGES_PER_SIGNATURE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_to_multiple_of_32_from_various_counts() {
        assert_eq!(pad_to_multiple_of_32(1), 32);
        assert_eq!(pad_to_multiple_of_32(32), 32);
        assert_eq!(pad_to_multiple_of_32(33), 64);
        assert_eq!(pad_to_multiple_of_32(500), 512);
    }
}
