mod arvore;
mod prova;

use crate::consenso::tipos::*;
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::info;

pub use arvore::*;
pub use prova::*;

pub struct CamadaMerkle {
    arvore: RwLock<Option<ArvoreMerkle>>,
    gerador_prova: GeradorProva,
}

impl CamadaMerkle {
    pub fn new() -> Self {
        Self {
            arvore: RwLock::new(None),
            gerador_prova: GeradorProva::new(),
        }
    }
    
    pub async fn gerar_arvore(&self, fila: &FilaOrdenada) -> Result<Vec<u8>> {
        info!("Gerando Merkle Tree para fila ordenada");
        
        let arvore = ArvoreMerkle::construir(&fila.nos)?;
        let merkle_root = arvore.obter_root();
        
        *self.arvore.write().await = Some(arvore);
        
        Ok(merkle_root)
    }
    
    pub async fn gerar_prova_inclusao(&self, no_id: &str) -> Result<Option<ProvaInclusao>> {
        if let Some(arvore) = self.arvore.read().await.as_ref() {
            self.gerador_prova.gerar_prova(arvore, no_id)
        } else {
            Ok(None)
        }
    }
    
    pub async fn verificar_prova(&self, prova: &ProvaInclusao, no_id: &str) -> Result<bool> {
        if let Some(arvore) = self.arvore.read().await.as_ref() {
            self.gerador_prova.verificar_prova(arvore, prova, no_id)
        } else {
            Ok(false)
        }
    }
}