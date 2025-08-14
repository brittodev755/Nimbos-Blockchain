mod processador;
mod rotacao;
mod estado;

use crate::consenso::tipos::*;
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::info;

pub use processador::*;
pub use rotacao::*;
pub use estado::*;

pub struct CamadaProcessamento {
    processador: ProcessadorTransacao,
    rotacao: GerenciadorRotacao,
    estado: GerenciadorEstado,
    hash_cadeia_anterior: RwLock<Vec<u8>>,
}

impl CamadaProcessamento {
    pub fn new() -> Self {
        Self {
            processador: ProcessadorTransacao::new(),
            rotacao: GerenciadorRotacao::new(),
            estado: GerenciadorEstado::new(),
            hash_cadeia_anterior: RwLock::new(vec![0; 32]), // Genesis hash
        }
    }
    
    pub async fn processar_transacoes(&self, fila: &FilaOrdenada) -> Result<()> {
        info!("Iniciando processamento rotativo de transações");
        
        if let Some(no_processador) = fila.nos.first() {
            // Simular transação para processamento
            let transacao = self.criar_transacao_exemplo().await;
            
            // Processar transação
            let resultado = self.processador.processar(&transacao, &no_processador.id).await?;
            
            // Calcular hash combinado
            let hash_anterior = self.hash_cadeia_anterior.read().await.clone();
            let hash_combinado = self.calcular_hash_combinado(&resultado, &hash_anterior).await;
            
            // Atualizar hash da cadeia
            *self.hash_cadeia_anterior.write().await = hash_combinado;
            
            // Rotacionar fila
            self.rotacao.rotacionar_fila().await;
            
            info!("Transação processada e nó rotacionado");
        }
        
        Ok(())
    }
    
    async fn criar_transacao_exemplo(&self) -> Transacao {
        Transacao {
            id: "tx_exemplo".to_string(),
            dados: b"dados_exemplo".to_vec(),
            estado_anterior: b"estado_anterior".to_vec(),
            estado_final: b"estado_final".to_vec(),
            timestamp: chrono::Utc::now(),
            nonce: 12345,
            assinatura: b"assinatura_exemplo".to_vec(),
        }
    }
    
    async fn calcular_hash_combinado(&self, transacao: &Transacao, hash_anterior: &[u8]) -> Vec<u8> {
        let mut dados = Vec::new();
        dados.extend_from_slice(&serde_json::to_vec(transacao).unwrap_or_default());
        dados.extend_from_slice(hash_anterior);
        calcular_hash(&dados)
    }
}