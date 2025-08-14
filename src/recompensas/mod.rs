mod distribuidor;
mod calculadora;
mod ledger;

use anyhow::Result;
use tracing::info;

pub use distribuidor::*;
pub use calculadora::*;
pub use ledger::*;

pub struct CamadaRecompensas {
    distribuidor: DistribuidorRecompensas,
    calculadora: CalculadoraTaxas,
    ledger: LedgerRecompensas,
}

impl CamadaRecompensas {
    pub fn new() -> Self {
        Self {
            distribuidor: DistribuidorRecompensas::new(),
            calculadora: CalculadoraTaxas::new(),
            ledger: LedgerRecompensas::new(),
        }
    }
    
    pub async fn processar_recompensas(&self, transacao_id: &str, processador_id: &str, validadores: &[String], valor_taxa: u64) -> Result<()> {
        info!("Processando recompensas para transação {}", transacao_id);
        
        // Calcular distribuição
        let distribuicao = self.calculadora.calcular_distribuicao(valor_taxa, validadores.len())?;
        
        // Distribuir recompensas
        self.distribuidor.distribuir(processador_id, validadores, &distribuicao).await?;
        
        // Registrar no ledger
        self.ledger.registrar_distribuicao(transacao_id, &distribuicao).await?;
        
        Ok(())
    }
}