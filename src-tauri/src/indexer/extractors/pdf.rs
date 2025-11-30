use crate::error::{CortexError, Result};
use crate::indexer::extractors::ExtractedContent;
use std::path::Path;

/// Extractor for PDF files
pub struct PdfExtractor;

impl PdfExtractor {
    /// Extract text content from a PDF file
    pub fn extract(path: &Path) -> Result<ExtractedContent> {
        match Self::try_extract(path) {
            Ok(content) => Ok(content),
            Err(e) => {
                log::warn!("PDF extraction failed for {}: {}", path.display(), e);

                // Return error with context
                Err(CortexError::ExtractionFailed {
                    path: path.to_string_lossy().to_string(),
                    error: format!("Failed to extract PDF: {}", e),
                })
            }
        }
    }

    fn try_extract(path: &Path) -> Result<ExtractedContent> {
        // Extract text from PDF
        let text = pdf_extract::extract_text(path).map_err(|e| CortexError::ExtractionFailed {
            path: path.to_string_lossy().to_string(),
            error: format!("Failed to parse PDF: {}", e),
        })?;

        // Clean up extracted text (PDFs often have extra whitespace)
        let cleaned = Self::clean_pdf_text(&text);

        Ok(ExtractedContent::new(cleaned))
    }

    /// Clean up PDF text extraction artifacts
    fn clean_pdf_text(text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nonexistent_pdf() {
        let result = PdfExtractor::extract(Path::new("/nonexistent/file.pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_clean_pdf_text() {
        let messy = "  Line 1  \n\n  Line 2  \n   \n  Line 3  ";
        let cleaned = PdfExtractor::clean_pdf_text(messy);

        assert_eq!(cleaned, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_clean_pdf_text_empty_lines() {
        let text = "Content\n\n\n\nMore content";
        let cleaned = PdfExtractor::clean_pdf_text(text);

        assert_eq!(cleaned, "Content\nMore content");
    }

    // Note: Testing PDF extraction requires actual PDF files
    // In a real project, you would include test fixtures
}
