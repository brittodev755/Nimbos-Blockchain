use crate::consenso::tipos::*;
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstadoTransacao {
    pub id: String,
    pub estado_anterior: Vec<u8>,
    pub estado_atual: Vec<u8>,
    pub timestamp_processamento: chrono::DateTime<chrono::Utc>,
    pub processador_id: String,
    pub hash_cadeia: Vec<u8>,
}

pub struct GerenciadorEstado {
    estados: RwLock<HashMap<String, EstadoTransacao>>,
    snapshots: RwLock<Vec<HashMap<String, EstadoTransacao>>>,
}

impl GerenciadorEstado {
    pub fn new() -> Self {
        Self {
            estados: RwLock::new(HashMap::new()),
            snapshots: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn atualizar_estado(&self, transacao: &Transacao, processador_id: &str, hash_cadeia: Vec<u8>) -> Result<()> {
        let estado = EstadoTransacao {
            id: transacao.id.clone(),
            estado_anterior: transacao.estado_anterior.clone(),
            estado_atual: transacao.estado_final.clone(),
            timestamp_processamento: transacao.timestamp,
            processador_id: processador_id.to_string(),
            hash_cadeia,
        };
        
        self.estados.write().await.insert(transacao.id.clone(), estado);
        Ok(())
    }
    
    pub async fn obter_estado(&self, transacao_id: &str) -> Option<EstadoTransacao> {
        self.estados.read().await.get(transacao_id).cloned()
    }
    
    pub async fn criar_snapshot(&self) -> Result<()> {
        let estados_atuais = self.estados.read().await.clone();
        self.snapshots.write().await.push(estados_atuais);
        Ok(())
    }
    
    pub async fn obter_ultimo_snapshot(&self) -> Option<HashMap<String, EstadoTransacao>> {
        self.snapshots.read().await.last().cloned()
    }
    
    pub async fn validar_consistencia(&self) -> Result<bool> {
        let estados = self.estados.read().await;
        
        // Verificar se todos os estados têm hashes válidos
        for estado in estados.values() {
            if estado.hash_cadeia.len() != 32 {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}