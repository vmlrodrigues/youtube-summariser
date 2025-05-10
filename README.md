# YouTube Summariser

A Rust-based command line tool that extracts transcripts from YouTube videos and uses OpenAI to generate summaries and highlight unique information.

## Features

- Extract transcripts from YouTube videos
- Store transcripts locally for future use
- Generate comprehensive summaries of the video content using OpenAI
- Identify and highlight new, unique, or unusual information from the videos
- Saves all outputs in Markdown format for easy reading

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest version)
- OpenAI API key

### Setup

1. Clone this repository:
   ```
   git clone https://github.com/yourusername/youtube-summariser.git
   cd youtube-summariser
   ```

2. Create an `.env` file based on the example:
   ```
   cp .env.example .env
   ```

3. Add your OpenAI API key to the `.env` file:
   ```
   OPENAI_API_KEY=your_openai_api_key_here
   ```

4. Build the project:
   ```
   cargo build --release
   ```

## Usage

Run the tool by providing a YouTube URL:

```
cargo run -- https://www.youtube.com/watch?v=VIDEO_ID
```

Or use the built executable directly:

```
./target/release/youtube-summariser https://www.youtube.com/watch?v=VIDEO_ID
```

### Options

- `--force` or `-f`: Force re-fetching the transcript even if it exists locally

## How It Works

1. The tool extracts the video ID from the provided YouTube URL
2. It fetches the transcript from YouTube (or loads it from local cache if available)
3. The transcript is sent to OpenAI's API to generate:
   - A comprehensive summary of the video content
   - Highlights of new or unusual information from the video
4. Results are saved as Markdown files in their respective directories

## Directory Structure

- `transcripts/`: Stores the raw text transcripts (named by video ID)
- `summaries/`: Contains generated summaries in Markdown format
- `highlights/`: Contains highlighted unique information in Markdown format

## Example

```
cargo run -- https://www.youtube.com/watch?v=dQw4w9WgXcQ
```

This will:
1. Extract the transcript for the video with ID "dQw4w9WgXcQ"
2. Save the transcript to `transcripts/dQw4w9WgXcQ.txt`
3. Generate a summary and save it to `summaries/dQw4w9WgXcQ.md`
4. Generate highlights and save them to `highlights/dQw4w9WgXcQ.md`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.