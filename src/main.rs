use std::env;
use std::fs;
use std::io::{self, Read};
use pdf_extract::extract_text_from_mem;
use gtts::save_to_file; // Use gtts crate function
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

async fn text_to_audio_to_file(
    text: &str,
    filename: &str,
    chunk_size: usize,
) -> Result<(), AudioraError> {
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
        let chunks = sentence_chars.chunks(chunk_size);

        for chunk in chunks {
            let chunk_str: String = chunk.iter().collect();
            let chunk_filename = format!("{}/{}_chunk_{}.mp3", output_dir, filename, chunk_index);

            if !save_to_file(&chunk_str, &chunk_filename) {
                eprintln!("Error saving chunk {}.", chunk_index);
                continue;
            }

            println!("Saved chunk {} to file: {}", chunk_index, chunk_filename);

            chunk_index += 1;

            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

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

    match extract_text_from_pdf(pdf_path).await {
        Ok(text) => {
            println!("Extracted text:\n{}", text);

            if text.trim().is_empty() {
                eprintln!("Warning: The extracted text is empty.");
                return;
            }

            let output_file_base = "output_audio";
            if let Err(e) = text_to_audio_to_file(&text, output_file_base, 100).await {
                eprintln!("Audio conversion error: {}", e);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
