use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequest, Role},
    Client,
};
use std::env;

/// Creates and returns an OpenAI client using API key from environment variables
fn create_openai_client() -> Result<Client<OpenAIConfig>> {
    // Check if OPENAI_API_KEY is set
    let api_key = env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY environment variable not set. Please set it in your .env file")?;
    
    // Create a client with the API key
    let config = OpenAIConfig::new().with_api_key(api_key);
    Ok(Client::with_config(config))
}

/// Generates a summary from a transcript using OpenAI
pub async fn generate_summary(transcript: &str) -> Result<String> {
    let client = create_openai_client()?;
    
    // Truncate transcript if it's too long (OpenAI has token limits)
    let truncated_transcript = if transcript.len() > 10000 {
        // Truncate to approximately 10k chars (about 2.5k tokens)
        // In real-world scenarios, you might want to chunk the transcript
        &transcript[0..10000]
    } else {
        transcript
    };
    
    // Create the chat completion request
    let request = CreateChatCompletionRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are a helpful assistant that generates concise, informative summaries of YouTube video transcripts. Focus on the main points, key insights, and important details. Format your response in Markdown.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(format!("Please provide a comprehensive summary of the following YouTube video transcript. Organize it with appropriate headings and bullet points where relevant:\n\n{}", truncated_transcript))
                .build()?,
        ],
        temperature: Some(0.7),
        max_tokens: Some(1500),
        ..Default::default()
    };
    
    // Send the request to the OpenAI API
    let response = client.chat().create(request).await
        .context("Failed to get response from OpenAI API")?;
    
    // Extract the summary from the response
    if let Some(choice) = response.choices.first() {
        if let Some(content) = &choice.message.content {
            return Ok(content.clone());
        }
    }
    
    Err(anyhow::anyhow!("No content received from OpenAI"))
}

/// Generates highlights of new or unusual information from a transcript using OpenAI
pub async fn generate_highlights(transcript: &str) -> Result<String> {
    let client = create_openai_client()?;
    
    // Truncate transcript if it's too long (OpenAI has token limits)
    let truncated_transcript = if transcript.len() > 10000 {
        &transcript[0..10000]
    } else {
        transcript
    };
    
    // Create the chat completion request
    let request = CreateChatCompletionRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are a specialist at identifying and highlighting new, unique, or unusual information from video transcripts. Focus on extracting insights that are not commonly known or that represent innovative thinking. Format your response in Markdown.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(format!("Analyze the following transcript and identify any new, unique, or unusual information. Highlight key insights that might not be widely known or that represent innovative thinking. Format your response with appropriate headings and emphasis:\n\n{}", truncated_transcript))
                .build()?,
        ],
        temperature: Some(0.7),
        max_tokens: Some(1000),
        ..Default::default()
    };
    
    // Send the request to the OpenAI API
    let response = client.chat().create(request).await
        .context("Failed to get response from OpenAI API")?;
    
    // Extract the highlights from the response
    if let Some(choice) = response.choices.first() {
        if let Some(content) = &choice.message.content {
            return Ok(content.clone());
        }
    }
    
    Err(anyhow::anyhow!("No content received from OpenAI"))
}