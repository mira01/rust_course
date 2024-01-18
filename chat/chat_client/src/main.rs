use std::env;
use std::io::{self, BufWriter};
use std::net::TcpStream;

use log::{Level, error};
use anyhow::{Result,Error};

use chat_client::interactive;
const DEFAULT_ADDRESS: &str = "127.0.0.1:11111";

fn main() {
    stderrlog::new()
        .verbosity(Level::Error)
        .init()
        .unwrap();
    if let Err(e) = run() {
        error!("{}", e);
    }
}

fn run() -> Result<(), Error> {
    let host_port = env::args().nth(1).unwrap_or(DEFAULT_ADDRESS.into());

    let stdin = io::stdin();
    let stdout = BufWriter::new(io::stdout());
    let stderr = BufWriter::new(io::stderr());
    let tcp_read = TcpStream::connect(host_port)?;
    let tcp_write = tcp_read.try_clone()?;
    interactive::enter_loop(stdin, stdout, stderr, tcp_read, tcp_write)?;
    Ok(())
}
