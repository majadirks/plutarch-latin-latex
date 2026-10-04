//! Loads the Anthropic API key from a local `secrets.json` (kept out of shared
//! copies; see the README).

use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

/// Credentials read from `secrets.json`.
#[derive(Debug, Deserialize)]
pub struct Secrets {
    pub anthropic_api_key: String,
}

/// Read and parse `secrets.json` at `path`, reporting a clear error if the file
/// is missing or malformed.
pub fn load(path: &Path) -> anyhow::Result<Secrets> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read secrets file at {}", path.display()))?;
    let secrets: Secrets = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse secrets file at {}", path.display()))?;
    Ok(secrets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn secrets_load_reads_key() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        write!(file, r#"{{"anthropic_api_key": "sk-ant-test-123"}}"#).unwrap();
        let secrets = load(file.path()).unwrap();
        assert_eq!(secrets.anthropic_api_key, "sk-ant-test-123");
    }

    #[test]
    fn secrets_load_missing_file_errors() {
        let path = Path::new("this/file/does/not/exist.json");
        assert!(load(path).is_err());
    }
}
