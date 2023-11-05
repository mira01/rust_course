use std::env;
use std::net::{TcpStream, TcpListener};
use std::error::Error;
use std::io::{Read, Write};
use std::collections::HashMap;

use chat::message::{Message};
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

    let tcp = TcpListener::bind(host_port)?;
    let mut clients = HashMap::new();

    for stream in tcp.incoming() {
        let mut stream = stream.unwrap();
        let addr = stream.peer_addr().unwrap();
        clients.insert(addr.clone(), stream.try_clone().unwrap());
        loop {
            let message = Message::read_from_stream(&mut clients.get(&addr).unwrap());
            println!("message: {:?}", message);
            let message = message.unwrap();
            message.write_to_stream(&mut stream).unwrap();
        }
    }
    Ok(())
}
