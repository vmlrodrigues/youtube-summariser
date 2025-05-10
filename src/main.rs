use anyhow::{Context, Result};
use clap::Parser;
use dotenv::dotenv;
use std::path::Path;

mod transcript;
mod openai;
mod utils;

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
    
    // Get transcript (either from cache or by fetching)
    let transcript_path = Path::new("transcripts").join(format!("{}.txt", video_id));
    let transcript = if !transcript_path.exists() || args.force {
        println!("Fetching transcript...");
        let transcript_text = transcript::fetch_transcript(&video_id)
            .await
            .context("Failed to fetch transcript")?;
        
        // Save transcript to file
        utils::save_to_file(&transcript_path, &transcript_text)
            .context("Failed to save transcript")?;
        
        transcript_text
    } else {
        println!("Using cached transcript...");
        utils::read_from_file(&transcript_path)
            .context("Failed to read transcript from cache")?
    };
    
    // Generate summary
    println!("Generating summary...");
    let summary = openai::generate_summary(&transcript)
        .await
        .context("Failed to generate summary")?;
    
    // Save summary
    let summary_path = Path::new("summaries").join(format!("{}.md", video_id));
    utils::save_to_file(&summary_path, &summary)
        .context("Failed to save summary")?;
    
    // Generate highlights
    println!("Generating highlights...");
    let highlights = openai::generate_highlights(&transcript)
        .await
        .context("Failed to generate highlights")?;
    
    // Save highlights
    let highlights_path = Path::new("highlights").join(format!("{}.md", video_id));
    utils::save_to_file(&highlights_path, &highlights)
        .context("Failed to save highlights")?;
    
    println!("Process completed successfully!");
    println!("Summary saved to: {}", summary_path.display());
    println!("Highlights saved to: {}", highlights_path.display());
    
    Ok(())
}