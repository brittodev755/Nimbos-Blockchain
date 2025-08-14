mod gerador_fila;
mod seed_global;

use crate::consenso::tipos::*;
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::info;

pub use gerador_fila::*;
pub use seed_global::*;

pub struct CamadaOrdenacao {
    gerador_fila: GeradorFila,
    seed_manager: SeedGlobalManager,
    fila_atual: RwLock<Option<FilaOrdenada>>,
}

impl CamadaOrdenacao {
    pub fn new() -> Self {
        Self {
            gerador_fila: GeradorFila::new(),
            seed_manager: SeedGlobalManager::new(),
            fila_atual: RwLock::new(None),
        }
    }
    
    pub async fn gerar_fila(&self) -> Result<FilaOrdenada> {
        info!("Gerando fila ordenada determinística");
        
        let seed_global = self.seed_manager.obter_seed_atual().await;
        
        // Em implementação real, obteria nós aprovados da camada anterior
        let nos_aprovados = vec![];
        
        let fila = self.gerador_fila.gerar_fila_ordenada(nos_aprovados, seed_global).await?;
        
        *self.fila_atual.write().await = Some(fila.clone());
        
        Ok(fila)
    }
    
    pub async fn obter_fila_atual(&self) -> Option<FilaOrdenada> {
        self.fila_atual.read().await.clone()
    }
}