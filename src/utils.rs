use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Extracts the YouTube video ID from various formats of YouTube URLs
pub fn extract_video_id(url: &str) -> Result<String> {
    // Match common YouTube URL patterns
    let patterns = [
        r"(?:youtube\.com/watch\?v=|youtu\.be/|youtube\.com/embed/|youtube\.com/v/|youtube\.com/e/|youtube\.com/shorts/)([^&?/\s]{11})",
        r"youtube\.com/watch.*[\?&]v=([^&?/\s]{11})",
        r"youtube\.com/shorts/([^&?/\s]{11})"
    ];

    for pattern in patterns {
        let regex = Regex::new(pattern).context("Failed to compile regex")?;
        if let Some(captures) = regex.captures(url) {
            if let Some(id) = captures.get(1) {
                return Ok(id.as_str().to_string());
            }
        }
    }

    Err(anyhow::anyhow!("Could not extract YouTube video ID from URL: {}", url))
}

/// Saves content to a file, creating directories if they don't exist
pub fn save_to_file(path: &Path, content: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create directory")?;
    }
    
    fs::write(path, content).context(format!("Failed to write to file: {}", path.display()))?;
    Ok(())
}

/// Reads content from a file
pub fn read_from_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).context(format!("Failed to read file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id() {
        let test_cases = vec![
            ("https://www.youtube.com/watch?v=dQw4w9WgXcQ", Ok("dQw4w9WgXcQ".to_string())),
            ("https://youtu.be/dQw4w9WgXcQ", Ok("dQw4w9WgXcQ".to_string())),
            ("https://youtube.com/shorts/dQw4w9WgXcQ", Ok("dQw4w9WgXcQ".to_string())),
            ("https://www.youtube.com/embed/dQw4w9WgXcQ", Ok("dQw4w9WgXcQ".to_string())),
            ("https://invalid-url.com", Err(())),
        ];

        for (url, expected) in test_cases {
            match (extract_video_id(url), expected) {
                (Ok(id), Ok(expected_id)) => assert_eq!(id, expected_id),
                (Err(_), Err(_)) => {}, // Both are errors, that's fine
                _ => panic!("Mismatch for URL: {}", url),
            }
        }
    }
}