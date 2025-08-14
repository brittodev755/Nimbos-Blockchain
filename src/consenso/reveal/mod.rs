mod verificador;
mod fila_aprovados;

use crate::consenso::tipos::*;
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub use verificador::*;
pub use fila_aprovados::*;

pub struct CamadaReveal {
    reveals: RwLock<HashMap<String, Reveal>>,
    verificador: VerificadorReveal,
    fila_aprovados: FilaAprovados,
}

impl CamadaReveal {
    pub fn new() -> Self {
        Self {
            reveals: RwLock::new(HashMap::new()),
            verificador: VerificadorReveal::new(),
            fila_aprovados: FilaAprovados::new(),
        }
    }
    
    pub async fn processar_reveals(&self) -> Result<()> {
        info!("Processando reveals da fase de verificação");
        
        // Implementar lógica de processamento
        
        Ok(())
    }
    
    pub async fn adicionar_reveal(&self, reveal: Reveal, commitments: &HashMap<String, Commitment>) -> Result<bool> {
        if let Some(commitment) = commitments.get(&reveal.no_id) {
            if self.verificador.verificar(&reveal, commitment)? {
                let mut reveals = self.reveals.write().await;
                reveals.insert(reveal.no_id.clone(), reveal.clone());
                
                // Adicionar à fila de aprovados
                self.fila_aprovados.adicionar_no_aprovado(reveal.no_id.clone()).await;
                
                info!("Reveal verificado e nó aprovado");
                Ok(true)
            } else {
                warn!("Reveal não corresponde ao commitment");
                Ok(false)
            }
        } else {
            warn!("Commitment não encontrado para o reveal");
            Ok(false)
        }
    }
    
    pub async fn obter_nos_aprovados(&self) -> Vec<String> {
        self.fila_aprovados.obter_nos_aprovados().await
    }
}