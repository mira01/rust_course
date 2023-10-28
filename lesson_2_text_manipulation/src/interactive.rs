use crate::command::Command;
use std::error::Error;
use std::io::{BufRead, BufReader, Read, Result as IoResult, Write};
use std::sync::mpsc;
use std::thread;

// I use a lot of unwraps inside threads. I do not know how to recover from these situations
pub fn enter_loop<
    R: Read + Send + 'static,
    W1: Write + Send + 'static,
    W2: Write + Send + 'static,
>(
    stdin: R,
    mut stdout: W1,
    mut stderr: W2,
) -> Result<(W1, W2), Box<dyn Error>> {
    let (reader_out, processor_in) = mpsc::channel::<Command>();
    let (processor_out, writer_in) = mpsc::channel::<Result<String, String>>();
    let reader_err = processor_out.clone();

    let reader = thread::spawn(move || {
        let stdin = BufReader::new(stdin);
        for line in stdin.lines() {
            let c = get_command(line);
            match c {
                Ok(c) => {
                    reader_out.send(c).unwrap();
                }
                Err(c) => {
                    reader_err.send(Err(c)).unwrap();
                }
            }
        }
    });

    let processor = thread::spawn(move || {
        while let Ok(command) = processor_in.recv() {
            let res = command.execute().map_err(|e| e.to_string());
            processor_out.send(res).unwrap();
        }
    });

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
    processor
        .join()
        .map_err(|_e| "Error in processing thread".to_string())?;
    let (stdout, stderr) = writer
        .join()
        .map_err(|_e| "Error in writing thread".to_string())?;
    Ok((stdout, stderr))
}

fn get_command(raw_line: IoResult<String>) -> Result<Command, String> {
    let line = raw_line.map_err(|e| e.to_string())?;
    Command::try_from(&line as &str).map_err(|e| e.to_string())
}

#[cfg(test)]
mod test {
    use super::enter_loop;
    use std::io::Cursor;

    fn test_streams(input: String, expected_out: &'static str, expected_err: &'static str) {
        let (stdout, stderr) = (vec![], vec![]);
        let stdin = String::into_bytes(input);
        let (stdout, stderr) = enter_loop(Cursor::new(stdin), stdout, stderr).unwrap();
        let stdout = std::str::from_utf8(&stdout).unwrap();
        let stderr = std::str::from_utf8(&stderr).unwrap();
        println!("stdout: {:?}, stderr {:?}", stdout, stderr);
        assert_eq!(expected_out, stdout);
        assert_eq!(expected_err, stderr);
    }

    #[test]
    fn uppercase_works() {
        test_streams("uppercase vole padni".to_string(), "VOLE PADNI \n", "")
    }

    #[test]
    fn unknown_method_works() {
        test_streams(
            "blabla vole padni".to_string(),
            "",
            "\u{1b}[0;31mUnknown method blabla\u{1b}[0m\n",
        )
    }
}
