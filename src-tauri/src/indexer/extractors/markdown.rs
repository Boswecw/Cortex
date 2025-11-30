use crate::error::{CortexError, Result};
use crate::indexer::extractors::ExtractedContent;
use pulldown_cmark::{Event, Parser, Tag};
use std::fs;
use std::path::Path;

/// Extractor for Markdown files
pub struct MarkdownExtractor;

impl MarkdownExtractor {
    /// Extract text content from a markdown file
    /// Converts markdown to plain text while preserving structure
    pub fn extract(path: &Path) -> Result<ExtractedContent> {
        // Read file
        let markdown = fs::read_to_string(path).map_err(|e| CortexError::ExtractionFailed {
            path: path.to_string_lossy().to_string(),
            error: format!("Failed to read file: {}", e),
        })?;

        // Parse markdown and convert to plain text
        let text = Self::markdown_to_text(&markdown);

        Ok(ExtractedContent::new(text))
    }

    /// Convert markdown to plain text
    fn markdown_to_text(markdown: &str) -> String {
        let parser = Parser::new(markdown);
        let mut output = String::new();
        let mut in_code_block = false;

        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(_)) => {
                    in_code_block = true;
                }
                Event::End(Tag::CodeBlock(_)) => {
                    in_code_block = false;
                    output.push('\n');
                }
                Event::Start(Tag::Heading { .. }) => {
                    // Add newline before headings
                    if !output.is_empty() {
                        output.push('\n');
                    }
                }
                Event::End(Tag::Heading { .. }) => {
                    output.push('\n');
                }
                Event::Start(Tag::Paragraph) => {
                    if !output.is_empty() && !output.ends_with('\n') {
                        output.push('\n');
                    }
                }
                Event::End(Tag::Paragraph) => {
                    output.push('\n');
                }
                Event::Start(Tag::List(_)) => {
                    if !output.is_empty() && !output.ends_with('\n') {
                        output.push('\n');
                    }
                }
                Event::Start(Tag::Item) => {
                    output.push_str("• ");
                }
                Event::End(Tag::Item) => {
                    output.push('\n');
                }
                Event::Text(text) => {
                    output.push_str(&text);
                }
                Event::Code(code) => {
                    if in_code_block {
                        output.push_str(&code);
                    } else {
                        output.push_str(&code);
                    }
                }
                Event::SoftBreak => {
                    output.push(' ');
                }
                Event::HardBreak => {
                    output.push('\n');
                }
                _ => {}
            }
        }

        output.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_simple_markdown() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"# Hello\n\nThis is a **test**.").unwrap();

        let result = MarkdownExtractor::extract(file.path()).unwrap();

        assert!(result.text.contains("Hello"));
        assert!(result.text.contains("test"));
        assert!(!result.text.contains("**")); // Bold markers removed
        assert!(result.word_count > 0);
    }

    #[test]
    fn test_markdown_to_text_heading() {
        let markdown = "# Heading 1\n## Heading 2\n### Heading 3";
        let text = MarkdownExtractor::markdown_to_text(markdown);

        assert!(text.contains("Heading 1"));
        assert!(text.contains("Heading 2"));
        assert!(text.contains("Heading 3"));
        assert!(!text.contains("#"));
    }

    #[test]
    fn test_markdown_to_text_lists() {
        let markdown = "- Item 1\n- Item 2\n- Item 3";
        let text = MarkdownExtractor::markdown_to_text(markdown);

        assert!(text.contains("•"));
        assert!(text.contains("Item 1"));
        assert!(text.contains("Item 2"));
        assert!(text.contains("Item 3"));
    }

    #[test]
    fn test_markdown_to_text_code_block() {
        let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let text = MarkdownExtractor::markdown_to_text(markdown);

        assert!(text.contains("fn main"));
        assert!(text.contains("println!"));
        assert!(!text.contains("```"));
    }

    #[test]
    fn test_markdown_to_text_inline_code() {
        let markdown = "Use `cargo build` to compile.";
        let text = MarkdownExtractor::markdown_to_text(markdown);

        assert!(text.contains("cargo build"));
        assert!(!text.contains("`"));
    }

    #[test]
    fn test_markdown_to_text_links() {
        let markdown = "Check [this link](https://example.com) for more.";
        let text = MarkdownExtractor::markdown_to_text(markdown);

        assert!(text.contains("this link"));
        // URL might or might not be included depending on parser
    }

    #[test]
    fn test_extract_empty_markdown() {
        let file = NamedTempFile::new().unwrap();

        let result = MarkdownExtractor::extract(file.path()).unwrap();

        assert_eq!(result.text, "");
        assert_eq!(result.word_count, 0);
    }

    #[test]
    fn test_markdown_preserves_paragraph_structure() {
        let markdown = "First paragraph.\n\nSecond paragraph.";
        let text = MarkdownExtractor::markdown_to_text(markdown);

        // Should have paragraph breaks
        assert!(text.lines().count() >= 2);
    }
}
