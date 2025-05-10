use anyhow::{Context, Result};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Structure to hold video metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub video_id: String,
    pub title: String,
    pub description: String,
    pub transcript: String,
}

/// Fetches the transcript and metadata for a YouTube video
pub async fn fetch_video_data(video_id: &str) -> Result<VideoMetadata> {
    // Create a reqwest client with appropriate timeouts
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;

    // First, we need to make a request to get the video page to extract metadata
    let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
    let response = client.get(&video_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await
        .context("Failed to fetch YouTube video page")?;
    
    let html = response.text().await.context("Failed to get YouTube page content")?;

    // Extract title, description, and captions URL from the HTML
    let title = extract_video_title(&html)
        .context("Failed to extract video title")?;
    
    let description = extract_video_description(&html)
        .context("Failed to extract video description")?;
    
    let captions_url = extract_captions_url(&html)
        .context("Failed to extract captions URL")?;
    
    // Fetch the transcript data from the captions URL
    let transcript_response = client.get(&captions_url)
        .send()
        .await
        .context("Failed to fetch transcript data")?;
    
    let transcript_data = transcript_response.text().await
        .context("Failed to get transcript content")?;
    
    // Parse and format the transcript
    let transcript = parse_transcript_data(&transcript_data)
        .context("Failed to parse transcript data")?;
    
    // Return the complete video metadata
    Ok(VideoMetadata {
        video_id: video_id.to_string(),
        title,
        description,
        transcript,
    })
}

/// Extract the captions URL from the video page HTML
fn extract_captions_url(html: &str) -> Result<String> {
    // Look for the captions track in the HTML
    let re = Regex::new(r#"\"captionTracks\":\[\{\"baseUrl\":\"(.*?)\","#)
        .context("Failed to compile regex")?;
    
    if let Some(captures) = re.captures(html) {
        if let Some(url) = captures.get(1) {
            // URL is escaped in the JSON, so we need to unescape it
            let escaped_url = url.as_str().replace("\\u0026", "&");
            return Ok(escaped_url);
        }
    }
    
    // Alternative method: try to find the playerCaptionsTracklistRenderer
    let re_alt = Regex::new(r#"\"playerCaptionsTracklistRenderer\".*?\"captionTracks\":\s*\[\s*\{\s*\"baseUrl\":\s*\"(.*?)\""#)
        .context("Failed to compile alternative regex")?;
    
    if let Some(captures) = re_alt.captures(html) {
        if let Some(url) = captures.get(1) {
            let escaped_url = url.as_str().replace("\\u0026", "&");
            return Ok(escaped_url);
        }
    }
    
    // If we couldn't find the captions URL, this video might not have captions
    Err(anyhow::anyhow!("No caption tracks found for this video"))
}

/// Parse and format the transcript data
fn parse_transcript_data(data: &str) -> Result<String> {
    // The transcript data is in XML format
    let re_text = Regex::new(r#"<text.*?>(.*?)</text>"#)
        .context("Failed to compile text regex")?;
    
    let mut transcript = String::new();
    
    for cap in re_text.captures_iter(data) {
        if let Some(text) = cap.get(1) {
            // Decode HTML entities
            let decoded = decode_html_entities(text.as_str());
            transcript.push_str(&decoded);
            transcript.push_str(" ");
        }
    }
    
    if transcript.is_empty() {
        return Err(anyhow::anyhow!("Failed to extract any text from transcript data"));
    }
    
    Ok(transcript)
}

/// Decode common HTML entities
fn decode_html_entities(text: &str) -> String {
    let mut result = text.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");
    
    // Handle numeric entities
    let re_numeric = Regex::new(r"&#(\d+);").unwrap();
    while let Some(cap) = re_numeric.captures(&result) {
        if let Some(num) = cap.get(1) {
            if let Ok(code) = num.as_str().parse::<u32>() {
                if let Some(c) = char::from_u32(code) {
                    result = result.replace(&cap[0], &c.to_string());
                    continue;
                }
            }
        }
        // If we couldn't parse the entity, just remove it
        result = result.replace(&cap[0], "");
    }
    
    result
}

/// Extract the video title from the HTML
fn extract_video_title(html: &str) -> Result<String> {
    // Try to find the title in various patterns used by YouTube
    let patterns = [
        r#"<meta property="og:title" content="(.*?)">"#,
        r#"<meta name="title" content="(.*?)">"#,
        r#"<title>(.*?) - YouTube</title>"#,
    ];
    
    for pattern in patterns {
        let re = Regex::new(pattern).context("Failed to compile title regex")?;
        
        if let Some(captures) = re.captures(html) {
            if let Some(title) = captures.get(1) {
                return Ok(decode_html_entities(title.as_str()));
            }
        }
    }
    
    // If we can't find a title, return a generic one
    Ok("Untitled YouTube Video".to_string())
}

/// Extract the video description from the HTML
fn extract_video_description(html: &str) -> Result<String> {
    // Try to find the description in various patterns used by YouTube
    let patterns = [
        r#"<meta property="og:description" content="(.*?)">"#,
        r#"<meta name="description" content="(.*?)">"#,
        r#""description":\s*"(.*?)(?<!\\)(?:\\\\)*""# // From the JSON data
    ];
    
    for pattern in patterns {
        let re = Regex::new(pattern).context("Failed to compile description regex")?;
        
        if let Some(captures) = re.captures(html) {
            if let Some(description) = captures.get(1) {
                // YouTube descriptions can be quite long, so we might want to truncate them
                let desc = decode_html_entities(description.as_str());
                return Ok(desc);
            }
        }
    }
    
    // If we can't find a description, return an empty one
    Ok("No description available.".to_string())
}

// Fallback method removed to avoid unused code warning