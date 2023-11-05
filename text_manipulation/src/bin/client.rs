use std::env;
use std::io::{self, BufWriter};
use std::net::TcpStream;
use std::error::Error;

use chat::interactive;
use chat::DEFAULT_ADDRESS;

fn main() {
    match run() {
        Err(e) => eprintln!("{}", e.to_string()),
        Ok(_) => (),
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let host_port = env::args()
        .nth(1)
        .or(Some(DEFAULT_ADDRESS.into()))
        .unwrap();

    let stdin = io::stdin();
    let stdout = BufWriter::new(io::stdout());
    let stderr = BufWriter::new(io::stderr());
    let tcp_read = TcpStream::connect(host_port)?;
    let tcp_write = tcp_read.try_clone()?;
    interactive::enter_loop(stdin, stdout, stderr, tcp_read, tcp_write)?;
    Ok(())
}
