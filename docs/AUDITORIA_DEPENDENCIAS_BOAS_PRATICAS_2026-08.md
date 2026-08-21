# Auditoria de dependências e boas práticas — Compactador

## Escopo

Workspace Rust do Compactador, pipeline ZIP, CLI, integração Windows, workflow de release e `Cargo.lock`.

## Resultado

A auditoria corretiva foi realizada com pesquisa em fontes primárias, inspeção do código, migração experimental de dependências, testes de regressão, auditoria RustSec e repetição da matriz end-to-end. Foram corrigidos os gaps que eram tecnicamente tratáveis sem enfraquecer as garantias já existentes: o CLI agora conecta Ctrl+C ao cancelamento cooperativo; a extração também pode ser cancelada durante a leitura e nunca publica o destino quando o cancelamento ocorre; `winreg` foi atualizado para a versão estável 0.56; e o workflow deixou de usar actions baseadas no runtime Node.js 20, passando para as versões Node.js 24.

A migração experimental do crate `zip` para a versão estável 8.6.0 foi deliberadamente revertida. O motivo não foi uma falha de compilação sem solução, mas uma mudança de semântica relevante: o parser moderno armazena o diretório central em um `IndexMap` indexado pelo nome e substitui entradas com o mesmo nome antes de expô-las por `ZipArchive`. Isso impede a política do Compactador de detectar todos os archives maliciosos com nomes duplicados. A versão 0.6.6 usada pelo projeto está fora da faixa afetada pelo advisory específico de `ZipArchive::extract`, e o Compactador não chama essa rotina; sua extração é própria e conserva validações adicionais.

A suíte Rust contém **42 testes aprovados**, a matriz externa contém **63 de 63 cenários aprovados**, o `cargo-audit` encontrou **zero vulnerabilidades**, não há dependências duplicadas no `cargo tree -d`, e o código compila com o MSRV declarado Rust 1.75. A compilação de `winreg`, os testes condicionais e a publicação foram confirmados pelo workflow; a experiência interativa do handler de console, do Explorer e do Registry real continua fora desta validação automatizada.

## Fontes

A análise de versões e segurança usa as páginas oficiais dos crates, a base RustSec, a documentação do crate de Ctrl+C, o blog oficial do Rust Secure Code Working Group e os repositórios oficiais das GitHub Actions. O procedimento segue a recomendação do Rust para executar `cargo audit` contra o `Cargo.lock` [5].

> “`cargo audit` checks your project's dependencies for known security vulnerabilities.” — Rust Secure Code Working Group [5]

As fontes relevantes e as decisões derivadas delas estão resumidas na tabela abaixo.

| Componente | Versão anterior ou estado | Versão pesquisada/atual | Decisão | Justificativa |
| --- | --- | --- | --- | --- |
| `zip` | 0.6.6 | 8.6.0 estável | **Manter 0.6.6** | A migração altera a visibilidade de duplicidades; a versão 0.6.6 não está na faixa afetada pelo advisory específico da rotina `extract`, e o projeto usa extração própria. |
| `flate2` | 1.1.9 resolvida | 1.1.9 | **Manter** | Já é a versão publicada resolvida no lockfile; o backend padrão `miniz_oxide` usa Rust seguro e atende à portabilidade. |
| `crc32fast` | 1.5.0 resolvida | 1.5.0 | **Manter** | Já é a versão publicada resolvida no lockfile; não foi encontrado motivo técnico para trocar. |
| `winreg` | 0.52 | 0.56.0 | **Atualizar** | A versão atual mantém MSRV compatível e é restrita ao alvo Windows; será validada no CI Windows com os testes de Registry em memória e compilação MSVC. |
| `ctrlc` | inexistente no binário | 3.5.2 | **Adicionar** | A API oficial fornece handler cross-platform em thread dedicada, adequado para marcar o `AtomicBool` do token sem executar I/O no callback. |
| `actions/checkout` | v4 | v5 | **Atualizar** | A versão oficial usa Node.js 24 e requer runner compatível. |
| `softprops/action-gh-release` | v2 | v3 | **Atualizar** | A documentação oficial indica v3 para o runtime Node.js 24; v2 usa Node.js 20 depreciado. |
| `cargo-audit` | ausente | 0.22.2 no workflow | **Adicionar ao gate** | A auditoria do Cargo.lock passa a bloquear releases com vulnerabilidades ou warnings configurados. |

## Advisory RustSec e decisão sobre `zip`

O advisory [RUSTSEC-2025-0168 / CVE-2025-29787] [2] descreve uma falha de canonicalização na rotina `zip::read::ZipArchive::extract`: um link simbólico criado por uma entrada anterior poderia ser usado por uma entrada posterior para escrever fora do destino. A base RustSec classifica como HIGH a faixa `>=1.3.0, <2.3.0`, indica correção em `>=2.3.0` e marca versões `<1.3.0` como não afetadas para a função descrita.

O Compactador usa `zip` 0.6.6 e não chama `ZipArchive::extract`. A extração própria valida cada caminho, rejeita links e reparse points, verifica CRC, impõe limites de tamanho e razão de expansão, usa staging e publica sem sobrescrever. Essas garantias próprias continuam necessárias mesmo quando o crate é atualizado.

A versão estável atual pesquisada no crates.io é `zip` 8.6.0 [1]. A tentativa de migração foi feita em branch isolado com Rust 1.88. A API exigiu mudanças em `SimpleFileOptions`, `ZipFile` genérico, `data_start()` opcional e erros não exaustivos. Depois dessas mudanças, três regressões revelaram uma incompatibilidade mais importante: o writer moderno não permite gerar duplicidades para os fixtures, e o leitor moderno deduplica nomes no mapa interno de metadados. Como consequência, archives com duas entradas `same.txt` ou duas entradas `folder/` não chegavam à política própria como duas entries distintas.

A correção escolhida foi **não adotar a atualização major neste ciclo**. Não é boa prática trocar uma dependência de segurança por uma versão numericamente mais nova se a troca remove uma propriedade de validação necessária. Uma futura migração do `zip` deverá incluir uma estratégia explícita para ler e rejeitar duplicidades do diretório central antes da deduplicação do mapa interno, além de validação em Linux e Windows.

## Correções implementadas

### Cancelamento de descompactação

Foi adicionada `extract_archive_with_cancel`, mantendo `extract_archive` como wrapper compatível. O callback de cancelamento é consultado antes da operação, entre entries, durante a cópia de cada entry e imediatamente antes da publicação do staging. Se a operação for cancelada, o staging é removido pelo caminho de erro e o destino final não é criado.

O teste `extraction_cancellation_discards_staging_without_publishing_destination` confirma que uma operação cancelada retorna `CoreError::Cancelled`, não deixa destino parcial e executa o callback durante a leitura.

### Handler de Ctrl+C no CLI

O binário `compactador-compressor` agora cria um `CancellationToken` no início do `main`, registra `ctrlc::set_handler` e captura o token por clone. O callback executa somente `signal_token.cancel()`. Não há escrita de arquivo, impressão ou limpeza dentro do callback; essas ações permanecem no fluxo normal e seguro da operação.

A mesma tokenização é usada tanto na compactação quanto na descompactação. O código retorna status 130 e informa o cancelamento ao usuário, enquanto os temporários são descartados pelo pipeline.

### Atualização de `winreg`

A dependência Windows foi atualizada de `winreg` 0.52 para 0.56.0 [4]. A API usada pelo backend — `RegKey::predef`, abertura, criação, escrita, exclusão de valores e poda de chaves vazias — permaneceu compatível, e a compilação MSVC com os testes condicionais foi confirmada pelo job Windows da release.

### Auditoria de dependências no workflow

O workflow instala `cargo-audit` 0.22.2 com `--locked` e executa `cargo audit` antes de `cargo check`, testes e Clippy. A execução local com a base RustSec carregou 1.225 advisories, examinou 25 dependências do lockfile e encontrou `vulnerabilities.found = false`, com contagem zero e nenhum warning.

### Actions do GitHub

O workflow foi atualizado para `actions/checkout@v5` e `softprops/action-gh-release@v3`. A documentação do checkout informa que v5 usa Node.js 24 e requer runner `v2.327.1` ou superior [6]. O repositório da action de release recomenda v3 para Node.js 24, enquanto a linha v2 usava o runtime Node.js 20 depreciado [7]. O job `windows-latest` confirmou a validação, o build e a publicação da v0.1.18.

### Fixtures de duplicidade

O writer do `zip` moderno passou a rejeitar duplicidades durante a criação, o que tornou inadequados os fixtures que dependiam de `start_file` duas vezes com o mesmo nome. Os testes agora constroem um ZIP armazenado mínimo com dois headers locais e dois registros do diretório central. Assim, a regressão continua representando um archive recebido de terceiros e verifica a defesa do produto, não apenas uma restrição do writer utilizado para fabricar o fixture.

## Validação executada

A matriz de validação foi repetida depois das correções. Os resultados são os seguintes.

| Gate ou cenário | Resultado |
| --- | --- |
| Rust 1.75 — `cargo check --workspace --locked` | Aprovado |
| Rust 1.75 — `cargo test --workspace --locked` | Aprovado |
| Rust 1.88 — `cargo fmt --all -- --check` | Aprovado |
| Rust 1.88 — `cargo check --workspace --locked` | Aprovado |
| Rust 1.88 — testes debug | 42/42 aprovados |
| Rust 1.88 — testes release | 42/42 aprovados |
| Rust 1.88 — Clippy com `-D warnings` | Aprovado |
| Rust 1.88 — build release | Aprovado |
| `cargo tree -d --locked` | Nenhuma dependência duplicada exibida |
| `cargo metadata --locked` | Aprovado |
| `git diff --check` | Aprovado |
| `cargo audit -D warnings` | 0 vulnerabilidades; 0 warnings |
| Matriz end-to-end externa | 63/63 aprovados |

A matriz externa cobriu ajuda e CLI, Unicode e espaços, arquivos vazios, diretórios vazios, múltipla seleção, os cinco níveis, Store/Deflate, round-trip byte a byte, nomes automáticos repetidos, destino existente, saída sobreposta, traversal, caminhos drive/UNC/absolutos, nomes reservados Windows, backslash, duplicidades, colisões case-insensitive, conflito hierárquico, razão de expansão, CRC corrompido, archive truncado, arquivo não ZIP, argumentos inválidos, launcher fora do Windows, permissões, symlinks, muitos arquivos, arquivo grande, caminhos relativos e nome não Unicode.

## Falhas encontradas durante a própria correção

A primeira compilação com `zip` 8.6.0 falhou por diferenças de API. Após a adaptação, os testes de duplicidade e paths falharam porque a semântica do crate moderno não preservava todas as entries e porque `enclosed_name()` sanitizava formas que o Compactador precisa rejeitar conservadoramente. Esses problemas foram resolvidos revertendo a migração major, restaurando `FileOptions` e validando o nome bruto da entry com a política própria de `safe_relative_path`.

Na validação seguinte, o Clippy encontrou um import não utilizado no novo teste de extração cancelável. O import foi removido e os gates foram executados novamente. Não há falhas conhecidas pendentes no código local ao final desta passagem.

## Riscos que permanecem documentados

A atualização não elimina janelas TOCTOU absolutas entre validar e abrir arquivos, verificar e publicar o destino, e ler e remover valores do Registry. A eliminação completa exigiria APIs de handles e primitivas específicas de Windows e Unix, com uma matriz de testes própria.

Também permanecem fora da validação local a experiência visual do menu no Explorer, a instalação e remoção contra o Registry real, versões Windows 10 e 11, caminhos UNC e caminhos longos em ambiente Windows real. O workflow `windows-latest` confirmou a compilação de `winreg` 0.56, os testes condicionais e a publicação do artefato; o comportamento interativo do handler de Ctrl+C ainda não foi exercitado por sinal real.

## Referências

[1]: https://crates.io/crates/zip "zip — crates.io"

[2]: https://rustsec.org/advisories/RUSTSEC-2025-0168.html "RUSTSEC-2025-0168: zip path canonicalization vulnerability"

[3]: https://crates.io/crates/flate2 "flate2 — crates.io"

[4]: https://crates.io/crates/winreg "winreg — crates.io"

[5]: https://blog.rust-lang.org/inside-rust/2023/09/04/keeping-secure-with-cargo-audit-0.18/ "Keeping Rust projects secure with cargo-audit — Inside Rust"

[6]: https://github.com/actions/checkout "actions/checkout — GitHub"

[7]: https://github.com/softprops/action-gh-release "softprops/action-gh-release — GitHub"

[8]: https://docs.rs/ctrlc/latest/ctrlc/ "ctrlc — docs.rs"
