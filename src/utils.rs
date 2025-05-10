use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use crate::transcript::VideoMetadata;

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

/// Creates a directory for a video and returns the path
pub fn create_video_directory(video_id: &str) -> Result<PathBuf> {
    let video_dir = Path::new("output").join(video_id);
    fs::create_dir_all(&video_dir).context(format!("Failed to create directory for video: {}", video_id))?;
    Ok(video_dir)
}

/// Creates all required files for a video in its directory
pub fn save_video_files(metadata: &VideoMetadata) -> Result<()> {
    // Create the video directory
    let video_dir = create_video_directory(&metadata.video_id)?;
    
    // Save the transcript
    save_to_file(&video_dir.join("transcript.txt"), &metadata.transcript)?;
    
    // Save the metadata (title and description)
    let info_content = format!("# {}\n\n{}", metadata.title, metadata.description);
    save_to_file(&video_dir.join("info.md"), &info_content)?;
    
    // Create empty summary and highlights files (to be filled later)
    save_to_file(&video_dir.join("summary.md"), "")?;
    save_to_file(&video_dir.join("highlights.md"), "")?;
    
    Ok(())
}

/// Updates or creates the summary file for a video
pub fn save_summary(video_id: &str, summary: &str) -> Result<PathBuf> {
    let video_dir = Path::new("output").join(video_id);
    let summary_path = video_dir.join("summary.md");
    save_to_file(&summary_path, summary)?;
    Ok(summary_path)
}

/// Updates or creates the highlights file for a video
pub fn save_highlights(video_id: &str, highlights: &str) -> Result<PathBuf> {
    let video_dir = Path::new("output").join(video_id);
    let highlights_path = video_dir.join("highlights.md");
    save_to_file(&highlights_path, highlights)?;
    Ok(highlights_path)
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

/// Checks if a video directory already exists
pub fn video_exists(video_id: &str) -> bool {
    let video_dir = Path::new("output").join(video_id);
    video_dir.exists()
}

/// Gets the transcript path for a video
pub fn get_transcript_path(video_id: &str) -> PathBuf {
    Path::new("output").join(video_id).join("transcript.txt")
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