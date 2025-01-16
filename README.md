# Audiora - Text-to-Speech PDF Reader
Audiora is a Rust-based application that converts text from a PDF document into speech. It extracts text from the PDF, splits it into smaller chunks, converts each chunk to an audio file, and plays it sequentially. This tool uses Google Text-to-Speech (GTTS) and the Rodio crate for audio playback.

#Features
PDF Text Extraction: Extracts text from PDF files.
Text-to-Speech (TTS): Converts the extracted text into speech using Google’s Text-to-Speech.
Audio Chunking: If a sentence is too long, it is broken into smaller chunks for better playback control.
Concurrent Audio Playback: Plays the audio files in sequence without blocking the program.
Audio File Output: Saves the audio as .mp3 files in the output directory.
Installation
To run the Audiora application, you will need Rust and some dependencies. Follow these steps to install it:

#1. Clone the repository:
**git clone https://github.com/your-username/audiora.git
cd audiora**

#2. Install Rust:
If you haven't installed Rust yet, you can install it using the following command:


**curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh**
Make sure to follow the instructions to set up your Rust environment.

#3. Install Dependencies:
Once you have Rust set up, navigate to your project directory and run:

**cargo build --release**
This will download and compile the required dependencies.

#Usage
Running the Application:
To run the program, provide a path to the PDF file you want to convert:

**cargo run -- <path_to_pdf>**
This will extract the text from the provided PDF, convert it into speech, save it as .mp3 files, and play the audio.

Example:
bash
Copy
Edit
cargo run -- example.pdf
This command will process example.pdf, save the audio files in the audio_output directory, and start playing them.

Project Structure
src/: Contains the main Rust code for text extraction, TTS conversion, and audio playback.
Cargo.toml: The configuration file that manages dependencies for the project.
audio_output/: The directory where the generated audio files are stored.
Dependencies
This project uses the following Rust crates:

clap: Command-line argument parser.
docx-rs: For handling DOCX files (not used in current version but may be useful for future updates).
pdf-extract: Extracts text from PDF files.
tts_rust: Google Text-to-Speech (TTS) client.
rodio: For audio playback.
tokio: Asynchronous runtime for running concurrent tasks.
Contributing
Contributions are welcome! Feel free to fork the repository and submit issues and pull requests.

To contribute:

Fork the repository.
Create a feature branch (git checkout -b feature-branch).
Commit your changes (git commit -am 'Add new feature').
Push to the branch (git push origin feature-branch).
Open a pull request.
License
This project is licensed under the MIT License - see the LICENSE file for details.

Contact
For questions or suggestions, feel free to open an issue on GitHub or contact me via email.

