use std::sync::mpsc;
use std::thread;
use std::io::{Result as IoResult, stdin, stderr, stdout};
use crate::command::{Command};
use crate::StringResult;

pub fn enter_loop() {

    let (reader_out, processor_in) = mpsc::channel::<Command>();
    let (processor_out, writer_in) = mpsc::channel::<Result<String, String>>();
    let reader_err = processor_out.clone();

    let processor = thread::spawn(move||{
        while let Ok(command) = processor_in.recv(){
           let res = command.execute().map_err(|e|e.to_string()); 
           processor_out.send(res);
        }
    });

    let reader = thread::spawn(move ||{
        for line in stdin().lines(){
           let c = get_command(line);
           match c {
             Ok(c) => {reader_out.send(c);},
             Err(c) => {reader_err.send(Err(c));},
           }
        }
    });

    let writer = thread::spawn(move ||{
        while let Ok(text) = writer_in.recv(){
            match text {
                Ok(out) => println!("{}", out), 
                Err(out) => eprintln!("{}", out), 
            }
        }
    });

    reader.join();
    processor.join();
    writer.join();
}

fn get_command(raw_line: IoResult<String>) -> Result<Command, String> {
   let line = raw_line.map_err(|e|e.to_string())?;
   Command::try_from(&line as &str).map_err(|e|e.to_string())
}
