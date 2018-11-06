use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, RwLock};
use std::thread;

use cluster::client::ClusterClient;
use cluster::manager::Manager;
use cluster::message::IridiumMessage;

pub fn listen(my_alias: String, addr: SocketAddr, connection_manager: Arc<RwLock<Manager>>) {
    info!("Initializing Cluster server...");
    let listener = TcpListener::bind(addr).unwrap();
    for stream in listener.incoming() {
        let tmp_alias = my_alias.clone();
        let mut cmgr = connection_manager.clone();
        info!("New Node connected!");
        let stream = stream.unwrap();
        thread::spawn(move || {
            debug!("Handling an incoming connection on a thread");
            let mut client = ClusterClient::new(stream, cmgr.clone());
            let result: bincode::Result<IridiumMessage> =
                bincode::deserialize_from(&mut client.reader);
            match result {
                Ok(message) => {
                    match message {
                        IridiumMessage::Hello { alias } => {
                            debug!("Received hello");
                            let mut members: Vec<(
                                String,
                                String,
                                String,
                            )> = Vec::new();

                            // Now we need to send back a list of cluster members in the form of a Vector of tuples, containing their alias
                            debug!("Generating member list");
                            {
                                let mut cmgr_lock = cmgr.read().unwrap();
                                debug!("Grabbed read lock on manager");
                                for (key, value) in &cmgr_lock.clients {
                                    debug!("Processing kv: {:#?} {:#?}", key, value);
                                    let tuple = (
                                        key.0.to_string(),
                                        key.1.to_string(),
                                        key.2.to_string(),
                                    );
                                    members.push(tuple);
                                }
                            }
                            debug!("Generating hello_ack");
                            let hello_ack = IridiumMessage::HelloAck {
                                nodes: members,
                                alias: (
                                    tmp_alias.clone(),
                                    addr.ip().to_string(),
                                    addr.port().to_string(),
                                ),
                            };

                            client.write_bytes(&bincode::serialize(&hello_ack).unwrap());
                            debug!("Adding {} to clients. Client info: {:?}", alias, client);
                            {
                                let mut cmgr_lock = cmgr.write().unwrap();
                                let client_tuple = (alias.to_string(), client.ip_as_string().unwrap(), client.port_as_string().unwrap());
                                cmgr_lock.add_client(client_tuple, client);
                            }
                            debug!("Client added to managr");
                        }
                        // Handles another node sending a Join message. In this case, we don't want to send back a list of all known nodes.
                        IridiumMessage::Join { alias } => {
                            debug!("Received join message from alias: {:?}", alias);
                            if let Ok(mut connection_manager) = cmgr.write() {
                                debug!("Added new client {} to conneciton manager", alias);
                                let client_tuple = (alias.to_string(), client.ip_as_string().unwrap(), client.port_as_string().unwrap());
                                connection_manager.add_client(client_tuple, client);
                                return;
                            } else {
                                error!("Unable to add {} to connection manager", alias);
                            }
                        }

                        _ => {
                            error!("Unknown message received from node");
                        }
                    }
                }
                Err(e) => {
                    error!("Error deserializing Iridium message: {:?}", e);
                }
            }
        });
    }
}
