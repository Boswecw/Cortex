use crate::error::{CortexError, Result};
use crate::indexer::extractors::ExtractedContent;
use std::path::Path;

/// Extractor for DOCX files
pub struct DocxExtractor;

impl DocxExtractor {
    /// Extract text content from a DOCX file
    pub fn extract(path: &Path) -> Result<ExtractedContent> {
        match Self::try_extract(path) {
            Ok(content) => Ok(content),
            Err(e) => {
                log::warn!("DOCX extraction failed for {}: {}", path.display(), e);

                // Return error with context
                Err(CortexError::ExtractionFailed {
                    path: path.to_string_lossy().to_string(),
                    error: format!("Failed to extract DOCX: {}", e),
                })
            }
        }
    }

    fn try_extract(path: &Path) -> Result<ExtractedContent> {
        // Read the DOCX file into memory
        let bytes = std::fs::read(path).map_err(|e| CortexError::ExtractionFailed {
            path: path.to_string_lossy().to_string(),
            error: format!("Failed to read file: {}", e),
        })?;

        // Parse the DOCX
        let docx = docx_rs::read_docx(&bytes).map_err(|e| CortexError::ExtractionFailed {
            path: path.to_string_lossy().to_string(),
            error: format!("Failed to parse DOCX: {}", e),
        })?;

        // Extract text from all paragraphs
        let mut text = String::new();

        for child in docx.document.children {
            if let docx_rs::DocumentChild::Paragraph(para) = child {
                for child in para.children {
                    if let docx_rs::ParagraphChild::Run(run) = child {
                        for child in run.children {
                            if let docx_rs::RunChild::Text(t) = child {
                                text.push_str(&t.text);
                            }
                        }
                    }
                }
                text.push('\n');
            }
        }

        Ok(ExtractedContent::new(text.trim().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nonexistent_docx() {
        let result = DocxExtractor::extract(Path::new("/nonexistent/file.docx"));
        assert!(result.is_err());
    }

    // Note: Testing DOCX extraction requires actual DOCX files
    // In a real project, you would include test fixtures
}
