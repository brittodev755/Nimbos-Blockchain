mod validador;
mod quorum;
mod deteccao_maliciosos;

use crate::consenso::tipos::*;
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub use validador::*;
pub use quorum::*;
pub use deteccao_maliciosos::*;

pub struct CamadaValidacao {
    validador: ValidadorDistribuido,
    quorum: GerenciadorQuorum,
    detector_maliciosos: DetectorMaliciosos,
    validacoes: RwLock<HashMap<String, Vec<ValidacaoConsenso>>>,
}

impl CamadaValidacao {
    pub fn new() -> Self {
        Self {
            validador: ValidadorDistribuido::new(),
            quorum: GerenciadorQuorum::new(0.7), // 70% de quórum
            detector_maliciosos: DetectorMaliciosos::new(),
            validacoes: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn validar_consenso(&self) -> Result<()> {
        info!("Iniciando validação distribuída");
        
        // Simular validação de transação
        // Em implementação real, receberia dados da camada de processamento
        
        Ok(())
    }
    
    pub async fn adicionar_validacao(&self, validacao: ValidacaoConsenso) -> Result<bool> {
        // Verificar se a validação é válida
        if !self.validador.verificar_validacao(&validacao).await? {
            warn!("Validação inválida rejeitada");
            return Ok(false);
        }
        
        // Adicionar à lista de validações
        let transacao_id = "tx_exemplo".to_string(); // Em implementação real, extrair do contexto
        self.validacoes.write().await
            .entry(transacao_id.clone())
            .or_insert_with(Vec::new)
            .push(validacao.clone());
        
        // Verificar se atingiu quórum
        let validacoes_transacao = self.validacoes.read().await
            .get(&transacao_id)
            .cloned()
            .unwrap_or_default();
        
        if self.quorum.verificar_quorum(&validacoes_transacao).await {
            info!("Quórum atingido para transação {}", transacao_id);
            
            // Detectar possíveis nós maliciosos
            let maliciosos = self.detector_maliciosos.analisar_validacoes(&validacoes_transacao).await?;
            if !maliciosos.is_empty() {
                warn!("Nós maliciosos detectados: {:?}", maliciosos);
            }
            
            return Ok(true);
        }
        
        Ok(false)
    }
    
    pub async fn obter_validacoes(&self, transacao_id: &str) -> Vec<ValidacaoConsenso> {
        self.validacoes.read().await
            .get(transacao_id)
            .cloned()
            .unwrap_or_default()
    }
}