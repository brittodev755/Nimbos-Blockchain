<div align="center">
<img src="https://raw.githubusercontent.com/gist/Micaelilobo/6bb596a75051e704771694f794957f72/raw/e3b6fd37722955f159518bf9710344b1c28c8942/logo-nimbos.svg" alt="Nimbos Logo" width="150"/>
<h1>Nimbos Blockchain</h1>
<p>
<strong>Uma implementação de blockchain distribuída, de código aberto, desenvolvida em Rust com um mecanismo de consenso inovador e arquitetura modular.</strong>
</p>
<p>
<a href="#"><img src="https://img.shields.io/badge/Rust-2021-orange.svg" alt="Rust 2021"></a>
<a href="https://github.com/Micaelilobo/nimbos-blockchain/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
<a href="#"><img src="https://img.shields.io/badge/status-funcional-success.svg" alt="Status"></a>
<a href="https://github.com/Micaelilobo/nimbos-blockchain/pulls"><img src="https://img.shields.io/badge/PRs-bem--vindos-brightgreen.svg" alt="PRs Welcome"></a>
</p>
</div>

📋 Visão Geral
Nimbos é uma blockchain experimental projetada para explorar um mecanismo de consenso único, baseado em múltiplas camadas de validação. O projeto foca em transparência, eficiência e tolerância a falhas. Desenvolvido inteiramente em Rust, sua arquitetura modular facilita a manutenção e a extensibilidade, tornando-o um campo de testes ideal para novas tecnologias de contabilidade distribuída.

🏗️ Arquitetura do Sistema
A Nimbos é organizada em camadas lógicas, cada uma com responsabilidades bem definidas, garantindo a separação de interesses e a coesão do sistema.

🎯 Camada de Aplicação (main.rs): Ponto de entrada e orquestração geral do nó.

🔄 Camada de Consenso: Núcleo do mecanismo de validação e acordo da rede.

Registro/Commit: Submissão de commitments anônimos.

Reveal/Verificação: Validação dos commitments revelados.

Ordenação Determinística: Fila de transações baseada em hash global.

Processamento Rotativo: Distribuição de carga entre nós processadores.

Validação Distribuída: Verificação final por quórum e mecanismos anti-maliciosos.

Merkle Tree: Geração de provas de inclusão de transações.

🔗 Camada de Blockchain: Gerenciamento da cadeia de blocos imutável.

Estrutura de Blocos: Definição de blocos com Hash, Merkle Root e assinaturas.

Cadeia de Blocos: Lógica de validação e persistência da cadeia.

Sistema de Checkpoints: Salva estados periódicos para recuperação rápida.

Validação Avançada: Otimizações com cache e coleta de estatísticas.

Migração de Dados: Conversão entre formatos JSON e binário.

🌐 Camada de Comunicação: Protocolos para a interação entre os nós da rede.

Protocolo de Mensagens: Serialização, assinatura e validação de mensagens.

Sistema de Broadcast: Envio de mensagens em paralelo com timeouts.

Gerenciamento de Rede: Descoberta de nós e manutenção da topologia (gossip).

Mecanismo de Retry: Tentativas de reconexão com backoff exponencial.

💰 Camada de Recompensas: Sistema de incentivos para os participantes da rede.

Calculadora de Taxas: Distribuição de recompensas (80% processador / 20% validadores).

Distribuidor de Recompensas: Gerenciamento de contas, saldos e saques.

Ledger de Transações: Registro auditável de todas as recompensas.

🛡️ Camada de Detecção de Falhas: Mecanismos para garantir a resiliência da rede.

Monitoramento de Nós: Verificação de atividade e latência com heartbeats.

Gerenciamento de Timeouts: Recuperação automática de falhas de comunicação.

Mecanismos de Recuperação: Reingresso seguro de nós na rede.

🔧 Módulos Implementados
<details>
<summary><strong>🔗 Blockchain (Camada de Imutabilidade)</strong></summary>

Estrutura de Blocos Completa: Hash SHA-256, Merkle Root, timestamps e nonce.

Cadeia de Blocos Persistente: Armazenamento em memória com suporte opcional a RocksDB.

Sistema de Checkpoints: Snapshots automáticos do estado da cadeia a cada 100 blocos.

Validação Avançada: Cache de resultados, detecção de duplicatas e estatísticas de performance.

Mineração PoW: Algoritmo de Prova de Trabalho com ajuste de dificuldade.

Serialização Otimizada: Suporte a JSON para depuração e formato binário (bincode + LZ4) para produção.

Localização: src/blockchain/

</details>

<details>
<summary><strong>🔄 Consenso (Mecanismo Inovador Multi-Camadas)</strong></summary>

O fluxo de consenso foi projetado para garantir segurança e justiça através de um processo de cinco etapas:

<div align="center">

1️⃣ REGISTRO <br/> (Commitments Anônimos)
<br/>⬇️<br/>
2️⃣ REVEAL <br/> (Verificação de Reveals)
<br/>⬇️<br/>
3️⃣ ORDENAÇÃO <br/> (Fila Determinística via Hash)
<br/>⬇️<br/>
4️⃣ PROCESSAMENTO <br/> (Rotação de Nós e Distribuição de Carga)
<br/>⬇️<br/>
5️⃣ VALIDAÇÃO <br/> (Quórum, Anti-Malware e Finalização)

</div>

Localização: src/consenso/

</details>

<details>
<summary><strong>🌐 Comunicação (Rede Distribuída)</strong></summary>

Protocolo de Mensagens: Mensagens serializadas em JSON com assinaturas para garantir a integridade.

Sistema de Broadcast: Envio de mensagens em paralelo com timeouts e estatísticas de entrega.

Gerenciamento de Rede: Descoberta automática de nós e protocolo gossip para manter a topologia.

Mecanismo de Retry: Utiliza backoff exponencial com jitter para gerenciar tentativas de conexão.

Monitoramento: Heartbeats contínuos para medir a saúde e a latência dos nós.

Localização: src/comunicacao/

</details>

<details>
<summary><strong>💰 Recompensas (Sistema de Incentivos)</strong></summary>

O modelo de distribuição de recompensas incentiva a participação honesta:

80% da recompensa do bloco é destinada ao Nó Processador.

20% restantes são divididos igualmente entre os Nós Validadores.

Funcionalidades implementadas:

Calculadora de Taxas: Implementa a regra de distribuição 80/20.

Distribuidor: Gerencia contas individuais e o sistema de saques.

Ledger de Transações: Mantém um registro auditável de todas as recompensas.

Localização: src/recompensas/

</details>

<details>
<summary><strong>🛡️ Detecção de Falhas (Tolerância a Falhas)</strong></summary>

Monitoramento Contínuo: Detecta nós offline ou com alta latência através de heartbeats.

Gerenciamento de Timeouts: Descarta nós que não respondem para não bloquear o consenso.

Recuperação Automática: Permite que nós recuperados reingressem na rede de forma segura.

Localização: src/deteccao_falhas/

</details>

🚀 Características Técnicas
Característica

Descrição

⚡ Performance

Serialização binária (bincode + LZ4), persistência com RocksDB e cache inteligente de validação.

🔒 Segurança

Validação multi-camadas, detecção de nós maliciosos e checkpoints automáticos para recuperação.

🌐 Escalabilidade

Arquitetura modular, processamento distribuído de carga e configuração flexível de parâmetros da rede.

⚙️ Assincronia

Uso massivo de Tokio para operações de I/O não-bloqueantes e concorrência eficiente.

🛠️ Tecnologias Utilizadas
Categoria

Tecnologias

Core

Rust 2021, Tokio, Serde

Otimização

Bincode, LZ4_flex, RocksDB

Criptografia

SHA-2, Hex

Utilitários

Chrono, Rand, Tracing, Anyhow, Thiserror

🎯 Status do Projeto
[x] Implementado e Funcional: Estrutura completa da blockchain, consenso, comunicação, recompensas e detecção de falhas.

[x] Otimizações Prontas (Não Integradas): Persistência com RocksDB e serialização binária com compressão.

[ ] Próximos Passos:

[ ] Integração final da persistência em disco.

[ ] Testes de estresse e performance em larga escala.

[ ] Desenvolvimento de uma interface de monitoramento web.

[ ] Criação de documentação detalhada da API.

[ ] Realização de benchmarks comparativos com outras blockchains.

🚀 Como Executar
Pré-requisitos
Rust 1.70+

Cargo

Execução
Clone o repositório:

git clone https://github.com/Micaelilobo/nimbos-blockchain.git
cd nimbos-blockchain

Compile e execute em modo de desenvolvimento:

cargo run

Execute os testes:

cargo test

Compile uma versão otimizada para produção:

cargo build --release

Configuração
As configurações padrão (intervalo de consenso, dificuldade, timeouts) podem ser ajustadas em um arquivo de configuração central.

🤝 Contribuição
Contribuições são muito bem-vindas! Para contribuir:

Faça um Fork do projeto.

Crie uma nova Branch (git checkout -b feature/nova-feature).

Faça o Commit de suas mudanças (git commit -m 'Adiciona nova feature').

Envie para a sua branch (git push origin feature/nova-feature).

Abra um Pull Request.

📝 Licença
Este projeto é licenciado sob a Licença MIT. Veja o arquivo LICENSE para mais detalhes.

📞 Contato
Micael Lobo - micael.lobo@nimbos.dev

Link do Projeto: https://github.com/Micaelilobo/nimbos-blockchain

<div align="center">
<i>Nimbos Blockchain – Explorando o futuro da tecnologia de contabilidade distribuída.</i>
</div>