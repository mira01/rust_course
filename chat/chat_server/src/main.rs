/// Application for keeping connection from clients and brodcasting received messages.

use std::collections::HashMap;
use std::env;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use threadpool::ThreadPool;
use log::{Level, info, error, warn};
use stderrlog;
use anyhow::{Result, Context, Error, bail};

use chat_lib::message::{Message, ChatLibError};

const DEFAULT_ADDRESS: &str = "127.0.0.1:11111";
const THREAD_COUNT: usize = 8;

fn main() -> Result<()> {
    stderrlog::new()
        .verbosity(Level::Info)
        .init()?;
    run()
        .map_err(|e| {
            error!("{}", e);
            e.context("Server crashed")
        })
}

/// Structure that encapsulates client identification, stream and an event that happened
/// to the client. This structure is sent between threads. 
#[derive(Debug)]
struct StoredData {
    address: SocketAddr,
    stream: TcpStream,
    event: Event,
}

/// Type of event that happened to a client.
#[derive(Debug)]
enum Event {
    Message(Message),
    ClientError(Error),
    Connected,
    Disconnected,
}

/// All logic happens here:
fn run() -> Result<()> {
    let host_port = env::args()
        .nth(1)
        .unwrap_or(DEFAULT_ADDRESS.into());

    let tcp = TcpListener::bind(&host_port)
        .with_context(|| format!("Cannot listen on {}", &host_port))?;
    let pool = ThreadPool::new(THREAD_COUNT);
    let (tx, rx) = mpsc::channel::<StoredData>();
    let (responses_out, responses_in) = mpsc::channel::<(Message, TcpStream)>();

    // Thread that writes messages to clients' streams
    let _responder: JoinHandle<Result<()>> = thread::spawn(move || {
        while let Ok((msg, mut stream)) = responses_in.recv() {
            msg.write_to_stream(&mut stream)?;
        }
        Ok(())
    });

    // Thread that keeps clients' connections and acts on incoming messages
    let _processor: JoinHandle<Result<()>> = thread::spawn(move || {
        let mut clients: HashMap<SocketAddr, TcpStream> = HashMap::new();
        while let Ok(data) = rx.recv() {
            match data.event {
                Event::Message(message) => {
                    for (address, stream) in &clients {
                        if address != &data.address {  // do not send to the author
                            info!("sending a message");
                            responses_out.send((message.clone(), stream.try_clone()?))?;
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
                Event::ClientError(e) => {
                    warn!("client error {}", e);
                    responses_out
                        .send((Message::Text(format!("You caused an error on server {}", e)), data.stream.try_clone()?))?;
                }
            }
        }
        Ok(())
    });

    /// Function for acting upon incoming message
    fn handle_stream(stream: std::io::Result<TcpStream>, tx: mpsc::Sender<StoredData>) -> Result<()> {
        let mut stream = stream.context("Cannot read from data stream")?;
        let address = stream.peer_addr().context("Cannot get peer addres")?;
        let tx = tx.clone();

        let data = StoredData {
            event: Event::Connected,
            address,
            stream: stream.try_clone()?,
        };

        tx.send(data)?;
        loop {
            let message = Message::read_from_stream(&mut stream);
            let event_type = match message {
                Ok(message) => Event::Message(message),
                Err(e @ ChatLibError::ComposeError) => Event::ClientError(e.into()),
                Err(_) => Event::Disconnected,
            };
            let mut should_exit = false;
            if let Event::Disconnected = event_type {
                should_exit = true;
            }
            let data = StoredData {
                event: event_type,
                address,
                stream: stream.try_clone()?,
            };
            tx.send(data)?;
            if should_exit {
                let _ = stream.shutdown(Shutdown::Both);
                break Ok(());
            }
        }
    }


    // Read data from tcp and handle them in thread pool
    for stream in tcp.incoming() {
        let tx2 = tx.clone();
        pool.execute(move || {
            match handle_stream(stream, tx2) {
                Ok(_) => info!("done processing stream"),
                Err(e) => error!("failed processing tcp stream {}", e), 
            }
        } );
    }
    Ok(())
}
