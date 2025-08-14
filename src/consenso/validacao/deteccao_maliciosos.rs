use crate::consenso::tipos::*;
use anyhow::Result;
use std::collections::HashMap;
use tracing::{warn, info};

#[derive(Debug, Clone)]
pub struct ComportamentoSuspeito {
    pub no_id: String,
    pub tipo_suspeita: TipoSuspeita,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub detalhes: String,
}

#[derive(Debug, Clone)]
pub enum TipoSuspeita {
    HashInconsistente,
    AssinaturaInvalida,
    TempoResposta,
    ComportamentoAnomalo,
}

pub struct DetectorMaliciosos {
    historico_suspeitas: std::sync::RwLock<Vec<ComportamentoSuspeito>>,
    threshold_suspeita: usize,
}

impl DetectorMaliciosos {
    pub fn new() -> Self {
        Self {
            historico_suspeitas: std::sync::RwLock::new(Vec::new()),
            threshold_suspeita: 3, // 3 comportamentos suspeitos = malicioso
        }
    }
    
    pub async fn analisar_validacoes(&self, validacoes: &[ValidacaoConsenso]) -> Result<Vec<String>> {
        let mut nos_maliciosos = Vec::new();
        
        // Analisar inconsistências nos hashes
        let inconsistencias = self.detectar_inconsistencias_hash(validacoes);
        
        for (no_id, _) in inconsistencias {
            self.registrar_suspeita(ComportamentoSuspeito {
                no_id: no_id.clone(),
                tipo_suspeita: TipoSuspeita::HashInconsistente,
                timestamp: chrono::Utc::now(),
                detalhes: "Hash inconsistente com maioria".to_string(),
            });
            
            if self.contar_suspeitas(&no_id) >= self.threshold_suspeita {
                nos_maliciosos.push(no_id);
            }
        }
        
        if !nos_maliciosos.is_empty() {
            warn!("Nós maliciosos detectados: {:?}", nos_maliciosos);
        }
        
        Ok(nos_maliciosos)
    }
    
    fn detectar_inconsistencias_hash(&self, validacoes: &[ValidacaoConsenso]) -> HashMap<String, Vec<u8>> {
        let mut contadores: HashMap<Vec<u8>, Vec<String>> = HashMap::new();
        
        // Agrupar validações por hash
        for validacao in validacoes {
            contadores
                .entry(validacao.hash_transacao.clone())
                .or_insert_with(Vec::new)
                .push(validacao.validador_id.clone());
        }
        
        // Encontrar hash majoritário
        let hash_majoritario = contadores
            .iter()
            .max_by_key(|(_, nos)| nos.len())
            .map(|(hash, _)| hash.clone());
        
        let mut inconsistentes = HashMap::new();
        
        if let Some(hash_correto) = hash_majoritario {
            for (hash, nos) in contadores {
                if hash != hash_correto {
                    for no_id in nos {
                        inconsistentes.insert(no_id, hash.clone());
                    }
                }
            }
        }
        
        inconsistentes
    }
    
    fn registrar_suspeita(&self, suspeita: ComportamentoSuspeito) {
        if let Ok(mut historico) = self.historico_suspeitas.write() {
            historico.push(suspeita);
        }
    }
    
    fn contar_suspeitas(&self, no_id: &str) -> usize {
        if let Ok(historico) = self.historico_suspeitas.read() {
            historico.iter().filter(|s| s.no_id == no_id).count()
        } else {
            0
        }
    }
    
    pub fn obter_historico_suspeitas(&self) -> Vec<ComportamentoSuspeito> {
        self.historico_suspeitas.read().unwrap_or_else(|_| std::sync::RwLockReadGuard::try_from(std::sync::RwLock::new(Vec::new()).read().unwrap()).unwrap()).clone()
    }
    
    pub fn limpar_historico(&self) {
        if let Ok(mut historico) = self.historico_suspeitas.write() {
            historico.clear();
        }
    }
}