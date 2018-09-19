use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpStream;

pub struct Client {
    stream: BufReader<TcpStream>,
}

impl Client {
    pub fn new(stream: BufReader<TcpStream>) -> Client {
        Client { stream }
    }

    pub fn run(&mut self) {
        let mut buf = String::new();
        loop {
            match self.stream.read_line(&mut buf) {
                Ok(_) => {
                    println!("{:#?}", buf);
                }
                Err(e) => {
                    println!("Error receiving: {:#?}", e);
                }
            };
        }
    }
}
