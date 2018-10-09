pub mod server;
pub mod client;
pub mod manager;
pub mod message;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use cluster::client::ClusterClient;

type NodeAlias = String;
type NodeCollection = HashMap<NodeAlias, ClusterClient>;
