# Camada de Registro/Commit

Implementa a primeira fase do protocolo de consenso commit-reveal, onde os nós enviam commitments criptográficos anônimos de suas intenções sem revelar o conteúdo real. Esta fase garante que os nós se comprometam com suas decisões antes de conhecer as escolhas dos outros participantes.

## Arquivos e Funcionalidades

### `mod.rs` - Coordenação Principal
Arquivo principal que integra todos os componentes da camada de registro. Define a `CamadaRegistro` que coordena o recebimento, validação e armazenamento de commitments. Gerencia o estado temporário dos commitments durante a fase de commit, antes da fase de reveal. Utiliza `RwLock` para acesso concorrente seguro aos dados compartilhados.

### `commitment.rs` - Estruturas e Geração
Implementa o `GeradorCommitment` responsável pela criação de commitments criptográficos. Gera nonces aleatórios para garantir unicidade e imprevisibilidade dos commitments. Cria estruturas de commitment que combinam chaves públicas, nonces e identificadores de nós em hashes criptográficos seguros.

### `validador.rs` - Validação de Commitments
Contém o `ValidadorCommitment` que implementa as regras de validação para commitments recebidos. Verifica timing (janela de tempo válida), formato dos hashes, integridade dos dados e validade dos identificadores de nós. Garante que apenas commitments válidos sejam aceitos no sistema.

## Funcionalidades Implementadas

### Recebimento de Commitments
- Aceitação de commitments de múltiplos nós participantes
- Armazenamento temporário em estrutura HashMap thread-safe
- Identificação única por ID do nó
- Prevenção de commitments duplicados

### Validação de Formato e Timing
- Verificação de janela temporal (5 minutos por padrão)
- Validação do tamanho e formato dos hashes (32 bytes)
- Verificação de integridade dos identificadores de nós
- Rejeição automática de commitments inválidos

### Armazenamento Temporário
- Manutenção de commitments em memória durante a fase de commit
- Acesso concorrente seguro através de RwLock
- Limpeza automática após conclusão da fase
- Recuperação eficiente de commitments por ID

### Garantia de Anonimato
- Commitments não revelam o conteúdo real das decisões
- Uso de hashes criptográficos para ocultar informações
- Nonces aleatórios para prevenir ataques de dicionário
- Separação temporal entre commit e reveal

## Implementações Fictícias/Simuladas

### Criptografia Real
- **Algoritmos de Hash**: Usa funções de hash simples em vez de algoritmos criptográficos robustos (SHA-256, Blake2)
- **Geração de Nonces**: Gerador pseudo-aleatório básico, não criptograficamente seguro
- **Verificação Criptográfica**: Não implementa verificação real de assinaturas digitais
- **Proteção contra Ataques**: Falta proteção contra ataques de timing e side-channel

### Comunicação de Rede
- **Recebimento Real**: Simula recebimento de commitments, não há comunicação de rede real
- **Broadcast**: Não implementa disseminação real de commitments entre nós
- **Sincronização**: Não há sincronização real de tempo entre nós distribuídos
- **Tolerância a Falhas**: Não lida com falhas de rede ou nós offline

### Persistência e Durabilidade
- **Armazenamento em Memória**: Todos os commitments são mantidos apenas em RAM
- **Recuperação**: Não há mecanismo de recuperação após falhas do sistema
- **Backup**: Não implementa backup ou replicação de commitments
- **Durabilidade**: Dados são perdidos em caso de reinicialização

### Validação Avançada
- **Prova de Stake**: Não verifica se o nó tem direito de participar
- **Reputação**: Não considera histórico ou reputação dos nós
- **Detecção de Fraude**: Não detecta tentativas de manipulação
- **Auditoria**: Não mantém trilhas de auditoria detalhadas

## O Que Ainda Falta Implementar

### Integração com Outras Camadas
- Conexão real com a camada de comunicação para receber commitments
- Integração com sistema de validação distribuída
- Coordenação com a fase de reveal do protocolo
- Sincronização com sistema de consenso global

### Criptografia e Segurança
- Implementação de algoritmos criptográficos robustos
- Sistema de gerenciamento de chaves públicas/privadas
- Verificação de assinaturas digitais
- Proteção contra ataques criptográficos

### Comunicação de Rede
- Protocolos de rede para recebimento de commitments
- Sistema de broadcast para disseminação
- Sincronização de tempo entre nós distribuídos
- Tolerância a falhas de comunicação

### Persistência e Durabilidade
- Sistema de armazenamento persistente
- Mecanismos de backup e recuperação
- Replicação de dados entre nós
- Garantias de durabilidade

### Validação Avançada
- Sistema de prova de stake ou autoridade
- Verificação de elegibilidade dos participantes
- Detecção de comportamento malicioso
- Auditoria e compliance

### Performance e Escalabilidade
- Otimização para grandes volumes de commitments
- Processamento paralelo de validações
- Cache inteligente para acesso rápido
- Compressão e otimização de dados

## Melhorias Futuras

### Criptografia Avançada
- **Zero-Knowledge Proofs**: Commitments que provam validade sem revelar conteúdo
- **Criptografia Homomórfica**: Operações sobre commitments criptografados
- **Assinaturas Cegas**: Commitments que preservam privacidade total
- **Threshold Cryptography**: Commitments que requerem múltiplas chaves

### Protocolos Avançados
- **Commit-Reveal Otimizado**: Versões mais eficientes do protocolo
- **Timelock Commitments**: Commitments com liberação temporal automática
- **Conditional Commitments**: Commitments condicionais baseados em eventos
- **Batch Processing**: Processamento em lote de múltiplos commitments

### Privacidade e Anonimato
- **Mixing Networks**: Redes de mistura para anonimato total
- **Ring Signatures**: Assinaturas que ocultam identidade do signatário
- **Stealth Addresses**: Endereços únicos para cada commitment
- **Differential Privacy**: Proteção estatística de privacidade

### Governança e Transparência
- **Auditoria Pública**: Mecanismos para auditoria externa
- **Transparência Seletiva**: Revelação controlada de informações
- **Compliance Automática**: Verificação automática de regras
- **Relatórios de Integridade**: Relatórios automáticos de saúde do sistema

### Integração e Interoperabilidade
- **Cross-Chain Commitments**: Commitments entre diferentes blockchains
- **Oracle Integration**: Integração com oráculos externos
- **API Padronizada**: Interfaces padrão para integração
- **Bridges**: Pontes para comunicação com outros sistemas

### Análise e Inteligência
- **Análise de Padrões**: Detecção de padrões em commitments
- **Machine Learning**: IA para otimização e detecção de anomalias
- **Métricas Avançadas**: Análise estatística detalhada
- **Predição**: Algoritmos para prever comportamentos

### Sustentabilidade
- **Eficiência Energética**: Algoritmos otimizados para menor consumo
- **Recursos Adaptativos**: Uso dinâmico baseado na demanda
- **Green Computing**: Técnicas para reduzir impacto ambiental
- **Métricas de Sustentabilidade**: Monitoramento de pegada ecológica