# Mecanismo de Detecção de Falhas / Timeout

Implementa a detecção e recuperação de falhas na rede.

## Arquivos:
- `mod.rs` - Módulo principal de detecção
- `monitor.rs` - Monitoramento de nós
- `timeout.rs` - Gerenciamento de timeouts
- `recuperacao.rs` - Mecanismos de recuperação

## Funcionalidades:
- Identificação de nós offline ou lentos
- Pular nós que não processaram a tempo
- Reentrada de nós após recuperação
- Garantia de que nenhuma transação fique sem validação