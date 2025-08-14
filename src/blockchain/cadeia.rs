use super::bloco::*;
use crate::consenso::tipos::Transacao;
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use rocksdb::{DB, Options};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub struct CadeiaBlockchain {
    blocos: RwLock<Vec<Bloco>>,
    indice_hash: RwLock<HashMap<Vec<u8>, usize>>,
    altura_atual: RwLock<u64>,
    dificuldade_atual: RwLock<u32>,
    // Novo campo para persistência
    db: Option<Arc<DB>>,
}

impl CadeiaBlockchain {
    pub fn new() -> Self {
        Self {
            blocos: RwLock::new(Vec::new()),
            indice_hash: RwLock::new(HashMap::new()),
            altura_atual: RwLock::new(0),
            dificuldade_atual: RwLock::new(1),
            db: None,
        }
    }
    
    /// Novo método para criar cadeia com persistência
    pub fn new_com_persistencia<P: AsRef<Path>>(caminho_db: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        let db = DB::open(&opts, caminho_db)?;
        let db_arc = Arc::new(db);
        
        let mut cadeia = Self {
            blocos: RwLock::new(Vec::new()),
            indice_hash: RwLock::new(HashMap::new()),
            altura_atual: RwLock::new(0),
            dificuldade_atual: RwLock::new(1),
            db: Some(db_arc.clone()),
        };
        
        // Carregar dados existentes do banco
        cadeia.carregar_dados_do_banco()?;
        
        Ok(cadeia)
    }
    
    /// Carrega dados existentes do banco de dados
    fn carregar_dados_do_banco(&mut self) -> Result<()> {
        if let Some(ref db) = self.db {
            let mut blocos_carregados = Vec::new();
            let mut indice_carregado = HashMap::new();
            let mut altura_maxima = 0u64;
            
            // Iterar sobre todas as chaves no banco
            let iter = db.iterator(rocksdb::IteratorMode::Start);
            for item in iter {
                let (key, value) = item?;
                if key.starts_with(b"bloco_") {
                    // Deserializar bloco usando formato binário
                    let bloco = Bloco::deserializar_binario(&value)?;
                    let posicao = blocos_carregados.len();
                    
                    indice_carregado.insert(bloco.hash_bloco.clone(), posicao);
                    altura_maxima = altura_maxima.max(bloco.numero);
                    blocos_carregados.push(bloco);
                }
            }
            
            // Ordenar blocos por número
            blocos_carregados.sort_by_key(|b| b.numero);
            
            // Atualizar estruturas em memória
            *self.blocos.get_mut() = blocos_carregados;
            *self.indice_hash.get_mut() = indice_carregado;
            *self.altura_atual.get_mut() = altura_maxima;
            
            info!("Carregados {} blocos do banco de dados", self.blocos.get_mut().len());
        }
        Ok(())
    }
    
    /// Persiste um bloco no banco de dados
    async fn persistir_bloco(&self, bloco: &Bloco) -> Result<()> {
        if let Some(ref db) = self.db {
            let chave = format!("bloco_{:010}", bloco.numero);
            let dados_binarios = bloco.serializar_binario()?;
            db.put(chave.as_bytes(), &dados_binarios)?;
            
            // Também salvar metadados
            let altura_key = b"altura_atual";
            db.put(altura_key, &bloco.numero.to_le_bytes())?;
        }
        Ok(())
    }
    
    pub async fn inicializar_com_genesis(&self) -> Result<()> {
        let bloco_genesis = Bloco::genesis()?;
        self.adicionar_bloco(bloco_genesis).await?;
        info!("Blockchain inicializada com bloco genesis");
        Ok(())
    }
    
    pub async fn adicionar_bloco(&self, bloco: Bloco) -> Result<()> {
        // Validar bloco antes de adicionar
        if !self.validar_bloco_para_adicao(&bloco).await? {
            return Err(anyhow::anyhow!("Bloco inválido para adição"));
        }
        
        // Persistir no banco antes de adicionar à memória
        self.persistir_bloco(&bloco).await?;
        
        let mut blocos = self.blocos.write().await;
        let mut indice = self.indice_hash.write().await;
        let mut altura = self.altura_atual.write().await;
        
        let posicao = blocos.len();
        indice.insert(bloco.hash_bloco.clone(), posicao);
        blocos.push(bloco.clone());
        *altura = bloco.numero;
        
        info!("Bloco {} adicionado à cadeia na posição {} e persistido", bloco.numero, posicao);
        
        // Ajustar dificuldade se necessário
        self.ajustar_dificuldade().await?;
        
        Ok(())
    }
    
    pub async fn criar_proximo_bloco(&self, transacoes: Vec<Transacao>) -> Result<Bloco> {
        let blocos = self.blocos.read().await;
        let dificuldade = *self.dificuldade_atual.read().await;
        
        let (numero, hash_anterior) = if let Some(ultimo_bloco) = blocos.last() {
            (ultimo_bloco.numero + 1, ultimo_bloco.hash_bloco.clone())
        } else {
            (0, vec![0; 32])
        };
        
        drop(blocos);
        
        let mut bloco = Bloco::new(
            numero,
            hash_anterior,
            transacoes,
            "minerador_atual".to_string(),
            dificuldade,
        )?;
        
        // Minerar o bloco
        if !bloco.minerar(1000000)? {
            warn!("Falha ao minerar bloco após 1M tentativas");
        }
        
        Ok(bloco)
    }
    
    pub async fn obter_bloco_por_hash(&self, hash: &[u8]) -> Option<Bloco> {
        let indice = self.indice_hash.read().await;
        let blocos = self.blocos.read().await;
        
        if let Some(&posicao) = indice.get(hash) {
            blocos.get(posicao).cloned()
        } else {
            None
        }
    }
    
    pub async fn obter_bloco_por_numero(&self, numero: u64) -> Option<Bloco> {
        let blocos = self.blocos.read().await;
        blocos.get(numero as usize).cloned()
    }
    
    pub async fn obter_ultimo_bloco(&self) -> Option<Bloco> {
        let blocos = self.blocos.read().await;
        blocos.last().cloned()
    }
    
    pub async fn obter_altura(&self) -> u64 {
        *self.altura_atual.read().await
    }
    
    pub async fn obter_tamanho_cadeia(&self) -> usize {
        self.blocos.read().await.len()
    }
    
    async fn validar_bloco_para_adicao(&self, bloco: &Bloco) -> Result<bool> {
        // Validar estrutura do bloco
        if !bloco.validar_estrutura()? {
            error!("Estrutura do bloco inválida");
            return Ok(false);
        }
        
        // Validar dificuldade
        if !bloco.validar_dificuldade() {
            error!("Dificuldade do bloco inválida");
            return Ok(false);
        }
        
        // Validar sequência
        let blocos = self.blocos.read().await;
        if let Some(ultimo_bloco) = blocos.last() {
            if bloco.numero != ultimo_bloco.numero + 1 {
                error!("Número de bloco inválido. Esperado: {}, Recebido: {}", 
                       ultimo_bloco.numero + 1, bloco.numero);
                return Ok(false);
            }
            
            if bloco.hash_anterior != ultimo_bloco.hash_bloco {
                error!("Hash anterior inválido");
                return Ok(false);
            }
        } else if bloco.numero != 0 {
            error!("Primeiro bloco deve ser genesis (número 0)");
            return Ok(false);
        }
        
        Ok(true)
    }
    
    async fn ajustar_dificuldade(&self) -> Result<()> {
        let blocos = self.blocos.read().await;
        let tamanho = blocos.len();
        
        // Ajustar dificuldade a cada 10 blocos
        if tamanho % 10 == 0 && tamanho > 0 {
            let mut dificuldade = self.dificuldade_atual.write().await;
            
            // Lógica simples: aumentar dificuldade se cadeia crescendo rapidamente
            if tamanho > 20 {
                *dificuldade += 1;
                info!("Dificuldade ajustada para: {}", *dificuldade);
            }
        }
        
        Ok(())
    }
    
    pub async fn validar_cadeia_completa(&self) -> Result<bool> {
        let blocos = self.blocos.read().await;
        
        for (i, bloco) in blocos.iter().enumerate() {
            // Validar estrutura do bloco
            if !bloco.validar_estrutura()? {
                error!("Bloco {} tem estrutura inválida", i);
                return Ok(false);
            }
            
            // Validar ligação com bloco anterior
            if i > 0 {
                let bloco_anterior = &blocos[i - 1];
                if bloco.hash_anterior != bloco_anterior.hash_bloco {
                    error!("Bloco {} não está ligado corretamente ao anterior", i);
                    return Ok(false);
                }
            }
        }
        
        info!("Validação completa da cadeia: SUCESSO");
        Ok(true)
    }
    
    pub async fn obter_estatisticas(&self) -> HashMap<String, u64> {
        let blocos = self.blocos.read().await;
        let mut stats = HashMap::new();
        
        stats.insert("total_blocos".to_string(), blocos.len() as u64);
        stats.insert("altura_atual".to_string(), *self.altura_atual.read().await);
        stats.insert("dificuldade_atual".to_string(), *self.dificuldade_atual.read().await as u64);
        
        let total_transacoes: usize = blocos.iter().map(|b| b.transacoes.len()).sum();
        stats.insert("total_transacoes".to_string(), total_transacoes as u64);
        
        stats
    }
}