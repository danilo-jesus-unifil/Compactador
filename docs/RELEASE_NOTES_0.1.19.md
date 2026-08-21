# Compactador v0.1.19

> **Estado:** release publicada em 21 de agosto de 2026 após CI Windows aprovado; artefatos e checksum verificados.

## Correção confirmada

A auditoria autônoma reproduziu uma condição de corrida na extração: depois que o Compactador verificava que o destino não existia, outro processo podia criar esse diretório antes da publicação final. No baseline, `fs::rename(&staging, destination)` permitia que o staging substituísse um destino concorrente vazio no Linux.

A publicação de diretórios agora usa `renameat2(RENAME_NOREPLACE)` no Linux. Se o destino surgir depois da checagem, a operação falha com `File exists`, o destino concorrente permanece intacto e o staging é removido pelo caminho de limpeza. No Windows, a implementação usa `fs::rename` como primitive nativa e a semântica precisa ser confirmada no runner Windows; em plataformas diferentes de Linux e Windows, o código retorna `Unsupported` para não prometer uma garantia não comprovada.

Foi adicionada a dependência condicional `libc = "0.2"` somente para o alvo Linux. O lockfile foi atualizado com a relação direta do `compactador-core` para `libc`.

## Testes adicionados e reprodução

Foi adicionado o teste unitário `container::tests::directory_publication_does_not_replace_existing_destination`, que preserva um sentinel no destino e confirma que o staging permanece após a falha de publicação.

O reproducer `/home/ubuntu/prompt4_toc_t_race.sh` foi executado uma vez no baseline e reproduziu a falha. Depois da correção, foi executado três vezes e passou nas três, com o resultado esperado de erro `File exists` e preservação do destino.

## Follow-up do CI Windows

A primeira execução do workflow para a tag v0.1.19 parou no Clippy estrito para Windows por causa de um `return Ok(())` desnecessário no bloco condicionado ao sistema operacional. O retorno foi substituído pela expressão final `Ok(())`, sem alteração funcional. O commit corretivo é `a6087d3`; a tag foi reposicionada para o commit final `81beb35` antes da execução aprovada do workflow `#32505778882`.

## Validação local

| Verificação | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --workspace --locked` | Aprovado |
| `cargo test --workspace --locked` | 43 testes aprovados |
| `cargo test --workspace --release --locked` | 43 testes aprovados |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Aprovado |
| `cargo build --workspace --release --locked` | Aprovado |
| `cargo tree -d --locked` | Nenhuma duplicidade exibida |
| `cargo metadata --locked` | Aprovado |
| `git diff --check` | Aprovado |
| `cargo check --workspace --target x86_64-pc-windows-gnu --locked` | Aprovado como cross-check de compilação |
| E2E CLI local | 10/10 cenários aprovados |
| Reproducer concorrente pós-correção | 3/3 repetições aprovadas |
| CI Windows `#32505778882` | Aprovado; cargo-audit, validação, build, empacotamento e publicação concluídos |
| Artefato Windows | ZIP x86_64, 357.624 bytes; checksum `bc3adaaeef0a017ffd50b49cf94f09ea9fa38cb4e101a7a9b322729135ef3e09` |

Cross-compilação não é execução nativa. Ainda não foram validados interativamente o Explorer, o Registry real, Windows 10/11, UNC, caminhos longos, seleção extensa, Ctrl+C real e filesystems de rede.

## Segurança e dependências

O `cargo-audit` 0.22.2 não foi concluído localmente porque a instalação ficou bloqueada por timeouts de crates.io; o workflow Windows instalou a versão declarada e concluiu o gate RustSec com sucesso. O resultado do CI é a evidência autoritativa desta release, sem ampliar o claim para ambientes diferentes do lockfile validado.

A decisão de manter `zip` 0.6.6, registrada nas notas da v0.1.18, não foi alterada nesta correção. Nenhum segredo foi adicionado ao código, logs, commit ou artefato.

## Entrega

A versão declarada do workspace foi entregue como `0.1.19`. Os commits `34da600`, `74d5ffc`, `a6087d3` e `81beb35` foram enviados ao branch `main`; a tag anotada `v0.1.19` foi publicada no commit `81beb35`, e a release pública foi criada após o CI Windows aprovado. O ZIP e o sidecar SHA-256 foram baixados e verificados localmente.
