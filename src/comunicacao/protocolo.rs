use serde::{Deserialize, Serialize};
use crate::consenso::tipos::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mensagem {
    pub id: String,
    pub tipo: TipoMensagem,
    pub remetente: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub dados: Vec<u8>,
    pub assinatura: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TipoMensagem {
    Commitment,
    Reveal,
    Validacao,
    Transacao,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespostaMensagem {
    pub id_mensagem: String,
    pub sucesso: bool,
    pub erro: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Mensagem {
    pub fn nova(tipo: TipoMensagem, remetente: String, dados: Vec<u8>) -> Self {
        let id = format!("msg_{}_{}", chrono::Utc::now().timestamp_nanos(), rand::random::<u32>());
        
        Self {
            id,
            tipo,
            remetente: remetente.clone(),
            timestamp: chrono::Utc::now(),
            dados: dados.clone(),
            assinatura: Self::calcular_assinatura(&remetente, &dados),
        }
    }
    
    fn calcular_assinatura(remetente: &str, dados: &[u8]) -> Vec<u8> {
        let mut conteudo = Vec::new();
        conteudo.extend_from_slice(remetente.as_bytes());
        conteudo.extend_from_slice(dados);
        calcular_hash(&conteudo)
    }
    
    pub fn verificar_assinatura(&self) -> bool {
        let assinatura_esperada = Self::calcular_assinatura(&self.remetente, &self.dados);
        assinatura_esperada == self.assinatura
    }
}