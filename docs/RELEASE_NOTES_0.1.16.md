# Compactador v0.1.16

## Resumo

Esta versão corrige problemas descobertos somente quando o workflow Windows da v0.1.15 foi analisado em detalhe. A v0.1.15 não deve ser considerada validada: dois testes falharam no Windows e o shell do workflow não propagou o erro antes da publicação dos artefatos.

## Correções

A validação de caminhos agora respeita o separador nativo de `Path` no Windows. A implementação anterior tratava qualquer backslash como nome ZIP não portátil, o que rejeitava caminhos internos legítimos gerados durante a compactação de diretórios em Windows. Em hosts não Windows, o backslash continua sendo rejeitado como separador externo não portátil.

O workflow `.github/workflows/release.yml` agora executa a etapa `Validate` com `bash` e `set -euo pipefail`. Uma falha de `cargo fmt`, `cargo check`, testes ou Clippy interrompe o job e impede build/publicação. Isso corrige a falsa conclusão verde observada no v0.1.15.

A versão também contém as correções exploratórias da v0.1.15: ordenação determinística de diretórios, rejeição de colisões case-insensitive, comparação de sobreposição de paths apropriada para Windows e nomes reservados COM¹–COM³/LPT¹–LPT³.

## Evidência da falha v0.1.15

O log do workflow [`Run #32421723076`](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32421723076) registrou `test result: FAILED` para `directory_entries_are_emitted_in_sorted_order` e `supports_directory_and_multiple_selection_without_following_symlinks`, com erro de caminho `Projeto Rust\\a.txt`. Apesar disso, o shell padrão continuou até as etapas posteriores. A falha foi corrigida na v0.1.16.

## Validação local

Após a correção, passaram `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, `cargo build --workspace --release --locked`, `git diff --check` e o E2E dos binários release.

A suíte possui **40 testes** em Linux e Windows: 18 do core, 11 do container, 5 do compressor e 6 da integração Windows em memória. No Windows, o teste `accepts_windows_native_relative_path` foi executado; o teste Unix de symlink é substituído pelo teste Windows de sobreposição case-insensitive, mantendo a contagem total.

## Riscos e limitações

As janelas TOCTOU entre validar e abrir entradas, entre verificar e publicar a extração e entre ler e remover valores do Registry permanecem como riscos residuais. Temporários, CRC, hard link sem sobrescrita e rejeição de links reduzem a superfície, mas não constituem proteção absoluta contra concorrência adversarial sem APIs de handles específicas.

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C; `cargo-audit` permanece indisponível; e o workflow ainda informa um aviso não bloqueante sobre o runtime Node.js 20 de actions atuais.

## Compatibilidade e artefatos

A tag `v0.1.16` acionou o [workflow Windows #32422248254](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32422248254) em `windows-latest`. O job `Build Windows release` terminou com **success**; a etapa `Validate` executou as suítes debug e release com o shell bloqueador e todos os testes passaram.

[1]: https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32421723076 "Workflow v0.1.15 com falha de testes descoberta no log"
[2]: https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file "Microsoft Learn: Naming Files, Paths, and Namespaces"

O pacote [`Compactador-v0.1.16-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.16/Compactador-v0.1.16-windows-x86_64.zip) tem 338.693 bytes e contém `compactador-launcher.exe` e `compactador-compressor.exe`. O arquivo [`Compactador-v0.1.16-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.16/Compactador-v0.1.16-windows-x86_64.zip.sha256) tem 106 bytes; a verificação local com `sha256sum -c` retornou **OK**.

A compilação MSVC e o pacote final foram publicados somente após o workflow corrigido concluir com sucesso.

---

**Versão:** `0.1.16`
**Status:** release Windows publicado; workflow bloqueador e checksum verificados.
