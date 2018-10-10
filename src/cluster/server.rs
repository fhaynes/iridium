use std::sync::{Arc, RwLock};
use std::net::{TcpListener, SocketAddr};
use std::thread;
use std::io::Read;

use cluster::client::ClusterClient;
use cluster::manager::Manager;

pub fn listen(addr: SocketAddr, connection_manager: Arc<RwLock<Manager>>) {
    info!("Initializing Cluster server...");
    let listener = TcpListener::bind(addr).unwrap();
    for stream in listener.incoming() {
        info!("New Node connected!");
        let stream = stream.unwrap();
        thread::spawn(move || {
            let mut buf = [0; 1024];
            let mut client = ClusterClient::new(stream);
            let bytes_read = client.reader.read(&mut buf);
            let alias = String::from_utf8_lossy(&buf);
            println!("Alias is: {:?}", alias);
            client.run();
        });
    }
}
