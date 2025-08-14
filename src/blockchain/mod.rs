mod bloco;
mod cadeia;
mod checkpoint;
mod validador_cadeia;
mod migrador;

use anyhow::Result;
use tracing::info;
use std::path::Path;

pub use bloco::*;
pub use cadeia::*;
pub use checkpoint::*;
pub use validador_cadeia::*;
pub use migrador::*;

pub struct CamadaBlockchain {
    cadeia: CadeiaBlockchain,
    checkpoint: SistemaCheckpoint,
    validador: ValidadorCadeia,
    persistencia_ativa: bool,
}

impl CamadaBlockchain {
    pub fn new() -> Self {
        Self {
            cadeia: CadeiaBlockchain::new(),
            checkpoint: SistemaCheckpoint::new(),
            validador: ValidadorCadeia::new(),
            persistencia_ativa: false,
        }
    }
    
    /// Cria nova camada com persistência otimizada
    pub fn new_com_persistencia<P: AsRef<Path>>(caminho_db: P) -> Result<Self> {
        info!("Inicializando blockchain com persistência em: {:?}", caminho_db.as_ref());
        
        Ok(Self {
            cadeia: CadeiaBlockchain::new_com_persistencia(caminho_db)?,
            checkpoint: SistemaCheckpoint::new(),
            validador: ValidadorCadeia::new(),
            persistencia_ativa: true,
        })
    }
    
    /// Verifica se a persistência está ativa
    pub fn tem_persistencia(&self) -> bool {
        self.persistencia_ativa
    }
    
    /// Migra dados existentes para formato otimizado
    pub async fn migrar_dados<P: AsRef<Path>>(&self, caminho_db_destino: P) -> Result<()> {
        if self.persistencia_ativa {
            info!("Blockchain já usa persistência, migração não necessária");
            return Ok(());
        }
        
        let mut migrador = MigradorDados::new(caminho_db_destino)?;
        migrador.migrar_para_formato_otimizado().await?;
        migrador.validar_migracao().await?;
        
        Ok(())
    }

    pub async fn adicionar_bloco(&mut self, transacoes: Vec<crate::consenso::tipos::Transacao>) -> Result<()> {
        info!("Adicionando novo bloco à cadeia (persistência: {})", self.persistencia_ativa);
        
        // Criar novo bloco
        let bloco = self.cadeia.criar_proximo_bloco(transacoes).await?;
        
        // Validar bloco
        if !self.validador.validar_bloco(&bloco, &self.cadeia).await? {
            return Err(anyhow::anyhow!("Bloco inválido"));
        }
        
        // Adicionar à cadeia (com persistência automática se ativa)
        self.cadeia.adicionar_bloco(bloco.clone()).await?;
        
        // Processar checkpoint se necessário
        self.checkpoint.processar_bloco(&bloco, &self.cadeia).await?;
        
        info!("Bloco {} adicionado com sucesso", bloco.numero);
        Ok(())
    }
}