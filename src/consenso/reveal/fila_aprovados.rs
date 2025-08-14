use tokio::sync::RwLock;
use std::collections::HashSet;

pub struct FilaAprovados {
    nos_aprovados: RwLock<HashSet<String>>,
}

impl FilaAprovados {
    pub fn new() -> Self {
        Self {
            nos_aprovados: RwLock::new(HashSet::new()),
        }
    }
    
    pub async fn adicionar_no_aprovado(&self, no_id: String) {
        self.nos_aprovados.write().await.insert(no_id);
    }
    
    pub async fn obter_nos_aprovados(&self) -> Vec<String> {
        self.nos_aprovados.read().await.iter().cloned().collect()
    }
    
    pub async fn limpar(&self) {
        self.nos_aprovados.write().await.clear();
    }
}