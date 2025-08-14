use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

use crate::recompensas::DistribuicaoRecompensa;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContaNo {
    pub no_id: String,
    pub saldo: u64,
    pub total_recebido: u64,
    pub numero_transacoes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstatisticasDistribuicao {
    pub total_distribuido: u64,
    pub numero_distribuicoes: u64,
    pub media_por_distribuicao: f64,
    pub maior_distribuicao: u64,
    pub menor_distribuicao: u64,
}

pub struct DistribuidorRecompensas {
    contas: Arc<RwLock<HashMap<String, ContaNo>>>,
    estatisticas: Arc<RwLock<EstatisticasDistribuicao>>,
    taxa_minima: u64,
}

impl DistribuidorRecompensas {
    pub fn new() -> Self {
        Self {
            contas: Arc::new(RwLock::new(HashMap::new())),
            estatisticas: Arc::new(RwLock::new(EstatisticasDistribuicao {
                total_distribuido: 0,
                numero_distribuicoes: 0,
                media_por_distribuicao: 0.0,
                maior_distribuicao: 0,
                menor_distribuicao: u64::MAX,
            })),
            taxa_minima: 1000, // Taxa mínima em unidades base
        }
    }
    
    pub async fn distribuir(
        &self,
        processador_id: &str,
        validadores: &[String],
        distribuicao: &DistribuicaoRecompensa,
    ) -> Result<()> {
        info!("Iniciando distribuição de recompensas");
        
        // Validar distribuição
        self.validar_distribuicao(distribuicao)?;
        
        let mut contas = self.contas.write().await;
        
        // Distribuir para o processador
        self.creditar_conta(
            &mut contas,
            processador_id,
            distribuicao.processador.valor,
        ).await?;
        
        info!(
            "Creditado {} para processador {}",
            distribuicao.processador.valor,
            processador_id
        );
        
        // Distribuir para validadores
        for (i, validador_id) in validadores.iter().enumerate() {
            if let Some(recompensa_validador) = distribuicao.validadores.get(i) {
                self.creditar_conta(
                    &mut contas,
                    validador_id,
                    recompensa_validador.valor,
                ).await?;
                
                info!(
                    "Creditado {} para validador {}",
                    recompensa_validador.valor,
                    validador_id
                );
            }
        }
        
        // Atualizar estatísticas
        self.atualizar_estatisticas(distribuicao.total).await;
        
        info!("Distribuição de recompensas concluída com sucesso");
        Ok(())
    }
    
    async fn creditar_conta(
        &self,
        contas: &mut HashMap<String, ContaNo>,
        no_id: &str,
        valor: u64,
    ) -> Result<()> {
        let conta = contas.entry(no_id.to_string()).or_insert(ContaNo {
            no_id: no_id.to_string(),
            saldo: 0,
            total_recebido: 0,
            numero_transacoes: 0,
        });
        
        conta.saldo += valor;
        conta.total_recebido += valor;
        conta.numero_transacoes += 1;
        
        Ok(())
    }
    
    fn validar_distribuicao(&self, distribuicao: &DistribuicaoRecompensa) -> Result<()> {
        if distribuicao.total < self.taxa_minima {
            return Err(anyhow::anyhow!(
                "Valor total da distribuição ({}) é menor que a taxa mínima ({})",
                distribuicao.total,
                self.taxa_minima
            ));
        }
        
        let soma_calculada = distribuicao.processador.valor +
            distribuicao.validadores.iter().map(|v| v.valor).sum::<u64>();
        
        if soma_calculada != distribuicao.total {
            return Err(anyhow::anyhow!(
                "Soma da distribuição ({}) não confere com o total ({})",
                soma_calculada,
                distribuicao.total
            ));
        }
        
        Ok(())
    }
    
    async fn atualizar_estatisticas(&self, valor_distribuido: u64) {
        let mut stats = self.estatisticas.write().await;
        
        stats.total_distribuido += valor_distribuido;
        stats.numero_distribuicoes += 1;
        stats.media_por_distribuicao = stats.total_distribuido as f64 / stats.numero_distribuicoes as f64;
        
        if valor_distribuido > stats.maior_distribuicao {
            stats.maior_distribuicao = valor_distribuido;
        }
        
        if valor_distribuido < stats.menor_distribuicao {
            stats.menor_distribuicao = valor_distribuido;
        }
    }
    
    pub async fn obter_saldo(&self, no_id: &str) -> Option<u64> {
        let contas = self.contas.read().await;
        contas.get(no_id).map(|conta| conta.saldo)
    }
    
    pub async fn obter_conta(&self, no_id: &str) -> Option<ContaNo> {
        let contas = self.contas.read().await;
        contas.get(no_id).cloned()
    }
    
    pub async fn listar_contas(&self) -> Vec<ContaNo> {
        let contas = self.contas.read().await;
        contas.values().cloned().collect()
    }
    
    pub async fn obter_estatisticas(&self) -> EstatisticasDistribuicao {
        let stats = self.estatisticas.read().await;
        stats.clone()
    }
    
    pub async fn sacar(&self, no_id: &str, valor: u64) -> Result<()> {
        let mut contas = self.contas.write().await;
        
        if let Some(conta) = contas.get_mut(no_id) {
            if conta.saldo >= valor {
                conta.saldo -= valor;
                info!("Saque de {} realizado para nó {}", valor, no_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Saldo insuficiente. Saldo atual: {}, valor solicitado: {}",
                    conta.saldo,
                    valor
                ))
            }
        } else {
            Err(anyhow::anyhow!("Conta não encontrada para nó: {}", no_id))
        }
    }
    
    pub fn definir_taxa_minima(&mut self, nova_taxa: u64) {
        self.taxa_minima = nova_taxa;
        info!("Taxa mínima atualizada para: {}", nova_taxa);
    }
}

impl Default for DistribuidorRecompensas {
    fn default() -> Self {
        Self::new()
    }
}