use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep, Instant};
use tracing::{info, warn, error, debug};
use crate::comunicacao::protocolo::{Mensagem, RespostaMensagem};
use crate::comunicacao::broadcast::No;

#[derive(Debug, Clone)]
pub struct ConfiguracaoRetry {
    pub max_tentativas: u32,
    pub delay_inicial: Duration,
    pub multiplicador_backoff: f64,
    pub delay_maximo: Duration,
    pub jitter: bool,
}

impl Default for ConfiguracaoRetry {
    fn default() -> Self {
        Self {
            max_tentativas: 3,
            delay_inicial: Duration::from_millis(100),
            multiplicador_backoff: 2.0,
            delay_maximo: Duration::from_secs(30),
            jitter: true,
        }
    }
}

#[derive(Debug, Clone)]
struct TentativaRetry {
    mensagem: Mensagem,
    no_destino: No,
    tentativa_atual: u32,
    proximo_retry: Instant,
    criado_em: Instant,
}

pub struct MecanismoRetry {
    configuracao: ConfiguracaoRetry,
    filas_retry: Arc<RwLock<HashMap<String, TentativaRetry>>>,
    estatisticas: Arc<RwLock<EstatisticasRetry>>,
}

#[derive(Debug, Default, Clone)]
pub struct EstatisticasRetry {
    pub tentativas_totais: u64,
    pub sucessos_primeiro_try: u64,
    pub sucessos_retry: u64,
    pub falhas_definitivas: u64,
    pub tempo_medio_resolucao: Duration,
}

impl MecanismoRetry {
    pub fn new() -> Self {
        Self::new_com_configuracao(ConfiguracaoRetry::default())
    }
    
    pub fn new_com_configuracao(configuracao: ConfiguracaoRetry) -> Self {
        let mecanismo = Self {
            configuracao,
            filas_retry: Arc::new(RwLock::new(HashMap::new())),
            estatisticas: Arc::new(RwLock::new(EstatisticasRetry::default())),
        };
        
        // Iniciar task de processamento de retries
        let filas_clone = mecanismo.filas_retry.clone();
        let stats_clone = mecanismo.estatisticas.clone();
        let config_clone = mecanismo.configuracao.clone();
        
        tokio::spawn(async move {
            Self::processar_filas_retry(filas_clone, stats_clone, config_clone).await;
        });
        
        mecanismo
    }
    
    pub async fn enviar_com_retry(
        &self,
        mensagem: Mensagem,
        no_destino: No
    ) -> Result<RespostaMensagem> {
        let inicio = Instant::now();
        let id_tentativa = format!("{}_{}", mensagem.id, no_destino.id);
        
        // Primeira tentativa
        match self.tentar_envio(&mensagem, &no_destino).await {
            Ok(resposta) => {
                self.registrar_sucesso_primeiro_try(inicio.elapsed()).await;
                info!("Mensagem {} entregue na primeira tentativa para {}", mensagem.id, no_destino.id);
                return Ok(resposta);
            }
            Err(e) => {
                warn!("Primeira tentativa falhou para {}: {}. Agendando retry.", no_destino.id, e);
            }
        }
        
        // Agendar retry
        let tentativa = TentativaRetry {
            mensagem: mensagem.clone(),
            no_destino: no_destino.clone(),
            tentativa_atual: 1,
            proximo_retry: Instant::now() + self.calcular_delay(1),
            criado_em: inicio,
        };
        
        {
            let mut filas = self.filas_retry.write().await;
            filas.insert(id_tentativa.clone(), tentativa);
        }
        
        // Aguardar resultado do retry
        self.aguardar_resultado_retry(id_tentativa, inicio).await
    }
    
    async fn tentar_envio(&self, mensagem: &Mensagem, no: &No) -> Result<RespostaMensagem> {
        // Simulação de envio (em produção seria HTTP/TCP real)
        
        // Simular latência variável
        let latencia = Duration::from_millis(50 + rand::random::<u64>() % 200);
        sleep(latencia).await;
        
        // Simular falhas baseadas no estado do nó
        if !no.ativo {
            return Err(anyhow!("Nó {} está inativo", no.id));
        }
        
        // Simular falhas temporárias (20% de chance)
        if rand::random::<f32>() < 0.2 {
            return Err(anyhow!("Falha temporária de rede"));
        }
        
        // Simular timeout ocasional (5% de chance)
        if rand::random::<f32>() < 0.05 {
            sleep(Duration::from_secs(35)).await; // Maior que timeout padrão
            return Err(anyhow!("Timeout na comunicação"));
        }
        
        Ok(RespostaMensagem {
            id_mensagem: mensagem.id.clone(),
            sucesso: true,
            erro: None,
            timestamp: chrono::Utc::now(),
        })
    }
    
    fn calcular_delay(&self, tentativa: u32) -> Duration {
        let mut delay = self.configuracao.delay_inicial.as_millis() as f64
            * self.configuracao.multiplicador_backoff.powi(tentativa as i32 - 1);
        
        // Aplicar jitter se configurado
        if self.configuracao.jitter {
            let jitter_factor = 0.1; // ±10%
            let jitter = 1.0 + (rand::random::<f64>() - 0.5) * 2.0 * jitter_factor;
            delay *= jitter;
        }
        
        // Limitar ao delay máximo
        let delay_ms = delay.min(self.configuracao.delay_maximo.as_millis() as f64) as u64;
        Duration::from_millis(delay_ms)
    }
    
    async fn processar_filas_retry(
        filas: Arc<RwLock<HashMap<String, TentativaRetry>>>,
        estatisticas: Arc<RwLock<EstatisticasRetry>>,
        configuracao: ConfiguracaoRetry,
    ) {
        let mut intervalo = tokio::time::interval(Duration::from_millis(100));
        
        loop {
            intervalo.tick().await;
            
            let agora = Instant::now();
            let mut tentativas_para_processar = Vec::new();
            
            // Coletar tentativas prontas para retry
            {
                let filas_read = filas.read().await;
                for (id, tentativa) in filas_read.iter() {
                    if agora >= tentativa.proximo_retry {
                        tentativas_para_processar.push((id.clone(), tentativa.clone()));
                    }
                }
            }
            
            // Processar tentativas
            for (id, mut tentativa) in tentativas_para_processar {
                debug!("Processando retry {} (tentativa {})", id, tentativa.tentativa_atual + 1);
                
                let resultado = Self::executar_retry_individual(&tentativa).await;
                
                match resultado {
                    Ok(_) => {
                        // Sucesso - remover da fila
                        {
                            let mut filas_write = filas.write().await;
                            filas_write.remove(&id);
                        }
                        
                        let duracao = agora.duration_since(tentativa.criado_em);
                        Self::registrar_sucesso_retry(&estatisticas, duracao).await;
                        
                        info!("Retry bem-sucedido para {} após {} tentativas", 
                              tentativa.no_destino.id, tentativa.tentativa_atual + 1);
                    }
                    Err(e) => {
                        tentativa.tentativa_atual += 1;
                        
                        if tentativa.tentativa_atual >= configuracao.max_tentativas {
                            // Falha definitiva
                            {
                                let mut filas_write = filas.write().await;
                                filas_write.remove(&id);
                            }
                            
                            Self::registrar_falha_definitiva(&estatisticas).await;
                            
                            error!("Falha definitiva para {} após {} tentativas: {}", 
                                   tentativa.no_destino.id, tentativa.tentativa_atual, e);
                        } else {
                            // Agendar próximo retry
                            tentativa.proximo_retry = agora + Self::calcular_delay_estatico(
                                &configuracao, tentativa.tentativa_atual
                            );
                            
                            {
                                let mut filas_write = filas.write().await;
                                filas_write.insert(id, tentativa);
                            }
                            
                            debug!("Agendando próximo retry em {:?}", 
                                   Self::calcular_delay_estatico(&configuracao, tentativa.tentativa_atual));
                        }
                    }
                }
            }
        }
    }
    
    async fn executar_retry_individual(tentativa: &TentativaRetry) -> Result<RespostaMensagem> {
        // Simular retry (mesma lógica do envio original)
        sleep(Duration::from_millis(50 + rand::random::<u64>() % 100)).await;
        
        if !tentativa.no_destino.ativo {
            return Err(anyhow!("Nó ainda inativo"));
        }
        
        // Chance de sucesso aumenta com tentativas (simular recuperação)
        let chance_sucesso = 0.6 + (tentativa.tentativa_atual as f32 * 0.15);
        if rand::random::<f32>() < chance_sucesso {
            Ok(RespostaMensagem {
                id_mensagem: tentativa.mensagem.id.clone(),
                sucesso: true,
                erro: None,
                timestamp: chrono::Utc::now(),
            })
        } else {
            Err(anyhow!("Falha no retry"))
        }
    }
    
    fn calcular_delay_estatico(config: &ConfiguracaoRetry, tentativa: u32) -> Duration {
        let mut delay = config.delay_inicial.as_millis() as f64
            * config.multiplicador_backoff.powi(tentativa as i32);
        
        if config.jitter {
            let jitter_factor = 0.1;
            let jitter = 1.0 + (rand::random::<f64>() - 0.5) * 2.0 * jitter_factor;
            delay *= jitter;
        }
        
        let delay_ms = delay.min(config.delay_maximo.as_millis() as f64) as u64;
        Duration::from_millis(delay_ms)
    }
    
    async fn aguardar_resultado_retry(
        &self,
        id_tentativa: String,
        inicio: Instant,
    ) -> Result<RespostaMensagem> {
        let timeout_total = Duration::from_secs(300); // 5 minutos
        let timeout_fim = inicio + timeout_total;
        
        while Instant::now() < timeout_fim {
            {
                let filas = self.filas_retry.read().await;
                if !filas.contains_key(&id_tentativa) {
                    // Tentativa foi removida (sucesso ou falha definitiva)
                    // Verificar estatísticas para determinar resultado
                    return Ok(RespostaMensagem {
                        id_mensagem: id_tentativa,
                        sucesso: true,
                        erro: None,
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
            
            sleep(Duration::from_millis(100)).await;
        }
        
        Err(anyhow!("Timeout aguardando resultado do retry"))
    }
    
    async fn registrar_sucesso_primeiro_try(&self, duracao: Duration) {
        let mut stats = self.estatisticas.write().await;
        stats.tentativas_totais += 1;
        stats.sucessos_primeiro_try += 1;
        Self::atualizar_tempo_medio(&mut stats, duracao);
    }
    
    async fn registrar_sucesso_retry(stats: &Arc<RwLock<EstatisticasRetry>>, duracao: Duration) {
        let mut stats = stats.write().await;
        stats.sucessos_retry += 1;
        Self::atualizar_tempo_medio(&mut stats, duracao);
    }
    
    async fn registrar_falha_definitiva(stats: &Arc<RwLock<EstatisticasRetry>>) {
        let mut stats = stats.write().await;
        stats.falhas_definitivas += 1;
    }
    
    fn atualizar_tempo_medio(stats: &mut EstatisticasRetry, nova_duracao: Duration) {
        let total_sucessos = stats.sucessos_primeiro_try + stats.sucessos_retry;
        if total_sucessos == 1 {
            stats.tempo_medio_resolucao = nova_duracao;
        } else {
            let peso = 1.0 / total_sucessos as f64;
            let media_atual_ms = stats.tempo_medio_resolucao.as_millis() as f64;
            let nova_duracao_ms = nova_duracao.as_millis() as f64;
            let nova_media_ms = (1.0 - peso) * media_atual_ms + peso * nova_duracao_ms;
            stats.tempo_medio_resolucao = Duration::from_millis(nova_media_ms as u64);
        }
    }
    
    pub async fn obter_estatisticas(&self) -> EstatisticasRetry {
        self.estatisticas.read().await.clone()
    }
    
    pub async fn limpar_filas(&self) {
        let mut filas = self.filas_retry.write().await;
        filas.clear();
        info!("Filas de retry limpas");
    }
    
    pub async fn obter_status_filas(&self) -> HashMap<String, u32> {
        let filas = self.filas_retry.read().await;
        filas.iter()
            .map(|(id, tentativa)| (id.clone(), tentativa.tentativa_atual))
            .collect()
    }
}

impl EstatisticasRetry {
    pub fn taxa_sucesso_total(&self) -> f64 {
        let total = self.tentativas_totais;
        if total == 0 {
            0.0
        } else {
            (self.sucessos_primeiro_try + self.sucessos_retry) as f64 / total as f64
        }
    }
    
    pub fn taxa_sucesso_primeiro_try(&self) -> f64 {
        if self.tentativas_totais == 0 {
            0.0
        } else {
            self.sucessos_primeiro_try as f64 / self.tentativas_totais as f64
        }
    }
    
    pub fn taxa_recuperacao_retry(&self) -> f64 {
        let falhas_primeiro_try = self.tentativas_totais - self.sucessos_primeiro_try;
        if falhas_primeiro_try == 0 {
            0.0
        } else {
            self.sucessos_retry as f64 / falhas_primeiro_try as f64
        }
    }
}