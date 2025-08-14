mod consenso;
mod comunicacao;
mod blockchain;
mod recompensas;
mod deteccao_falhas;

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("🚀 Iniciando Nimbos Blockchain");
    
    // Inicializar todas as camadas
    let mut sistema_consenso = consenso::SistemaConsenso::new().await?;
    let comunicacao = comunicacao::CamadaComunicacao::new().await?;
    let mut blockchain = blockchain::CamadaBlockchain::new();
    let recompensas = recompensas::CamadaRecompensas::new();
    let deteccao_falhas = deteccao_falhas::CamadaDeteccaoFalhas::new();
    
    info!("✅ Todas as camadas inicializadas com sucesso");
    
    // Loop principal da blockchain
    loop {
        // Executar ciclo de consenso
        if let Err(e) = sistema_consenso.executar().await {
            tracing::error!("Erro no consenso: {}", e);
        }
        
        // Verificar saúde da rede
        if let Err(e) = deteccao_falhas.verificar_saude_rede().await {
            tracing::error!("Erro na detecção de falhas: {}", e);
        }
        
        // Aguardar próximo ciclo
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}