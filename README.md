# Audiora

## Description

Audiora is a command-line application that extracts text from PDF files and converts it into audio using text-to-speech technology. It allows users to listen to the content of their documents in a natural-sounding voice.

## Features

- Extracts text from PDF files.
- Converts text and saved into audio files (MP3 format).
- Plays audio files sequentially.
- Processes long text by splitting it into manageable chunks for smoother playback.

## Requirements

- **Operating System**: Linux (Ubuntu recommended) or macOS.
- **Rust**: Version 1.42.0 or later.
- A terminal or command-line interface.

## Installation

1. **Clone the repository:**

    ```bash
    git clone https://github.com/Huntspeedy/Audiora.git
    cd Audiora
    ```

2. **Build the application in release mode:**

    ```bash
    cargo build --release
    ```

The compiled binary will be located at
```bash
target/release/audiora`.
```

## Usage

### Running the Program

Navigate to the release directory:

```bash
cd target/release
```
Run the program with a PDF file as input:

```bash
./audiora sample.pdf
```
Replace sample.pdf with the path to your PDF file.

## Output
The extracted audio files will be saved in the audio_output directory.

Each section of the PDF text will be saved as a separate audio chunk.

## Notes
Ensure the program has execute permissions:

```bash
chmod +x audiora
```
A sample PDF (sample.pdf) is included in the release directory for testing purposes.

The application can handle large PDFs by splitting the text into smaller chunks for efficient processing and playback.

## Contributing
Contributions are welcome! Feel free to submit a pull request or open an issue for bug reports or feature requests.

## Contact
For any questions or feedback, please contact:

GitHub: Huntspeedy
Email: vibro22@example.com
