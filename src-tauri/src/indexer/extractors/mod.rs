// Content extraction module
mod text;
mod markdown;
mod docx;
mod pdf;

pub use text::TextExtractor;
pub use markdown::MarkdownExtractor;
pub use docx::DocxExtractor;
pub use pdf::PdfExtractor;

use crate::error::Result;
use std::path::Path;

/// Result of content extraction
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    /// Extracted text content
    pub text: String,

    /// Word count
    pub word_count: usize,

    /// Brief summary (first 200 chars or first paragraph)
    pub summary: Option<String>,

    /// Any warnings encountered during extraction
    pub warnings: Vec<String>,
}

impl ExtractedContent {
    pub fn new(text: String) -> Self {
        let word_count = text.split_whitespace().count();
        let summary = Self::generate_summary(&text);

        Self {
            text,
            word_count,
            summary,
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    fn generate_summary(text: &str) -> Option<String> {
        if text.is_empty() {
            return None;
        }

        // Take first paragraph or first 200 chars
        let first_para = text.lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("");

        let summary = if first_para.len() > 200 {
            format!("{}...", &first_para[..200])
        } else {
            first_para.to_string()
        };

        Some(summary)
    }
}

/// Main extractor that dispatches to appropriate sub-extractor
pub struct ContentExtractor;

impl ContentExtractor {
    /// Extract content from a file based on its extension
    pub fn extract(path: &Path) -> Result<ExtractedContent> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        match extension.as_str() {
            "txt" => TextExtractor::extract(path),
            "md" => MarkdownExtractor::extract(path),
            "docx" => DocxExtractor::extract(path),
            "pdf" => PdfExtractor::extract(path),
            _ => TextExtractor::extract(path), // Fallback to text
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extracted_content_word_count() {
        let content = ExtractedContent::new("Hello world this is a test".to_string());
        assert_eq!(content.word_count, 6);
    }

    #[test]
    fn test_summary_generation() {
        let short = ExtractedContent::new("Short text".to_string());
        assert_eq!(short.summary, Some("Short text".to_string()));

        let long = ExtractedContent::new("a".repeat(250));
        assert!(long.summary.unwrap().ends_with("..."));
        assert!(long.summary.unwrap().len() <= 203); // 200 + "..."
    }

    #[test]
    fn test_with_warning() {
        let content = ExtractedContent::new("test".to_string())
            .with_warning("Encoding issue".to_string());

        assert_eq!(content.warnings.len(), 1);
        assert_eq!(content.warnings[0], "Encoding issue");
    }
}
