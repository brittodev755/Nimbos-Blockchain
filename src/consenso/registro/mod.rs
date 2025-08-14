mod commitment;
mod validador;

use crate::consenso::tipos::*;
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub use commitment::*;
pub use validador::*;

pub struct CamadaRegistro {
    commitments: RwLock<HashMap<String, Commitment>>,
    validador: ValidadorCommitment,
}

impl CamadaRegistro {
    pub fn new() -> Self {
        Self {
            commitments: RwLock::new(HashMap::new()),
            validador: ValidadorCommitment::new(),
        }
    }
    
    pub async fn processar_commitments(&self) -> Result<()> {
        info!("Processando commitments da fase de registro");
        
        // Simular recebimento de commitments
        // Em implementação real, isso viria da camada de comunicação
        
        Ok(())
    }
    
    pub async fn adicionar_commitment(&self, commitment: Commitment) -> Result<bool> {
        if self.validador.validar(&commitment)? {
            let mut commitments = self.commitments.write().await;
            commitments.insert(commitment.no_id.clone(), commitment);
            info!("Commitment adicionado com sucesso");
            Ok(true)
        } else {
            warn!("Commitment inválido rejeitado");
            Ok(false)
        }
    }
    
    pub async fn obter_commitments(&self) -> HashMap<String, Commitment> {
        self.commitments.read().await.clone()
    }
    
    pub async fn limpar_commitments(&self) {
        self.commitments.write().await.clear();
    }
}