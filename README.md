🚀 Nimbos Blockchain
Uma implementação de blockchain distribuída, de código aberto, desenvolvida em Rust com um mecanismo de consenso inovador e arquitetura modular.

📋 Visão Geral
Nimbos é uma blockchain experimental projetada para explorar um mecanismo de consenso único, baseado em múltiplas camadas de validação. O projeto foca em transparência, eficiência e tolerância a falhas. Desenvolvido inteiramente em Rust, sua arquitetura modular facilita a manutenção e a extensibilidade, tornando-o um campo de testes ideal para novas tecnologias de contabilidade distribuída.

🏗️ Arquitetura do Sistema
A Nimbos é organizada em camadas lógicas, cada uma com responsabilidades bem definidas, garantindo a separação de interesses e a coesão do sistema.

┌─────────────────────────────────────────────────────────────┐
│                     NIMBOS BLOCKCHAIN                     │
├─────────────────────────────────────────────────────────────┤
│             🎯 Camada de Aplicação (main.rs)                │
├─────────────────────────────────────────────────────────────┤
│             🔄 Camada de Consenso                           │
│   ├── Registro/Commit (Commitments Anônimos)                │
│   ├── Reveal/Verificação (Validação de Commitments)         │
│   ├── Ordenação Determinística (Fila baseada em Hash)       │
│   ├── Processamento Rotativo (Distribuição de Carga)        │
│   ├── Validação Distribuída (Quorum e Anti-maliciosos)      │
│   └── Merkle Tree (Provas de Inclusão)                      │
├─────────────────────────────────────────────────────────────┤
│             🔗 Camada de Blockchain                         │
│   ├── Estrutura de Blocos (Hash, Merkle Root, Assinaturas)  │
│   ├── Cadeia de Blocos (Validação e Persistência)           │
│   ├── Sistema de Checkpoints (Estado Periódico)             │
│   ├── Validação Avançada (Cache e Estatísticas)             │
│   └── Migração de Dados (JSON ↔ Binário)                    │
├─────────────────────────────────────────────────────────────┤
│             🌐 Camada de Comunicação                        │
│   ├── Protocolo de Mensagens (Serialização/Assinaturas)     │
│   ├── Sistema de Broadcast (Paralelo com Timeout)           │
│   ├── Gerenciamento de Rede (Descoberta e Topologia)        │
│   └── Mecanismo de Retry (Backoff Exponencial)              │
├─────────────────────────────────────────────────────────────┤
│             💰 Camada de Recompensas                         │
│   ├── Calculadora de Taxas (Distribuição 80/20)             │
│   ├── Distribuidor de Recompensas (Contas e Saques)         │
│   └── Ledger de Transações (Histórico e Auditoria)          │
├─────────────────────────────────────────────────────────────┤
│             🛡️ Camada de Detecção de Falhas                  │
│   ├── Monitoramento de Nós (Heartbeat e Latência)           │
│   ├── Gerenciamento de Timeouts (Recuperação Automática)    │
│   └── Mecanismos de Recuperação (Reentrada de Nós)          │
└─────────────────────────────────────────────────────────────┘

🔧 Módulos Implementados
🔗 Blockchain (Camada de Imutabilidade)
Localização: src/blockchain/

✅ Estrutura de Blocos Completa: Hash SHA-256, Merkle Root, timestamps e nonce para mineração.

✅ Cadeia de Blocos Persistente: Armazenamento em memória com suporte opcional a RocksDB para persistência em disco.

✅ Sistema de Checkpoints: Snapshots automáticos do estado da cadeia a cada 100 blocos para recuperação rápida.

✅ Validação Avançada: Cache de resultados de validação, detecção de transações duplicadas e coleta de estatísticas.

✅ Mineração PoW: Algoritmo de Prova de Trabalho com ajuste automático de dificuldade.

✅ Serialização Otimizada: Suporte a JSON para desenvolvimento e a um formato binário com compressão LZ4 para produção.

Arquivos Principais:

bloco.rs: Define a estrutura dos blocos e suas regras de validação.

cadeia.rs: Gerencia a cadeia principal, incluindo a adição de novos blocos.

checkpoint.rs: Implementa o sistema de snapshots periódicos.

validador_cadeia.rs: Realiza a validação contínua da cadeia e gerencia o cache.

migrador.rs: Facilita a migração de dados entre diferentes formatos.

🔄 Consenso (Mecanismo Inovador Multi-Camadas)
Localização: src/consenso/

O fluxo de consenso foi projetado para garantir segurança e justiça através de um processo de cinco etapas:

1. REGISTRO   →  2. REVEAL   →  3. ORDENAÇÃO   →  4. PROCESSAMENTO   →  5. VALIDAÇÃO
     ↓              ↓              ↓                  ↓                  ↓
 Commitments     Verificação    Fila Hash       Rotação de Nós      Quorum e Anti-Malware
  Anônimos       de Reveals   Determinística     Distribuída          Validação Final

Submódulos:

Registro (registro/): Permite que os nós submetam commitments anônimos.

Reveal (reveal/): Valida os commitments revelados na fase anterior.

Ordenação (ordenacao/): Organiza as transações em uma fila determinística baseada em um hash global.

Processamento (processamento/): Distribui a carga de processamento através de um sistema de rotação de nós.

Validação (validacao/): Executa a validação final por um quorum distribuído e aplica mecanismos anti-maliciosos.

Merkle (merkle/): Gera Árvores de Merkle para criar provas de inclusão de transações de forma eficiente.

🌐 Comunicação (Rede Distribuída)
Localização: src/comunicacao/

✅ Protocolo de Mensagens: Define mensagens serializadas em JSON com assinaturas baseadas em hash para garantir a integridade.

✅ Sistema de Broadcast: Realiza o envio de mensagens em paralelo para a rede, com timeouts e coleta de estatísticas de entrega.

✅ Gerenciamento de Rede: Implementa a descoberta automática de nós via seeds e um protocolo gossip para manter a topologia da rede.

✅ Mecanismo de Retry: Utiliza backoff exponencial com jitter para gerenciar tentativas de conexão e evitar sobrecarga da rede.

✅ Monitoramento: Envia heartbeats contínuos para monitorar a saúde dos nós e medir a latência da rede.

💰 Recompensas (Sistema de Incentivos)
Localização: src/recompensas/

O modelo de distribuição de recompensas incentiva a participação honesta na rede:

80% da recompensa do bloco é destinada ao Nó Processador.

20% restantes são divididos igualmente entre os Nós Validadores.

Funcionalidades:

✅ Calculadora de Taxas: Implementa a regra de distribuição 80/20.

✅ Distribuidor: Gerencia contas individuais, histórico de recompensas e um sistema de saques.

✅ Ledger de Transações: Mantém um registro auditável de todas as transações de recompensa.

🛡️ Detecção de Falhas (Tolerância a Falhas)
Localização: src/deteccao_falhas/

✅ Monitoramento Contínuo: Detecta nós offline ou com alta latência através de heartbeats.

✅ Gerenciamento de Timeouts: Descarta automaticamente nós que não respondem a tempo, garantindo que o consenso não seja bloqueado.

✅ Recuperação Automática: Permite que nós recuperados reingressem na rede de forma segura.

🚀 Características Técnicas
Performance e Otimização
Serialização Binária: Usa bincode e compressão LZ4 para reduzir o tráfego de rede em 60-80%.

Persistência Eficiente: Integração com RocksDB para armazenamento rápido em disco.

Cache Inteligente: Armazena resultados de validação para evitar reprocessamento desnecessário.

Processamento Assíncrono: Utiliza Tokio para operações de I/O não-bloqueantes.

Segurança e Confiabilidade
Validação Multi-Camadas: Garante que transações e blocos passem por múltiplos pontos de verificação.

Detecção de Nós Maliciosos: Algoritmos integrados para identificar e penalizar comportamento fraudulento.

Checkpoints Automáticos: Permitem a recuperação rápida do estado da rede em caso de falhas críticas.

Escalabilidade
Arquitetura Modular: Permite a fácil adição de novos componentes e funcionalidades.

Processamento Distribuído: Balanceia a carga de trabalho entre os nós da rede.

Configuração Flexível: Parâmetros do sistema, como dificuldade e timeouts, podem ser ajustados por ambiente.

🛠️ Tecnologias Utilizadas
Core: Rust 2021, Tokio, Serde

Otimização: Bincode, LZ4_flex, RocksDB

Criptografia: SHA-2, Hex

Utilitários: Chrono, Rand, Tracing, Anyhow/Thiserror

🎯 Status do Projeto
✅ Implementado e Funcional
Estrutura completa da blockchain.

Sistema de consenso multi-camadas.

Comunicação de rede distribuída.

Sistema de recompensas e detecção de falhas.

Validação avançada com cache e sistema de checkpoints.

🔧 Otimizações Prontas (Não Integradas)
Persistência com RocksDB.

Serialização binária com compressão.

🚧 Próximos Passos
Integração final da persistência em disco.

Testes de estresse e performance em larga escala.

Desenvolvimento de uma interface de monitoramento web.

Criação de documentação detalhada da API.

Realização de benchmarks comparativos com outras blockchains.

🚀 Como Executar
Pré-requisitos
Rust 1.70+

Cargo

Execução
# Clone o repositório
git clone <URL_DO_REPOSITORIO>
cd nimbos

# Compile e execute em modo de desenvolvimento
cargo run

# Execute os testes unitários e de integração
cargo test

# Compile uma versão otimizada para produção
cargo build --release

Configuração
O sistema é iniciado com configurações padrão, que podem ser ajustadas em um arquivo de configuração:

Intervalo de consenso: 5 segundos

Intervalo de checkpoints: 100 blocos

Dificuldade de mineração inicial: 4

Timeout de rede: 30 segundos

📝 Licença
Este projeto é licenciado sob a Licença MIT.

🤝 Contribuição
Contribuições são muito bem-vindas! Para contribuir:

Faça um fork do projeto.

Crie uma branch para sua nova funcionalidade (git checkout -b feature/nova-feature).

Faça o commit de suas mudanças (git commit -m 'Adiciona nova feature').

Envie para a sua branch (git push origin feature/nova-feature).

Abra um Pull Request.

📞 Contato
[Suas informações de contato ou da equipe]

Nimbos Blockchain – Explorando o futuro da tecnologia de contabilidade distribuída.