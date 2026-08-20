# Compactador v0.1.9

## Resumo

Esta versão registra uma nova auditoria completa após a v0.1.8. A revisão confrontou novamente requisitos anteriores, funcionamento real, arquitetura, segurança, desempenho, compatibilidade Windows, organização, dependências, documentação e regressões.

## Correção documental

A decisão arquitetural do container ainda descrevia a publicação de cada arquivo extraído como `rename`, embora o código utilize hard link no mesmo diretório para publicação sem sobrescrita. A redação foi corrigida para refletir a política efetiva: o arquivo é escrito em temporário, sincronizado, validado por CRC e publicado sem substituir um destino existente.

Esta passagem não alterou o comportamento funcional do programa. Nenhuma funcionalidade correta foi removida ou substituída; somente a documentação da decisão de segurança foi alinhada ao código real.

## Validação

A auditoria repetida aprovou `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release.

A suíte possui 34 testes: 17 testes unitários do core, 9 testes de integração do container, 4 testes do compressor e 4 testes da integração Windows em memória. O E2E confirmou novamente ajuda, Unicode, espaços, arquivos e diretórios vazios, múltiplas entradas, os cinco níveis, Store, extração, colisões, repetição, erros e launcher fora do Windows.

## Compatibilidade, artefatos e limitações

A tag `v0.1.9` acionou o [workflow Windows](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32416149877), que terminou com sucesso em `windows-latest`, confirmou a compilação MSVC e publicou os artefatos. O pacote é [`Compactador-v0.1.9-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.9/Compactador-v0.1.9-windows-x86_64.zip), acompanhado de [`Compactador-v0.1.9-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.9/Compactador-v0.1.9-windows-x86_64.zip.sha256); o checksum foi verificado localmente. A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C; o cancelamento validado é cooperativo pela API. O host Linux não possui target MSVC local e `cargo-audit` permanece indisponível.

Os contratos públicos reservados continuam não sendo anunciados como funcionalidades ativas: não há IPC baseado em `CompressionRequest`, store global de `OperationStatus` nem paralelismo de arquivos independentes nesta versão.
