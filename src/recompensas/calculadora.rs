use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistribuicaoRecompensa {
    pub processador: RecompensaNo,
    pub validadores: Vec<RecompensaNo>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecompensaNo {
    pub no_id: String,
    pub valor: u64,
    pub porcentagem: f64,
}

pub struct CalculadoraTaxas {
    porcentagem_processador: f64,
    porcentagem_validadores: f64,
}

impl CalculadoraTaxas {
    pub fn new() -> Self {
        Self {
            porcentagem_processador: 0.8, // 80%
            porcentagem_validadores: 0.2, // 20%
        }
    }
    
    pub fn calcular_distribuicao(&self, valor_total: u64, num_validadores: usize) -> Result<DistribuicaoRecompensa> {
        if num_validadores == 0 {
            return Err(anyhow::anyhow!("Número de validadores não pode ser zero"));
        }
        
        let valor_processador = (valor_total as f64 * self.porcentagem_processador) as u64;
        let valor_total_validadores = valor_total - valor_processador;
        let valor_por_validador = valor_total_validadores / num_validadores as u64;
        
        let processador = RecompensaNo {
            no_id: "processador".to_string(),
            valor: valor_processador,
            porcentagem: self.porcentagem_processador * 100.0,
        };
        
        let validadores: Vec<RecompensaNo> = (0..num_validadores)
            .map(|i| RecompensaNo {
                no_id: format!("validador_{}", i),
                valor: valor_por_validador,
                porcentagem: (self.porcentagem_validadores * 100.0) / num_validadores as f64,
            })
            .collect();
        
        Ok(DistribuicaoRecompensa {
            processador,
            validadores,
            total: valor_total,
        })
    }
}