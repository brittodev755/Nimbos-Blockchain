use crate::consenso::tipos::*;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ArvoreMerkle {
    root: Vec<u8>,
    folhas: Vec<Vec<u8>>,
    niveis: Vec<Vec<Vec<u8>>>,
    mapa_nos: HashMap<String, usize>,
}

impl ArvoreMerkle {
    pub fn construir(nos: &[No]) -> Result<Self> {
        if nos.is_empty() {
            return Err(anyhow::anyhow!("Lista de nós não pode estar vazia"));
        }
        
        // Criar folhas (hash dos IDs dos nós)
        let folhas: Vec<Vec<u8>> = nos
            .iter()
            .map(|no| calcular_hash(no.id.as_bytes()))
            .collect();
        
        // Criar mapa de nós para posições
        let mapa_nos: HashMap<String, usize> = nos
            .iter()
            .enumerate()
            .map(|(i, no)| (no.id.clone(), i))
            .collect();
        
        // Construir árvore
        let mut niveis = vec![folhas.clone()];
        let mut nivel_atual = folhas.clone();
        
        while nivel_atual.len() > 1 {
            let mut proximo_nivel = Vec::new();
            
            for chunk in nivel_atual.chunks(2) {
                let hash_combinado = if chunk.len() == 2 {
                    let mut dados = Vec::new();
                    dados.extend_from_slice(&chunk[0]);
                    dados.extend_from_slice(&chunk[1]);
                    calcular_hash(&dados)
                } else {
                    chunk[0].clone() // Nó ímpar, duplicar
                };
                
                proximo_nivel.push(hash_combinado);
            }
            
            niveis.push(proximo_nivel.clone());
            nivel_atual = proximo_nivel;
        }
        
        let root = nivel_atual[0].clone();
        
        Ok(Self {
            root,
            folhas,
            niveis,
            mapa_nos,
        })
    }
    
    pub fn obter_root(&self) -> Vec<u8> {
        self.root.clone()
    }
    
    pub fn obter_posicao_no(&self, no_id: &str) -> Option<usize> {
        self.mapa_nos.get(no_id).copied()
    }
    
    pub fn obter_caminho_prova(&self, posicao: usize) -> Vec<Vec<u8>> {
        let mut caminho = Vec::new();
        let mut pos_atual = posicao;
        
        for nivel in &self.niveis[..self.niveis.len() - 1] {
            let pos_irmao = if pos_atual % 2 == 0 {
                pos_atual + 1
            } else {
                pos_atual - 1
            };
            
            if pos_irmao < nivel.len() {
                caminho.push(nivel[pos_irmao].clone());
            }
            
            pos_atual /= 2;
        }
        
        caminho
    }
}