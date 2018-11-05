use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use cluster::client::ClusterClient;
use cluster::NodeAlias;

#[derive(Default)]
pub struct Manager {
    pub clients: HashMap<NodeAlias, Arc<RwLock<ClusterClient>>>,
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
        let client = Arc::new(RwLock::new(client));
        self.clients.insert(alias.clone(), client);
        let cloned_client = self.get_client(alias).unwrap();
        thread::spawn(move || {
            cloned_client.write().unwrap().run();
        });
        true
    }

    pub fn get_client(&mut self, alias: NodeAlias) -> Option<Arc<RwLock<ClusterClient>>> {
        Some(self.clients.get_mut(&alias).unwrap().clone())
    }

    pub fn del_client(&mut self, alias: &NodeAlias) {
        self.clients.remove(alias);
    }

    pub fn get_client_names(&self) -> Vec<String> {
        debug!("Getting client names");
        let mut results = vec![];
        for alias in self.clients.keys() {
            results.push(alias.to_owned());
        }
        results
    }
}

#[cfg(test)]
mod test {
    use super::Manager;

    #[allow(dead_code)]
    fn test_create_manager() {
        let _test_manager = Manager::new();
    }
}
