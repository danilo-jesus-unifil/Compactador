# Compactador v0.1.7

## Resumo

Esta versão registra uma nova revisão completa após a publicação da v0.1.6. A auditoria confrontou novamente os requisitos anteriores, código, arquitetura, segurança, desempenho, CLI, documentação, dependências, contratos públicos, regressões e funcionamento real. Foram feitas somente alterações justificadas por inconsistências confirmadas.

## Correções

| Área | Correção |
| --- | --- |
| Heurística | Removida a condição que alterava apenas a justificativa com base em memória disponível quando o algoritmo e o pipeline permaneciam iguais. O seletor agora é determinístico e não simula tuning de recursos. |
| Recursos | `ResourceProfile` foi documentado como conjunto de hints reservado para futura evolução de agendamento. A implementação atual continua single-threaded, com `parallel = false`. |
| Contratos operacionais | `OperationStatus` foi documentado como contrato público reservado para consumidores futuros; o CLI atual usa eventos de progresso e não mantém store global. |
| Protocolo | `launcher_protocol` foi documentado como contrato passivo para evolução futura. Ele não é IPC, não serializa requisições em produção e não substitui o parser CLI atual. |
| Documentação | O README foi alinhado à publicação sem sobrescrita e deixou de repetir a política de não sobrescrita. |
| Regressão | Foi incluído teste que compara perfis de recursos baixo e alto e confirma a mesma estratégia, justificativa e ausência de paralelismo. |

## Validação

A nova passagem aprovou `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release.

A suíte final desta passagem possui 32 testes: 15 testes unitários do core, 9 testes de integração do container, 4 testes do compressor e 4 testes da integração Windows em memória. O E2E confirmou novamente ajuda, Unicode, espaços, arquivos e diretórios vazios, múltiplas entradas, os cinco níveis, Store, extração, colisões, repetição, erros e launcher fora do Windows.

## Compatibilidade e limitações

A tag `v0.1.7` acionará o workflow Windows em `windows-latest` para validar as mudanças específicas de compilação MSVC. A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C; o cancelamento validado é cooperativo pela API. O host Linux não possui target MSVC local e `cargo-audit` permanece indisponível.

Os contratos públicos reservados não são anunciados como funcionalidades ativas: não há IPC baseado em `CompressionRequest`, store global de `OperationStatus` nem paralelismo de arquivos independentes nesta versão.
