mod monitor;
mod timeout;
mod recuperacao;

use anyhow::Result;
use tracing::{info, warn};

pub use monitor::*;
pub use timeout::*;
pub use recuperacao::*;

pub struct CamadaDeteccaoFalhas {
    monitor: MonitorNos,
    timeout: GerenciadorTimeout,
    recuperacao: SistemaRecuperacao,
}

impl CamadaDeteccaoFalhas {
    pub fn new() -> Self {
        Self {
            monitor: MonitorNos::new(),
            timeout: GerenciadorTimeout::new(),
            recuperacao: SistemaRecuperacao::new(),
        }
    }
    
    pub async fn verificar_saude_rede(&self) -> Result<()> {
        info!("Verificando saúde da rede");
        
        // Monitorar nós
        let nos_problematicos = self.monitor.verificar_nos().await?;
        
        // Processar timeouts
        let nos_timeout = self.timeout.verificar_timeouts().await?;
        
        // Iniciar recuperação se necessário
        for no_id in nos_problematicos.iter().chain(nos_timeout.iter()) {
            warn!("Iniciando recuperação para nó: {}", no_id);
            self.recuperacao.iniciar_recuperacao(no_id).await?;
        }
        
        Ok(())
    }
}