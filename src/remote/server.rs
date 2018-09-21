use remote::client::Client;
use std::io::BufReader;
use std::net::TcpListener;
use std::thread;

pub struct Server {
    bind_hostname: String,
    bind_port: String,
}

impl Server {
    pub fn new(bind_hostname: String, bind_port: String) -> Server {
        Server {
            bind_hostname,
            bind_port,
        }
    }

    pub fn listen(&mut self) {
        println!("Initializing TCP server...");
        let listener =
            TcpListener::bind(self.bind_hostname.clone() + ":" + &self.bind_port).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            thread::spawn(|| {
                let mut client = Client::new(stream);
                client.run();
            });
        }
    }
}
