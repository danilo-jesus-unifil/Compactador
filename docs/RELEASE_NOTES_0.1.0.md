# Compactador v0.1.0

## Destaques

Esta primeira versão entrega a fundação modular Rust, o Launcher/Manager com instalação, verificação, reparo e remoção idempotentes, a definição centralizada da integração por menu estático em cascata, parsing seguro de arquivos e pastas, análise amostral, seletor heurístico explicável, catálogo Deflate/Store, container ZIP padrão, compressão e extração em streaming, CRC, proteção contra traversal, escrita temporária, progresso por fases e cancelamento cooperativo.

## Validação

O release candidate passou por `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --release`, `cargo tree -d` e `git diff --check`. Foram executados 17 testes unitários no workspace.

## Limitações conhecidas

A validação executada neste ambiente ocorreu em Linux. A integração efetiva com o Registro e a experiência do Windows Explorer precisam ser confirmadas em Windows 10 e Windows 11, especialmente para seleções múltiplas extensas, caminhos UNC, caminhos longos, reinicialização do Explorer e remoção após reparo. O release não anexa binários Linux como se fossem `.exe` Windows.

O workflow de release incluído no repositório constrói os executáveis Windows em `windows-latest`, executa a suíte e publica um ZIP com checksum quando uma tag SemVer é enviada.

## Documentação

Consulte o [`README.md`](https://github.com/danilo-jesus-unifil/Compactador/blob/main/README.md), a [auditoria final](https://github.com/danilo-jesus-unifil/Compactador/blob/main/docs/AUDITORIA_FINAL.md), a [checklist Windows](https://github.com/danilo-jesus-unifil/Compactador/blob/main/docs/COMPATIBILIDADE_WINDOWS.md) e as decisões de [integração](https://github.com/danilo-jesus-unifil/Compactador/blob/main/docs/DECISAO_INTEGRACAO_WINDOWS.md) e [container](https://github.com/danilo-jesus-unifil/Compactador/blob/main/docs/DECISAO_CONTAINER_ZIP.md).
