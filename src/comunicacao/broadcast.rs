use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, timeout};
use tracing::{info, warn, error};
use crate::comunicacao::protocolo::{Mensagem, RespostaMensagem};

#[derive(Debug, Clone)]
pub struct No {
    pub id: String,
    pub endereco: String,
    pub porta: u16,
    pub ativo: bool,
    pub ultima_resposta: chrono::DateTime<chrono::Utc>,
}

pub struct SistemaBroadcast {
    timeout_padrao: Duration,
    max_tentativas: u32,
    estatisticas: Arc<RwLock<EstatisticasBroadcast>>,
}

#[derive(Debug, Default)]
struct EstatisticasBroadcast {
    mensagens_enviadas: u64,
    mensagens_entregues: u64,
    mensagens_falharam: u64,
    tempo_medio_entrega: Duration,
}

impl SistemaBroadcast {
    pub fn new() -> Self {
        Self {
            timeout_padrao: Duration::from_secs(30),
            max_tentativas: 3,
            estatisticas: Arc::new(RwLock::new(EstatisticasBroadcast::default())),
        }
    }
    
    pub async fn enviar_para_nos(&self, mensagem: &Mensagem, nos: &[No]) -> Result<Vec<RespostaMensagem>> {
        info!("Iniciando broadcast para {} nós", nos.len());
        
        let inicio = std::time::Instant::now();
        let mut respostas = Vec::new();
        let mut handles = Vec::new();
        
        // Enviar para todos os nós em paralelo
        for no in nos {
            if !no.ativo {
                warn!("Nó {} está inativo, pulando", no.id);
                continue;
            }
            
            let mensagem_clone = mensagem.clone();
            let no_clone = no.clone();
            let timeout_duracao = self.timeout_padrao;
            
            let handle = tokio::spawn(async move {
                Self::enviar_para_no_individual(mensagem_clone, no_clone, timeout_duracao).await
            });
            
            handles.push(handle);
        }
        
        // Aguardar todas as respostas
        for handle in handles {
            match handle.await {
                Ok(Ok(resposta)) => {
                    respostas.push(resposta);
                }
                Ok(Err(e)) => {
                    error!("Erro ao enviar mensagem: {}", e);
                }
                Err(e) => {
                    error!("Erro na task de envio: {}", e);
                }
            }
        }
        
        // Atualizar estatísticas
        let duracao = inicio.elapsed();
        self.atualizar_estatisticas(nos.len(), respostas.len(), duracao).await;
        
        info!("Broadcast concluído: {}/{} nós responderam", respostas.len(), nos.len());
        Ok(respostas)
    }
    
    async fn enviar_para_no_individual(
        mensagem: Mensagem,
        no: No,
        timeout_duracao: Duration
    ) -> Result<RespostaMensagem> {
        // Simulação de envio de mensagem via rede
        // Em uma implementação real, isso seria uma chamada HTTP/TCP
        
        let resultado = timeout(timeout_duracao, async {
            // Simular latência de rede
            tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % 100)).await;
            
            // Simular falha ocasional (5% de chance)
            if rand::random::<f32>() < 0.05 {
                return Err(anyhow!("Falha simulada na rede"));
            }
            
            Ok(RespostaMensagem {
                id_mensagem: mensagem.id.clone(),
                sucesso: true,
                erro: None,
                timestamp: chrono::Utc::now(),
            })
        }).await;
        
        match resultado {
            Ok(Ok(resposta)) => {
                info!("Mensagem entregue com sucesso para nó {}", no.id);
                Ok(resposta)
            }
            Ok(Err(e)) => {
                warn!("Falha ao enviar para nó {}: {}", no.id, e);
                Err(e)
            }
            Err(_) => {
                warn!("Timeout ao enviar para nó {}", no.id);
                Err(anyhow!("Timeout na comunicação com nó {}", no.id))
            }
        }
    }
    
    pub async fn broadcast_com_fanout(
        &self,
        mensagem: &Mensagem,
        nos: &[No],
        fator_fanout: usize
    ) -> Result<Vec<RespostaMensagem>> {
        info!("Iniciando broadcast com fanout {} para {} nós", fator_fanout, nos.len());
        
        let mut respostas = Vec::new();
        let chunks: Vec<_> = nos.chunks(fator_fanout).collect();
        
        for (i, chunk) in chunks.iter().enumerate() {
            info!("Processando lote {} de {}", i + 1, chunks.len());
            
            let mut chunk_respostas = self.enviar_para_nos(mensagem, chunk).await?;
            respostas.append(&mut chunk_respostas);
            
            // Pequena pausa entre lotes para evitar sobrecarga
            if i < chunks.len() - 1 {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
        
        Ok(respostas)
    }
    
    async fn atualizar_estatisticas(&self, total_nos: usize, sucessos: usize, duracao: Duration) {
        let mut stats = self.estatisticas.write().await;
        stats.mensagens_enviadas += total_nos as u64;
        stats.mensagens_entregues += sucessos as u64;
        stats.mensagens_falharam += (total_nos - sucessos) as u64;
        
        // Atualizar tempo médio (média móvel simples)
        if stats.mensagens_enviadas == total_nos as u64 {
            stats.tempo_medio_entrega = duracao;
        } else {
            let peso_novo = 0.1;
            let duracao_ms = duracao.as_millis() as f64;
            let media_atual_ms = stats.tempo_medio_entrega.as_millis() as f64;
            let nova_media_ms = (1.0 - peso_novo) * media_atual_ms + peso_novo * duracao_ms;
            stats.tempo_medio_entrega = Duration::from_millis(nova_media_ms as u64);
        }
    }
    
    pub async fn obter_estatisticas(&self) -> EstatisticasBroadcast {
        self.estatisticas.read().await.clone()
    }
    
    pub fn configurar_timeout(&mut self, timeout: Duration) {
        self.timeout_padrao = timeout;
    }
    
    pub fn configurar_max_tentativas(&mut self, max_tentativas: u32) {
        self.max_tentativas = max_tentativas;
    }
}

#[derive(Debug, Clone)]
pub struct EstatisticasBroadcast {
    pub mensagens_enviadas: u64,
    pub mensagens_entregues: u64,
    pub mensagens_falharam: u64,
    pub tempo_medio_entrega: Duration,
}

impl EstatisticasBroadcast {
    pub fn taxa_sucesso(&self) -> f64 {
        if self.mensagens_enviadas == 0 {
            0.0
        } else {
            self.mensagens_entregues as f64 / self.mensagens_enviadas as f64
        }
    }
    
    pub fn taxa_falha(&self) -> f64 {
        if self.mensagens_enviadas == 0 {
            0.0
        } else {
            self.mensagens_falharam as f64 / self.mensagens_enviadas as f64
        }
    }
}