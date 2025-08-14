use tokio::sync::RwLock;
use std::collections::VecDeque;
use tracing::info;

pub struct GerenciadorRotacao {
    fila_nos: RwLock<VecDeque<String>>,
    historico_processamento: RwLock<Vec<String>>,
}

impl GerenciadorRotacao {
    pub fn new() -> Self {
        Self {
            fila_nos: RwLock::new(VecDeque::new()),
            historico_processamento: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn inicializar_fila(&self, nos: Vec<String>) {
        let mut fila = self.fila_nos.write().await;
        fila.clear();
        for no in nos {
            fila.push_back(no);
        }
        info!("Fila de rotação inicializada com {} nós", fila.len());
    }
    
    pub async fn rotacionar_fila(&self) {
        let mut fila = self.fila_nos.write().await;
        if let Some(no_processador) = fila.pop_front() {
            // Adicionar ao histórico
            self.historico_processamento.write().await.push(no_processador.clone());
            
            // Mover para o final da fila
            fila.push_back(no_processador.clone());
            
            info!("Nó {} rotacionado para o final da fila", no_processador);
        }
    }
    
    pub async fn obter_proximo_processador(&self) -> Option<String> {
        self.fila_nos.read().await.front().cloned()
    }
    
    pub async fn obter_historico(&self) -> Vec<String> {
        self.historico_processamento.read().await.clone()
    }
    
    pub async fn obter_fila_atual(&self) -> Vec<String> {
        self.fila_nos.read().await.iter().cloned().collect()
    }
}