use std::io::{BufRead, Write, Read};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc};
use std::thread;
use repl;

pub struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    raw_stream: TcpStream,
    repl: repl::REPL,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        // TODO: Handle this better
        let reader = stream.try_clone().unwrap();
        let writer = stream.try_clone().unwrap();
        let mut repl = repl::REPL::new();

        Client {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
            raw_stream: stream,
            repl: repl
        }
    }

    fn w(&mut self, msg: &str) -> bool {
        match self.writer.write_all(msg.as_bytes()) {
            Ok(_) => {
                match self.writer.flush() {
                    Ok(_) => {
                        true
                    }
                    Err(e) => {
                        println!("Error flushing to client: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                println!("Error writing to client: {}", e);
                false
            }
        }
    }

    fn write_prompt(&mut self) {
        self.w(repl::PROMPT);
    }

    fn recv_loop(&mut self) {
        let rx = self.repl.rx_pipe.take();
        // TODO: Make this safer on unwrap
        let mut writer = self.raw_stream.try_clone().unwrap();
        let t = thread::spawn(move || {
            let chan = rx.unwrap();
            loop {
                match chan.recv() {
                    Ok(msg) => {
                        writer.write_all(msg.as_bytes());
                        writer.flush();
                    },
                    Err(e) => {}
                }
            }
        });
    }

    pub fn run(&mut self) {
        self.recv_loop();
        let mut buf = String::new();
        let banner = repl::REMOTE_BANNER.to_owned() + "\n" + repl::PROMPT;
        self.w(&banner);
        loop {
            match self.reader.read_line(&mut buf) {
                Ok(_) => {
                    buf.trim_right();
                    self.repl.run_single(&buf);
                }
                Err(e) => {
                    println!("Error receiving: {:#?}", e);
                }
            }
        }
    }
}
