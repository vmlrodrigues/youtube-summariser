use anyhow::{Context, Result};
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

/// Fetches the transcript for a YouTube video
pub async fn fetch_transcript(video_id: &str) -> Result<String> {
    // Create a reqwest client with appropriate timeouts
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;

    // First, we need to make a request to get the video page to extract some metadata
    let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
    let response = client.get(&video_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await
        .context("Failed to fetch YouTube video page")?;
    
    let html = response.text().await.context("Failed to get YouTube page content")?;

    // Extract captions URL from the HTML
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
    parse_transcript_data(&transcript_data)
        .context("Failed to parse transcript data")
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

// Fallback method removed to avoid unused code warning