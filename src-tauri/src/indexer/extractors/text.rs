use crate::error::{CortexError, Result};
use crate::indexer::extractors::ExtractedContent;
use encoding_rs::Encoding;
use std::fs;
use std::path::Path;

/// Extractor for plain text files
pub struct TextExtractor;

impl TextExtractor {
    /// Extract text content from a file
    pub fn extract(path: &Path) -> Result<ExtractedContent> {
        // Read file as bytes
        let bytes = fs::read(path).map_err(|e| CortexError::ExtractionFailed {
            path: path.to_string_lossy().to_string(),
            error: format!("Failed to read file: {}", e),
        })?;

        // Detect encoding and decode
        let (text, encoding_used, had_errors) = Self::decode_with_detection(&bytes);

        let mut content = ExtractedContent::new(text);

        // Add warning if we had to fallback to lossy conversion
        if had_errors {
            content = content.with_warning(format!(
                "File decoded as {} with replacement characters for invalid sequences",
                encoding_used.name()
            ));
        }

        Ok(content)
    }

    /// Detect encoding and decode bytes to string
    fn decode_with_detection(bytes: &[u8]) -> (String, &'static Encoding, bool) {
        // Check for BOM first
        let (encoding, bom_length) = encoding_rs::Encoding::for_bom(bytes)
            .unwrap_or((encoding_rs::UTF_8, 0));

        // Skip BOM if present
        let bytes_without_bom = &bytes[bom_length..];

        // Try UTF-8 first (most common)
        if let Ok(text) = std::str::from_utf8(bytes_without_bom) {
            return (text.to_string(), encoding, false);
        }

        // Decode with detected encoding
        let (decoded, _, had_errors) = encoding.decode(bytes_without_bom);

        (decoded.to_string(), encoding, had_errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_utf8_text() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, world!\nThis is a test.").unwrap();

        let result = TextExtractor::extract(file.path()).unwrap();

        assert_eq!(result.text, "Hello, world!\nThis is a test.");
        assert_eq!(result.word_count, 6);
        assert!(result.summary.is_some());
    }

    #[test]
    fn test_extract_multiline() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Line 1\nLine 2\nLine 3").unwrap();

        let result = TextExtractor::extract(file.path()).unwrap();

        assert_eq!(result.text, "Line 1\nLine 2\nLine 3");
        assert_eq!(result.word_count, 6);
    }

    #[test]
    fn test_extract_empty_file() {
        let file = NamedTempFile::new().unwrap();

        let result = TextExtractor::extract(file.path()).unwrap();

        assert_eq!(result.text, "");
        assert_eq!(result.word_count, 0);
        assert_eq!(result.summary, None);
    }

    #[test]
    fn test_extract_nonexistent_file() {
        let result = TextExtractor::extract(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_encoding_detection() {
        // Test UTF-8
        let (text, encoding, errors) = TextExtractor::decode_with_detection(b"Hello");
        assert_eq!(text, "Hello");
        assert_eq!(encoding.name(), "UTF-8");
        assert!(!errors);

        // Test with BOM
        let utf8_bom = b"\xEF\xBB\xBFHello";
        let (text, _, _) = TextExtractor::decode_with_detection(utf8_bom);
        assert_eq!(text, "Hello");
    }
}
