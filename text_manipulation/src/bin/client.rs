use std::env;
use std::error::Error;
use std::io::{self, BufWriter};
use std::net::TcpStream;

use chat::interactive;
use chat::DEFAULT_ADDRESS;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let host_port = env::args().nth(1).unwrap_or(DEFAULT_ADDRESS.into());

    let stdin = io::stdin();
    let stdout = BufWriter::new(io::stdout());
    let stderr = BufWriter::new(io::stderr());
    let tcp_read = TcpStream::connect(host_port)?;
    let tcp_write = tcp_read.try_clone()?;
    interactive::enter_loop(stdin, stdout, stderr, tcp_read, tcp_write)?;
    Ok(())
}
