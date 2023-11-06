use std::env;
use std::net::{TcpStream, TcpListener, Incoming, SocketAddr};
use std::error::Error;
use std::io::{Read, Write, Error as IoError};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use threadpool::ThreadPool;

use chat::message::{Message};
use chat::DEFAULT_ADDRESS;

const THREAD_COUNT: usize = 8;

fn main() {
    match run() {
        Err(e) => eprintln!("{}", e.to_string()),
        Ok(_) => (),
    }
}

#[derive(Debug)]
struct StoredData {
    address: SocketAddr,
    stream: TcpStream,
    message: Message,
}

fn run() -> Result<(), Box<dyn Error>> {
    let host_port = env::args()
        .nth(1)
        .or(Some(DEFAULT_ADDRESS.into()))
        .unwrap(); // always Some

    let tcp = TcpListener::bind(host_port)?;
    let pool = ThreadPool::new(THREAD_COUNT);
    let (tx, rx) = mpsc::channel::<StoredData>();
    let (responses_out, responses_in) = mpsc::channel::<(Message, TcpStream)>();

    let responder = thread::spawn(move || {
        while let Ok((msg, mut stream)) = responses_in.recv() {
           msg.write_to_stream(&mut stream).unwrap();
        }
    });

    let processor = thread::spawn(move || {
        let mut clients = HashMap::new();
        while let Ok(data) = rx.recv(){
            println!("data in thread: {:?}", data);
            clients.insert(data.address.clone(), data.stream);
            for (address, stream) in &clients {
               if address != &data.address {
                    println!("would send message");
                    responses_out.send((data.message.clone(), stream.try_clone().unwrap())).unwrap();
               }
            }
        }
    });

    for stream in tcp.incoming() {
        let tx = tx.clone();
        pool.execute(move || {
            let mut stream = stream.unwrap();
            let address = stream.peer_addr().unwrap();
            let tx = tx.clone();
            //clients.insert(addr.clone(), stream.try_clone().unwrap());
            loop {
                let message = Message::read_from_stream(&mut stream);
                println!("message: {:?}", message);
                let message = message.unwrap();
                let data = StoredData{
                    message,
                    address,
                    stream: stream.try_clone().unwrap(),
                };
                let send_result = tx.send(data).unwrap();
            }
       
        });
    }

    Ok(())
}
