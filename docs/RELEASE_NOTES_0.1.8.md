# Compactador v0.1.8

## Resumo

Esta versão registra outra revisão completa após a v0.1.7. A auditoria confrontou novamente os requisitos anteriores, o código, os fluxos reais, segurança, desempenho, compatibilidade, documentação e regressões. Foi encontrada uma falha concreta na API pública de análise: entradas inválidas podiam ser ignoradas silenciosamente em vez de falhar antes da seleção de estratégia.

## Correção

| Área | Problema | Correção |
| --- | --- | --- |
| Análise de seleção | `analyze_selection(&[])` retornava sucesso e `AnalysisAccumulator::add_file` retornava `Ok(())` quando um caminho declarado como arquivo deixava de ser arquivo regular. Isso permitia totais parciais e justificativas enganosas. | Seleção vazia agora retorna `InvalidInput`; links/reparse points retornam `Unsupported`; diretórios, tipos especiais e outros caminhos incompatíveis retornam `InvalidInput`. |
| Regressão | Não havia cobertura direta para seleção vazia ou divergência entre tipo declarado e filesystem. | Adicionados `rejects_empty_selection_analysis` e `rejects_file_entry_that_is_not_a_regular_file`. |

A correção preserva o comportamento normal de arquivos e diretórios válidos e não remove funcionalidades anteriores.

## Validação

A passagem repetida aprovou `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release.

A suíte possui 34 testes: 17 testes unitários do core, 9 testes de integração do container, 4 testes do compressor e 4 testes da integração Windows em memória. O E2E confirmou novamente ajuda, Unicode, espaços, arquivos e diretórios vazios, múltiplas entradas, os cinco níveis, Store, extração, colisões, repetição, erros e launcher fora do Windows.

## Compatibilidade, artefatos e limitações

A tag `v0.1.8` acionará o workflow Windows em `windows-latest` para confirmar a compilação MSVC e publicar os artefatos. A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C; o cancelamento validado é cooperativo pela API. O host Linux não possui target MSVC local e `cargo-audit` permanece indisponível.

Os contratos públicos reservados continuam não sendo anunciados como funcionalidades ativas: não há IPC baseado em `CompressionRequest`, store global de `OperationStatus` nem paralelismo de arquivos independentes nesta versão.
