pub enum IridiumMessage {
    Hello {
        alias: String,
    },
    HelloAck {
        alias: String,
        nodes: Vec<(String, String, String)>,
    },
}

#[cfg(test)]
mod test {}
