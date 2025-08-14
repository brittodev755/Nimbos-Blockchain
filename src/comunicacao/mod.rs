mod broadcast;
mod retry;
mod protocolo;
mod rede;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, error};

pub use broadcast::*;
pub use retry::*;
pub use protocolo::*;
pub use rede::*;

pub struct CamadaComunicacao {
    broadcast: SistemaBroadcast,
    retry: MecanismoRetry,
    rede: GerenciadorRede,
    canal_mensagens: mpsc::UnboundedSender<Mensagem>,
}

impl CamadaComunicacao {
    pub async fn new() -> Result<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        let sistema = Self {
            broadcast: SistemaBroadcast::new(),
            retry: MecanismoRetry::new(),
            rede: GerenciadorRede::new(),
            canal_mensagens: tx,
        };
        
        // Iniciar loop de processamento de mensagens
        tokio::spawn(async move {
            while let Some(mensagem) = rx.recv().await {
                if let Err(e) = Self::processar_mensagem(mensagem).await {
                    error!("Erro ao processar mensagem: {}", e);
                }
            }
        });
        
        Ok(sistema)
    }
    
    pub async fn enviar_broadcast(&self, mensagem: Mensagem) -> Result<()> {
        info!("Enviando broadcast: {:?}", mensagem.tipo);
        
        let nos_destino = self.rede.obter_nos_ativos().await;
        self.broadcast.enviar_para_nos(&mensagem, &nos_destino).await?;
        
        Ok(())
    }
    
    async fn processar_mensagem(mensagem: Mensagem) -> Result<()> {
        match mensagem.tipo {
            TipoMensagem::Commitment => {
                info!("Processando commitment recebido");
            },
            TipoMensagem::Reveal => {
                info!("Processando reveal recebido");
            },
            TipoMensagem::Validacao => {
                info!("Processando validação recebida");
            },
            TipoMensagem::Transacao => {
                info!("Processando transação recebida");
            },
        }
        Ok(())
    }
}