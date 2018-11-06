pub mod client;
pub mod manager;
pub mod message;
pub mod server;
pub mod alias;

type NodeAlias = String;
type NodeIP = String;
type NodePort = String;
type NodeInfo = (NodeAlias, NodeIP, NodePort);
