use std::io::Read;
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
            let mut client = ClusterClient::new(stream);
            let result: bincode::Result<IridiumMessage> =
                bincode::deserialize_from(&mut client.reader);
            match result {
                Ok(message) => {
                    match message {
                        IridiumMessage::Hello { alias } => {
                            debug!("Found a hello message with alias: {:?}", alias);
                            let mut cmgr_lock = cmgr.write().unwrap();
                            let mut members: Vec<(
                                String,
                                String,
                                String,
                            )> = Vec::new();

                            // Now we need to send back a list of cluster members in the form of a Vector of tuples, containing their alias
                            for (key, value) in &cmgr_lock.clients {
                                if let Ok(client) = value.read() {
                                    let tuple = (
                                        key.to_string(),
                                        client.ip_as_string().unwrap(),
                                        client.port_as_string().unwrap(),
                                    );
                                    members.push(tuple);
                                }
                            }
                            let hello_ack = IridiumMessage::HelloAck {
                                nodes: members,
                                alias: (
                                    tmp_alias.clone(),
                                    addr.ip().to_string(),
                                    addr.port().to_string(),
                                ),
                            };

                            client.write_bytes(&bincode::serialize(&hello_ack).unwrap());
                            cmgr_lock.add_client(alias.to_string(), client);
                        }
                        _ => {
                            error!("Non-hello message received from node trying to join");
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
