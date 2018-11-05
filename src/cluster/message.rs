use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use bincode::*;

use cluster::NodeAlias;
use cluster::client::ClusterClient;

#[derive(Serialize, Deserialize, Debug)]
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
    pub fn hello(alias: &str) -> Result<Vec<u8>> {
        trace!("Generating hello message");
        let new_message = IridiumMessage::Hello{
            alias: alias.into()
        };
        serialize(&new_message)
    }

    pub fn hello_ack(clients: &HashMap<NodeAlias, Arc<RwLock<ClusterClient>>>) -> Result<Vec<u8>> {
        let results: Vec<(String, String, String)> = Vec::new();
        for (key, value) in clients.iter() {
            if let Ok(client_data) = value.read() {
                let client_tuple = (client_data.alias_as_string(), );
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
mod test {

}