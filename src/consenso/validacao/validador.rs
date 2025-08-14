use crate::consenso::tipos::*;
use anyhow::Result;
use tracing::{info, warn};

pub struct ValidadorDistribuido {
    id_validador: String,
}

impl ValidadorDistribuido {
    pub fn new() -> Self {
        Self {
            id_validador: "validador_local".to_string(),
        }
    }
    
    pub async fn validar_transacao(&self, transacao: &Transacao, hash_cadeia_anterior: &[u8]) -> Result<ValidacaoConsenso> {
        info!("Validando transação {}", transacao.id);
        
        // Recalcular hash da transação
        let hash_transacao = self.calcular_hash_transacao(transacao)?;
        
        // Recalcular hash combinado da cadeia
        let hash_cadeia = self.calcular_hash_cadeia(&hash_transacao, hash_cadeia_anterior)?;
        
        // Validar estado anterior/final
        self.validar_estados(transacao)?;
        
        // Criar validação
        let validacao = ValidacaoConsenso {
            hash_transacao,
            hash_cadeia,
            validador_id: self.id_validador.clone(),
            assinatura: self.assinar_validacao(&hash_transacao, &hash_cadeia)?,
            timestamp: chrono::Utc::now(),
        };
        
        Ok(validacao)
    }
    
    pub async fn verificar_validacao(&self, validacao: &ValidacaoConsenso) -> Result<bool> {
        // Verificar assinatura
        if !self.verificar_assinatura(validacao)? {
            return Ok(false);
        }
        
        // Verificar timestamp (não muito antigo)
        let agora = chrono::Utc::now();
        let limite = agora - chrono::Duration::minutes(10);
        if validacao.timestamp < limite {
            return Ok(false);
        }
        
        // Verificar formato dos hashes
        if validacao.hash_transacao.len() != 32 || validacao.hash_cadeia.len() != 32 {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    fn calcular_hash_transacao(&self, transacao: &Transacao) -> Result<Vec<u8>> {
        let dados = serde_json::to_vec(transacao)?;
        Ok(calcular_hash(&dados))
    }
    
    fn calcular_hash_cadeia(&self, hash_transacao: &[u8], hash_anterior: &[u8]) -> Result<Vec<u8>> {
        let mut dados = Vec::new();
        dados.extend_from_slice(hash_transacao);
        dados.extend_from_slice(hash_anterior);
        Ok(calcular_hash(&dados))
    }
    
    fn validar_estados(&self, transacao: &Transacao) -> Result<()> {
        // Validar que o estado final é derivado corretamente do anterior
        if transacao.estado_anterior.is_empty() {
            return Err(anyhow::anyhow!("Estado anterior não pode estar vazio"));
        }
        
        if transacao.estado_final.is_empty() {
            return Err(anyhow::anyhow!("Estado final não pode estar vazio"));
        }
        
        // Em implementação real, validaria a transição de estado
        Ok(())
    }
    
    fn assinar_validacao(&self, hash_transacao: &[u8], hash_cadeia: &[u8]) -> Result<Vec<u8>> {
        let mut dados = Vec::new();
        dados.extend_from_slice(hash_transacao);
        dados.extend_from_slice(hash_cadeia);
        dados.extend_from_slice(self.id_validador.as_bytes());
        Ok(calcular_hash(&dados))
    }
    
    fn verificar_assinatura(&self, validacao: &ValidacaoConsenso) -> Result<bool> {
        let mut dados = Vec::new();
        dados.extend_from_slice(&validacao.hash_transacao);
        dados.extend_from_slice(&validacao.hash_cadeia);
        dados.extend_from_slice(validacao.validador_id.as_bytes());
        
        let assinatura_esperada = calcular_hash(&dados);
        Ok(assinatura_esperada == validacao.assinatura)
    }
}