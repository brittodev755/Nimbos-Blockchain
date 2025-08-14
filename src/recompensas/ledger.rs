use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::recompensas::DistribuicaoRecompensa;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroRecompensa {
    pub id: String,
    pub transacao_id: String,
    pub timestamp: DateTime<Utc>,
    pub distribuicao: DistribuicaoRecompensa,
    pub status: StatusRegistro,
    pub hash_verificacao: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StatusRegistro {
    Pendente,
    Confirmado,
    Falhou,
    Revertido,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumoLedger {
    pub total_registros: usize,
    pub valor_total_distribuido: u64,
    pub registros_por_status: HashMap<String, usize>,
    pub periodo_inicio: Option<DateTime<Utc>>,
    pub periodo_fim: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiltroLedger {
    pub transacao_id: Option<String>,
    pub status: Option<StatusRegistro>,
    pub data_inicio: Option<DateTime<Utc>>,
    pub data_fim: Option<DateTime<Utc>>,
    pub no_id: Option<String>,
}

pub struct LedgerRecompensas {
    registros: Arc<RwLock<HashMap<String, RegistroRecompensa>>>,
    indice_transacao: Arc<RwLock<HashMap<String, Vec<String>>>>, // transacao_id -> registro_ids
    indice_no: Arc<RwLock<HashMap<String, Vec<String>>>>, // no_id -> registro_ids
    limite_registros: usize,
}

impl LedgerRecompensas {
    pub fn new() -> Self {
        Self {
            registros: Arc::new(RwLock::new(HashMap::new())),
            indice_transacao: Arc::new(RwLock::new(HashMap::new())),
            indice_no: Arc::new(RwLock::new(HashMap::new())),
            limite_registros: 10000, // Limite para evitar uso excessivo de memória
        }
    }
    
    pub async fn registrar_distribuicao(
        &self,
        transacao_id: &str,
        distribuicao: &DistribuicaoRecompensa,
    ) -> Result<String> {
        let registro_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        let registro = RegistroRecompensa {
            id: registro_id.clone(),
            transacao_id: transacao_id.to_string(),
            timestamp,
            distribuicao: distribuicao.clone(),
            status: StatusRegistro::Pendente,
            hash_verificacao: self.calcular_hash_verificacao(transacao_id, distribuicao, timestamp),
        };
        
        // Verificar limite de registros
        self.verificar_limite_registros().await;
        
        // Inserir registro
        {
            let mut registros = self.registros.write().await;
            registros.insert(registro_id.clone(), registro);
        }
        
        // Atualizar índices
        self.atualizar_indices(&registro_id, transacao_id, distribuicao).await;
        
        info!(
            "Registro de recompensa criado: {} para transação {}",
            registro_id,
            transacao_id
        );
        
        Ok(registro_id)
    }
    
    pub async fn confirmar_registro(&self, registro_id: &str) -> Result<()> {
        let mut registros = self.registros.write().await;
        
        if let Some(registro) = registros.get_mut(registro_id) {
            registro.status = StatusRegistro::Confirmado;
            info!("Registro {} confirmado", registro_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Registro não encontrado: {}", registro_id))
        }
    }
    
    pub async fn falhar_registro(&self, registro_id: &str) -> Result<()> {
        let mut registros = self.registros.write().await;
        
        if let Some(registro) = registros.get_mut(registro_id) {
            registro.status = StatusRegistro::Falhou;
            warn!("Registro {} marcado como falhou", registro_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Registro não encontrado: {}", registro_id))
        }
    }
    
    pub async fn reverter_registro(&self, registro_id: &str) -> Result<()> {
        let mut registros = self.registros.write().await;
        
        if let Some(registro) = registros.get_mut(registro_id) {
            if registro.status == StatusRegistro::Confirmado {
                registro.status = StatusRegistro::Revertido;
                warn!("Registro {} revertido", registro_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Apenas registros confirmados podem ser revertidos. Status atual: {:?}",
                    registro.status
                ))
            }
        } else {
            Err(anyhow::anyhow!("Registro não encontrado: {}", registro_id))
        }
    }
    
    pub async fn obter_registro(&self, registro_id: &str) -> Option<RegistroRecompensa> {
        let registros = self.registros.read().await;
        registros.get(registro_id).cloned()
    }
    
    pub async fn buscar_por_transacao(&self, transacao_id: &str) -> Vec<RegistroRecompensa> {
        let indice = self.indice_transacao.read().await;
        let registros = self.registros.read().await;
        
        if let Some(registro_ids) = indice.get(transacao_id) {
            registro_ids
                .iter()
                .filter_map(|id| registros.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub async fn buscar_por_no(&self, no_id: &str) -> Vec<RegistroRecompensa> {
        let indice = self.indice_no.read().await;
        let registros = self.registros.read().await;
        
        if let Some(registro_ids) = indice.get(no_id) {
            registro_ids
                .iter()
                .filter_map(|id| registros.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub async fn buscar_com_filtro(&self, filtro: &FiltroLedger) -> Vec<RegistroRecompensa> {
        let registros = self.registros.read().await;
        
        registros
            .values()
            .filter(|registro| self.aplicar_filtro(registro, filtro))
            .cloned()
            .collect()
    }
    
    pub async fn obter_historico_no(&self, no_id: &str, limite: Option<usize>) -> Vec<RegistroRecompensa> {
        let mut registros = self.buscar_por_no(no_id).await;
        
        // Ordenar por timestamp (mais recente primeiro)
        registros.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(limite) = limite {
            registros.truncate(limite);
        }
        
        registros
    }
    
    pub async fn obter_resumo(&self) -> ResumoLedger {
        let registros = self.registros.read().await;
        
        let mut registros_por_status = HashMap::new();
        let mut valor_total = 0u64;
        let mut periodo_inicio: Option<DateTime<Utc>> = None;
        let mut periodo_fim: Option<DateTime<Utc>> = None;
        
        for registro in registros.values() {
            // Contar por status
            let status_str = format!("{:?}", registro.status);
            *registros_por_status.entry(status_str).or_insert(0) += 1;
            
            // Somar valores
            valor_total += registro.distribuicao.total;
            
            // Atualizar período
            if periodo_inicio.is_none() || registro.timestamp < periodo_inicio.unwrap() {
                periodo_inicio = Some(registro.timestamp);
            }
            if periodo_fim.is_none() || registro.timestamp > periodo_fim.unwrap() {
                periodo_fim = Some(registro.timestamp);
            }
        }
        
        ResumoLedger {
            total_registros: registros.len(),
            valor_total_distribuido: valor_total,
            registros_por_status,
            periodo_inicio,
            periodo_fim,
        }
    }
    
    pub async fn limpar_registros_antigos(&self, dias: i64) -> usize {
        let limite_tempo = Utc::now() - chrono::Duration::days(dias);
        let mut registros = self.registros.write().await;
        let mut indice_transacao = self.indice_transacao.write().await;
        let mut indice_no = self.indice_no.write().await;
        
        let registros_antigos: Vec<String> = registros
            .iter()
            .filter(|(_, registro)| registro.timestamp < limite_tempo)
            .map(|(id, _)| id.clone())
            .collect();
        
        let count = registros_antigos.len();
        
        for registro_id in registros_antigos {
            if let Some(registro) = registros.remove(&registro_id) {
                // Remover dos índices
                self.remover_dos_indices(&registro_id, &registro, &mut indice_transacao, &mut indice_no);
            }
        }
        
        if count > 0 {
            info!("Removidos {} registros antigos (mais de {} dias)", count, dias);
        }
        
        count
    }
    
    fn calcular_hash_verificacao(
        &self,
        transacao_id: &str,
        distribuicao: &DistribuicaoRecompensa,
        timestamp: DateTime<Utc>,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        transacao_id.hash(&mut hasher);
        distribuicao.total.hash(&mut hasher);
        timestamp.timestamp().hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    async fn atualizar_indices(
        &self,
        registro_id: &str,
        transacao_id: &str,
        distribuicao: &DistribuicaoRecompensa,
    ) {
        // Índice por transação
        {
            let mut indice = self.indice_transacao.write().await;
            indice
                .entry(transacao_id.to_string())
                .or_insert_with(Vec::new)
                .push(registro_id.to_string());
        }
        
        // Índice por nó
        {
            let mut indice = self.indice_no.write().await;
            
            // Processador
            indice
                .entry(distribuicao.processador.no_id.clone())
                .or_insert_with(Vec::new)
                .push(registro_id.to_string());
            
            // Validadores
            for validador in &distribuicao.validadores {
                indice
                    .entry(validador.no_id.clone())
                    .or_insert_with(Vec::new)
                    .push(registro_id.to_string());
            }
        }
    }
    
    fn remover_dos_indices(
        &self,
        registro_id: &str,
        registro: &RegistroRecompensa,
        indice_transacao: &mut HashMap<String, Vec<String>>,
        indice_no: &mut HashMap<String, Vec<String>>,
    ) {
        // Remover do índice de transação
        if let Some(ids) = indice_transacao.get_mut(&registro.transacao_id) {
            ids.retain(|id| id != registro_id);
            if ids.is_empty() {
                indice_transacao.remove(&registro.transacao_id);
            }
        }
        
        // Remover do índice de nós
        let nos_ids = std::iter::once(&registro.distribuicao.processador.no_id)
            .chain(registro.distribuicao.validadores.iter().map(|v| &v.no_id));
        
        for no_id in nos_ids {
            if let Some(ids) = indice_no.get_mut(no_id) {
                ids.retain(|id| id != registro_id);
                if ids.is_empty() {
                    indice_no.remove(no_id);
                }
            }
        }
    }
    
    fn aplicar_filtro(&self, registro: &RegistroRecompensa, filtro: &FiltroLedger) -> bool {
        if let Some(ref transacao_id) = filtro.transacao_id {
            if registro.transacao_id != *transacao_id {
                return false;
            }
        }
        
        if let Some(ref status) = filtro.status {
            if registro.status != *status {
                return false;
            }
        }
        
        if let Some(data_inicio) = filtro.data_inicio {
            if registro.timestamp < data_inicio {
                return false;
            }
        }
        
        if let Some(data_fim) = filtro.data_fim {
            if registro.timestamp > data_fim {
                return false;
            }
        }
        
        if let Some(ref no_id) = filtro.no_id {
            let tem_no = registro.distribuicao.processador.no_id == *no_id ||
                registro.distribuicao.validadores.iter().any(|v| v.no_id == *no_id);
            if !tem_no {
                return false;
            }
        }
        
        true
    }
    
    async fn verificar_limite_registros(&self) {
        let registros = self.registros.read().await;
        if registros.len() >= self.limite_registros {
            drop(registros);
            warn!(
                "Limite de registros atingido ({}). Considere limpar registros antigos.",
                self.limite_registros
            );
        }
    }
}

impl Default for LedgerRecompensas {
    fn default() -> Self {
        Self::new()
    }
}