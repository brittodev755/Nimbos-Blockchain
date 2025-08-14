# Camada de Consenso

Esta pasta contém toda a implementação do mecanismo de consenso da blockchain Nimbos.

## Estrutura:

- `mod.rs` - Módulo principal que exporta todos os submódulos
- `registro/` - Camada de Registro/Commit (commitment anônimo)
- `reveal/` - Camada de Reveal/Verificação (validação de commitments)
- `ordenacao/` - Camada de Ordenação Determinística (fila baseada em hash)
- `merkle/` - Prova de Inclusão com Merkle Tree
- `processamento/` - Camada de Processamento Rotativo
- `validacao/` - Camada de Validação Distribuída
- `tipos.rs` - Tipos e estruturas comuns do consenso