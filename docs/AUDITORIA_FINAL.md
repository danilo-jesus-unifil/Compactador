# Auditoria final — versão 0.1.3

## Resultado

A auditoria funcional e arquitetural disponível no ambiente foi concluída antes da publicação da versão corrigida. As primeiras execuções do workflow Windows encontraram problemas específicos de plataforma e de versões mais novas do Clippy; todos foram corrigidos antes da publicação da v0.1.3. O working tree estava limpo, o workspace foi compilado em modo de desenvolvimento e release, e as verificações Rust passaram sem warnings do Clippy configurado como erro.

| Verificação | Resultado | Observação |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Aprovado | Código formatado |
| `cargo check --workspace` | Aprovado | Todos os crates compilam |
| `cargo test --workspace` | Aprovado | 17 testes unitários executados no workspace |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Aprovado | Nenhum lint permitido ficou pendente |
| `cargo build --workspace --release` | Aprovado | Builds Linux otimizados dos dois binários |
| `cargo tree -d` | Aprovado | Nenhum grupo duplicado foi reportado |
| `git diff --check` | Aprovado | Nenhum erro de whitespace |
| `git status --porcelain` | Aprovado | Working tree limpo antes da publicação |
| `cargo audit` | Não executado | A ferramenta não está instalada no ambiente |
| Workflow Windows v0.1.3 | Será executado após a tag | Responsável por confirmar compilação MSVC e anexar os executáveis Windows |
| Windows 10/11 real | Pendente | O ambiente disponível é Linux |

## Auditoria funcional

A seleção aceita arquivos, pastas e múltiplas entradas preservando caminhos com espaços e Unicode. A análise usa extensões e amostras limitadas, a decisão do seletor é explicável, e o container ZIP padrão suporta Deflate, CRC, múltiplas entradas e extração. O pipeline usa temporário, sincronização, validação e renomeação final. Os testes cobrem arquivo Unicode, diretório heterogêneo, corrupção, traversal, estratégia conservadora, cancelamento, instalação idempotente e remoção restrita.

## Auditoria de segurança

A extração rejeita caminhos absolutos, componentes pai, letras de unidade, UNC e separadores invertidos não portáveis. A compactação não segue links simbólicos. Limites de entrada, quantidade de itens e bytes expandidos são aplicados. O launcher registra apenas recursos de uma definição centralizada, e o backend em memória permite validar o fluxo sem tocar no Registro real.

O próximo nível de auditoria deve ser executado em Windows com foco em quoting efetivo do verbo estático, seleções grandes, caminhos UNC, caminhos longos, permissões e comportamento após reinicialização do Explorer. A ausência dessa validação impede declarar compatibilidade de produção, mas não impede a publicação do código-fonte versionado com a limitação explícita.

## Auditoria arquitetural e de desempenho

Os crates mantêm o domínio e o container fora do ponto de entrada do menu, e o launcher não depende da lógica de compressão para construir o menu. A operação usa buffers limitados e não carrega arquivos inteiros em memória. A análise de diretórios é limitada a uma amostra de arquivos para decisões em escala; a compactação percorre os dados em streaming. O paralelismo de múltiplos arquivos ainda não foi ativado, pois a primeira versão prioriza previsibilidade e não possui benchmark Windows que demonstre ganho real.

## Política de artefatos

O build release produzido neste ambiente contém binários Linux chamados `compactador-launcher` e `compactador-compressor`. Eles não serão anexados ao release como se fossem executáveis Windows. O release publicado conterá o código-fonte e as notas com a limitação de validação; a geração de `.exe` deve ocorrer em CI ou em uma máquina Windows após a checklist de compatibilidade.

## Referências

[1]: https://doc.rust-lang.org/cargo/commands/cargo-test.html "The Cargo Book: cargo test"
[2]: https://doc.rust-lang.org/cargo/commands/cargo-clippy.html "The Cargo Book: cargo clippy"
[3]: https://rustsec.org/ "RustSec Advisory Database"
