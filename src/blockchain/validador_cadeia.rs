use super::{bloco::Bloco, cadeia::CadeiaBlockchain};
use crate::consenso::tipos::Transacao;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone)]
pub struct ResultadoValidacao {
    pub valido: bool,
    pub erros: Vec<String>,
    pub avisos: Vec<String>,
    pub tempo_validacao: Duration,
}

#[derive(Debug, Clone)]
pub struct EstatisticasValidacao {
    pub total_validacoes: u64,
    pub validacoes_bem_sucedidas: u64,
    pub validacoes_falharam: u64,
    pub tempo_medio_validacao: Duration,
    pub ultimo_bloco_validado: Option<u64>,
}

pub struct ValidadorCadeia {
    estatisticas: RwLock<EstatisticasValidacao>,
    cache_validacao: RwLock<HashMap<Vec<u8>, ResultadoValidacao>>,
    transacoes_conhecidas: RwLock<HashSet<String>>,
    configuracao: ConfiguracaoValidador,
}

#[derive(Debug, Clone)]
pub struct ConfiguracaoValidador {
    pub validar_assinaturas: bool,
    pub validar_merkle_trees: bool,
    pub validar_dificuldade: bool,
    pub validar_transacoes_duplicadas: bool,
    pub timeout_validacao: Duration,
    pub tamanho_cache: usize,
}

impl Default for ConfiguracaoValidador {
    fn default() -> Self {
        Self {
            validar_assinaturas: true,
            validar_merkle_trees: true,
            validar_dificuldade: true,
            validar_transacoes_duplicadas: true,
            timeout_validacao: Duration::seconds(30),
            tamanho_cache: 1000,
        }
    }
}

impl ValidadorCadeia {
    pub fn new() -> Self {
        Self {
            estatisticas: RwLock::new(EstatisticasValidacao {
                total_validacoes: 0,
                validacoes_bem_sucedidas: 0,
                validacoes_falharam: 0,
                tempo_medio_validacao: Duration::zero(),
                ultimo_bloco_validado: None,
            }),
            cache_validacao: RwLock::new(HashMap::new()),
            transacoes_conhecidas: RwLock::new(HashSet::new()),
            configuracao: ConfiguracaoValidador::default(),
        }
    }
    
    pub fn new_com_configuracao(config: ConfiguracaoValidador) -> Self {
        Self {
            estatisticas: RwLock::new(EstatisticasValidacao {
                total_validacoes: 0,
                validacoes_bem_sucedidas: 0,
                validacoes_falharam: 0,
                tempo_medio_validacao: Duration::zero(),
                ultimo_bloco_validado: None,
            }),
            cache_validacao: RwLock::new(HashMap::new()),
            transacoes_conhecidas: RwLock::new(HashSet::new()),
            configuracao: config,
        }
    }
    
    pub async fn validar_bloco(&self, bloco: &Bloco, cadeia: &CadeiaBlockchain) -> Result<bool> {
        let inicio = Utc::now();
        
        // Verificar cache primeiro
        if let Some(resultado_cache) = self.obter_do_cache(&bloco.hash_bloco).await {
            info!("Resultado de validação obtido do cache");
            return Ok(resultado_cache.valido);
        }
        
        let mut resultado = ResultadoValidacao {
            valido: true,
            erros: Vec::new(),
            avisos: Vec::new(),
            tempo_validacao: Duration::zero(),
        };
        
        // Validações estruturais
        self.validar_estrutura_bloco(bloco, &mut resultado).await?;
        
        // Validações de cadeia
        self.validar_ligacao_cadeia(bloco, cadeia, &mut resultado).await?;
        
        // Validações de transações
        self.validar_transacoes_bloco(bloco, &mut resultado).await?;
        
        // Validações de consenso
        if self.configuracao.validar_dificuldade {
            self.validar_dificuldade_bloco(bloco, &mut resultado).await;
        }
        
        // Calcular tempo de validação
        resultado.tempo_validacao = Utc::now() - inicio;
        
        // Atualizar estatísticas
        self.atualizar_estatisticas(&resultado, bloco.numero).await;
        
        // Armazenar no cache
        self.armazenar_no_cache(bloco.hash_bloco.clone(), resultado.clone()).await;
        
        // Log do resultado
        if resultado.valido {
            info!("Bloco {} validado com sucesso em {:?}", bloco.numero, resultado.tempo_validacao);
        } else {
            error!("Bloco {} falhou na validação: {:?}", bloco.numero, resultado.erros);
        }
        
        Ok(resultado.valido)
    }
    
    async fn validar_estrutura_bloco(&self, bloco: &Bloco, resultado: &mut ResultadoValidacao) -> Result<()> {
        // Validar estrutura básica
        if !bloco.validar_estrutura()? {
            resultado.valido = false;
            resultado.erros.push("Estrutura do bloco inválida".to_string());
        }
        
        // Validar Merkle tree se configurado
        if self.configuracao.validar_merkle_trees {
            let merkle_calculado = Bloco::calcular_merkle_root(&bloco.transacoes)?;
            if merkle_calculado != bloco.merkle_root {
                resultado.valido = false;
                resultado.erros.push("Merkle root inválido".to_string());
            }
        }
        
        // Validar timestamp
        let agora = Utc::now();
        if bloco.timestamp > agora + Duration::minutes(10) {
            resultado.valido = false;
            resultado.erros.push("Timestamp do bloco muito no futuro".to_string());
        }
        
        // Validar assinatura se configurado
        if self.configuracao.validar_assinaturas {
            // Em implementação real, validaria assinatura criptográfica
            if bloco.assinatura_minerador.is_empty() {
                resultado.avisos.push("Assinatura do minerador vazia".to_string());
            }
        }
        
        Ok(())
    }
    
    async fn validar_ligacao_cadeia(&self, bloco: &Bloco, cadeia: &CadeiaBlockchain, resultado: &mut ResultadoValidacao) -> Result<()> {
        if bloco.numero == 0 {
            // Bloco genesis - validações especiais
            if !bloco.hash_anterior.iter().all(|&b| b == 0) {
                resultado.valido = false;
                resultado.erros.push("Bloco genesis deve ter hash anterior zero".to_string());
            }
            return Ok(());
        }
        
        // Obter bloco anterior
        if let Some(bloco_anterior) = cadeia.obter_bloco_por_numero(bloco.numero - 1).await {
            // Verificar ligação
            if bloco.hash_anterior != bloco_anterior.hash_bloco {
                resultado.valido = false;
                resultado.erros.push("Hash anterior não confere com bloco anterior".to_string());
            }
            
            // Verificar sequência de números
            if bloco.numero != bloco_anterior.numero + 1 {
                resultado.valido = false;
                resultado.erros.push("Número de bloco fora de sequência".to_string());
            }
            
            // Verificar timestamp
            if bloco.timestamp <= bloco_anterior.timestamp {
                resultado.avisos.push("Timestamp não é estritamente crescente".to_string());
            }
        } else {
            resultado.valido = false;
            resultado.erros.push("Bloco anterior não encontrado".to_string());
        }
        
        Ok(())
    }
    
    async fn validar_transacoes_bloco(&self, bloco: &Bloco, resultado: &mut ResultadoValidacao) -> Result<()> {
        let mut transacoes_no_bloco = HashSet::new();
        
        for transacao in &bloco.transacoes {
            // Verificar duplicatas no bloco
            if !transacoes_no_bloco.insert(&transacao.id) {
                resultado.valido = false;
                resultado.erros.push(format!("Transação duplicada no bloco: {}", transacao.id));
            }
            
            // Verificar se já foi processada antes (se configurado)
            if self.configuracao.validar_transacoes_duplicadas {
                let conhecidas = self.transacoes_conhecidas.read().await;
                if conhecidas.contains(&transacao.id) {
                    resultado.valido = false;
                    resultado.erros.push(format!("Transação já processada anteriormente: {}", transacao.id));
                }
            }
            
            // Validar estrutura da transação
            if let Err(e) = self.validar_transacao_individual(transacao).await {
                resultado.valido = false;
                resultado.erros.push(format!("Transação inválida {}: {}", transacao.id, e));
            }
        }
        
        // Atualizar conjunto de transações conhecidas
        if resultado.valido {
            let mut conhecidas = self.transacoes_conhecidas.write().await;
            for transacao in &bloco.transacoes {
                conhecidas.insert(transacao.id.clone());
            }
            
            // Limitar tamanho do conjunto
            if conhecidas.len() > 10000 {
                let ids_para_remover: Vec<_> = conhecidas.iter().take(1000).cloned().collect();
                for id in ids_para_remover {
                    conhecidas.remove(&id);
                }
            }
        }
        
        Ok(())
    }
    
    async fn validar_transacao_individual(&self, transacao: &Transacao) -> Result<()> {
        // Validações básicas
        if transacao.id.is_empty() {
            return Err(anyhow::anyhow!("ID da transação vazio"));
        }
        
        if transacao.dados.is_empty() {
            return Err(anyhow::anyhow!("Dados da transação vazios"));
        }
        
        if transacao.assinatura.is_empty() {
            return Err(anyhow::anyhow!("Assinatura da transação vazia"));
        }
        
        // Validar timestamp
        let agora = Utc::now();
        if transacao.timestamp > agora + Duration::minutes(5) {
            return Err(anyhow::anyhow!("Timestamp da transação muito no futuro"));
        }
        
        // Em implementação real, validaria assinatura criptográfica
        
        Ok(())
    }
    
    async fn validar_dificuldade_bloco(&self, bloco: &Bloco, resultado: &mut ResultadoValidacao) {
        if !bloco.validar_dificuldade() {
            resultado.valido = false;
            resultado.erros.push("Dificuldade do bloco não atendida".to_string());
        }
    }
    
    async fn obter_do_cache(&self, hash_bloco: &[u8]) -> Option<ResultadoValidacao> {
        let cache = self.cache_validacao.read().await;
        cache.get(hash_bloco).cloned()
    }
    
    async fn armazenar_no_cache(&self, hash_bloco: Vec<u8>, resultado: ResultadoValidacao) {
        let mut cache = self.cache_validacao.write().await;
        
        // Limitar tamanho do cache
        if cache.len() >= self.configuracao.tamanho_cache {
            // Remover entradas mais antigas (implementação simples)
            let chaves_para_remover: Vec<_> = cache.keys().take(100).cloned().collect();
            for chave in chaves_para_remover {
                cache.remove(&chave);
            }
        }
        
        cache.insert(hash_bloco, resultado);
    }
    
    async fn atualizar_estatisticas(&self, resultado: &ResultadoValidacao, numero_bloco: u64) {
        let mut stats = self.estatisticas.write().await;
        
        stats.total_validacoes += 1;
        
        if resultado.valido {
            stats.validacoes_bem_sucedidas += 1;
        } else {
            stats.validacoes_falharam += 1;
        }
        
        // Atualizar tempo médio
        let total_tempo = stats.tempo_medio_validacao * (stats.total_validacoes as i32 - 1) + resultado.tempo_validacao;
        stats.tempo_medio_validacao = total_tempo / (stats.total_validacoes as i32);
        
        stats.ultimo_bloco_validado = Some(numero_bloco);
    }
    
    pub async fn obter_estatisticas(&self) -> EstatisticasValidacao {
        self.estatisticas.read().await.clone()
    }
    
    pub async fn limpar_cache(&self) {
        let mut cache = self.cache_validacao.write().await;
        cache.clear();
        info!("Cache de validação limpo");
    }
    
    pub async fn validar_cadeia_completa(&self, cadeia: &CadeiaBlockchain) -> Result<bool> {
        info!("Iniciando validação completa da cadeia");
        
        let altura = cadeia.obter_altura().await;
        let mut todos_validos = true;
        
        for numero in 0..=altura {
            if let Some(bloco) = cadeia.obter_bloco_por_numero(numero).await {
                if !self.validar_bloco(&bloco, cadeia).await? {
                    error!("Bloco {} falhou na validação", numero);
                    todos_validos = false;
                }
            } else {
                error!("Bloco {} não encontrado", numero);
                todos_validos = false;
            }
        }
        
        if todos_validos {
            info!("Validação completa da cadeia: SUCESSO");
        } else {
            error!("Validação completa da cadeia: FALHOU");
        }
        
        Ok(todos_validos)
    }
}