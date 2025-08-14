use crate::consenso::tipos::*;
use anyhow::Result;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct NoComPosicao {
    no: No,
    position_hash: Vec<u8>,
}

impl PartialEq for NoComPosicao {
    fn eq(&self, other: &Self) -> bool {
        self.position_hash == other.position_hash
    }
}

impl Eq for NoComPosicao {}

impl PartialOrd for NoComPosicao {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NoComPosicao {
    fn cmp(&self, other: &Self) -> Ordering {
        // Ordenar por hash de posição (menor primeiro)
        self.position_hash.cmp(&other.position_hash)
    }
}

pub struct GeradorFila;

impl GeradorFila {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn gerar_fila_ordenada(&self, nos: Vec<No>, seed_global: Vec<u8>) -> Result<FilaOrdenada> {
        let mut nos_com_posicao: Vec<NoComPosicao> = nos
            .into_iter()
            .map(|no| {
                let position_hash = calcular_position_hash(&no.chave_publica, &seed_global);
                NoComPosicao {
                    no,
                    position_hash,
                }
            })
            .collect();
        
        // Ordenar por position_hash
        nos_com_posicao.sort();
        
        let nos_ordenados: Vec<No> = nos_com_posicao
            .into_iter()
            .map(|ncp| ncp.no)
            .collect();
        
        Ok(FilaOrdenada {
            nos: nos_ordenados,
            seed_global,
            timestamp: chrono::Utc::now(),
        })
    }
}