//! Model Downloader Utility
//!
//! Downloads the all-MiniLM-L6-v2 ONNX model and tokenizer from Hugging Face.

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// URLs for model files on Hugging Face
const MODEL_REPO: &str = "sentence-transformers/all-MiniLM-L6-v2";
const HF_BASE_URL: &str = "https://huggingface.co";

/// Model files to download
const MODEL_FILES: &[(&str, &str)] = &[
    ("model.onnx", "onnx/model.onnx"),
    ("tokenizer.json", "tokenizer.json"),
];

/// Get the path to the models directory
pub fn get_models_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let models_dir = home.join(".cortex").join("models").join("all-MiniLM-L6-v2");

    Ok(models_dir)
}

/// Check if model files are already downloaded
pub fn is_model_downloaded() -> Result<bool> {
    let models_dir = get_models_dir()?;

    for (filename, _) in MODEL_FILES {
        let file_path = models_dir.join(filename);
        if !file_path.exists() {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Download all required model files
pub fn download_model() -> Result<()> {
    let models_dir = get_models_dir()?;

    // Create models directory if it doesn't exist
    fs::create_dir_all(&models_dir)
        .context("Failed to create models directory")?;

    log::info!("Downloading all-MiniLM-L6-v2 model to {:?}", models_dir);

    for (filename, repo_path) in MODEL_FILES {
        let file_path = models_dir.join(filename);

        // Skip if already exists
        if file_path.exists() {
            log::info!("{} already exists, skipping", filename);
            continue;
        }

        // Construct download URL
        let url = format!("{}/{}/resolve/main/{}", HF_BASE_URL, MODEL_REPO, repo_path);

        log::info!("Downloading {} from {}", filename, url);

        // Download file
        let response = reqwest::blocking::get(&url)
            .context(format!("Failed to download {}", filename))?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download {}: HTTP {}", filename, response.status());
        }

        // Save to file
        let bytes = response.bytes()
            .context(format!("Failed to read response for {}", filename))?;

        let mut file = File::create(&file_path)
            .context(format!("Failed to create file {}", filename))?;

        file.write_all(&bytes)
            .context(format!("Failed to write {}", filename))?;

        log::info!("Downloaded {} ({} bytes)", filename, bytes.len());
    }

    log::info!("Model download complete!");

    Ok(())
}

/// Ensure model is downloaded, download if not
pub fn ensure_model_downloaded() -> Result<()> {
    if !is_model_downloaded()? {
        log::info!("Model not found, downloading...");
        download_model()?;
    } else {
        log::info!("Model already downloaded");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_models_dir() {
        let dir = get_models_dir().unwrap();
        assert!(dir.to_str().unwrap().contains(".cortex"));
        assert!(dir.to_str().unwrap().contains("models"));
        assert!(dir.to_str().unwrap().contains("all-MiniLM-L6-v2"));
    }

    #[test]
    #[ignore] // Requires network access
    fn test_download_model() {
        // Clean up first if exists
        let models_dir = get_models_dir().unwrap();
        if models_dir.exists() {
            fs::remove_dir_all(&models_dir).ok();
        }

        // Download
        download_model().unwrap();

        // Verify files exist
        assert!(is_model_downloaded().unwrap());
    }
}
