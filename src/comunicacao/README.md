# Camada de Comunicação Distribuída

Implementa a comunicação entre nós da rede blockchain.

## Arquivos:

### `mod.rs` - Módulo Principal de Comunicação
**O que faz:**
- Define a estrutura `CamadaComunicacao` que integra todos os componentes
- Gerencia um canal `mpsc` para processamento assíncrono de mensagens
- Coordena `SistemaBroadcast`, `MecanismoRetry` e `GerenciadorRede`
- Processa diferentes tipos de mensagens (Commitment, Reveal, Validação, Transação)

**Implementação atual:** Funcional mas simplificada - apenas registra mensagens recebidas

### `broadcast.rs` - Sistema de Broadcast de Mensagens
**O que faz:**
- Implementa broadcast paralelo para múltiplos nós
- Gerencia timeout e tratamento de falhas
- Coleta estatísticas de entrega
- Simula latência de rede

**Implementação atual:** Simulada - não há comunicação real de rede

### `retry.rs` - Mecanismo de Retry e Garantia de Entrega
**O que faz:**
- Sistema de retry com backoff exponencial e jitter
- Fila assíncrona para tentativas falhadas
- Estatísticas de recuperação e falhas
- Configuração flexível de parâmetros de retry

**Implementação atual:** Lógica completa mas sem persistência real

### `protocolo.rs` - Definição do Protocolo de Comunicação
**O que faz:**
- Define estruturas `Mensagem` e `TipoMensagem`
- Implementa serialização/deserialização
- Sistema básico de assinaturas com hash
- Geração de IDs únicos para mensagens

**Implementação atual:** Funcional mas com criptografia simplificada

### `rede.rs` - Gerenciamento da Topologia de Rede
**O que faz:**
- Descoberta automática de nós (seeds + gossip)
- Monitoramento contínuo via heartbeat
- Gerenciamento dinâmico da topologia
- Métricas de latência e disponibilidade

**Implementação atual:** Simulada - não há descoberta real de rede

## Funcionalidades Implementadas:
- Broadcast de hashes, provas e transações
- Sistema de retry com backoff exponencial
- Fan-out para múltiplos nós
- Garantia de entrega (simulada)
- Detecção de nós offline (simulada)
- Processamento assíncrono de mensagens
- Coleta de estatísticas e métricas

## Implementações Fictícias/Simuladas:
- **Comunicação de rede real:** Atualmente apenas simula envios
- **Descoberta de nós:** Não há integração com protocolos reais
- **Criptografia:** Assinaturas usam hash simples, não criptografia real
- **Persistência:** Dados mantidos apenas em memória
- **Falhas de rede:** Simuladas com delays e probabilidades

## O que Falta Implementar:

### Integração com Outras Camadas:
- Conexão real com a camada de consenso
- Integração com sistema de recompensas
- Comunicação com detecção de falhas

### Protocolos de Rede Reais:
- Implementação TCP/UDP para comunicação
- Protocolos de descoberta (mDNS, DHT)
- Handshake e autenticação de nós
- Compressão e otimização de mensagens

### Criptografia e Segurança:
- Assinaturas digitais reais (Ed25519, ECDSA)
- Criptografia de mensagens sensíveis
- Verificação de identidade de nós
- Prevenção contra ataques de replay

### Persistência e Confiabilidade:
- Armazenamento de estado de rede
- Recuperação após falhas
- Sincronização de estado entre nós
- Backup e restauração de configurações

### Performance e Escalabilidade:
- Pool de conexões reutilizáveis
- Balanceamento de carga
- Otimização de bandwidth
- Compressão de dados

### Observabilidade:
- Métricas detalhadas de rede
- Logs estruturados
- Tracing distribuído
- Dashboards de monitoramento

## Melhorias Futuras:

### Arquitetura:
- Implementar padrão pub/sub para eventos
- Adicionar circuit breakers para falhas
- Sistema de rate limiting
- Cache distribuído para mensagens

### Protocolos Avançados:
- Suporte a múltiplos protocolos de transporte
- Roteamento inteligente de mensagens
- Agregação de mensagens para eficiência
- Protocolos de gossip otimizados

### Segurança Avançada:
- Zero-knowledge proofs para privacidade
- Rotação automática de chaves
- Auditoria de comunicações
- Detecção de comportamento malicioso

### Escalabilidade:
- Sharding de rede
- Hierarquia de nós (super-nós)
- Otimização para redes de alta latência
- Suporte a milhares de nós simultâneos