# Compactador v0.1.2

## Correções finais

Esta versão corrige os últimos avisos tratados como erro pelo Clippy do runner Windows: o cálculo de progresso usa divisão verificada, e o resultado da operação reporta seu identificador e sua estratégia. As correções foram feitas após a segunda execução do workflow Windows e validadas localmente.

## Validação

A quality gate local passou por `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --release` e `git diff --check`. O workflow associado à tag valida novamente o workspace em Windows, constrói os dois executáveis, empacota-os e gera checksum SHA-256.

## Limitações conhecidas

A aparência e o comportamento final do menu do Explorer ainda devem ser confirmados em Windows 10 e Windows 11 com a checklist do projeto, especialmente para seleções múltiplas extensas, caminhos UNC, caminhos longos, reinicialização do Explorer e remoção após reparo. O workflow de release é o responsável por publicar os artefatos `.exe` Windows; este ambiente Linux não publica binários incompatíveis como se fossem Windows.

## Documentação

Consulte o [`README.md`](https://github.com/danilo-jesus-unifil/Compactador/blob/main/README.md), a [auditoria final](https://github.com/danilo-jesus-unifil/Compactador/blob/main/docs/AUDITORIA_FINAL.md), a [checklist Windows](https://github.com/danilo-jesus-unifil/Compactador/blob/main/docs/COMPATIBILIDADE_WINDOWS.md) e as decisões de [integração](https://github.com/danilo-jesus-unifil/Compactador/blob/main/docs/DECISAO_INTEGRACAO_WINDOWS.md) e [container](https://github.com/danilo-jesus-unifil/Compactador/blob/main/docs/DECISAO_CONTAINER_ZIP.md).
