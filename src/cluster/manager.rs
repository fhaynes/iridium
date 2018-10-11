use std::collections::HashMap;

use cluster::client::ClusterClient;
use cluster::NodeAlias;

#[derive(Default)]
pub struct Manager {
    clients: HashMap<NodeAlias, ClusterClient>,
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, alias: NodeAlias, client: ClusterClient) -> bool {
        if self.clients.contains_key(&alias) {
            error!("Tried to add a client that already existed");
            return false;
        }
        debug!("Adding {}", alias);
        self.clients.insert(alias, client);
        true
    }

    pub fn del_client(&mut self, alias: NodeAlias) {
        self.clients.remove(&alias);
    }

    pub fn get_client_names(&self) -> Vec<String> {
        let mut results = vec![];
        for (alias, _) in &self.clients {
            results.push(alias.to_owned());
        }
        results
    }
}

#[cfg(test)]
mod test {
    use super::Manager;

    fn test_create_manager() {
        let test_manager = Manager::new();
    }
}
