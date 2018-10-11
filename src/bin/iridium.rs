use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::thread;

extern crate byteorder;
extern crate chrono;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate num_cpus;
extern crate uuid;

extern crate iridium;

use clap::App;
use iridium::assembler::Assembler;
use iridium::repl::REPL;
use iridium::vm::VM;

fn main() {
    env_logger::init();
    let mut _repl_receiver: Receiver<String>;
    info!("Starting logging!");
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let data_root_dir = matches
        .value_of("DATA_ROOT_DIR")
        .unwrap_or("/var/lib/iridium/");

    if make_directory(data_root_dir).is_err() {
        println!("There was an error creating the default root data directory");
        std::process::exit(1);
    };

    if matches.is_present("ENABLE_REMOTE_ACCESS") {
        let port = matches.value_of("LISTEN_PORT").unwrap_or("2244");
        let host = matches.value_of("LISTEN_HOST").unwrap_or("127.0.0.1");
        start_remote_server(host.to_string(), port.to_string());
    }

    let alias = matches.value_of("NODE_ALIAS").unwrap_or("");
    let server_addr = matches
        .value_of("SERVER_LISTEN_HOST")
        .unwrap_or("127.0.0.1");
    let server_port = matches.value_of("SERVER_LISTEN_PORT").unwrap_or("2254");

    let num_threads = match matches.value_of("THREADS") {
        Some(number) => match number.parse::<usize>() {
            Ok(v) => v,
            Err(_e) => {
                println!(
                    "Invalid argument for number of threads: {}. Using default.",
                    number
                );
                num_cpus::get()
            }
        },
        None => num_cpus::get(),
    };

    let target_file = matches.value_of("INPUT_FILE");
    match target_file {
        Some(filename) => {
            let program = read_file(filename);
            let mut asm = Assembler::new();
            let mut vm = VM::new()
                .with_alias(alias.to_string())
                .with_cluster_bind(server_addr.into(), server_port.into());
            vm.logical_cores = num_threads;
            let program = asm.assemble(&program);
            match program {
                Ok(p) => {
                    vm.add_bytes(p);
                    let _events = vm.run();
                    println!("{:#?}", vm.registers);
                    std::process::exit(0);
                }
                Err(_e) => {}
            }
        }
        None => {
            let mut vm = VM::new()
                .with_alias(alias.to_string())
                .with_cluster_bind(server_addr.into(), server_port.into());
            let mut repl = REPL::new(vm);
            let mut rx = repl.rx_pipe.take();
            thread::spawn(move || {
                let chan = rx.unwrap();
                loop {
                    match chan.recv() {
                        Ok(msg) => {
                            println!("{}", msg);
                        }
                        Err(_e) => {}
                    }
                }
            });
            repl.run();
        }
    }
}

fn read_file(tmp: &str) -> String {
    let filename = Path::new(tmp);
    match File::open(Path::new(&filename)) {
        Ok(mut fh) => {
            let mut contents = String::new();
            match fh.read_to_string(&mut contents) {
                Ok(_) => contents,
                Err(e) => {
                    println!("There was an error reading file: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("File not found: {:?}", e);
            std::process::exit(1)
        }
    }
}

fn start_remote_server(listen_host: String, listen_port: String) {
    let _t = std::thread::spawn(move || {
        let mut sh = iridium::remote::server::Server::new(listen_host, listen_port);
        sh.listen();
    });
}

fn make_directory(dir: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    Ok(())
}
