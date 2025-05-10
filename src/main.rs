use anyhow::{Context, Result};
use clap::Parser;
use dotenv::dotenv;

mod transcript;
mod openai;
mod utils;

use transcript::VideoMetadata;

#[derive(Parser, Debug)]
#[command(name = "YouTube Summariser")]
#[command(author = "Rust Dev")]
#[command(version = "1.0")]
#[command(about = "Summarizes YouTube videos using their transcripts", long_about = None)]
struct Cli {
    /// URL of the YouTube video to summarize
    #[arg(required = true)]
    youtube_url: String,

    /// Force re-fetching transcript even if it exists locally
    #[arg(short, long, default_value = "false")]
    force: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Parse command line arguments
    let args = Cli::parse();
    
    // Extract video ID from URL
    let video_id = utils::extract_video_id(&args.youtube_url)
        .context("Failed to extract video ID from URL")?;
    
    println!("Processing YouTube video: {}", video_id);
    
    // Get video data (either from cache or by fetching)
    let transcript_path = utils::get_transcript_path(&video_id);
    let metadata = if !utils::video_exists(&video_id) || args.force {
        println!("Fetching video data...");
        let video_metadata = transcript::fetch_video_data(&video_id)
            .await
            .context("Failed to fetch video data")?;
        
        // Save video files
        utils::save_video_files(&video_metadata)
            .context("Failed to save video files")?;
        
        video_metadata
    } else {
        println!("Using cached transcript...");
        let transcript = utils::read_from_file(&transcript_path)
            .context("Failed to read transcript from cache")?;
        
        // Create a basic metadata object from the cached transcript
        // We don't have title/description from cache, but that's OK
        VideoMetadata {
            video_id: video_id.clone(),
            title: format!("YouTube Video {}", video_id),
            description: "Description not available for cached video.".to_string(),
            transcript,
        }
    };
    
    // Generate summary
    println!("Generating summary...");
    let summary = openai::generate_summary(&metadata.transcript)
        .await
        .context("Failed to generate summary")?;
    
    // Save summary
    let _summary_path = utils::save_summary(&video_id, &summary)
        .context("Failed to save summary")?;
    
    // Generate highlights
    println!("Generating highlights...");
    let highlights = openai::generate_highlights(&metadata.transcript)
        .await
        .context("Failed to generate highlights")?;
    
    // Save highlights
    let _highlights_path = utils::save_highlights(&video_id, &highlights)
        .context("Failed to save highlights")?;
    
    println!("Process completed successfully!");
    println!("Video: {}", metadata.title);
    println!("Files saved to: output/{}/", video_id);
    println!("  - info.md (title and description)");
    println!("  - transcript.txt");
    println!("  - summary.md");
    println!("  - highlights.md");
    
    Ok(())
}