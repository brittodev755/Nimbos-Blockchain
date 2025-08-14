use crate::consenso::tipos::*;
use anyhow::Result;

pub struct VerificadorReveal;

impl VerificadorReveal {
    pub fn new() -> Self {
        Self
    }
    
    pub fn verificar(&self, reveal: &Reveal, commitment: &Commitment) -> Result<bool> {
        // Recalcular o commitment usando os dados revelados
        let commitment_calculado = calcular_commitment(&reveal.chave_publica, &reveal.nonce);
        
        // Verificar se o commitment calculado corresponde ao original
        Ok(commitment_calculado == commitment.hash)
    }
}