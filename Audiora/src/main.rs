use std::env;
use std::fs;
use std::io::{self, Read, BufReader};
use pdf_extract::extract_text_from_mem;
use tts_rust::tts::GTTSClient;
use tts_rust::languages::Languages;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
enum AudioraError {
    IoError(io::Error),
    PdfError(String),
}

impl std::fmt::Display for AudioraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioraError::IoError(e) => write!(f, "IO Error: {}", e),
            AudioraError::PdfError(e) => write!(f, "PDF Error: {}", e),
        }
    }
}

impl From<io::Error> for AudioraError {
    fn from(error: io::Error) -> Self {
        AudioraError::IoError(error)
    }
}

async fn extract_text_from_pdf(pdf_path: &str) -> Result<String, AudioraError> {
    println!("Attempting to open file: {}", pdf_path);
    let file = fs::File::open(pdf_path)?;
    println!("PDF file opened successfully!");
    let buffer = read_file(file)?;
    extract_text_from_mem(&buffer)
        .map_err(|e| AudioraError::PdfError(format!("Failed to extract text from PDF: {}", e)))
}

fn read_file(mut file: fs::File) -> Result<Vec<u8>, io::Error> {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut start = 0;

    for (i, c) in text.char_indices() {
        // Check for sentence-ending punctuation
        if c == '.' || c == '!' || c == '?' {
            let sentence = &text[start..=i]; // Include punctuation mark
            sentences.push(sentence.trim().to_string());
            start = i + 1; // Skip past the punctuation
        }
    }

    // Push the remaining part of the text if any
    if start < text.len() {
        sentences.push(text[start..].trim().to_string());
    }

    sentences
}

async fn text_to_audio_to_file_and_play(
    text: &str,
    filename: &str,
    chunk_size: usize,
    audio_sender: mpsc::Sender<String>,
) -> Result<(), AudioraError> {
    let narrator = Arc::new(GTTSClient {
        volume: 1.0,
        language: Languages::English,
        tld: "com",
    });

    let output_dir = "audio_output";
    fs::create_dir_all(output_dir).map_err(|e| {
        AudioraError::IoError(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create output directory: {}", e),
        ))
    })?;

    let sentences = split_into_sentences(text);
    let mut chunk_index = 0;

    for sentence in sentences {
        let sentence_chars: Vec<_> = sentence.chars().collect();
        
        // If the sentence is too long, break it into smaller chunks
        let mut chunks = sentence_chars.chunks(chunk_size);
        while let Some(chunk) = chunks.next() {
            let chunk_str: String = chunk.iter().collect();
            let chunk_filename = format!("{}/{}_chunk_{}.mp3", output_dir, filename, chunk_index);

            // Save the audio to a file
            if let Err(e) = narrator.save_to_file(&chunk_str, &chunk_filename) {
                eprintln!("Error saving chunk {}: {}", chunk_index, e);
                continue;
            }

            // Send the filename to the playback task
            audio_sender.send(chunk_filename).await.map_err(|e| {
                AudioraError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to send chunk {}: {}", chunk_index, e),
                ))
            })?;

            println!("Successfully sent chunk {} to the receiver.", chunk_index);
            chunk_index += 1;

            // Add a slight pause for smoother transitions between chunks
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

    Ok(())
}

async fn play_audio_concurrently(
    mut receiver: mpsc::Receiver<String>,
    stream_handle: OutputStreamHandle,
) -> Result<(), AudioraError> {
    let sink = Sink::try_new(&stream_handle).unwrap();

    while let Some(file_path) = receiver.recv().await {
        println!("Received audio file path: {}", file_path);
        let file = fs::File::open(file_path).map_err(|e| {
            AudioraError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to open audio file: {}", e),
            ))
        })?;
        let source = rodio::Decoder::new(BufReader::new(file)).map_err(|e| {
            AudioraError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to decode audio: {}", e),
            ))
        })?;

        sink.append(source);
    }

    println!("Finished playing all audio.");
    sink.sleep_until_end();

    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_pdf>", args[0]);
        return;
    }

    let pdf_path = &args[1];
    println!("Running with PDF path: {}", pdf_path);

    let (audio_sender, audio_receiver) = mpsc::channel(100);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let playback_handle = tokio::spawn(play_audio_concurrently(audio_receiver, stream_handle));

    match extract_text_from_pdf(pdf_path).await {
        Ok(text) => {
            println!("Extracted text:\n{}", text);

            if text.trim().is_empty() {
                eprintln!("Warning: The extracted text is empty.");
                return;
            }

            let output_file_base = "output_audio";
            if let Err(e) = text_to_audio_to_file_and_play(&text, output_file_base, 100, audio_sender).await {
                eprintln!("Audio conversion or playback error: {}", e);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    if let Err(e) = playback_handle.await {
        eprintln!("Error in playback task: {}", e);
    }
}
