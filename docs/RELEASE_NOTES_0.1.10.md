# Compactador v0.1.10

## Resumo

Esta versão registra uma nova auditoria completa do projeto após a v0.1.9. A revisão confrontou novamente funcionamento real, arquitetura, segurança, desempenho, compatibilidade Windows, organização, dependências, documentação e regressões.

## Correções de análise

A análise de diretórios grandes agora separa a contagem total de arquivos descobertos do número de arquivos usados na classificação amostral. Seleções com mais de 4.096 arquivos continuam marcadas como `sampled`, acumulam o tamanho total e reportam a quantidade real de arquivos, sem permitir que a heurística trate uma análise parcial como se cobrisse todo o conteúdo.

Foi adicionado um teste de regressão que cria 4.097 arquivos e verifica contagem, tamanho total, amostragem e política conservadora para o campo `already_compressed`.

## Correções de cancelamento

O pipeline operacional agora informa explicitamente a fase `Cancelled` quando o cancelamento ocorre antes da próxima etapa ou durante o streaming. O evento mantém o progresso conhecido quando disponível e continua acompanhado do erro `Cancelled`; a política de temporários sem publicação parcial permanece preservada.

Foram adicionados testes que verificam a emissão observável da fase de cancelamento nos dois caminhos.

## Validação

A auditoria passou `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release.

A suíte possui 35 testes: 18 testes do core, 9 testes de integração do container, 4 testes do compressor e 4 testes da integração Windows em memória. O E2E confirmou novamente ajuda, Unicode, espaços, arquivos e diretórios vazios, múltiplas entradas, os cinco níveis, Store, extração, colisões, repetição, erros e launcher fora do Windows. `cargo-audit` permanece indisponível no ambiente.

## Compatibilidade, artefatos e limitações

A tag `v0.1.10` acionou o [workflow Windows](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32417679053), concluído com **success** em `windows-latest`. O job `Build Windows release` confirmou a validação e a compilação MSVC, empacotou os executáveis e publicou os artefatos.

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C; o cancelamento exposto pelo pipeline é cooperativo pela API. O host Linux não possui target MSVC local, e a proteção contra TOCTOU do diretório final de extração não é absoluta em Unix.

Os contratos públicos reservados continuam não sendo anunciados como funcionalidades ativas: não há IPC baseado em `CompressionRequest`, store global de `OperationStatus` nem paralelismo de arquivos independentes nesta versão.

## Artefatos Windows verificados

O pacote [`Compactador-v0.1.10-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.10/Compactador-v0.1.10-windows-x86_64.zip) tem 314.906 bytes e contém `compactador-launcher.exe` e `compactador-compressor.exe`. O arquivo de checksum [`Compactador-v0.1.10-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.10/Compactador-v0.1.10-windows-x86_64.zip.sha256) tem 106 bytes; a verificação local com `sha256sum -c` retornou **OK**.

## Referências

[1]: https://github.com/danilo-jesus-unifil/Compactador/actions "GitHub Actions do Compactador"
[2]: https://github.com/danilo-jesus-unifil/Compactador "Repositório do Compactador"

A publicação e a verificação dos artefatos acima foram concluídas após o workflow Windows; as limitações de validação manual permanecem explicitamente separadas e não são tratadas como cobertas pelo CI.

## Histórico da auditoria

A implementação, os testes e a revisão foram realizados sobre o código fonte do repositório. As limitações de plataforma permanecem explicitamente registradas até a validação pós-CI em Windows.

---

**Versão:** `0.1.10`
**Status:** release Windows publicado e checksum verificado.
