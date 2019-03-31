use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::PathBuf;

pub fn read_node_id(path: &str) -> Result<String, Error> {
    let path = PathBuf::from(path);
    let mut alias = String::new();
    debug!("Reading node_id from file");
    let mut f = File::open(path)?;
    match f.read_to_string(&mut alias) {
        Ok(bytes) => {
            debug!("Read node_id from file. {} bytes.", bytes);
            Ok(alias)
        }
        Err(e) => {
            error!("Error reading node ID from disk: {}", e);
            Err(e)
        }
    }
}

pub fn write_node_id(path: &str, alias: &str) -> Result<(), Error> {
    let path = PathBuf::from(path);
    let mut f = File::create(path)?;
    match f.write_all(alias.as_bytes()) {
        Ok(_) => {
            info!("Node ID {} from CLI written to disk", alias);
            Ok(())
        }
        Err(e) => {
            error!("Error writing node ID to disk: {}", e);
            Err(e)
        }
    }
}
