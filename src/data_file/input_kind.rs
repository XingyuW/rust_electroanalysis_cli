//! Centralized input classification for parser dispatch and batch filtering.
//!
//! Every runner that accepts file inputs must use this module to determine
//! whether a file is supported and which parser should handle it.

use std::fmt;
use std::fs;
use std::path::Path;

/// Classification of a candidate input file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// CHI-format CSV with `Freq/Hz` impedance columns.
    ChiEisCsv,
    /// CHI-format CSV with time‑series data (e.g., OCPT).
    ChiOcptCsv,
    /// Generic CSV or text time‑series.
    GeneralCsv,
    /// Microsoft Excel `.xlsx` workbook (Office Open XML).
    ExcelXlsx,
    /// Microsoft Excel `.xls` workbook (legacy binary format).
    /// Only classified when the selected library supports it.
    ExcelXls,
    /// A known binary extension that is intentionally unsupported.
    UnsupportedBinary,
    /// A known binary file with a misleading text extension.
    UnsupportedContentBinary,
    /// An extension that is not in the supported list and whose file may be
    /// binary or text; callers should attempt content-based detection.
    Unknown,
}

impl fmt::Display for InputKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            InputKind::ChiEisCsv => "CHI EIS CSV",
            InputKind::ChiOcptCsv => "CHI OCPT CSV",
            InputKind::GeneralCsv => "general CSV",
            InputKind::ExcelXlsx => "Excel XLSX",
            InputKind::ExcelXls => "Excel XLS",
            InputKind::UnsupportedBinary => "unsupported binary",
            InputKind::UnsupportedContentBinary => "unsupported binary content",
            InputKind::Unknown => "unknown format",
        };
        write!(f, "{label}")
    }
}

/// The set of extensions that are recognised as binary and intentionally
/// unsupported.  Files ending with any of these are never opened as text or
/// spreadsheet input.
const BINARY_EXTENSIONS: &[&str] = &["bin", "raw"];

/// Extensions that the project supports for CSV‑style text input.
const TEXT_EXTENSIONS: &[&str] = &["csv", "txt", "dat"];

/// Extensions that the project supports as Excel workbooks.
const EXCEL_EXTENSIONS: &[&str] = &["xlsx"];

/// Extensions that the project supports as legacy Excel workbooks.
const LEGACY_EXCEL_EXTENSIONS: &[&str] = &["xls"];

impl InputKind {
    /// Classify a file by its extension alone.
    pub fn classify_by_extension(path: &Path) -> Self {
        let extension = path
            .extension()
            .and_then(|v| v.to_str())
            .map(|v| v.to_ascii_lowercase());

        match extension.as_deref() {
            Some(ext) if BINARY_EXTENSIONS.contains(&ext) => InputKind::UnsupportedBinary,
            Some(ext) if TEXT_EXTENSIONS.contains(&ext) => InputKind::GeneralCsv,
            Some(ext) if EXCEL_EXTENSIONS.contains(&ext) => InputKind::ExcelXlsx,
            Some(ext) if LEGACY_EXCEL_EXTENSIONS.contains(&ext) => InputKind::ExcelXls,
            Some(_) => InputKind::Unknown,
            None => InputKind::Unknown,
        }
    }

    /// Classify a path using extension and lightweight content checks.
    pub fn classify_path(path: &Path) -> Self {
        let by_extension = Self::classify_by_extension(path);
        if by_extension == InputKind::UnsupportedBinary {
            return by_extension;
        }

        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase());
        let is_text_extension = extension
            .as_deref()
            .is_some_and(|ext| TEXT_EXTENSIONS.contains(&ext));
        if !is_text_extension {
            return by_extension;
        }

        if let Ok(bytes) = Self::read_prefix(path, 4096)
            && Self::looks_binary(&bytes)
        {
            return InputKind::UnsupportedContentBinary;
        }
        by_extension
    }

    fn read_prefix(path: &Path, max_len: usize) -> Result<Vec<u8>, std::io::Error> {
        let data = fs::read(path)?;
        Ok(data.into_iter().take(max_len).collect())
    }

    fn looks_binary(bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }
        if bytes.contains(&0) {
            return true;
        }
        std::str::from_utf8(bytes).is_err()
    }

    /// Returns `true` when this classification means the file must be
    /// skipped and must never reach a text or spreadsheet parser.
    pub fn is_unsupported_binary(&self) -> bool {
        matches!(
            self,
            InputKind::UnsupportedBinary | InputKind::UnsupportedContentBinary
        )
    }

    /// Returns `true` when this classification means the file is a
    /// supported text-based format (CSV, TXT, DAT).
    pub fn is_supported_text(&self) -> bool {
        matches!(
            self,
            InputKind::ChiEisCsv | InputKind::ChiOcptCsv | InputKind::GeneralCsv
        )
    }

    /// Returns `true` when this classification means the file is a
    /// supported spreadsheet format.
    pub fn is_supported_spreadsheet(&self) -> bool {
        matches!(self, InputKind::ExcelXlsx | InputKind::ExcelXls)
    }

    /// Returns `true` when the file is supported by any parser.
    pub fn is_supported(&self) -> bool {
        self.is_supported_text() || self.is_supported_spreadsheet()
    }

    /// Human-readable skip reason for batch summaries.
    pub fn skip_reason(&self) -> &'static str {
        match self {
            InputKind::UnsupportedBinary => "unsupported binary extension",
            InputKind::UnsupportedContentBinary => "binary content with misleading text extension",
            InputKind::Unknown => "unknown or unsupported extension",
            _ => "unsupported",
        }
    }

    /// Documented list of supported text extensions.
    pub fn supported_text_extensions() -> &'static [&'static str] {
        TEXT_EXTENSIONS
    }

    /// Documented list of supported Excel extensions.
    pub fn supported_excel_extensions() -> &'static [&'static str] {
        EXCEL_EXTENSIONS
    }

    /// Documented list of unsupported binary extensions.
    pub fn unsupported_binary_extensions() -> &'static [&'static str] {
        BINARY_EXTENSIONS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn classifies_text_extensions_as_general_csv() {
        for ext in ["csv", "txt", "dat"] {
            let kind = InputKind::classify_by_extension(Path::new(&format!("file.{ext}")));
            assert_eq!(kind, InputKind::GeneralCsv, "extension .{ext}");
        }
    }

    #[test]
    fn classifies_xlsx_as_excel() {
        let kind = InputKind::classify_by_extension(Path::new("data.xlsx"));
        assert_eq!(kind, InputKind::ExcelXlsx);
    }

    #[test]
    fn classifies_xls_as_legacy_excel() {
        let kind = InputKind::classify_by_extension(Path::new("data.xls"));
        assert_eq!(kind, InputKind::ExcelXls);
    }

    #[test]
    fn classifies_bin_as_unsupported_binary() {
        let kind = InputKind::classify_by_extension(Path::new("data.bin"));
        assert_eq!(kind, InputKind::UnsupportedBinary);
        assert!(kind.is_unsupported_binary());
        assert!(!kind.is_supported());
    }

    #[test]
    fn classifies_raw_as_unsupported_binary() {
        let kind = InputKind::classify_by_extension(Path::new("data.raw"));
        assert_eq!(kind, InputKind::UnsupportedBinary);
    }

    #[test]
    fn classifies_uppercase_extensions() {
        let kind = InputKind::classify_by_extension(Path::new("DATA.CSV"));
        assert_eq!(kind, InputKind::GeneralCsv);
        let kind = InputKind::classify_by_extension(Path::new("DATA.BIN"));
        assert_eq!(kind, InputKind::UnsupportedBinary);
    }

    #[test]
    fn classifies_extensionless_as_unknown() {
        let kind = InputKind::classify_by_extension(Path::new("no_extension"));
        assert_eq!(kind, InputKind::Unknown);
    }

    #[test]
    fn classifies_binary_content_with_text_extension() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("input-kind-binary-{suffix}.csv"));
        fs::write(&path, [0x00, 0xFF, 0x10, 0x20]).expect("write fixture");
        let kind = InputKind::classify_path(&path);
        assert_eq!(kind, InputKind::UnsupportedContentBinary);
        fs::remove_file(path).ok();
    }
}
