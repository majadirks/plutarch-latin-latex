//! Signature-padding arithmetic: the inner book pages are bound in 32-page
//! signatures, so each volume's inner page count is rounded up to a whole
//! signature (with the final verso left blank).

/// Book pages in one signature (8 sheets, 4 book pages per sheet).
pub const PAGES_PER_SIGNATURE: u32 = 32;

/// Round a page count up to the next whole signature.
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
