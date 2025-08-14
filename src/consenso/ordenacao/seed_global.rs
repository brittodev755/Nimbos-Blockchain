use anyhow::Result;
use rand::Rng;
use tokio::sync::RwLock;

pub struct SeedGlobalManager {
    seed_atual: RwLock<Vec<u8>>,
}

impl SeedGlobalManager {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let seed_inicial: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        
        Self {
            seed_atual: RwLock::new(seed_inicial),
        }
    }
    
    pub async fn obter_seed_atual(&self) -> Vec<u8> {
        self.seed_atual.read().await.clone()
    }
    
    pub async fn atualizar_seed(&self, novo_seed: Vec<u8>) {
        *self.seed_atual.write().await = novo_seed;
    }
    
    pub async fn gerar_novo_seed(&self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let novo_seed: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        self.atualizar_seed(novo_seed.clone()).await;
        novo_seed
    }
}