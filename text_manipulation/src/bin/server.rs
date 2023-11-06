use std::env;
use std::net::{TcpStream, TcpListener, Shutdown, SocketAddr};
use std::error::Error;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use threadpool::ThreadPool;

use chat::message::Message;
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
    event: Event,
}

#[derive(Debug)]
enum Event {
    Message(Message),
    Disconnected,
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

    let _responder = thread::spawn(move || {
        while let Ok((msg, mut stream)) = responses_in.recv() {
           msg.write_to_stream(&mut stream).unwrap();
        }
    });

    let _processor = thread::spawn(move || {
        let mut clients = HashMap::new();
        while let Ok(data) = rx.recv(){
            match data.event {
                Event::Message(message) => {
                    clients.insert(data.address.clone(), data.stream);
                    for (address, stream) in &clients {
                        if address != &data.address {
                            println!("sending a message");
                            responses_out.send((message.clone(), stream.try_clone().unwrap())).unwrap();
                        }
                    }
                },
                Event::Disconnected => {
                    println!("a client disconected");
                    clients.remove(&data.address);
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
            loop {
                let message = Message::read_from_stream(&mut stream);
                match message {
                    Ok(message) => {
                        let data = StoredData{
                            event: Event::Message(message),
                            address,
                            stream: stream.try_clone().unwrap(),
                        };
                        tx.send(data).unwrap();
                    },
                    Err(_) => {
                        let _ = stream.shutdown(Shutdown::Both);
                        let data = StoredData{
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
