use std::io::{Write};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use bincode;
use cluster::message::IridiumMessage;
use cluster::{NodeInfo};
use cluster::manager::Manager;

#[derive(Debug)]
pub struct ClusterClient {
    alias: Option<String>,
    pub reader: BufReader<TcpStream>,
    pub writer: BufWriter<TcpStream>,
    pub connection_manager: Arc<RwLock<Manager>>,
    pub bind_port: Option<String>,
    rx: Option<Arc<Mutex<Receiver<String>>>>,
    _tx: Option<Arc<Mutex<Sender<String>>>>,
    pub raw_stream: TcpStream,
}

impl ClusterClient {
    /// Creates and returns a new ClusterClient that wraps TcpStreams for communicating with it
    pub fn new(stream: TcpStream, manager: Arc<RwLock<Manager>>, bind_port: String) -> ClusterClient {
        // TODO: Handle this better
        let reader = stream.try_clone().unwrap();
        let writer = stream.try_clone().unwrap();
        let (tx, rx) = channel();
        ClusterClient {
            connection_manager: manager,
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
            raw_stream: stream,
            _tx: Some(Arc::new(Mutex::new(tx))),
            rx: Some(Arc::new(Mutex::new(rx))),
            alias: None,
            bind_port: Some(bind_port),
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
                if let Ok(mut hello) = IridiumMessage::hello(alias, &self.bind_port.clone().unwrap()) {
                    if self.raw_stream.write_all(&hello).is_ok() {
                        trace!("Hello sent: {:#?}", hello);
                    } else {
                        error!("Error sending hello");
                    }
                }
            }
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

    pub fn remote_ip_as_string(&self) -> Option<String> {
        if let Ok(addr) = self.raw_stream.peer_addr() {
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

    pub fn remote_port_as_string(&self) -> Option<String> {
        if let Ok(addr) = self.raw_stream.peer_addr() {
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
                    error!("Error flushing to client: {}", e);
                    false
                }
            },
            Err(e) => {
                error!("Error writing to client: {}", e);
                false
            }
        }
    }

    pub fn write_bytes(&mut self, msg: &[u8]) {
        match self.writer.write_all(msg) {
            Ok(_) => match self.writer.flush() {
                Ok(_) => {}
                Err(e) => {
                    error!("Error flushing to client: {}", e);
                }
            },
            Err(e) => {
                error!("Error writing to client: {}", e);
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
                            Ok(_) => {}
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
        loop {
            let result: bincode::Result<IridiumMessage> =
                bincode::deserialize_from(&mut self.reader);
            if let Err(e) = result {
                match *e {
                    bincode::ErrorKind::Io(inner_error) => {
                        error!("There was an IO error with node: {:?}", inner_error);
                        return;
                    },
                    _ => {
                        error!("There was an unknown error communicating with the client: {:?}", e);
                        return;
                    }
                }
            }
            match result {
                Ok(ref message) => {
                    match message {
                        &IridiumMessage::HelloAck {
                            ref nodes,
                            ref alias,
                        } => {
                            let join_message: std::result::Result<std::vec::Vec<u8>, std::boxed::Box<bincode::ErrorKind>>;
                            if let Some(ref alias) = self.alias_as_string() {
                                join_message = IridiumMessage::join(&alias, &self.port_as_string().unwrap());
                            } else {
                                error!("Unable to get my own alias to send a join message to other cluster members");
                                continue;
                            }
                            let join_message = join_message.unwrap();
                            for node in nodes {
                                debug!("Sending join to {:#?}", node);
                                let remote_alias = &node.0;
                                let remote_ip = &node.1;
                                let remote_port = &node.2;
                                let addr = remote_ip.to_owned() + ":" + remote_port;
                                if let Ok(stream) = TcpStream::connect(addr) {
                                    let mut cluster_client = ClusterClient::new(stream, self.connection_manager.clone(), self.bind_port.clone().unwrap());
                                    cluster_client.write_bytes(&join_message);
                                    if let Ok(mut cm) = self.connection_manager.write() {
                                        let client_tuple = (remote_alias.to_string(), cluster_client.ip_as_string().unwrap(), cluster_client.port_as_string().unwrap());
                                        cm.add_client(client_tuple, cluster_client);
                                    }
                                } else {
                                    error!("Unable to establish connection to: {:?}", node);
                                }
                            }
                        }
                        _ => {
                            error!("Unknown message received");
                        }
                    }
                    debug!("Received message: {:?}", message);
                }
                Err(e) => {
                    error!("Error deserializing Iridium message: {:?}", e);
                }
            }
        }
    }
}