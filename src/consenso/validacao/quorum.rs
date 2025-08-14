use crate::consenso::tipos::*;
use std::collections::HashMap;
use tracing::info;

pub struct GerenciadorQuorum {
    threshold: f64, // Porcentagem mínima para quórum (ex: 0.7 = 70%)
    total_nos: usize,
}

impl GerenciadorQuorum {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            total_nos: 10, // Em implementação real, seria dinâmico
        }
    }
    
    pub async fn verificar_quorum(&self, validacoes: &[ValidacaoConsenso]) -> bool {
        let total_validacoes = validacoes.len();
        let minimo_necessario = (self.total_nos as f64 * self.threshold).ceil() as usize;
        
        if total_validacoes >= minimo_necessario {
            // Verificar consenso nos hashes
            if self.verificar_consenso_hashes(validacoes) {
                info!("Quórum atingido: {}/{} validações com consenso", total_validacoes, self.total_nos);
                return true;
            }
        }
        
        false
    }
    
    fn verificar_consenso_hashes(&self, validacoes: &[ValidacaoConsenso]) -> bool {
        if validacoes.is_empty() {
            return false;
        }
        
        // Contar frequência de cada hash
        let mut contadores_transacao: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut contadores_cadeia: HashMap<Vec<u8>, usize> = HashMap::new();
        
        for validacao in validacoes {
            *contadores_transacao.entry(validacao.hash_transacao.clone()).or_insert(0) += 1;
            *contadores_cadeia.entry(validacao.hash_cadeia.clone()).or_insert(0) += 1;
        }
        
        // Verificar se há consenso majoritário
        let total = validacoes.len();
        let maioria = (total as f64 * 0.51).ceil() as usize;
        
        let consenso_transacao = contadores_transacao.values().any(|&count| count >= maioria);
        let consenso_cadeia = contadores_cadeia.values().any(|&count| count >= maioria);
        
        consenso_transacao && consenso_cadeia
    }
    
    pub fn atualizar_total_nos(&mut self, total: usize) {
        self.total_nos = total;
    }
    
    pub fn obter_threshold(&self) -> f64 {
        self.threshold
    }
    
    pub fn calcular_minimo_necessario(&self) -> usize {
        (self.total_nos as f64 * self.threshold).ceil() as usize
    }
}