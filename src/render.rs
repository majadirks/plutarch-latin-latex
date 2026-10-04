//! Rasterizes source PDF pages to PNG via poppler `pdftoppm` and hashes the
//! rendered bytes for the transcription cache.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use sha2::{Digest, Sha256};

/// DPI at which body pages are rasterized for transcription. Higher is more
/// legible but costs more vision input tokens; tunable after the first real run.
pub const TRANSCRIBE_DPI: u32 = 300;

/// DPI for the cheap page-role classification thumbnails. Low, because the
/// classifier only needs the page's gross shape, not its text.
pub const THUMBNAIL_DPI: u32 = 50;

/// Fraction of a body page's width lying left of the Latin column. Body pages are
/// Greek on the left and Latin on the right, so the rightmost `1 − CROP_LEFT_FRAC`
/// of the width is what gets transcribed. Tuned by eye; if the gutter sits
/// differently between the two volumes this may need to become a per-source pair.
pub const CROP_LEFT_FRAC: f32 = 0.48;

/// The `pdftoppm` binary, overridable with `PLUTARCH_PDFTOPPM` (default
/// `pdftoppm` on `PATH`).
fn pdftoppm_bin() -> String {
    std::env::var("PLUTARCH_PDFTOPPM").unwrap_or_else(|_| "pdftoppm".to_string())
}

/// The `pdfinfo` binary, overridable with `PLUTARCH_PDFINFO`.
fn pdfinfo_bin() -> String {
    std::env::var("PLUTARCH_PDFINFO").unwrap_or_else(|_| "pdfinfo".to_string())
}

/// The `pdftotext` binary, overridable with `PLUTARCH_PDFTOTEXT` (default
/// `pdftotext` on `PATH`).
fn pdftotext_bin() -> String {
    std::env::var("PLUTARCH_PDFTOTEXT").unwrap_or_else(|_| "pdftotext".to_string())
}

/// Whether `pdftotext` is callable, so detection tests can gate on it.
pub fn pdftotext_available() -> bool {
    Command::new(pdftotext_bin())
        .arg("-v")
        .output()
        .map(|o| o.status.success() || !o.stderr.is_empty())
        .unwrap_or(false)
}

/// Read one 1-based `page` of `pdf`'s embedded OCR text layer via
/// `pdftotext -f page -l page pdf -`. Returns an empty string when the binary is
/// absent or the call fails, so a missing text layer surfaces as a reported gap in
/// detection rather than a hard error.
pub fn page_text(pdf: &Path, page: u32) -> Result<String> {
    let page_arg = page.to_string();
    // `-enc UTF-8` so ligatures and accents (Æ, Œ) arrive as valid UTF-8 rather than
    // raw Latin-1 bytes, which lossy decoding would turn into replacement characters
    // and lose before folding.
    let output = match Command::new(pdftotext_bin())
        .arg("-enc")
        .arg("UTF-8")
        .arg("-f")
        .arg(&page_arg)
        .arg("-l")
        .arg(&page_arg)
        .arg(pdf)
        .arg("-")
        .output()
    {
        Ok(output) => output,
        Err(_) => return Ok(String::new()),
    };
    if !output.status.success() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// One OCR word with its page-fraction box (origin top-left, y down).
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

/// Read normalized OCR boxes, returning no words when poppler or text is absent.
pub fn page_words(pdf: &Path, page: u32) -> Result<Vec<Word>> {
    let p = page.to_string();
    let output = match Command::new(pdftotext_bin())
        .args(["-bbox", "-enc", "UTF-8", "-f", &p, "-l", &p])
        .arg(pdf)
        .arg("-")
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return Ok(Vec::new()),
    };
    Ok(parse_bbox(&String::from_utf8_lossy(&output.stdout)))
}

fn bbox_attr(tag: &str, name: &str) -> Option<f32> {
    let prefix = format!("{name}=\"");
    let value = tag.split_once(&prefix)?.1.split_once('"')?.0;
    value.parse::<f32>().ok().filter(|v| v.is_finite())
}

fn parse_bbox(xml: &str) -> Vec<Word> {
    let Some(page) = xml
        .split_once("<page ")
        .and_then(|(_, s)| s.split_once('>'))
    else {
        return Vec::new();
    };
    let Some((w, h)) = bbox_attr(page.0, "width").zip(bbox_attr(page.0, "height")) else {
        return Vec::new();
    };
    if w <= 0.0 || h <= 0.0 {
        return Vec::new();
    }
    page.1
        .split("</page>")
        .next()
        .unwrap_or("")
        .split("<word ")
        .skip(1)
        .filter_map(|s| {
            let (tag, rest) = s.split_once('>')?;
            let text = rest.split_once("</word>")?.0;
            Some(Word {
                text: text
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&quot;", "\"")
                    .replace("&apos;", "'")
                    .replace("&amp;", "&"),
                x_min: bbox_attr(tag, "xMin")? / w,
                y_min: bbox_attr(tag, "yMin")? / h,
                x_max: bbox_attr(tag, "xMax")? / w,
                y_max: bbox_attr(tag, "yMax")? / h,
            })
        })
        .collect()
}

/// The per-page title candidate: from the first few non-empty lines, the ALL-CAPS
/// title tokens (folio digits and punctuation-only tokens dropped), joined by
/// spaces. On a body page this is the running head that names the current unit.
pub fn running_head(page_text: &str) -> String {
    let mut tokens = Vec::new();
    for line in page_text.lines().filter(|l| !l.trim().is_empty()).take(3) {
        for word in line.split_whitespace() {
            if let Some(token) = title_token(word) {
                tokens.push(token);
            }
        }
    }
    tokens.join(" ")
}

/// A single title token: the word with its leading/trailing non-alphanumerics
/// stripped, kept only when it is non-empty, not a pure-digit folio, and its
/// letters are all upper-case (so `THESEUS.` and `NUMÆ` survive, `115` and `,` do
/// not).
pub fn title_token(word: &str) -> Option<String> {
    let cleaned: String = word
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_string();
    if cleaned.is_empty() {
        return None;
    }
    if cleaned.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let letters: Vec<char> = cleaned.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return None;
    }
    if letters.iter().all(|c| c.is_uppercase()) {
        Some(cleaned)
    } else {
        None
    }
}

/// Whether `pdftoppm` is callable, so render tests can gate on it.
pub fn pdftoppm_available() -> bool {
    Command::new(pdftoppm_bin())
        .arg("-h")
        .output()
        .map(|o| o.status.success() || !o.stderr.is_empty())
        .unwrap_or(false)
}

/// A source PDF handle: its path and page count.
#[derive(Debug, Clone)]
pub struct Pdf {
    pub path: PathBuf,
    pub page_count: u32,
}

impl Pdf {
    /// Open a source PDF, reading its page count from `pdfinfo`.
    pub fn open(path: &Path) -> Result<Pdf> {
        let page_count = page_count(path)?;
        Ok(Pdf {
            path: path.to_path_buf(),
            page_count,
        })
    }
}

/// Read a PDF's page count from `pdfinfo`'s `Pages:` line.
pub fn page_count(pdf: &Path) -> Result<u32> {
    let output = Command::new(pdfinfo_bin())
        .arg(pdf)
        .output()
        .with_context(|| format!("running pdfinfo on {}", pdf.display()))?;
    if !output.status.success() {
        bail!(
            "pdfinfo failed on {}: {}",
            pdf.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_page_count(&text)
        .ok_or_else(|| anyhow!("no Pages: line in pdfinfo output for {}", pdf.display()))
}

/// Parse the `Pages:` line out of `pdfinfo` output.
fn parse_page_count(info: &str) -> Option<u32> {
    for line in info.lines() {
        if let Some(rest) = line.strip_prefix("Pages:") {
            if let Ok(n) = rest.trim().parse::<u32>() {
                return Some(n);
            }
        }
    }
    None
}

/// Render one 1-based `page` of `pdf` to a PNG in `out_dir` at `dpi`, returning
/// the produced PNG path. Uses `-singlefile` so the output is exactly
/// `<prefix>.png` with no page-number suffix to reconstruct.
pub fn render_page(pdf: &Path, page: u32, dpi: u32, out_dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("creating render dir {}", out_dir.display()))?;
    let stem = pdf
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "page".to_string());
    let prefix = out_dir.join(format!("{stem}-p{page:04}-{dpi}dpi"));
    let out_png = prefix.with_extension("png");

    let page_arg = page.to_string();
    let dpi_arg = dpi.to_string();
    // poppler's open can fail transiently while a OneDrive Files-On-Demand
    // placeholder is being recalled, so retry a few times before giving up.
    let mut last_err = String::new();
    for attempt in 1..=OPEN_RETRIES {
        let output = Command::new(pdftoppm_bin())
            .arg("-png")
            .arg("-f")
            .arg(&page_arg)
            .arg("-l")
            .arg(&page_arg)
            .arg("-r")
            .arg(&dpi_arg)
            .arg("-singlefile")
            .arg(pdf)
            .arg(&prefix)
            .output()
            .with_context(|| format!("running pdftoppm on {} page {page}", pdf.display()))?;
        if output.status.success() && out_png.exists() {
            return Ok(out_png);
        }
        last_err = String::from_utf8_lossy(&output.stderr).into_owned();
        if attempt < OPEN_RETRIES {
            std::thread::sleep(std::time::Duration::from_millis(500 * attempt as u64));
        }
    }
    bail!(
        "pdftoppm failed on {} page {page} after {OPEN_RETRIES} attempts: {last_err}",
        pdf.display()
    );
}

/// How many times a poppler open is retried through a transient cloud-file recall.
const OPEN_RETRIES: u32 = 3;

/// Crop a rendered page PNG to its right column: keep `x ∈ [round(w·left_frac), w)`
/// at full height and re-encode as PNG. Used so body transcription sends only the
/// Latin column, halving image input tokens.
pub fn crop_right_column(png_bytes: &[u8], left_frac: f32) -> Result<Vec<u8>> {
    let img = image::load_from_memory(png_bytes).context("decoding page PNG to crop")?;
    let (w, h) = (img.width(), img.height());
    let left = ((w as f32) * left_frac).round() as u32;
    let left = left.min(w.saturating_sub(1));
    let cropped = img.crop_imm(left, 0, w - left, h);
    let mut out = Vec::new();
    cropped
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .context("re-encoding cropped column PNG")?;
    Ok(out)
}

/// Render one page's Latin column: rasterize the whole page with [`render_page`],
/// then crop to the right column with [`crop_right_column`] and write the cropped
/// PNG beside the full one. Only body pages use this; front-matter prose is
/// full-width and rendered with [`render_page`] directly.
pub fn render_latin_column(pdf: &Path, page: u32, dpi: u32, out_dir: &Path) -> Result<PathBuf> {
    let full = render_page(pdf, page, dpi, out_dir)?;
    let bytes = std::fs::read(&full).with_context(|| format!("reading {}", full.display()))?;
    let cropped = crop_right_column(&bytes, CROP_LEFT_FRAC)?;
    let out_png = full.with_extension("latin.png");
    std::fs::write(&out_png, cropped)
        .with_context(|| format!("writing cropped column {}", out_png.display()))?;
    Ok(out_png)
}

/// Hex SHA-256 of a PNG file's bytes, kept as a per-page integrity field on the
/// rendered image (not the cache key).
pub fn page_image_hash(png: &Path) -> Result<String> {
    let bytes = std::fs::read(png).with_context(|| format!("reading {}", png.display()))?;
    Ok(hash_bytes(&bytes))
}

/// Hex SHA-256 of raw bytes.
pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Hex SHA-256 of a whole file's bytes, used to hash a source PDF for the cache
/// key and staleness check.
pub fn file_hash(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(hash_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_words_parses_bbox_fractions() {
        let words = parse_bbox(
            r#"<page width="400" height="700"><word xMin="200" yMin="280" xMax="300" yMax="294">ROMULUS.</word><word xMin="210" yMin="320" xMax="250" yMax="334">Ingens</word></page>"#,
        );
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].text, "ROMULUS.");
        assert_eq!(words[1].text, "Ingens");
        assert!((words[0].y_min - 0.4).abs() < 1e-4);
        assert_eq!(words[0].x_min, 0.5);
        assert_eq!(words[0].x_max, 0.75);
        assert!((words[1].y_max - 334.0 / 700.0).abs() < 1e-4);
        assert!(parse_bbox("<page width=\"0\" height=\"700\">").is_empty());
    }

    #[test]
    fn parse_bbox_unescapes_entities() {
        let words = parse_bbox(
            r#"<page width="1" height="1"><word xMin="0" yMin="0" xMax="1" yMax="1">R&amp;R &lt; &gt; &quot; &apos; &amp;lt;</word></page>"#,
        );
        assert_eq!(words[0].text, "R&R < > \" ' &lt;");
    }

    #[test]
    fn parse_page_count_reads_pages_line() {
        let info = "Title:          Plutarch\nPages:          643\nEncrypted:      no\n";
        assert_eq!(parse_page_count(info), Some(643));
    }

    #[test]
    fn parse_page_count_missing_returns_none() {
        assert_eq!(parse_page_count("Title: x\nEncrypted: no\n"), None);
    }

    #[test]
    fn page_image_hash_is_stable() {
        let dir = tempfile::tempdir().unwrap();
        let png = dir.path().join("a.png");
        std::fs::write(&png, b"not really a png but stable bytes").unwrap();
        let h1 = page_image_hash(&png).unwrap();
        let h2 = page_image_hash(&png).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn hash_bytes_differs_on_different_input() {
        assert_ne!(hash_bytes(b"a"), hash_bytes(b"b"));
    }

    /// A synthetic solid-color PNG of the given dimensions, for crop tests.
    fn solid_rgb_png(w: u32, h: u32) -> Vec<u8> {
        let img: image::RgbImage =
            image::ImageBuffer::from_pixel(w, h, image::Rgb([200, 200, 200]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        bytes
    }

    #[test]
    fn crop_right_column_keeps_right_fraction() {
        let bytes = solid_rgb_png(100, 20);
        let out = crop_right_column(&bytes, 0.48).unwrap();
        let decoded = image::load_from_memory(&out).unwrap();
        // left = round(100 * 0.48) = 48, so the kept width is 100 - 48 = 52.
        assert_eq!(decoded.width(), 52);
        assert_eq!(decoded.height(), 20);
    }

    #[test]
    fn crop_preserves_png() {
        let bytes = solid_rgb_png(64, 48);
        let out = crop_right_column(&bytes, 0.5).unwrap();
        // The output is a valid, decodable PNG.
        assert!(image::load_from_memory(&out).is_ok());
    }

    #[test]
    fn render_latin_column_produces_png() {
        if !pdftoppm_available() {
            eprintln!("skipping: pdftoppm not available");
            return;
        }
        let v1 = Path::new("Ploutarchou_Bioi_v1.pdf");
        if !v1.exists() {
            eprintln!("skipping: source PDF not present");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let png = render_latin_column(v1, 40, 150, dir.path()).unwrap();
        assert!(png.exists());
        assert!(std::fs::metadata(&png).unwrap().len() > 0);
    }

    #[test]
    fn running_head_strips_folio_and_keeps_title() {
        assert_eq!(running_head("ROMULUS. 18"), "ROMULUS");
        assert_eq!(running_head("115 SOLON."), "SOLON");
        assert_eq!(
            running_head("LYCURGI ET NUMÆ COMPARATIO. 89"),
            "LYCURGI ET NUMÆ COMPARATIO"
        );
    }

    #[test]
    fn running_head_empty_on_noise() {
        assert_eq!(running_head("-- , . ; !"), "");
        assert_eq!(running_head(""), "");
    }

    #[test]
    fn page_text_reads_embedded_layer() {
        if !pdftotext_available() {
            eprintln!("skipping: pdftotext not available");
            return;
        }
        let v1 = Path::new("Ploutarchou_Bioi_v1.pdf");
        if !v1.exists() {
            eprintln!("skipping: source PDF not present");
            return;
        }
        // Page 16 carries the Theseus running head in its OCR layer.
        let text = page_text(v1, 16).unwrap();
        assert!(
            running_head(&text).contains("THESEUS"),
            "unexpected running head on v1 p16: {:?}",
            running_head(&text)
        );
    }

    #[test]
    fn render_page_produces_png() {
        if !pdftoppm_available() {
            eprintln!("skipping: pdftoppm not available");
            return;
        }
        let v1 = Path::new("Ploutarchou_Bioi_v1.pdf");
        if !v1.exists() {
            eprintln!("skipping: source PDF not present");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let png = render_page(v1, 40, 150, dir.path()).unwrap();
        assert!(png.exists());
        assert!(std::fs::metadata(&png).unwrap().len() > 0);
    }
}
