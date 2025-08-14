use super::*;
use anyhow::Result;
use std::path::Path;
use std::time::Instant;
use tracing::{info, warn};

pub struct MigradorDados {
    cadeia_origem: CadeiaBlockchain,
    cadeia_destino: CadeiaBlockchain,
}

impl MigradorDados {
    pub fn new<P: AsRef<Path>>(caminho_db_destino: P) -> Result<Self> {
        Ok(Self {
            cadeia_origem: CadeiaBlockchain::new(),
            cadeia_destino: CadeiaBlockchain::new_com_persistencia(caminho_db_destino)?,
        })
    }
    
    /// Migração otimizada para formato binário com persistência
    pub async fn migrar_para_formato_otimizado(&mut self) -> Result<()> {
        info!("Iniciando migração para formato otimizado");
        let inicio = Instant::now();
        
        let blocos_origem = self.cadeia_origem.blocos.read().await;
        let total_blocos = blocos_origem.len();
        
        for (i, bloco) in blocos_origem.iter().enumerate() {
            // Migrar usando serialização binária otimizada
            self.cadeia_destino.adicionar_bloco(bloco.clone()).await?;
            
            if i % 100 == 0 {
                info!("Migrados {}/{} blocos", i + 1, total_blocos);
            }
        }
        
        let duracao = inicio.elapsed();
        info!("Migração concluída em {:?}. {} blocos migrados", duracao, total_blocos);
        
        Ok(())
    }
    
    /// Comparação detalhada de performance entre formatos
    pub async fn comparar_performance(&self) -> Result<()> {
        info!("Iniciando comparação de performance");
        
        let blocos = self.cadeia_origem.blocos.read().await;
        if blocos.is_empty() {
            warn!("Nenhum bloco disponível para comparação");
            return Ok(());
        }
        
        let mut total_json_size = 0usize;
        let mut total_binary_size = 0usize;
        let mut tempo_json = std::time::Duration::new(0, 0);
        let mut tempo_binary = std::time::Duration::new(0, 0);
        
        for bloco in blocos.iter().take(100) { // Testar com 100 blocos
            // Teste JSON
            let inicio = Instant::now();
            let dados_json = bloco.serializar_json()?;
            tempo_json += inicio.elapsed();
            total_json_size += dados_json.len();
            
            // Teste Binário
            let inicio = Instant::now();
            let dados_binary = bloco.serializar_binario()?;
            tempo_binary += inicio.elapsed();
            total_binary_size += dados_binary.len();
        }
        
        let reducao_tamanho = ((total_json_size as f64 - total_binary_size as f64) / total_json_size as f64) * 100.0;
        let melhoria_velocidade = tempo_json.as_nanos() as f64 / tempo_binary.as_nanos() as f64;
        
        info!("=== COMPARAÇÃO DE PERFORMANCE ===");
        info!("Tamanho JSON: {} bytes", total_json_size);
        info!("Tamanho Binário: {} bytes", total_binary_size);
        info!("Redução de tamanho: {:.2}%", reducao_tamanho);
        info!("Tempo JSON: {:?}", tempo_json);
        info!("Tempo Binário: {:?}", tempo_binary);
        info!("Melhoria de velocidade: {:.2}x", melhoria_velocidade);
        
        Ok(())
    }
    
    /// Validação da integridade após migração
    pub async fn validar_migracao(&self) -> Result<bool> {
        info!("Validando integridade da migração");
        
        let blocos_origem = self.cadeia_origem.blocos.read().await;
        let blocos_destino = self.cadeia_destino.blocos.read().await;
        
        if blocos_origem.len() != blocos_destino.len() {
            warn!("Número de blocos diferente: origem={}, destino={}", 
                  blocos_origem.len(), blocos_destino.len());
            return Ok(false);
        }
        
        for (i, (bloco_origem, bloco_destino)) in blocos_origem.iter().zip(blocos_destino.iter()).enumerate() {
            if bloco_origem.hash_bloco != bloco_destino.hash_bloco {
                warn!("Hash diferente no bloco {}", i);
                return Ok(false);
            }
        }
        
        info!("Validação de migração: SUCESSO");
        Ok(true)
    }
}