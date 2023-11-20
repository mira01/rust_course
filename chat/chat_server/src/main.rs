use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

use threadpool::ThreadPool;
use log::{Level, info, error};
use stderrlog;

use chat_lib::message::Message;

const DEFAULT_ADDRESS: &str = "127.0.0.1:11111";
const THREAD_COUNT: usize = 8;

fn main() {
    stderrlog::new()
        .verbosity(Level::Info)
        .init()
        .unwrap();
    if let Err(e) = run() {
        error!("{}", e);
    }
}

#[derive(Debug)]
struct StoredData {
    address: SocketAddr,
    stream: TcpStream,
    event: Event,
}

#[derive(Debug)]
enum Event {
    Message(Message),
    Connected,
    Disconnected,
}

fn run() -> Result<(), Box<dyn Error>> {
    let host_port = env::args().nth(1).unwrap_or(DEFAULT_ADDRESS.into());

    let tcp = TcpListener::bind(host_port)?;
    let pool = ThreadPool::new(THREAD_COUNT);
    let (tx, rx) = mpsc::channel::<StoredData>();
    let (responses_out, responses_in) = mpsc::channel::<(Message, TcpStream)>();

    let _responder = thread::spawn(move || {
        while let Ok((msg, mut stream)) = responses_in.recv() {
            msg.write_to_stream(&mut stream).unwrap();
        }
    });

    let _processor = thread::spawn(move || {
        let mut clients: HashMap<SocketAddr, TcpStream> = HashMap::new();
        while let Ok(data) = rx.recv() {
            match data.event {
                Event::Message(message) => {
                    for (address, stream) in &clients {
                        if address != &data.address {
                            info!("sending a message");
                            responses_out
                                .send((message.clone(), stream.try_clone().unwrap()))
                                .unwrap();
                        }
                    }
                }
                Event::Disconnected => {
                    info!("a client disconected");
                    clients.remove(&data.address);
                }
                Event::Connected => {
                    info!("a client conected");
                    clients.insert(data.address, data.stream);
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

            let data = StoredData {
                event: Event::Connected,
                address,
                stream: stream.try_clone().unwrap(),
            };

            tx.send(data).unwrap();
            loop {
                let message = Message::read_from_stream(&mut stream);
                match message {
                    Ok(message) => {
                        let data = StoredData {
                            event: Event::Message(message),
                            address,
                            stream: stream.try_clone().unwrap(),
                        };
                        tx.send(data).unwrap();
                    }
                    Err(_) => {
                        let _ = stream.shutdown(Shutdown::Both);
                        let data = StoredData {
                            event: Event::Disconnected,
                            address,
                            stream: stream.try_clone().unwrap(),
                        };
                        tx.send(data).unwrap();
                        break;
                    }
                }
            }
        });
    }

    Ok(())
}
