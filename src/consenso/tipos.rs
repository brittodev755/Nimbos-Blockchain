use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct No {
    pub id: String,
    pub chave_publica: Vec<u8>,
    pub endereco: String,
    pub ativo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub hash: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub no_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reveal {
    pub chave_publica: Vec<u8>,
    pub nonce: Vec<u8>,
    pub no_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilaOrdenada {
    pub nos: Vec<No>,
    pub seed_global: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvaInclusao {
    pub merkle_proof: Vec<Vec<u8>>,
    pub posicao: usize,
    pub merkle_root: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transacao {
    pub id: String,
    pub dados: Vec<u8>,
    pub estado_anterior: Vec<u8>,
    pub estado_final: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub nonce: u64,
    pub assinatura: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidacaoConsenso {
    pub hash_transacao: Vec<u8>,
    pub hash_cadeia: Vec<u8>,
    pub validador_id: String,
    pub assinatura: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub fn calcular_hash(dados: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(dados);
    hasher.finalize().to_vec()
}

pub fn calcular_commitment(chave_publica: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut dados = Vec::new();
    dados.extend_from_slice(chave_publica);
    dados.extend_from_slice(nonce);
    calcular_hash(&dados)
}

pub fn calcular_position_hash(chave_publica: &[u8], seed_global: &[u8]) -> Vec<u8> {
    let mut dados = Vec::new();
    dados.extend_from_slice(chave_publica);
    dados.extend_from_slice(seed_global);
    calcular_hash(&dados)
}