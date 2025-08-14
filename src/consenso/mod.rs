pub mod registro;
pub mod reveal;
pub mod ordenacao;
pub mod merkle;
pub mod processamento;
pub mod validacao;
pub mod tipos;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

pub use tipos::*;

pub struct SistemaConsenso {
    registro: registro::CamadaRegistro,
    reveal: reveal::CamadaReveal,
    ordenacao: ordenacao::CamadaOrdenacao,
    merkle: merkle::CamadaMerkle,
    processamento: processamento::CamadaProcessamento,
    validacao: validacao::CamadaValidacao,
}

impl SistemaConsenso {
    pub async fn new() -> Result<Self> {
        info!("Inicializando Sistema de Consenso");
        
        Ok(Self {
            registro: registro::CamadaRegistro::new(),
            reveal: reveal::CamadaReveal::new(),
            ordenacao: ordenacao::CamadaOrdenacao::new(),
            merkle: merkle::CamadaMerkle::new(),
            processamento: processamento::CamadaProcessamento::new(),
            validacao: validacao::CamadaValidacao::new(),
        })
    }
    
    pub async fn executar(&mut self) -> Result<()> {
        info!("Executando ciclo de consenso");
        
        // Implementar loop principal do consenso
        loop {
            // 1. Fase de Registro
            self.registro.processar_commitments().await?;
            
            // 2. Fase de Reveal
            self.reveal.processar_reveals().await?;
            
            // 3. Ordenação Determinística
            let fila = self.ordenacao.gerar_fila().await?;
            
            // 4. Gerar Merkle Tree
            let merkle_root = self.merkle.gerar_arvore(&fila).await?;
            
            // 5. Processamento Rotativo
            self.processamento.processar_transacoes(&fila).await?;
            
            // 6. Validação Distribuída
            self.validacao.validar_consenso().await?;
            
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}