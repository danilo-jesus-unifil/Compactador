# Compactador v0.1.11

## Resumo

Esta versão registra uma nova auditoria completa após a v0.1.10. A revisão confrontou novamente funcionamento real, arquitetura, segurança, desempenho, compatibilidade Windows, organização, dependências, documentação, integração com o Explorer e regressões.

## Segurança de caminhos Windows

A validação de componentes relativos usados na extração ZIP agora rejeita caracteres de controle e proibidos pelo Windows, componentes terminados em ponto ou espaço e nomes reservados de dispositivos como `CON`, `NUL`, `COM1` e `LPT1`, inclusive quando recebem uma extensão. Os testes unitários e de integração cobrem traversal, caminhos absolutos, separadores perigosos e esses nomes incompatíveis.

## Rollback da instalação do Registry

A instalação da integração captura os valores anteriores das entradas declaradas. Se uma escrita intermediária falhar, o manager tenta restaurar as entradas já processadas em ordem reversa; se o próprio rollback falhar, o erro de restauração é preservado. O comportamento foi coberto por backend de teste que falha no meio da instalação e confirma o retorno ao estado `NotInstalled` quando a restauração é possível.

## Cancelamento entre fases

O container verifica o token de cancelamento antes da validação, depois do início da validação, após a validação, depois do início da finalização e antes da publicação. O compressor agora possui uma regressão que cancela pelo evento `Validating` e confirma que nenhum arquivo de saída é publicado.

## Correção documental

A auditoria corrigiu a contagem histórica da v0.1.10: as notas informavam 36 testes, mas a suíte comprovada antes desta passagem possuía 35 — 18 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória.

## Validação local

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release.

A suíte possui 37 testes: 18 testes do core, 9 testes de integração do container, 5 testes do compressor e 5 testes da integração Windows em memória. O E2E confirmou ajuda, Unicode, espaços, arquivos e diretórios vazios, múltiplas entradas, cinco níveis, Store, extração, colisões, repetição, erros e launcher fora do Windows. `cargo-audit` permanece indisponível no ambiente.

## Compatibilidade, artefatos e limitações

A tag `v0.1.11` acionou o [workflow Windows](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32418733216), concluído com **success** em `windows-latest`. O job `Build Windows release` confirmou a validação e a compilação MSVC, empacotou os executáveis e publicou os artefatos.

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C; o cancelamento do pipeline é cooperativo pela API. O rollback do Registry é de melhor esforço quando o próprio backend falha durante a restauração. O host Linux não possui target MSVC local, e a proteção contra TOCTOU do diretório final de extração não é absoluta em Unix.

Os contratos públicos reservados continuam não sendo anunciados como funcionalidades ativas: não há IPC baseado em `CompressionRequest`, store global de `OperationStatus` nem paralelismo de arquivos independentes nesta versão.

## Artefatos Windows verificados

O pacote [`Compactador-v0.1.11-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.11/Compactador-v0.1.11-windows-x86_64.zip) tem 317.786 bytes e contém `compactador-launcher.exe` e `compactador-compressor.exe`. O arquivo de checksum [`Compactador-v0.1.11-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.11/Compactador-v0.1.11-windows-x86_64.zip.sha256) tem 106 bytes; a verificação local com `sha256sum -c` retornou **OK**.

## Referências

[1]: https://github.com/danilo-jesus-unifil/Compactador/actions "GitHub Actions do Compactador"
[2]: https://github.com/danilo-jesus-unifil/Compactador "Repositório do Compactador"

A publicação e a verificação dos artefatos acima foram concluídas após o workflow Windows; as limitações de validação manual permanecem explicitamente separadas e não são tratadas como cobertas pelo CI.

---

**Versão:** `0.1.11`
**Status:** release Windows publicado e checksum verificado.
