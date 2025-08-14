use super::{bloco::Bloco, cadeia::CadeiaBlockchain};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub numero_bloco: u64,
    pub hash_bloco: Vec<u8>,
    pub merkle_root_estado: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub assinatura: Vec<u8>,
    pub validadores: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstadoCheckpoint {
    pub balances: HashMap<String, u64>,
    pub contratos: HashMap<String, Vec<u8>>,
    pub nonces: HashMap<String, u64>,
    pub metadata: HashMap<String, String>,
}

pub struct SistemaCheckpoint {
    checkpoints: RwLock<Vec<Checkpoint>>,
    estados: RwLock<HashMap<u64, EstadoCheckpoint>>,
    intervalo_checkpoint: u64,
    ultimo_checkpoint: RwLock<Option<u64>>,
}

impl SistemaCheckpoint {
    pub fn new() -> Self {
        Self {
            checkpoints: RwLock::new(Vec::new()),
            estados: RwLock::new(HashMap::new()),
            intervalo_checkpoint: 100, // Checkpoint a cada 100 blocos
            ultimo_checkpoint: RwLock::new(None),
        }
    }
    
    pub fn new_com_intervalo(intervalo: u64) -> Self {
        Self {
            checkpoints: RwLock::new(Vec::new()),
            estados: RwLock::new(HashMap::new()),
            intervalo_checkpoint: intervalo,
            ultimo_checkpoint: RwLock::new(None),
        }
    }
    
    pub async fn deve_criar_checkpoint(&self, cadeia: &CadeiaBlockchain) -> bool {
        let altura_atual = cadeia.obter_altura().await;
        let ultimo = *self.ultimo_checkpoint.read().await;
        
        match ultimo {
            Some(ultimo_numero) => {
                altura_atual >= ultimo_numero + self.intervalo_checkpoint
            },
            None => altura_atual >= self.intervalo_checkpoint
        }
    }
    
    pub async fn criar_checkpoint(&self, cadeia: &CadeiaBlockchain) -> Result<()> {
        let altura_atual = cadeia.obter_altura().await;
        
        info!("Criando checkpoint para bloco {}", altura_atual);
        
        // Obter bloco atual
        let bloco = cadeia.obter_bloco_por_numero(altura_atual).await
            .ok_or_else(|| anyhow::anyhow!("Bloco não encontrado para checkpoint"))?;
        
        // Calcular estado atual
        let estado = self.calcular_estado_atual(cadeia, altura_atual).await?;
        
        // Criar checkpoint
        let checkpoint = Checkpoint {
            numero_bloco: altura_atual,
            hash_bloco: bloco.hash_bloco.clone(),
            merkle_root_estado: self.calcular_merkle_root_estado(&estado)?,
            timestamp: Utc::now(),
            assinatura: self.assinar_checkpoint(&bloco, &estado)?,
            validadores: vec!["validador_principal".to_string()], // Em implementação real, seria dinâmico
        };
        
        // Armazenar checkpoint e estado
        let mut checkpoints = self.checkpoints.write().await;
        let mut estados = self.estados.write().await;
        
        checkpoints.push(checkpoint.clone());
        estados.insert(altura_atual, estado);
        
        *self.ultimo_checkpoint.write().await = Some(altura_atual);
        
        info!("Checkpoint criado com sucesso para bloco {}", altura_atual);
        Ok(())
    }
    
    pub async fn validar_checkpoint(&self, checkpoint: &Checkpoint, cadeia: &CadeiaBlockchain) -> Result<bool> {
        // Verificar se o bloco existe
        let bloco = cadeia.obter_bloco_por_numero(checkpoint.numero_bloco).await
            .ok_or_else(|| anyhow::anyhow!("Bloco do checkpoint não encontrado"))?;
        
        // Verificar hash do bloco
        if bloco.hash_bloco != checkpoint.hash_bloco {
            warn!("Hash do bloco no checkpoint não confere");
            return Ok(false);
        }
        
        // Recalcular estado
        let estado = self.calcular_estado_atual(cadeia, checkpoint.numero_bloco).await?;
        let merkle_calculado = self.calcular_merkle_root_estado(&estado)?;
        
        // Verificar Merkle root do estado
        if merkle_calculado != checkpoint.merkle_root_estado {
            warn!("Merkle root do estado no checkpoint não confere");
            return Ok(false);
        }
        
        // Verificar assinatura
        let assinatura_calculada = self.assinar_checkpoint(&bloco, &estado)?;
        if assinatura_calculada != checkpoint.assinatura {
            warn!("Assinatura do checkpoint inválida");
            return Ok(false);
        }
        
        Ok(true)
    }
    
    pub async fn obter_checkpoint_mais_recente(&self) -> Option<Checkpoint> {
        let checkpoints = self.checkpoints.read().await;
        checkpoints.last().cloned()
    }
    
    pub async fn obter_estado_checkpoint(&self, numero_bloco: u64) -> Option<EstadoCheckpoint> {
        let estados = self.estados.read().await;
        estados.get(&numero_bloco).cloned()
    }
    
    pub async fn restaurar_de_checkpoint(&self, numero_bloco: u64) -> Result<EstadoCheckpoint> {
        let estados = self.estados.read().await;
        
        estados.get(&numero_bloco)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Estado do checkpoint {} não encontrado", numero_bloco))
    }
    
    async fn calcular_estado_atual(&self, cadeia: &CadeiaBlockchain, ate_bloco: u64) -> Result<EstadoCheckpoint> {
        let mut estado = EstadoCheckpoint {
            balances: HashMap::new(),
            contratos: HashMap::new(),
            nonces: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        // Processar todos os blocos até o número especificado
        for numero in 0..=ate_bloco {
            if let Some(bloco) = cadeia.obter_bloco_por_numero(numero).await {
                self.aplicar_transacoes_ao_estado(&mut estado, &bloco).await?;
            }
        }
        
        // Adicionar metadata
        estado.metadata.insert("ultimo_bloco".to_string(), ate_bloco.to_string());
        estado.metadata.insert("timestamp".to_string(), Utc::now().to_rfc3339());
        
        Ok(estado)
    }
    
    async fn aplicar_transacoes_ao_estado(&self, estado: &mut EstadoCheckpoint, bloco: &Bloco) -> Result<()> {
        for transacao in &bloco.transacoes {
            // Simular aplicação de transação ao estado
            // Em implementação real, isso seria mais complexo
            
            // Atualizar nonce
            let nonce_atual = estado.nonces.get(&transacao.id).unwrap_or(&0);
            estado.nonces.insert(transacao.id.clone(), nonce_atual + 1);
            
            // Simular mudança de balance (exemplo)
            let balance_atual = estado.balances.get(&transacao.id).unwrap_or(&1000);
            estado.balances.insert(transacao.id.clone(), balance_atual.saturating_sub(1));
        }
        
        Ok(())
    }
    
    fn calcular_merkle_root_estado(&self, estado: &EstadoCheckpoint) -> Result<Vec<u8>> {
        use crate::consenso::tipos::calcular_hash;
        
        let dados_estado = serde_json::to_vec(estado)?;
        Ok(calcular_hash(&dados_estado))
    }
    
    fn assinar_checkpoint(&self, bloco: &Bloco, estado: &EstadoCheckpoint) -> Result<Vec<u8>> {
        use crate::consenso::tipos::calcular_hash;
        
        let mut dados = Vec::new();
        dados.extend_from_slice(&bloco.hash_bloco);
        
        let estado_serializado = serde_json::to_vec(estado)?;
        dados.extend_from_slice(&estado_serializado);
        
        Ok(calcular_hash(&dados))
    }
    
    pub async fn limpar_checkpoints_antigos(&self, manter_ultimos: usize) -> Result<()> {
        let mut checkpoints = self.checkpoints.write().await;
        let mut estados = self.estados.write().await;
        
        if checkpoints.len() > manter_ultimos {
            let remover_ate = checkpoints.len() - manter_ultimos;
            
            // Remover checkpoints antigos
            for checkpoint in checkpoints.drain(0..remover_ate) {
                estados.remove(&checkpoint.numero_bloco);
            }
            
            info!("Removidos {} checkpoints antigos", remover_ate);
        }
        
        Ok(())
    }
    
    pub async fn obter_estatisticas(&self) -> HashMap<String, u64> {
        let checkpoints = self.checkpoints.read().await;
        let estados = self.estados.read().await;
        
        let mut stats = HashMap::new();
        stats.insert("total_checkpoints".to_string(), checkpoints.len() as u64);
        stats.insert("total_estados".to_string(), estados.len() as u64);
        stats.insert("intervalo_checkpoint".to_string(), self.intervalo_checkpoint);
        
        if let Some(ultimo) = *self.ultimo_checkpoint.read().await {
            stats.insert("ultimo_checkpoint".to_string(), ultimo);
        }
        
        stats
    }
}