use std::io::{BufRead, Write};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::thread;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

use cluster::message::IridiumMessage;
use cluster::manager::ClientManager;

pub struct ClusterClient {
    alias: Option<String>,
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    rx: Option<Receiver<String>>,
    tx: Option<Sender<String>>,
    raw_stream: TcpStream,
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
            tx: Some(tx),
            rx: Some(rx),
            alias: None,
        }
    }

    pub fn send_hello(&self) {
        if let Some(ref a) = self.alias {
            let msg = IridiumMessage::Hello{alias: a.to_string()};
        }

    }

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
        let _t = thread::spawn(move || {
            loop {
                match chan.recv() {
                    Ok(msg) => {
                        match writer.write_all(msg.as_bytes()) {
                            Ok(_) => {}
                            Err(_e) => {}
                        };
                        match writer.flush() {
                            Ok(_) => {}
                            Err(_e) => {}
                        }
                    }
                    Err(_e) => {}
                }
            }
        });
    }

    pub fn run(&mut self) {
        self.recv_loop();
        let mut buf = String::new();
        loop {
            match self.reader.read_line(&mut buf) {
                Ok(_) => {
                    buf.trim_right();
                }
                Err(e) => {
                    println!("Error receiving: {:#?}", e);
                }
            }
        }
    }
}
