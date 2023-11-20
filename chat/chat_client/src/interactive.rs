use std::env::current_dir;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Lines, Read, Result as IoResult, Write, Cursor};
use std::path::PathBuf;
use std::process;
use std::sync::mpsc;
use std::thread;

use chrono;
use image::{io::Reader as ImageReader, ImageOutputFormat};

use crate::command::Command;
use chat_lib::message::Message;

/// An event to be handled; Either incoming message or a command typed by user
#[derive(Debug)]
enum Event {
    Command(Command),
    Message(Message),
}

// I use a lot of unwraps inside threads. I do not know how to recover from these situations

/// A function that starts threads that do reading, executing and writing results
pub fn enter_loop<
    R1: Read + Send + 'static,
    R2: Read + Send + 'static,
    W1: Write + Send + 'static,
    W2: Write + Send + 'static,
    W3: Write + Send + 'static,
>(
    stdin: R1,
    mut stdout: W1,
    mut stderr: W2,
    net_in: R2,
    mut net_out: W3,
) -> Result<(), Box<dyn Error>> {
    let (reader_out, processor_in) = mpsc::channel::<Event>();
    let (processor_out, writer_in) = mpsc::channel::<Result<String, String>>();
    let (to_download, downloader_in) = mpsc::channel::<Message>();
    let reader_err = processor_out.clone();
    let downloader_out = processor_out.clone();
    let net_reader_out = reader_out.clone();

    // Thread that reads commands from input stream. In case of succes it passes command
    // to executing thread; otherwise it sends error to writer thread
    let reader = thread::spawn(move || {
        let stdin = BufReader::new(stdin);
        for line in LineIterator(stdin.lines()) {
            let c = get_command(line);
            match c {
                Ok(Command::Quit) => {
                    reader_out.send(Event::Command(Command::Quit)).unwrap();
                    return;
                }
                Ok(c) => {
                    reader_out.send(Event::Command(c)).unwrap();
                }
                Err(c) => {
                    reader_err.send(Err(c)).unwrap();
                }
            }
        }
    });

    // Thread that read from Tcp
    let _net_reader = thread::spawn(move || {
        let mut reader = BufReader::new(net_in);
        loop {
            if let Ok(message) = Message::read_from_stream(&mut reader) {
                net_reader_out.send(Event::Message(message)).unwrap();
            } else {
                // Reconnection not yet implemented
                process::exit(1);
            }
        }
    });

    // Thread that executes the command and sends the result to writer thread
    // or reads incoming message
    let processor = thread::spawn(move || {
        while let Ok(event) = processor_in.recv() {
            match event {
                Event::Message(Message::Text(text)) => {
                    let _ = processor_out.send(Ok(format!("> {}", text).to_string()));
                }
                Event::Message(message) => {
                    let _ = to_download.send(message);
                }
                Event::Command(Command::Quit) => return,
                Event::Command(command) => {
                    match send_message(command, &mut net_out) {
                        Err(e) => {
                            let _ = processor_out.send(Err(e.to_string()));
                        }
                        Ok(_) => continue,
                    };
                }
            };
        }
    });

    // Thread for downloading
    let _downloader = thread::spawn(move || {
        while let Ok(message) = downloader_in.recv() {
            match message {
                Message::File(ref name, _) => {
                    let _ = downloader_out.send(Ok(format!("downloading {}", name).to_string()));
                    let _ = downloader_out.send(download(message));
                }
                Message::Image(ref _content) => {
                    let _ = downloader_out.send(Ok("downloading an image".to_string()));
                    let _ = downloader_out.send(download(message));
                }
                _ => continue,
            }
        }
    });

    // Thread that writes results to two output buffers: success buffer and error buffer
    let _writer = thread::spawn(move || {
        while let Ok(text) = writer_in.recv() {
            match text {
                Ok(out) => {
                    writeln!(stdout, "{}", out).unwrap();
                    stdout.flush().unwrap();
                }
                Err(out) => {
                    writeln!(stderr, "\x1b[0;31m{}\x1b[0m", out).unwrap();
                    stderr.flush().unwrap();
                }
            }
        }
        (stdout, stderr)
    });

    reader
        .join()
        .map_err(|_e| "Error in reading thread".to_string())?;
    processor
        .join()
        .map_err(|_e| "Error in processing thread".to_string())?;
    // no need to wait for other threads
    Ok(())
}

/// Reads a Command
fn get_command(raw_line: IoResult<String>) -> Result<Command, String> {
    let line = raw_line.map_err(|e| e.to_string())?;
    Command::try_from(&line as &str).map_err(|e| e.to_string())
}

/// Given a Command and a Stream, tries to compose a message and write it to the stream
fn send_message<T: Write>(command: Command, stream: &mut T) -> Result<(), Box<dyn Error>> {
    let message: Message = command.try_into()?;
    message.write_to_stream(stream)?;
    Ok(())
}

/// Given an incoming file or image message, stores its content as a file
fn download(msg: Message) -> Result<String, String> {
    match msg {
        Message::File(name, content) => {
            store("files", name.clone(), content).map_err(|e| e.to_string())?;
            Ok(format!("> {} downloaded", name).to_string())
        }
        Message::Image(content) => {
            let mut name = chrono::offset::Local::now().to_string();
            name.push_str(".png");
            let converted = {
                let mut buf: Vec<u8> = Vec::new();
                let _ = convert(content, &mut buf).map_err(|e| e.to_string());
                buf
            };
            store("images", name.clone(), converted).map_err(|e| e.to_string())?;
            Ok(format!("image downloaded as {}", name).to_string())
        }
        _ => Err("cannot download".into()),
    }
}

/// Convert image buffer into png image buffer
fn convert(orig_buf: Vec<u8>, output_buf: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    let img = ImageReader::new(Cursor::new(orig_buf))
        .with_guessed_format()?
        .decode()?;
    img.write_to(&mut Cursor::new(output_buf), ImageOutputFormat::Png)?;
    Ok(())
}

/// Given directory_name, file_name and content prepares directory structure and strore content as
/// a file.
fn store(directory_name: &str, file_name: String, content: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut path = PathBuf::new();
    path.push(current_dir()?);
    path.push(directory_name);
    fs::create_dir_all(&path)?;
    path.push(file_name);
    Ok(fs::write(path, content)?)
}

pub struct LineIterator<B: BufRead>(Lines<B>);
impl<B: BufRead> Iterator for LineIterator<B> {
    type Item = IoResult<String>;
    fn next(&mut self) -> Option<IoResult<String>> {
        let mut s = String::new();
        loop {
            let next = self.0.next();
            match next {
                Some(Ok(line)) => {
                    if !line.ends_with('\\') {
                        s.push_str(&line);
                        return Some(Ok(s));
                    } else {
                        // cut tre trailing backshlash;
                        s.push_str(line.strip_suffix('\\').unwrap());
                        s.push('\n');
                    }
                }
                _ => return next,
            }
        }
    }
}
