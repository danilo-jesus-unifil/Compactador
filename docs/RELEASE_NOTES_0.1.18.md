# Compactador v0.1.18

## Resumo

Esta versão aplica uma auditoria corretiva de dependências, segurança e boas práticas ao workspace Rust. O ciclo corrige o gap de Ctrl+C no CLI, adiciona cancelamento durante a descompactação, atualiza `winreg`, elimina o aviso operacional do runtime Node.js 20 no workflow e incorpora `cargo-audit` ao gate de release.

## Correções e melhorias

A descompactação agora aceita cancelamento cooperativo durante a leitura de entries e antes da publicação. O staging é removido quando a operação é cancelada, e o destino final não é publicado parcialmente. O compressor CLI registra um handler cross-platform usando `ctrlc` 3.5.2; o callback apenas marca o token atômico e não executa I/O.

A integração Windows atualiza `winreg` de 0.52 para 0.56. A compilação MSVC e os testes condicionais foram confirmados pelo workflow Windows da release.

O workflow atualiza `actions/checkout` para v5 e `softprops/action-gh-release` para v3, versões compatíveis com Node.js 24. A validação passa a instalar `cargo-audit` 0.22.2 e a auditar o lockfile antes dos gates de compilação e testes.

Os testes de duplicidade ZIP usam agora fixtures armazenados com headers centrais duplicados. Isso evita depender de um writer que rejeita a duplicidade durante a criação e preserva a regressão contra archives externos malformados ou maliciosos.

## Decisão sobre o crate `zip`

A versão estável pesquisada do crate `zip` é 8.6.0, mas ela não foi adotada neste ciclo. A migração experimental mostrou que o leitor moderno deduplica nomes no mapa interno de metadados antes da API pública, impedindo que o Compactador detecte todas as entradas duplicadas. A política de segurança existente é mais importante que uma atualização major automática; por isso, `zip` permanece em 0.6.6.

O advisory RustSec RUSTSEC-2025-0168 foi analisado. Ele afeta a rotina `ZipArchive::extract` nas versões `>=1.3.0,<2.3.0`; o Compactador usa a versão 0.6.6, não chama essa rotina e mantém uma extração própria com validação de caminhos, links, CRC, limites, staging e publicação sem sobrescrita.

## Validação local

| Verificação | Resultado |
| --- | --- |
| Rust 1.75 — check locked | Aprovado |
| Rust 1.75 — testes locked | 42/42 aprovados |
| Rust 1.88 — fmt/check | Aprovado |
| Rust 1.88 — testes debug/release | 42/42 aprovados |
| Rust 1.88 — Clippy estrito | Aprovado |
| Rust 1.88 — build release | Aprovado |
| `cargo tree -d` | Nenhuma duplicidade exibida |
| `cargo metadata --locked` | Aprovado |
| `cargo audit -D warnings` | 0 vulnerabilidades; 0 warnings |
| Matriz externa de cenários | 63/63 aprovados |

## Validações que exigem Windows interativo

O workflow `windows-latest` já confirmou a compilação de `winreg` 0.56, os testes condicionais de paths e Registro em memória, a construção dos dois executáveis MSVC que incluem o handler de Ctrl+C, o empacotamento e o checksum. A recepção interativa de um Ctrl+C real, a validação visual do Explorer, do Registry real, do Windows 10/11, de UNC e de caminhos longos continuam exigindo um Windows interativo.

## Documentação

O relatório técnico completo está em `docs/AUDITORIA_DEPENDENCIAS_BOAS_PRATICAS_2026-08.md`. O histórico consolidado está em `docs/AUDITORIA_REVISAO_2026-08.md`.

## Resultado do CI Windows

O workflow [#32432217654](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32432217654) concluiu com `success` em `windows-latest`. Passaram a instalação do toolchain, a instalação do cargo-audit, a validação, o build release, o empacotamento e a publicação.

A release [Compactador v0.1.18](https://github.com/danilo-jesus-unifil/Compactador/releases/tag/v0.1.18) publicou `Compactador-v0.1.18-windows-x86_64.zip` com 356.432 bytes e o sidecar SHA-256 com 106 bytes. A verificação baixada retornou `OK`. O digest SHA-256 do ZIP publicado é `ef28034ca17a997e05080c79c6aed1430d6ecdb8dff3ec32de2393a5269a8287`. O pacote contém `compactador-compressor.exe`, `compactador-launcher.exe`, `README.md`, `LICENSE` e `CHANGELOG.md`.

## Referências

[1]: https://crates.io/crates/zip "zip — crates.io"

[2]: https://rustsec.org/advisories/RUSTSEC-2025-0168.html "RUSTSEC-2025-0168 — RustSec"

[3]: https://docs.rs/ctrlc/latest/ctrlc/ "ctrlc — docs.rs"

[4]: https://crates.io/crates/winreg "winreg — crates.io"

[5]: https://blog.rust-lang.org/inside-rust/2023/09/04/keeping-secure-with-cargo-audit-0.18/ "cargo-audit — Inside Rust"

[6]: https://github.com/actions/checkout "actions/checkout — GitHub"

[7]: https://github.com/softprops/action-gh-release "softprops/action-gh-release — GitHub"
