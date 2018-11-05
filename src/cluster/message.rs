use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use bincode::*;

use cluster::client::ClusterClient;
use cluster::NodeAlias;

#[derive(Serialize, Deserialize, Debug)]
/// These are the message types that cluster nodes can exchange between themselves
pub enum IridiumMessage {
    Hello {
        alias: String,
    },
    HelloAck {
        alias: (String, String, String),
        nodes: Vec<(String, String, String)>,
    },
}

impl IridiumMessage {
    /// Creates and serializes a Hello message
    pub fn hello(alias: &str) -> Result<Vec<u8>> {
        trace!("Generating hello message");
        let new_message = IridiumMessage::Hello {
            alias: alias.into(),
        };
        serialize(&new_message)
    }

    /// Creates and serializes a HelloAck message, whch sends back a list of all cluster nodes to the sender
    pub fn hello_ack(clients: &HashMap<NodeAlias, Arc<RwLock<ClusterClient>>>) -> Result<Vec<u8>> {
        let _results: Vec<(String, String, String)> = Vec::new();
        for (_key, value) in clients.iter() {
            if let Ok(client_data) = value.read() {
                let _client_tuple = (client_data.alias_as_string(),);
            }
        }
        Ok(Vec::new())
    }

    pub fn process_message(message: &[u8]) -> Result<IridiumMessage> {
        trace!("Deserializing message");
        deserialize(message)
    }
}

#[cfg(test)]
mod test {}
