use std::env;
use std::fs;
use std::io::{self, Read};
use pdf_extract::extract_text_from_mem;
use tts_rust::tts::GTTSClient;
use tts_rust::languages::Languages;
use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;

#[derive(Debug)]
enum AudioraError {
    IoError(io::Error),
    PdfError(String),
    AudioError(String),
}

impl std::fmt::Display for AudioraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioraError::IoError(e) => write!(f, "IO Error: {}", e),
            AudioraError::PdfError(e) => write!(f, "PDF Error: {}", e),
            AudioraError::AudioError(e) => write!(f, "Audio Error: {}", e),
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
        if c == '.' || c == '!' || c == '?' {
            let sentence = &text[start..=i];
            sentences.push(sentence.trim().to_string());
            start = i + 1;
        }
    }

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
        let mut chunks = sentence_chars.chunks(chunk_size);
        
        while let Some(chunk) = chunks.next() {
            let chunk_str: String = chunk.iter().collect();
            let chunk_filename = format!("{}/{}_chunk_{}.mp3", output_dir, filename, chunk_index);

            if let Err(e) = narrator.save_to_file(&chunk_str, &chunk_filename) {
                eprintln!("Error saving chunk {}: {}", chunk_index, e);
                continue;
            }

            audio_sender.send(chunk_filename).await.map_err(|e| {
                AudioraError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to send chunk {}: {}", chunk_index, e),
                ))
            })?;

            println!("Successfully sent chunk {} to the receiver.", chunk_index);
            chunk_index += 1;
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

    Ok(())
}

async fn play_audio_concurrently(
    mut receiver: mpsc::Receiver<String>,
) -> Result<(), AudioraError> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or(AudioraError::AudioError("No output device available".to_string()))?;

    while let Some(file_path) = receiver.recv().await {
        println!("Received audio file path: {}", file_path);
        play_audio_file(&file_path, &device)?;
    }

    Ok(())
}

fn play_audio_file(file_path: &str, device: &cpal::Device) -> Result<(), AudioraError> {
    let file = fs::File::open(file_path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    
    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &Default::default(),
        &Default::default(),
    ).map_err(|e| AudioraError::AudioError(format!("Failed to probe audio: {}", e)))?;

    let mut format = probed.format;
    let track = format.default_track().ok_or(AudioraError::AudioError("No audio track found".to_string()))?;
    
    let decoder = symphonia::default::get_codecs().make(
        &track.codec_params,
        &Default::default(),
    ).map_err(|e| AudioraError::AudioError(format!("Failed to create decoder: {}", e)))?;

    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(44100),
        buffer_size: cpal::BufferSize::Default,
    };

    let err_fn = |err| eprintln!("Audio stream error: {}", err);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Fill buffer with audio data from decoder
            let buffer = decoder.decode().unwrap();
            // Convert and copy samples to output buffer
            // (Implementation depends on your specific audio format)
        },
        err_fn,
    ).map_err(|e| AudioraError::AudioError(format!("Failed to build audio stream: {}", e)))?;

    stream.play().map_err(|e| AudioraError::AudioError(format!("Failed to play stream: {}", e)))?;

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
    let playback_handle = tokio::spawn(play_audio_concurrently(audio_receiver));

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