use std::io::Read;
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, RwLock};
use std::thread;

use cluster::client::ClusterClient;
use cluster::manager::Manager;

pub fn listen(addr: SocketAddr, connection_manager: Arc<RwLock<Manager>>) {
    info!("Initializing Cluster server...");
    let listener = TcpListener::bind(addr).unwrap();
    for stream in listener.incoming() {
        let mut cmgr = connection_manager.clone();
        info!("New Node connected!");
        let stream = stream.unwrap();
        thread::spawn(move || {
            let mut buf = [0; 1024];
            let mut client = ClusterClient::new(stream);
            let bytes_read = client.reader.read(&mut buf).unwrap();
            let alias = String::from_utf8_lossy(&buf[0..bytes_read]);
            let mut cmgr_lock = cmgr.write().unwrap();
            cmgr_lock.add_client(alias.to_string(), client);
        });
    }
}
