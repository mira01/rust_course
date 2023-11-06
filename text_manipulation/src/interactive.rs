use crate::command::Command;
use crate::message::Message;
use std::error::Error;
use std::io::{BufRead, BufReader, Read, Result as IoResult, Write, Lines};
use std::sync::mpsc;
use std::thread;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

use chrono;

#[derive(Debug)]
enum Event{
    Command(crate::command::Command),
    Message(crate::message::Message),
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
    mut net_in: R2,
    mut net_out: W3
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
                    return ();
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

    let net_reader = thread::spawn(move || {
        let mut reader = BufReader::new(net_in);
        loop {
            let message = Message::read_from_stream(&mut reader).unwrap();
            net_reader_out.send(Event::Message(message)).unwrap();
        }
    });

    // Thread that executes the command and sends the result to writer thread
    let processor = thread::spawn(move || {
        while let Ok(event) = processor_in.recv() {
            let _  = match event {
                Event::Message(Message::Text(text)) => {
                    processor_out.send(Ok(format!("> {}", text).to_string()));
                },
                Event::Message(message) => {to_download.send(message);},
                Event::Command(Command::Quit) => return (),
                Event::Command(command) =>  {
                    match send_message(command, &mut net_out) {
                        Err(e) => processor_out.send(Err(e.to_string())),
                        Ok(_) => continue,
                    };
                }
            };
        }
    });

    let downloader = thread::spawn(move || {
       while let Ok(message) = downloader_in.recv() {
            match message {
                Message::File(ref name, ref content) => {
                    downloader_out.send(Ok(format!("downloading {}", name).to_string()));
                    downloader_out.send(download(message));
                }
                Message::File(ref name, ref content) => {
                    downloader_out.send(Ok(format!("downloading an image").to_string()));
                    downloader_out.send(download(message));
                }
                _ => continue,
            }
       };
    });

    // Thread that writes results to two output buffers: success buffer and error buffer
    let writer = thread::spawn(move || {
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
    println!("readerjoin");
    processor
        .join()
        .map_err(|_e| "Error in processing thread".to_string())?;
    println!("processor join");
    //let (stdout, stderr) = writer
    //    .join()
    //    .map_err(|_e| "Error in writing thread".to_string())?;
    Ok(())
}

fn get_command(raw_line: IoResult<String>) -> Result<Command, String> {
    let line = raw_line.map_err(|e| e.to_string())?;
    Command::try_from(&line as &str).map_err(|e| e.to_string())
}

fn send_message<T: Write>(command: Command, stream: &mut T) -> Result<(), Box<dyn Error>> {
    let message: Message = command.try_into()?;
    message.write_to_stream(stream)?;
    Ok(())
}

fn download(msg: Message) -> Result<String, String> {
    match msg {
        Message::File(name, content) => {
            store("files", name.clone(), content).map_err(|e| e.to_string())?;
            Ok(format!("> {} downloaded", name).to_string())
        },
        Message::Image(content) => {
            let name = chrono::offset::Local::now().to_string();
            store("files", name.clone(), content).map_err(|e| e.to_string())?;
            Ok(format!("image downloaded as {}", name).to_string())
        },
        _ => Err("cannot download".into())
    }
}

fn store(directory_name: &str, file_name: String, content: Vec<u8>) -> Result<(), Box<dyn Error>>{
   let mut path = PathBuf::new();
   path.push(current_dir()?);
   path.push(directory_name);
   fs::create_dir_all(&path)?;
   path.push(file_name);
   Ok(fs::write(path, content)?)
}

pub struct LineIterator<B: BufRead>(Lines<B>);
impl<B: BufRead> Iterator for LineIterator<B>{
    type Item = IoResult<String>;
    fn next(&mut self) -> Option<IoResult<String>>{
        let mut s = String::new();
        loop {
            let next = self.0.next();
            match next {
                Some(Ok(line)) => {
                    if !line.ends_with('\\'){
                        s.push_str(&line);
                        return Some(Ok(s));
                    } else {
                        // cut tre trailing backshlash;
                        s.push_str(line.strip_suffix('\\').unwrap());       
                        s.push('\n');       
                    }
                },
                _ => return next
            }

        }
    }
}
