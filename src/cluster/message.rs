use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use bincode::*;

use cluster::client::ClusterClient;
use cluster::{NodeAlias, NodeInfo, NodePort};

#[derive(Serialize, Deserialize, Debug)]
/// These are the message types that cluster nodes can exchange between themselves
pub enum IridiumMessage {
    Hello {
        alias: NodeAlias,
        port: NodePort,
    },
    HelloAck {
        alias: NodeInfo,
        nodes: Vec<NodeInfo>,
    },
    Join {
        alias: NodeAlias,
        port: NodePort,
    },
}

impl IridiumMessage {
    pub fn join(alias: &str, port: &str) -> Result<Vec<u8>> {
        trace!("Generating join message!");
        let new_message = IridiumMessage::Join {
            alias: alias.into(),
            port: port.into(),
        };
        serialize(&new_message)
    }

    /// Creates and serializes a Hello message
    pub fn hello(alias: &str, port: &str) -> Result<Vec<u8>> {
        trace!("Generating hello message");
        let new_message = IridiumMessage::Hello {
            alias: alias.into(),
            port: port.into(),
        };
        serialize(&new_message)
    }

    /// Creates and serializes a HelloAck message, whch sends back a list of all cluster nodes to the sender
    pub fn hello_ack(
        myself: NodeInfo,
        clients: &HashMap<NodeAlias, Arc<RwLock<ClusterClient>>>,
    ) -> Result<Vec<u8>> {
        trace!("Generating helloack message");
        let mut results: Vec<NodeInfo> = Vec::new();
        for (key, value) in clients.iter() {
            if let Ok(client_data) = value.read() {
                results.push((
                    key.to_string(),
                    client_data.ip_as_string().unwrap(),
                    client_data.port_as_string().unwrap(),
                ));
            }
        }
        let new_message = IridiumMessage::HelloAck {
            alias: myself,
            nodes: results,
        };
        serialize(&new_message)
    }

    pub fn process_message(message: &[u8]) -> Result<IridiumMessage> {
        trace!("Deserializing message");
        deserialize(message)
    }
}

#[cfg(test)]
mod test {}
