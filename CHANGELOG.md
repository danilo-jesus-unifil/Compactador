# Changelog

## [0.1.0] — 2026-08-20

### Recursos

Esta versão inicial estabelece o workspace Rust modular com crates separados para domínio, integração com o Windows, launcher e compressor. Ela inclui instalação, verificação, reparo e remoção idempotentes por uma definição centralizada de registro; menu estático em cascata com os níveis Rápida, Baixa, Normal, Alta e Máxima; parsing de seleção com Unicode, espaços e múltiplas entradas; análise amostral; seletor heurístico explicável; catálogo Deflate/Store; container ZIP padrão; compressão e extração em streaming; CRC; proteção contra traversal; escrita temporária; progresso por fases e cancelamento cooperativo.

### Validação

A suíte foi executada em Linux com Rust/Cargo 1.75 e passou por `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` e `git diff --check`.

### Limitações conhecidas

A integração efetiva com o Registro, a aparência do menu e o comportamento de seleções múltiplas pelo Explorer ainda precisam ser validados em Windows 10 e Windows 11. O ambiente que produziu este release não é Windows e não publicou binários `.exe` não validados. A checklist de execução está em [`docs/COMPATIBILIDADE_WINDOWS.md`](docs/COMPATIBILIDADE_WINDOWS.md).
