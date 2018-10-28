use std::io::{BufRead, Write};
use std::io::Read;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use bincode;
use cluster::message::IridiumMessage;

pub struct ClusterClient {
    alias: Option<String>,
    pub reader: BufReader<TcpStream>,
    pub writer: BufWriter<TcpStream>,
    rx: Option<Arc<Mutex<Receiver<String>>>>,
    _tx: Option<Arc<Mutex<Sender<String>>>>,
    pub raw_stream: TcpStream,
}

impl ClusterClient {
    /// Creates and returns a new ClusterClient that wraps TcpStreams for communicating with it
    pub fn new(stream: TcpStream) -> ClusterClient {
        // TODO: Handle this better
        let reader = stream.try_clone().unwrap();
        let writer = stream.try_clone().unwrap();
        let (tx, rx) = channel();
        ClusterClient {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
            raw_stream: stream,
            _tx: Some(Arc::new(Mutex::new(tx))),
            rx: Some(Arc::new(Mutex::new(rx))),
            alias: None,
        }
    }

    /// Sets the alias of the ClusterClient and returns it
    pub fn with_alias(mut self, alias: String) -> Self {
        self.alias = Some(alias);
        self
    }

    pub fn send_hello(&mut self) {
        match self.alias {
            Some(ref alias) => {
                if let Ok(mut hello) = IridiumMessage::hello(alias) {
                    if self.raw_stream.write_all(&hello).is_ok() {
                        trace!("Hello sent: {:?}", hello);
                    } else {
                        error!("Error sending hello");
                    }
                }
            },
            None => {
                error!("Node has no ID to send hello");
            }
        }
    }

    pub fn alias_as_string(&self) -> Option<String> {
        if let Some(alias) = &self.alias {
            Some(alias.clone())
        } else {
            None
        }
    }

    pub fn ip_as_string(&self) -> Option<String> {
        if let Ok(addr) = self.raw_stream.local_addr() {
            Some(addr.ip().to_string())
        } else {
            None
        }
    }

    pub fn port_as_string(&self) -> Option<String> {
        if let Ok(addr) = self.raw_stream.local_addr() {
            Some(addr.port().to_string())
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn w(&mut self, msg: &str) -> bool {
        match self.writer.write_all(msg.as_bytes()) {
            Ok(_) => match self.writer.flush() {
                Ok(_) => true,
                Err(e) => {
                    println!("Error flushing to client: {}", e);
                    false
                }
            },
            Err(e) => {
                println!("Error writing to client: {}", e);
                false
            }
        }
    }

    fn recv_loop(&mut self) {
        let chan = self.rx.take().unwrap();
        let mut writer = self.raw_stream.try_clone().unwrap();
        let _t = thread::spawn(move || loop {
            if let Ok(locked_rx) = chan.lock() {
                match locked_rx.recv() {
                    Ok(msg) => {
                        match writer.write_all(msg.as_bytes()) {
                            Ok(_) => {
                                
                            }
                            Err(_e) => {}
                        };
                        match writer.flush() {
                            Ok(_) => {}
                            Err(_e) => {}
                        };
                    }
                    Err(_e) => {}
                }
            }
        });
    }

    pub fn run(&mut self) {
        self.recv_loop();
        // let mut buf = Vec::new();
        
        loop {
                match bincode::deserialize_from(self.reader) {
                    Ok(message) => {},
                    Err(e) => {}
                }

        }
    }
}
