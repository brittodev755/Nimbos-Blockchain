use crate::consenso::tipos::*;
use anyhow::Result;
use rand::Rng;

pub struct GeradorCommitment;

impl GeradorCommitment {
    pub fn gerar_nonce() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.gen()).collect()
    }
    
    pub fn criar_commitment(chave_publica: &[u8], nonce: &[u8], no_id: String) -> Commitment {
        let hash = calcular_commitment(chave_publica, nonce);
        
        Commitment {
            hash,
            timestamp: chrono::Utc::now(),
            no_id,
        }
    }
}