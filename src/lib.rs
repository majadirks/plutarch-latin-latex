//! Library backing the Plutarch Latin reader generator.
//!
//! The binary is a thin shell over these modules; keeping the logic in a
//! library lets the test suite exercise every stage without going through the
//! command line.

use base64::Engine as _;

/// Base64-encode bytes for an Anthropic image content block.
pub fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub mod assemble;
pub mod compile;
pub mod facsimile;
pub mod latex;
pub mod llm;
pub mod pages;
pub mod pipeline;
pub mod render;
pub mod secrets;
pub mod signatures;
pub mod tags;
pub mod verify;
pub mod volumes;

pub use pipeline::{run, RunArgs};
