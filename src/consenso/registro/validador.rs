use crate::consenso::tipos::*;
use anyhow::Result;
use chrono::{Duration, Utc};

pub struct ValidadorCommitment {
    janela_tempo: Duration,
}

impl ValidadorCommitment {
    pub fn new() -> Self {
        Self {
            janela_tempo: Duration::minutes(5), // 5 minutos de janela
        }
    }
    
    pub fn validar(&self, commitment: &Commitment) -> Result<bool> {
        // Validar timestamp
        let agora = Utc::now();
        let tempo_limite = agora - self.janela_tempo;
        
        if commitment.timestamp < tempo_limite {
            return Ok(false);
        }
        
        // Validar formato do hash
        if commitment.hash.len() != 32 {
            return Ok(false);
        }
        
        // Validar ID do nó
        if commitment.no_id.is_empty() {
            return Ok(false);
        }
        
        Ok(true)
    }
}