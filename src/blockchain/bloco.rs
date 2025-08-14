use serde::{Deserialize, Serialize};
use crate::consenso::tipos::{Transacao, calcular_hash};
use anyhow::Result;
use chrono::{DateTime, Utc};

// Estrutura otimizada para serialização binária
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bloco {
    pub numero: u64,
    pub hash_anterior: Vec<u8>,
    pub merkle_root: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub nonce: u64,
    pub transacoes: Vec<Transacao>,
    pub hash_bloco: Vec<u8>,
    pub assinatura_minerador: Vec<u8>,
    pub minerador_id: String,
    pub dificuldade: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CabecalhoBloco {
    pub numero: u64,
    pub hash_anterior: Vec<u8>,
    pub merkle_root: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub nonce: u64,
    pub dificuldade: u32,
}

// Implementação de serialização otimizada
impl Bloco {
    pub fn new(
        numero: u64,
        hash_anterior: Vec<u8>,
        transacoes: Vec<Transacao>,
        minerador_id: String,
        dificuldade: u32,
    ) -> Result<Self> {
        let timestamp = Utc::now();
        let merkle_root = Self::calcular_merkle_root(&transacoes)?;
        
        let mut bloco = Self {
            numero,
            hash_anterior,
            merkle_root,
            timestamp,
            nonce: 0,
            transacoes,
            hash_bloco: vec![],
            assinatura_minerador: vec![],
            minerador_id,
            dificuldade,
        };
        
        bloco.hash_bloco = bloco.calcular_hash()?;
        bloco.assinatura_minerador = bloco.assinar_bloco()?;
        
        Ok(bloco)
    }
    
    pub fn genesis() -> Result<Self> {
        let transacoes = vec![];
        let mut bloco = Self {
            numero: 0,
            hash_anterior: vec![0; 32], // Hash zero para bloco genesis
            merkle_root: vec![0; 32],
            timestamp: Utc::now(),
            nonce: 0,
            transacoes,
            hash_bloco: vec![],
            assinatura_minerador: vec![],
            minerador_id: "genesis".to_string(),
            dificuldade: 1,
        };
        
        bloco.hash_bloco = bloco.calcular_hash()?;
        bloco.assinatura_minerador = bloco.assinar_bloco()?;
        
        Ok(bloco)
    }
    
    pub fn calcular_hash(&self) -> Result<Vec<u8>> {
        let cabecalho = CabecalhoBloco {
            numero: self.numero,
            hash_anterior: self.hash_anterior.clone(),
            merkle_root: self.merkle_root.clone(),
            timestamp: self.timestamp,
            nonce: self.nonce,
            dificuldade: self.dificuldade,
        };
        
        let dados = serde_json::to_vec(&cabecalho)?;
        Ok(calcular_hash(&dados))
    }
    
    pub fn calcular_merkle_root(transacoes: &[Transacao]) -> Result<Vec<u8>> {
        if transacoes.is_empty() {
            return Ok(vec![0; 32]);
        }
        
        let mut hashes: Vec<Vec<u8>> = transacoes
            .iter()
            .map(|tx| {
                let dados = serde_json::to_vec(tx).unwrap();
                calcular_hash(&dados)
            })
            .collect();
        
        // Construir árvore Merkle
        while hashes.len() > 1 {
            let mut proxima_camada = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let hash_combinado = if chunk.len() == 2 {
                    let mut dados = Vec::new();
                    dados.extend_from_slice(&chunk[0]);
                    dados.extend_from_slice(&chunk[1]);
                    calcular_hash(&dados)
                } else {
                    // Se número ímpar, duplicar o último hash
                    let mut dados = Vec::new();
                    dados.extend_from_slice(&chunk[0]);
                    dados.extend_from_slice(&chunk[0]);
                    calcular_hash(&dados)
                };
                
                proxima_camada.push(hash_combinado);
            }
            
            hashes = proxima_camada;
        }
        
        Ok(hashes.into_iter().next().unwrap_or(vec![0; 32]))
    }
    
    fn assinar_bloco(&self) -> Result<Vec<u8>> {
        let mut dados = Vec::new();
        dados.extend_from_slice(&self.hash_bloco);
        dados.extend_from_slice(self.minerador_id.as_bytes());
        Ok(calcular_hash(&dados))
    }
    
    pub fn validar_estrutura(&self) -> Result<bool> {
        // Validar hash do bloco
        let hash_calculado = self.calcular_hash()?;
        if hash_calculado != self.hash_bloco {
            return Ok(false);
        }
        
        // Validar Merkle root
        let merkle_calculado = Self::calcular_merkle_root(&self.transacoes)?;
        if merkle_calculado != self.merkle_root {
            return Ok(false);
        }
        
        // Validar assinatura
        let assinatura_calculada = self.assinar_bloco()?;
        if assinatura_calculada != self.assinatura_minerador {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    pub fn validar_dificuldade(&self) -> bool {
        let zeros_necessarios = self.dificuldade as usize;
        if zeros_necessarios == 0 {
            return true;
        }
        
        let hash_hex = hex::encode(&self.hash_bloco);
        hash_hex.starts_with(&"0".repeat(zeros_necessarios))
    }
    
    pub fn minerar(&mut self, max_tentativas: u64) -> Result<bool> {
        for nonce in 0..max_tentativas {
            self.nonce = nonce;
            self.hash_bloco = self.calcular_hash()?;
            
            if self.validar_dificuldade() {
                self.assinatura_minerador = self.assinar_bloco()?;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Serialização binária otimizada com compressão LZ4
    pub fn serializar_binario(&self) -> Result<Vec<u8>> {
        let dados_binarios = bincode::serialize(self)
            .map_err(|e| anyhow::anyhow!("Erro na serialização binária: {}", e))?;
        
        // Aplicar compressão LZ4
        let dados_comprimidos = lz4_flex::compress_prepend_size(&dados_binarios);
        Ok(dados_comprimidos)
    }
    
    /// Deserialização binária otimizada com descompressão LZ4
    pub fn deserializar_binario(dados: &[u8]) -> Result<Self> {
        // Descomprimir dados LZ4
        let dados_descomprimidos = lz4_flex::decompress_size_prepended(dados)
            .map_err(|e| anyhow::anyhow!("Erro na descompressão LZ4: {}", e))?;
        
        // Deserializar usando bincode
        let bloco: Self = bincode::deserialize(&dados_descomprimidos)
            .map_err(|e| anyhow::anyhow!("Erro na deserialização binária: {}", e))?;
        
        Ok(bloco)
    }
    
    /// Serialização JSON (mantida para compatibilidade)
    pub fn serializar_json(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| anyhow::anyhow!("Erro na serialização JSON: {}", e))
    }
    
    /// Deserialização JSON (mantida para compatibilidade)
    pub fn deserializar_json(dados: &[u8]) -> Result<Self> {
        serde_json::from_slice(dados)
            .map_err(|e| anyhow::anyhow!("Erro na deserialização JSON: {}", e))
    }
    
    /// Método para comparar tamanhos de serialização
    pub fn comparar_formatos_serializacao(&self) -> Result<(usize, usize, f64)> {
        let json_size = self.serializar_json()?.len();
        let binary_size = self.serializar_binario()?.len();
        let reducao_percentual = ((json_size as f64 - binary_size as f64) / json_size as f64) * 100.0;
        
        Ok((json_size, binary_size, reducao_percentual))
    }
}