use crate::db::Database;
use crate::error::{CortexError, Result};
use crate::export::{
    ContextBuilder, ExportConfig, ExportResult, ExportStats, PathValidator, PromptBuilder
};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

/// Bundles export files into a complete VS Code Claude context package
pub struct BundleBuilder {
    db: Database,
}

impl BundleBuilder {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a complete export bundle
    pub async fn create_bundle(&self, config: &ExportConfig) -> Result<ExportResult> {
        // Validate and sanitize output path (security)
        let output_dir = PathValidator::validate_export_path(&config.output_path)?;
        self.ensure_directory(&output_dir)?;

        // Build CONTEXT.md
        let context_builder = ContextBuilder::new(self.db.clone());
        let (context_content, stats) = context_builder.build_context(config).await?;
        let context_file = self.write_file(
            &output_dir,
            "CONTEXT.md",
            &context_content
        )?;

        // Build STARTER_PROMPT.md
        let prompt_builder = PromptBuilder::new(self.db.clone());
        let starter_prompt = prompt_builder.build_starter_prompt(config).await?;
        let starter_prompt_file = self.write_file(
            &output_dir,
            "STARTER_PROMPT.md",
            &starter_prompt
        )?;

        // Build prompt templates (if enabled)
        let mut prompt_files = Vec::new();
        if config.include_prompts {
            let prompts_dir = output_dir.join("prompts");
            self.ensure_directory(&prompts_dir)?;

            // Create default prompt templates
            let templates = self.create_default_templates(&prompt_builder).await?;
            for (name, content) in templates {
                let file_path = self.write_file(&prompts_dir, &name, &content)?;
                prompt_files.push(file_path);
            }
        }

        // Create .claude/ directory with settings
        let claude_dir = output_dir.join(".claude");
        self.ensure_directory(&claude_dir)?;
        let settings = self.create_claude_settings(config)?;
        self.write_file(&claude_dir, "settings.json", &settings)?;

        // Create README for the export
        let readme = self.create_export_readme(config, &stats)?;
        self.write_file(&output_dir, "README.md", &readme)?;

        // Update stats with actual prompt count
        let mut final_stats = stats;
        final_stats.prompts_generated = if config.include_prompts {
            prompt_files.len() + 1 // +1 for STARTER_PROMPT.md
        } else {
            1 // Just STARTER_PROMPT.md
        };

        Ok(ExportResult {
            context_file,
            starter_prompt_file,
            prompt_files,
            stats: final_stats,
            exported_at: Utc::now(),
        })
    }

    /// Ensure a directory exists, create if necessary
    fn ensure_directory(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| CortexError::Internal {
                message: format!("Failed to create directory {}: {}", path.display(), e)
            })?;
        }
        Ok(())
    }

    /// Write content to a file
    fn write_file(&self, dir: &Path, filename: &str, content: &str) -> Result<String> {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).map_err(|e| CortexError::Internal {
            message: format!("Failed to write file {}: {}", file_path.display(), e)
        })?;

        Ok(file_path.to_string_lossy().to_string())
    }

    /// Create default prompt templates
    async fn create_default_templates(&self, builder: &PromptBuilder) -> Result<Vec<(String, String)>> {
        let mut templates = Vec::new();

        // ADD_FEATURE.md template
        let feature_prompt = builder.build_feature_prompt(
            "{FEATURE_NAME}",
            "{FEATURE_DESCRIPTION}\n\n\
             Replace the placeholders above with:\n\
             - FEATURE_NAME: Name of the feature to implement\n\
             - FEATURE_DESCRIPTION: Detailed description of the feature"
        ).await?;
        templates.push(("ADD_FEATURE.md".to_string(), feature_prompt));

        // FIX_BUG.md template
        let bugfix_prompt = builder.build_bugfix_prompt(
            "{BUG_DESCRIPTION}\n\n\
             Replace with a detailed description of the bug.",
            "{STEPS_TO_REPRODUCE}\n\n\
             Replace with steps to reproduce the bug:\n\
             1. Step 1\n\
             2. Step 2\n\
             3. Expected behavior\n\
             4. Actual behavior"
        ).await?;
        templates.push(("FIX_BUG.md".to_string(), bugfix_prompt));

        // REFACTOR.md template
        let refactor_prompt = builder.build_refactor_prompt(
            "{MODULE_PATH}",
            "{REFACTORING_GOALS}\n\n\
             Replace with specific refactoring goals:\n\
             - Improve code readability\n\
             - Reduce complexity\n\
             - Follow best practices\n\
             - etc."
        ).await?;
        templates.push(("REFACTOR.md".to_string(), refactor_prompt));

        // ADD_TESTS.md template
        let test_prompt = self.create_test_template()?;
        templates.push(("ADD_TESTS.md".to_string(), test_prompt));

        // DOCUMENTATION.md template
        let docs_prompt = self.create_documentation_template()?;
        templates.push(("DOCUMENTATION.md".to_string(), docs_prompt));

        Ok(templates)
    }

    /// Create test template
    fn create_test_template(&self) -> Result<String> {
        Ok(
            "# ADD TESTS\n\n\
             **Generated by Cortex**\n\n\
             ---\n\n\
             ## Testing Task\n\n\
             Module/Feature to test: `{MODULE_PATH}`\n\n\
             ## Test Requirements\n\n\
             Please create comprehensive tests following these guidelines:\n\n\
             1. **Unit Tests**: Test individual functions in isolation\n\
             2. **Integration Tests**: Test interactions between components\n\
             3. **Edge Cases**: Test boundary conditions and error cases\n\
             4. **Coverage**: Aim for high code coverage\n\n\
             ## Test Structure\n\n\
             Based on CONTEXT.md, follow the existing test patterns:\n\
             - Test file location\n\
             - Test naming conventions\n\
             - Assertion style\n\
             - Mock/fixture usage\n\n\
             ## Expected Deliverables\n\n\
             - [ ] Unit tests for all public functions\n\
             - [ ] Integration tests (if applicable)\n\
             - [ ] Edge case tests\n\
             - [ ] All tests passing\n\
             - [ ] Test coverage report\n\n\
             ---\n\n\
             **Replace {MODULE_PATH} with the actual module path and begin implementation.**\n".to_string()
        )
    }

    /// Create documentation template
    fn create_documentation_template(&self) -> Result<String> {
        Ok(
            "# DOCUMENTATION UPDATE\n\n\
             **Generated by Cortex**\n\n\
             ---\n\n\
             ## Documentation Task\n\n\
             Module/Feature to document: `{MODULE_PATH}`\n\n\
             ## Documentation Requirements\n\n\
             Please create or update documentation including:\n\n\
             1. **Code Comments**: Inline documentation for complex logic\n\
             2. **Doc Comments**: Function/class documentation\n\
             3. **README**: User-facing documentation\n\
             4. **Architecture Docs**: System design documentation\n\n\
             ## Documentation Style\n\n\
             Follow existing documentation patterns from CONTEXT.md:\n\
             - Markdown format\n\
             - Code examples where helpful\n\
             - Clear, concise language\n\
             - Diagrams for complex concepts (ASCII or Mermaid)\n\n\
             ## Expected Deliverables\n\n\
             - [ ] Updated code comments\n\
             - [ ] Doc comments for public APIs\n\
             - [ ] README updates (if applicable)\n\
             - [ ] Architecture documentation (if needed)\n\
             - [ ] Usage examples\n\n\
             ---\n\n\
             **Replace {MODULE_PATH} with the actual module path and begin documentation.**\n".to_string()
        )
    }

    /// Create VS Code Claude settings
    fn create_claude_settings(&self, _config: &ExportConfig) -> Result<String> {
        let settings = serde_json::json!({
            "cortex_export": {
                "version": "1.0",
                "generated_at": Utc::now().to_rfc3339(),
                "instructions": "Load CONTEXT.md first to understand the full project structure"
            },
            "recommended_workflow": [
                "1. Read CONTEXT.md to understand project structure",
                "2. Read STARTER_PROMPT.md for implementation guidelines",
                "3. Choose a prompt template from prompts/ directory",
                "4. Begin implementation following project patterns"
            ]
        });

        serde_json::to_string_pretty(&settings).map_err(|e| CortexError::Internal {
            message: format!("Failed to serialize settings: {}", e)
        })
    }

    /// Create README for the export bundle
    fn create_export_readme(&self, config: &ExportConfig, stats: &ExportStats) -> Result<String> {
        let project_name = config.project_name.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Project");

        Ok(format!(
            "# {} - VS Code Claude Export\n\n\
             **Generated by Cortex on {}**\n\n\
             This directory contains a complete context package for VS Code Claude development.\n\n\
             ## Contents\n\n\
             - **CONTEXT.md** - Comprehensive project context (architecture, structure, patterns)\n\
             - **STARTER_PROMPT.md** - Initial prompt for starting a development session\n\
             - **prompts/** - Feature-specific prompt templates\n\
             - **.claude/** - VS Code Claude configuration\n\n\
             ## Export Statistics\n\n\
             - **Files Exported**: {}\n\
             - **Files with Embeddings**: {}\n\
             - **Total Size**: {}\n\
             - **Estimated Tokens**: ~{}\n\n\
             ## How to Use\n\n\
             ### In VS Code Claude:\n\n\
             1. **Start a new session** in VS Code Claude\n\
             2. **Load CONTEXT.md** first: \"@CONTEXT.md\"\n\
             3. **Load STARTER_PROMPT.md**: \"@STARTER_PROMPT.md\"\n\
             4. **Begin development**: Claude now has full project context\n\n\
             ### For Specific Tasks:\n\n\
             1. Choose a prompt template from `prompts/`:\n\
                - `ADD_FEATURE.md` - Implement a new feature\n\
                - `FIX_BUG.md` - Fix a bug\n\
                - `REFACTOR.md` - Refactor code\n\
                - `ADD_TESTS.md` - Add test coverage\n\
                - `DOCUMENTATION.md` - Update documentation\n\n\
             2. Replace placeholders with your specific requirements\n\
             3. Load the customized prompt in Claude\n\
             4. Claude will implement following project patterns\n\n\
             ## Tips\n\n\
             - Always load CONTEXT.md at the start of each session\n\
             - Reference specific files when asking questions\n\
             - Ask Claude to explain architecture if unclear\n\
             - Use prompt templates for consistent implementations\n\n\
             ---\n\n\
             *Generated by [Cortex](https://github.com/yourusername/cortex) - \
             AI-powered local file intelligence system*\n",
            project_name,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            stats.total_files,
            stats.files_with_embeddings,
            crate::export::format_file_size(stats.total_size_bytes),
            stats.total_chunks
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_directory() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path().join("test_export");

        // Directory should not exist yet
        assert!(!test_dir.exists());

        // Create a BundleBuilder (we need a dummy database)
        // This is just a unit test for directory creation
        // Real integration tests would use a proper database
    }
}
