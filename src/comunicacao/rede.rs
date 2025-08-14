use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant, interval};
use tracing::{info, warn, error, debug};
use crate::comunicacao::broadcast::No;

#[derive(Debug, Clone)]
pub struct ConfiguracaoRede {
    pub timeout_heartbeat: Duration,
    pub intervalo_descoberta: Duration,
    pub max_nos_inativos: usize,
    pub timeout_conexao: Duration,
}

impl Default for ConfiguracaoRede {
    fn default() -> Self {
        Self {
            timeout_heartbeat: Duration::from_secs(30),
            intervalo_descoberta: Duration::from_secs(60),
            max_nos_inativos: 10,
            timeout_conexao: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusNo {
    pub no: No,
    pub ultima_atividade: Instant,
    pub tentativas_conexao: u32,
    pub latencia_media: Duration,
    pub historico_disponibilidade: Vec<bool>, // Últimas 100 verificações
}

pub struct GerenciadorRede {
    configuracao: ConfiguracaoRede,
    nos_conhecidos: Arc<RwLock<HashMap<String, StatusNo>>>,
    nos_seeds: Arc<RwLock<Vec<String>>>, // Nós iniciais para bootstrap
    estatisticas: Arc<RwLock<EstatisticasRede>>,
}

#[derive(Debug, Default, Clone)]
pub struct EstatisticasRede {
    pub total_nos_conhecidos: usize,
    pub nos_ativos: usize,
    pub nos_inativos: usize,
    pub latencia_media_rede: Duration,
    pub disponibilidade_media: f64,
    pub ultima_descoberta: Option<chrono::DateTime<chrono::Utc>>,
}

impl GerenciadorRede {
    pub fn new() -> Self {
        Self::new_com_configuracao(ConfiguracaoRede::default())
    }
    
    pub fn new_com_configuracao(configuracao: ConfiguracaoRede) -> Self {
        let gerenciador = Self {
            configuracao: configuracao.clone(),
            nos_conhecidos: Arc::new(RwLock::new(HashMap::new())),
            nos_seeds: Arc::new(RwLock::new(Vec::new())),
            estatisticas: Arc::new(RwLock::new(EstatisticasRede::default())),
        };
        
        // Iniciar tarefas de monitoramento
        gerenciador.iniciar_monitoramento();
        
        gerenciador
    }
    
    fn iniciar_monitoramento(&self) {
        // Task de heartbeat
        let nos_clone = self.nos_conhecidos.clone();
        let config_clone = self.configuracao.clone();
        let stats_clone = self.estatisticas.clone();
        
        tokio::spawn(async move {
            Self::task_heartbeat(nos_clone, config_clone, stats_clone).await;
        });
        
        // Task de descoberta de nós
        let nos_clone = self.nos_conhecidos.clone();
        let seeds_clone = self.nos_seeds.clone();
        let config_clone = self.configuracao.clone();
        
        tokio::spawn(async move {
            Self::task_descoberta_nos(nos_clone, seeds_clone, config_clone).await;
        });
        
        // Task de limpeza de nós inativos
        let nos_clone = self.nos_conhecidos.clone();
        let config_clone = self.configuracao.clone();
        
        tokio::spawn(async move {
            Self::task_limpeza_nos(nos_clone, config_clone).await;
        });
    }
    
    pub async fn adicionar_no(&self, no: No) -> Result<()> {
        info!("Adicionando nó {} à rede", no.id);
        
        let status = StatusNo {
            no: no.clone(),
            ultima_atividade: Instant::now(),
            tentativas_conexao: 0,
            latencia_media: Duration::from_millis(0),
            historico_disponibilidade: Vec::new(),
        };
        
        {
            let mut nos = self.nos_conhecidos.write().await;
            nos.insert(no.id.clone(), status);
        }
        
        // Testar conectividade inicial
        self.testar_conectividade_no(&no.id).await?;
        
        self.atualizar_estatisticas().await;
        Ok(())
    }
    
    pub async fn remover_no(&self, id_no: &str) -> Result<()> {
        info!("Removendo nó {} da rede", id_no);
        
        {
            let mut nos = self.nos_conhecidos.write().await;
            nos.remove(id_no);
        }
        
        self.atualizar_estatisticas().await;
        Ok(())
    }
    
    pub async fn obter_nos_ativos(&self) -> Vec<No> {
        let nos = self.nos_conhecidos.read().await;
        let agora = Instant::now();
        
        nos.values()
            .filter(|status| {
                status.no.ativo && 
                agora.duration_since(status.ultima_atividade) < self.configuracao.timeout_heartbeat
            })
            .map(|status| status.no.clone())
            .collect()
    }
    
    pub async fn obter_nos_por_latencia(&self, limite: Option<usize>) -> Vec<No> {
        let nos = self.nos_conhecidos.read().await;
        let mut nos_ordenados: Vec<_> = nos.values()
            .filter(|status| status.no.ativo)
            .collect();
        
        // Ordenar por latência (menor primeiro)
        nos_ordenados.sort_by_key(|status| status.latencia_media);
        
        let resultado: Vec<No> = nos_ordenados
            .into_iter()
            .take(limite.unwrap_or(usize::MAX))
            .map(|status| status.no.clone())
            .collect();
        
        debug!("Retornando {} nós ordenados por latência", resultado.len());
        resultado
    }
    
    pub async fn descobrir_nos_via_seed(&self, endereco_seed: &str) -> Result<Vec<No>> {
        info!("Descobrindo nós via seed: {}", endereco_seed);
        
        // Simulação de descoberta de nós (em produção seria uma chamada real)
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Simular descoberta de 3-7 nós
        let num_nos = 3 + rand::random::<usize>() % 5;
        let mut nos_descobertos = Vec::new();
        
        for i in 0..num_nos {
            let no = No {
                id: format!("no_descoberto_{}_{}", endereco_seed.replace(".", "_"), i),
                endereco: format!("192.168.1.{}", 100 + i),
                porta: 8080 + i as u16,
                ativo: rand::random::<f32>() > 0.1, // 90% de chance de estar ativo
                ultima_resposta: chrono::Utc::now(),
            };
            nos_descobertos.push(no);
        }
        
        // Adicionar nós descobertos
        for no in &nos_descobertos {
            if let Err(e) = self.adicionar_no(no.clone()).await {
                warn!("Erro ao adicionar nó descoberto {}: {}", no.id, e);
            }
        }
        
        info!("Descobertos {} nós via seed {}", nos_descobertos.len(), endereco_seed);
        Ok(nos_descobertos)
    }
    
    async fn testar_conectividade_no(&self, id_no: &str) -> Result<Duration> {
        let inicio = Instant::now();
        
        // Simulação de teste de conectividade
        tokio::time::sleep(Duration::from_millis(10 + rand::random::<u64>() % 50)).await;
        
        let latencia = inicio.elapsed();
        
        // Simular falha ocasional (10% de chance)
        if rand::random::<f32>() < 0.1 {
            return Err(anyhow!("Falha no teste de conectividade"));
        }
        
        // Atualizar status do nó
        {
            let mut nos = self.nos_conhecidos.write().await;
            if let Some(status) = nos.get_mut(id_no) {
                status.ultima_atividade = Instant::now();
                status.tentativas_conexao = 0;
                
                // Atualizar latência média
                if status.latencia_media.is_zero() {
                    status.latencia_media = latencia;
                } else {
                    let peso = 0.2; // 20% peso para nova medição
                    let latencia_atual_ms = status.latencia_media.as_millis() as f64;
                    let nova_latencia_ms = latencia.as_millis() as f64;
                    let media_ms = (1.0 - peso) * latencia_atual_ms + peso * nova_latencia_ms;
                    status.latencia_media = Duration::from_millis(media_ms as u64);
                }
                
                // Atualizar histórico de disponibilidade
                status.historico_disponibilidade.push(true);
                if status.historico_disponibilidade.len() > 100 {
                    status.historico_disponibilidade.remove(0);
                }
                
                status.no.ativo = true;
                status.no.ultima_resposta = chrono::Utc::now();
            }
        }
        
        debug!("Conectividade testada para {}: {:?}", id_no, latencia);
        Ok(latencia)
    }
    
    async fn task_heartbeat(
        nos: Arc<RwLock<HashMap<String, StatusNo>>>,
        configuracao: ConfiguracaoRede,
        estatisticas: Arc<RwLock<EstatisticasRede>>,
    ) {
        let mut intervalo = interval(configuracao.timeout_heartbeat / 3); // Verificar 3x mais frequente
        
        loop {
            intervalo.tick().await;
            
            let agora = Instant::now();
            let mut nos_para_testar = Vec::new();
            
            // Identificar nós que precisam de heartbeat
            {
                let nos_read = nos.read().await;
                for (id, status) in nos_read.iter() {
                    let tempo_desde_atividade = agora.duration_since(status.ultima_atividade);
                    if tempo_desde_atividade > configuracao.timeout_heartbeat / 2 {
                        nos_para_testar.push(id.clone());
                    }
                }
            }
            
            // Testar conectividade dos nós
            for id_no in nos_para_testar {
                match Self::testar_conectividade_individual(&nos, &id_no).await {
                    Ok(latencia) => {
                        debug!("Heartbeat OK para {}: {:?}", id_no, latencia);
                    }
                    Err(e) => {
                        warn!("Heartbeat falhou para {}: {}", id_no, e);
                        Self::marcar_no_inativo(&nos, &id_no).await;
                    }
                }
            }
            
            // Atualizar estatísticas
            Self::atualizar_estatisticas_task(&nos, &estatisticas).await;
        }
    }
    
    async fn task_descoberta_nos(
        nos: Arc<RwLock<HashMap<String, StatusNo>>>,
        seeds: Arc<RwLock<Vec<String>>>,
        configuracao: ConfiguracaoRede,
    ) {
        let mut intervalo = interval(configuracao.intervalo_descoberta);
        
        loop {
            intervalo.tick().await;
            
            info!("Iniciando descoberta periódica de nós");
            
            // Obter lista de seeds
            let seeds_list = {
                let seeds_read = seeds.read().await;
                seeds_read.clone()
            };
            
            // Descobrir via cada seed
            for seed in seeds_list {
                match Self::descobrir_nos_simulado(&seed).await {
                    Ok(novos_nos) => {
                        for no in novos_nos {
                            Self::adicionar_no_descoberto(&nos, no).await;
                        }
                    }
                    Err(e) => {
                        warn!("Falha na descoberta via seed {}: {}", seed, e);
                    }
                }
            }
            
            // Descobrir via nós conhecidos (gossip)
            Self::descobrir_via_gossip(&nos).await;
        }
    }
    
    async fn task_limpeza_nos(
        nos: Arc<RwLock<HashMap<String, StatusNo>>>,
        configuracao: ConfiguracaoRede,
    ) {
        let mut intervalo = interval(Duration::from_secs(300)); // A cada 5 minutos
        
        loop {
            intervalo.tick().await;
            
            let agora = Instant::now();
            let mut nos_para_remover = Vec::new();
            
            {
                let nos_read = nos.read().await;
                for (id, status) in nos_read.iter() {
                    let tempo_inativo = agora.duration_since(status.ultima_atividade);
                    
                    // Remover nós inativos há muito tempo
                    if !status.no.ativo && tempo_inativo > Duration::from_secs(3600) { // 1 hora
                        nos_para_remover.push(id.clone());
                    }
                    
                    // Remover nós com muitas falhas consecutivas
                    if status.tentativas_conexao > 10 {
                        nos_para_remover.push(id.clone());
                    }
                }
            }
            
            // Remover nós identificados
            if !nos_para_remover.is_empty() {
                let mut nos_write = nos.write().await;
                for id in &nos_para_remover {
                    nos_write.remove(id);
                    info!("Nó {} removido por inatividade prolongada", id);
                }
            }
        }
    }
    
    async fn testar_conectividade_individual(
        nos: &Arc<RwLock<HashMap<String, StatusNo>>>,
        id_no: &str,
    ) -> Result<Duration> {
        let inicio = Instant::now();
        
        // Simulação de teste
        tokio::time::sleep(Duration::from_millis(5 + rand::random::<u64>() % 20)).await;
        
        let latencia = inicio.elapsed();
        
        // Simular falha ocasional
        if rand::random::<f32>() < 0.05 {
            return Err(anyhow!("Falha no heartbeat"));
        }
        
        // Atualizar status
        {
            let mut nos_write = nos.write().await;
            if let Some(status) = nos_write.get_mut(id_no) {
                status.ultima_atividade = Instant::now();
                status.tentativas_conexao = 0;
            }
        }
        
        Ok(latencia)
    }
    
    async fn marcar_no_inativo(
        nos: &Arc<RwLock<HashMap<String, StatusNo>>>,
        id_no: &str,
    ) {
        let mut nos_write = nos.write().await;
        if let Some(status) = nos_write.get_mut(id_no) {
            status.no.ativo = false;
            status.tentativas_conexao += 1;
            status.historico_disponibilidade.push(false);
            
            if status.historico_disponibilidade.len() > 100 {
                status.historico_disponibilidade.remove(0);
            }
        }
    }
    
    async fn descobrir_nos_simulado(seed: &str) -> Result<Vec<No>> {
        // Simulação de descoberta
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        if rand::random::<f32>() < 0.1 {
            return Err(anyhow!("Falha na descoberta via seed"));
        }
        
        let num_nos = rand::random::<usize>() % 3; // 0-2 novos nós
        let mut nos = Vec::new();
        
        for i in 0..num_nos {
            let no = No {
                id: format!("descoberto_{}_{}", seed.replace(".", "_"), i),
                endereco: format!("10.0.{}.{}", rand::random::<u8>(), rand::random::<u8>()),
                porta: 8080,
                ativo: true,
                ultima_resposta: chrono::Utc::now(),
            };
            nos.push(no);
        }
        
        Ok(nos)
    }
    
    async fn adicionar_no_descoberto(
        nos: &Arc<RwLock<HashMap<String, StatusNo>>>,
        no: No,
    ) {
        let mut nos_write = nos.write().await;
        
        if !nos_write.contains_key(&no.id) {
            let status = StatusNo {
                no: no.clone(),
                ultima_atividade: Instant::now(),
                tentativas_conexao: 0,
                latencia_media: Duration::from_millis(0),
                historico_disponibilidade: Vec::new(),
            };
            
            nos_write.insert(no.id.clone(), status);
            info!("Novo nó descoberto e adicionado: {}", no.id);
        }
    }
    
    async fn descobrir_via_gossip(nos: &Arc<RwLock<HashMap<String, StatusNo>>>) {
        // Simulação de descoberta via gossip protocol
        let nos_ativos: Vec<String> = {
            let nos_read = nos.read().await;
            nos_read.iter()
                .filter(|(_, status)| status.no.ativo)
                .map(|(id, _)| id.clone())
                .collect()
        };
        
        if nos_ativos.len() < 2 {
            return; // Precisa de pelo menos 2 nós para gossip
        }
        
        // Simular troca de informações entre nós
        for _ in 0..3 { // Até 3 novos nós via gossip
            if rand::random::<f32>() < 0.3 {
                let no = No {
                    id: format!("gossip_{}", rand::random::<u32>()),
                    endereco: format!("172.16.{}.{}", rand::random::<u8>(), rand::random::<u8>()),
                    porta: 8080,
                    ativo: true,
                    ultima_resposta: chrono::Utc::now(),
                };
                
                Self::adicionar_no_descoberto(nos, no).await;
            }
        }
    }
    
    async fn atualizar_estatisticas_task(
        nos: &Arc<RwLock<HashMap<String, StatusNo>>>,
        estatisticas: &Arc<RwLock<EstatisticasRede>>,
    ) {
        let (total, ativos, latencia_total, disponibilidade_total) = {
            let nos_read = nos.read().await;
            let total = nos_read.len();
            let mut ativos = 0;
            let mut latencia_total = Duration::from_millis(0);
            let mut disponibilidade_total = 0.0;
            
            for status in nos_read.values() {
                if status.no.ativo {
                    ativos += 1;
                    latencia_total += status.latencia_media;
                }
                
                if !status.historico_disponibilidade.is_empty() {
                    let disponibilidade = status.historico_disponibilidade.iter()
                        .map(|&ativo| if ativo { 1.0 } else { 0.0 })
                        .sum::<f64>() / status.historico_disponibilidade.len() as f64;
                    disponibilidade_total += disponibilidade;
                }
            }
            
            (total, ativos, latencia_total, disponibilidade_total)
        };
        
        let mut stats = estatisticas.write().await;
        stats.total_nos_conhecidos = total;
        stats.nos_ativos = ativos;
        stats.nos_inativos = total - ativos;
        
        if ativos > 0 {
            stats.latencia_media_rede = latencia_total / ativos as u32;
        }
        
        if total > 0 {
            stats.disponibilidade_media = disponibilidade_total / total as f64;
        }
    }
    
    async fn atualizar_estatisticas(&self) {
        Self::atualizar_estatisticas_task(&self.nos_conhecidos, &self.estatisticas).await;
    }
    
    pub async fn adicionar_seed(&self, endereco: String) {
        let mut seeds = self.nos_seeds.write().await;
        if !seeds.contains(&endereco) {
            seeds.push(endereco.clone());
            info!("Seed adicionado: {}", endereco);
        }
    }
    
    pub async fn obter_estatisticas(&self) -> EstatisticasRede {
        self.estatisticas.read().await.clone()
    }
    
    pub async fn obter_status_detalhado(&self) -> HashMap<String, StatusNo> {
        self.nos_conhecidos.read().await.clone()
    }
    
    pub async fn forcar_descoberta(&self) -> Result<usize> {
        info!("Forçando descoberta de nós");
        
        let seeds = {
            let seeds_read = self.nos_seeds.read().await;
            seeds_read.clone()
        };
        
        let mut total_descobertos = 0;
        
        for seed in seeds {
            match self.descobrir_nos_via_seed(&seed).await {
                Ok(nos) => total_descobertos += nos.len(),
                Err(e) => warn!("Falha na descoberta forçada via {}: {}", seed, e),
            }
        }
        
        self.atualizar_estatisticas().await;
        Ok(total_descobertos)
    }
}

impl EstatisticasRede {
    pub fn taxa_disponibilidade(&self) -> f64 {
        if self.total_nos_conhecidos == 0 {
            0.0
        } else {
            self.nos_ativos as f64 / self.total_nos_conhecidos as f64
        }
    }
    
    pub fn saude_rede(&self) -> String {
        let taxa = self.taxa_disponibilidade();
        match taxa {
            t if t >= 0.9 => "Excelente".to_string(),
            t if t >= 0.7 => "Boa".to_string(),
            t if t >= 0.5 => "Regular".to_string(),
            _ => "Crítica".to_string(),
        }
    }
}