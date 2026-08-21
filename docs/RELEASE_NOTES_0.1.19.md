# Compactador v0.1.19 — preparação local

> **Estado:** preparada localmente; a tag e a release ainda não foram publicadas.

## Correção confirmada

A auditoria autônoma reproduziu uma condição de corrida na extração: depois que o Compactador verificava que o destino não existia, outro processo podia criar esse diretório antes da publicação final. No baseline, `fs::rename(&staging, destination)` permitia que o staging substituísse um destino concorrente vazio no Linux.

A publicação de diretórios agora usa `renameat2(RENAME_NOREPLACE)` no Linux. Se o destino surgir depois da checagem, a operação falha com `File exists`, o destino concorrente permanece intacto e o staging é removido pelo caminho de limpeza. No Windows, a implementação usa `fs::rename` como primitive nativa e a semântica precisa ser confirmada no runner Windows; em plataformas diferentes de Linux e Windows, o código retorna `Unsupported` para não prometer uma garantia não comprovada.

Foi adicionada a dependência condicional `libc = "0.2"` somente para o alvo Linux. O lockfile foi atualizado com a relação direta do `compactador-core` para `libc`.

## Testes adicionados e reprodução

Foi adicionado o teste unitário `container::tests::directory_publication_does_not_replace_existing_destination`, que preserva um sentinel no destino e confirma que o staging permanece após a falha de publicação.

O reproducer `/home/ubuntu/prompt4_toc_t_race.sh` foi executado uma vez no baseline e reproduziu a falha. Depois da correção, foi executado três vezes e passou nas três, com o resultado esperado de erro `File exists` e preservação do destino.

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

Cross-compilação não é execução nativa. Ainda não foram validados interativamente o Explorer, o Registry real, Windows 10/11, UNC, caminhos longos, seleção extensa, Ctrl+C real e filesystems de rede.

## Segurança e dependências

O `cargo-audit` 0.22.2 não foi concluído nesta preparação porque a instalação ficou bloqueada por timeouts de transferência em crates.io. Não há claim de “zero vulnerabilidades” baseado nesta execução local. O workflow versionado continua sendo o caminho para executar o gate RustSec em ambiente de CI.

A decisão de manter `zip` 0.6.6, registrada nas notas da v0.1.18, não foi alterada nesta correção. Nenhum segredo foi adicionado ao código, logs, commit ou artefato.

## Entrega pendente

A versão declarada do workspace foi preparada como `0.1.19`, mas não houve commit, tag anotada, push ou release publicados nesta etapa. A publicação deve ocorrer somente após revisão humana, confirmação do CI apropriado e validação da semântica Windows específica da operação.
