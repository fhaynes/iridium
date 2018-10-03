use std::net::{TcpListener, SocketAddr};
use std::thread;
use cluster::client::ClusterClient;

pub fn listen(addr: SocketAddr) {
    info!("Initializing Cluster server...");
    let listener =
        TcpListener::bind(addr).unwrap();
    for stream in listener.incoming() {
        info!("New Node connected!");
        let stream = stream.unwrap();
        thread::spawn(|| {
            let mut client = ClusterClient::new(stream);
            client.run();
        });
    }
}
