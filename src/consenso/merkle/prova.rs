use crate::consenso::tipos::*;
use super::arvore::ArvoreMerkle;
use anyhow::Result;

pub struct GeradorProva;

impl GeradorProva {
    pub fn new() -> Self {
        Self
    }
    
    pub fn gerar_prova(&self, arvore: &ArvoreMerkle, no_id: &str) -> Result<Option<ProvaInclusao>> {
        if let Some(posicao) = arvore.obter_posicao_no(no_id) {
            let merkle_proof = arvore.obter_caminho_prova(posicao);
            let merkle_root = arvore.obter_root();
            
            Ok(Some(ProvaInclusao {
                merkle_proof,
                posicao,
                merkle_root,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn verificar_prova(&self, arvore: &ArvoreMerkle, prova: &ProvaInclusao, no_id: &str) -> Result<bool> {
        // Calcular hash da folha
        let hash_folha = calcular_hash(no_id.as_bytes());
        
        // Reconstruir caminho até a raiz
        let mut hash_atual = hash_folha;
        let mut posicao_atual = prova.posicao;
        
        for hash_irmao in &prova.merkle_proof {
            let mut dados = Vec::new();
            
            if posicao_atual % 2 == 0 {
                // Nó à esquerda
                dados.extend_from_slice(&hash_atual);
                dados.extend_from_slice(hash_irmao);
            } else {
                // Nó à direita
                dados.extend_from_slice(hash_irmao);
                dados.extend_from_slice(&hash_atual);
            }
            
            hash_atual = calcular_hash(&dados);
            posicao_atual /= 2;
        }
        
        // Verificar se chegamos à raiz correta
        Ok(hash_atual == prova.merkle_root && prova.merkle_root == arvore.obter_root())
    }
}