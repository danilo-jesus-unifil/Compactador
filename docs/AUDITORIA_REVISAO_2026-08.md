# Auditoria e aprimoramento — achados iniciais

Este documento registra a primeira passagem da auditoria solicitada no prompt de revisão. Os achados foram obtidos por leitura do código atual, inspeção da estrutura, execução dos testes existentes e comparação com os requisitos anteriores. Ele será atualizado após as correções e a segunda revisão.

## Classificação inicial

| Área | Estado observado | Prioridade |
| --- | --- | --- |
| Fundação e modularização | Workspace separado e coerente; `container/mod.rs` concentra aproximadamente 517 linhas e merece divisão por responsabilidade se novas correções continuarem crescendo | Média |
| Container | ZIP padrão, validação CRC e streaming existem; o resumo é recalculado pela validação final, mas o caminho de escrita ainda precisa validar conflitos de saída dentro das pastas selecionadas | Alta |
| Seleção e análise | Unicode, múltiplas entradas e amostras existem; pastas vazias produzem perfil sem arquivos e podem ser rejeitadas pelo seletor | Alta |
| Estratégias | O seletor retorna nomes de estratégias, porém a operação sempre usa Deflate e não aplica o fallback `Store`; isso torna parte da explicação enganosa | Alta |
| Progresso | A operação emite fases, mas a compressão permanece em 0% e salta para 100% na finalização; não representa trabalho real | Alta |
| Segurança de entrada | Extração protege traversal textual e rejeita links durante a enumeração; entrada raiz por symlink e conflitos TOCTOU/output ainda precisam de política explícita | Alta |
| Extração | Arquivos individuais são temporários, mas uma falha posterior pode deixar saídas anteriores; destino existente também não possui política de não sobrescrita clara | Alta |
| Launcher | Fluxos idempotentes e backend Windows existem; resíduos de chaves vazias e testes reais em Windows ainda não estão cobertos pelo ambiente Linux | Média |
| Testes | Boa cobertura unitária de domínio; faltam testes end-to-end do binário, ZIP malicioso com traversal/ratio, colisão de saída, pasta vazia e symlink raiz | Alta |
| Dependências | `zip`, `flate2`, `crc32fast` e `winreg` são justificadas pelo fluxo; `cargo-audit` não está instalado no ambiente | Média |

## Requisitos anteriores confirmados

A base preserva os crates separados, o documento de boas práticas, o Launcher/Manager, a integração estática em cascata, níveis tipados, seleção Unicode, container ZIP padrão, CRC, validação, extração segura básica, análise limitada, seleção heurística, streaming, cancelamento cooperativo e workflow Windows com artefatos.

## Requisitos ainda incompletos ou superficiais

A integração visual do Explorer, a seleção múltipla real pelo verbo estático, caminhos UNC, reparse points e validação Windows 10/11 continuam dependentes de execução em Windows. O paralelismo de arquivos independentes ainda não foi implementado porque não há benchmark demonstrando ganho. O container não oferece ainda política completa de atualização/overwrite na extração e a operação não transforma todas as decisões do seletor em comportamento efetivo.

## Método de correção

As correções serão aplicadas em grupos pequenos: primeiro semântica do fluxo de compactação e progresso; depois validação de seleção, conflitos e extração; em seguida testes regressivos e documentação. Após cada grupo serão executados `cargo fmt`, `cargo check`, `cargo test`, `cargo clippy` e testes end-to-end apropriados. Nenhuma funcionalidade correta será removida apenas por preferência estética.

## Fontes externas consultadas

A documentação da Microsoft confirma que verbos estáticos exigem comandos devidamente citados e que cascatas podem ser construídas com `SubCommands` e `CommandStore` em versões modernas do Windows [1] [2]. A mesma documentação alerta que protocolos de linha de comando possuem limitações para múltiplas seleções e que `IDropTarget` evita as restrições de buffer quando a integração precisar preservar uma seleção extensa [2]. A documentação de links simbólicos e junctions confirma que links são objetos transparentes ao usuário e que junctions são implementados como reparse points; por isso a recusa preventiva de links e reparse points na raiz e durante a enumeração permanece intencional [3] [4].

### Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/shell/fa-verbs "Microsoft Learn: Verbs and File Associations"
[2]: https://learn.microsoft.com/en-us/windows/win32/shell/context-menu-handlers "Microsoft Learn: Creating Shortcut Menu Handlers"
[3]: https://learn.microsoft.com/en-us/windows/win32/fileio/symbolic-links "Microsoft Learn: Symbolic Links"
[4]: https://learn.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions "Microsoft Learn: Hard Links and Junctions"

## Correções aplicadas nesta revisão

A análise de conteúdo passou a marcar corretamente seleções compostas apenas por formatos já comprimidos, enquanto seleções mistas permanecem conservadoras sem forçar Store indevidamente. O seletor agora retorna `store` ou `deflate` de forma compatível com o método efetivamente usado pelo container. O compressor propaga progresso incremental por bytes, expõe a fase de validação, implementa `--help`/`-h` nos binários e oferece `--decompress` com validação CRC, limites, staging e destino novo.

A validação de entradas usa `symlink_metadata` e rejeita symlinks na raiz; no Windows, também considera o atributo de reparse point. A enumeração de diretórios de análise e compactação passou a processar `read_dir` em streaming. Saídas existentes e saídas dentro de entradas são rejeitadas, temporários usam `create_new`, a extração não sobrescreve destino existente, entradas duplicadas são recusadas e foi acrescentado limite de razão de expansão para reduzir risco de decompression bomb.

## Limitações que permanecem

A seleção múltipla pelo verbo estático continua dependente do limite de linha de comando do Explorer e ainda não foi validada em Windows 10/11 reais. O paralelismo de arquivos independentes permanece desativado; o campo de estratégia não anuncia paralelismo inexistente. A proteção contra TOCTOU entre a validação de um caminho e sua leitura posterior é reduzida por revalidações e rejeição de links, mas não é eliminada sem manter handles abertos ou implementar uma camada específica por plataforma. A auditoria de vulnerabilidades RustSec não foi executada porque `cargo-audit` não está instalado no ambiente.

## Estado final para release

Após a segunda revisão, o workspace foi preparado para o release **v0.1.4**. O arquivo `Cargo.lock` foi regenerado pela validação do workspace, a versão compartilhada foi atualizada e as notas públicas estão em `docs/RELEASE_NOTES_0.1.4.md`. O workflow Windows permanece a verificação autoritativa para os executáveis e os artefatos específicos do sistema operacional.

## Nova passagem de auditoria — prompt 4

A revisão foi reiniciada sobre o commit `5531f28` e a tag `v0.1.4`. O working tree estava limpo, `main` estava sincronizada com `origin/main` e o release possuía artefatos Windows publicados pelo workflow aprovado. O inventário atual confirma workspace Rust com crates separados para `core`, `windows-integration`, `launcher` e `compressor`, testes de integração do container, documentação de compatibilidade e workflow de release.

| Família de requisitos históricos | Situação de linha de base | Evidência inicial |
| --- | --- | --- |
| Compactação por Explorer e níveis | Implementação estática registrada e níveis disponíveis; integração visual real ainda depende de Windows | `crates/windows-integration`, `docs/DECISAO_INTEGRACAO_WINDOWS.md` |
| Launcher/Manager | Fluxos install/verify/repair/remove e definição centralizada presentes | `crates/windows-integration/src/manager.rs` |
| Seleção, Unicode e diretórios | Parser tipado, validação de links e enumeração em streaming presentes | `crates/core/src/selection`, `filesystem`, `container` |
| Estratégia e container | Deflate/Store conectados ao ZIP, CRC, limites e staging presentes | `crates/core/src/container`, `selection`, `analysis` |
| Descompactação | API do core e fluxo CLI `--decompress` presentes; UX de extração pelo Explorer não faz parte do escopo registrado | `crates/compressor/src/main.rs` |
| Segurança | Traversal, duplicidades, saída existente, links/reparse points, temporários exclusivos e razão de expansão cobertos | testes do core e documentação de auditoria |
| Desempenho | Streaming de dados e enumeração; paralelismo deliberadamente não anunciado | `README.md`, `RELEASE_NOTES_0.1.4.md` |
| Compatibilidade Windows | CI Windows verde para build e empacotamento; validação manual do Explorer ainda pendente | `.github/workflows/release.yml`, `docs/COMPATIBILIDADE_WINDOWS.md` |

O foco desta nova passagem será procurar falhas residuais ou afirmações ainda superficiais, especialmente em parsing de argumentos, estados vazios, fluxos repetidos, API pública, `expect` em caminhos operacionais, consistência entre documentação e implementação, política de Registro, integridade do container, testes de regressão e comportamento do workflow. Nenhuma alteração será feita apenas para aumentar o diff.

## Fechamento da passagem de auditoria — prompt 4

A implementação foi revisada sobre a linha de base do commit `5531f28` e recebeu correções incrementais antes do release `v0.1.5`. O primeiro bloqueador encontrado foi um import órfão em `crates/launcher/src/main.rs`, deixado após a correção do código de saída para plataformas não suportadas. O import foi removido e o launcher release foi recompilado e executado em Linux: `install` informa que a integração requer Windows e retorna código 1.

A inspeção do protocolo Explorer → CLI encontrou uma incompatibilidade concreta: o Registro construía comandos com o valor numérico de `CompressionLevel`, enquanto o parser público do compressor aceitava apenas os nomes textuais. A definição central de Registro passou a usar `cli_name()`, e um teste verifica os cinco comandos registrados (`fast`, `low`, `normal`, `high` e `maximum`).

O seletor heurístico foi ajustado para distinguir uma seleção sem arquivos de uma seleção legítima composta somente por diretórios. O `InputProfile` agora carrega `directory_count`, o compressor propaga essa informação e um teste confirma a compactação de diretório vazio. A validação de links foi centralizada em `is_link_or_reparse_point`: análise e compactação rejeitam links simbólicos e reparse points tanto na raiz quanto durante a travessia, evitando que as camadas adotem políticas diferentes.

A razão de expansão máxima passou a ser aplicada pela mesma função compartilhada em validação e extração e também é verificada durante a leitura incremental de cada entrada. Isso permite abortar mais cedo quando o conteúdo expandido já excede a razão permitida, sem substituir a validação final de checksum e tamanho.

| Verificação | Resultado observado |
| --- | --- |
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --workspace` | Aprovado |
| `cargo test --workspace` | Aprovado; 26 testes |
| `cargo test --workspace --release` | Aprovado; 26 testes |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Aprovado, sem warnings |
| `cargo build --workspace --release` | Aprovado |
| `cargo tree -d` | Sem duplicações para imprimir |
| `git diff --check` | Aprovado |
| `cargo-audit` | Não executado; ferramenta indisponível |
| `compactador-launcher install` em Linux | Retornou código 1, conforme esperado |

### Limitações remanescentes após a passagem

A integração efetiva com o Registro e a aparência do menu do Explorer não foram executadas neste ambiente Linux. Permanecem pendentes a validação manual em Windows 10/11, a confirmação de seleções múltiplas extensas, caminhos UNC e longos, e a verificação visual do verbo estático. A proteção contra TOCTOU entre a validação e a leitura posterior foi reduzida por revalidações e pela recusa de symlinks/reparse points, mas não é absoluta sem uma estratégia específica de handles por plataforma. O paralelismo de arquivos independentes permanece desativado e não é anunciado sem benchmark. A auditoria RustSec continua pendente porque `cargo-audit` não está instalado.

### Decisão de release

Com os gates locais aprovados e a documentação atualizada, a versão compartilhada do workspace foi incrementada para `0.1.5`. A tag anotada `v0.1.5` acionou o workflow `windows-latest`, que concluiu com sucesso as etapas de validação, build, empacotamento e publicação. A release contém `Compactador-v0.1.5-windows-x86_64.zip` e `Compactador-v0.1.5-windows-x86_64.zip.sha256`. O workflow emitiu apenas o aviso operacional de depreciação do Node.js 20 nas actions usadas, sem falha de job.

## Nova auditoria completa — prompt anexado

A revisão completa foi iniciada sobre o branch `main` limpo e sincronizado, após a publicação de `v0.1.5`. O prompt anexado foi lido integralmente, incluindo os critérios de auditoria funcional, padrões de implementação gerada por IA, arquitetura, segurança, desempenho, compatibilidade, UX/CLI, robustez, fallbacks, dependências, verificação real, regressão, segunda revisão e release. A inspeção não tratou afirmações históricas como prova: cada comportamento relevante foi localizado no código, exercitado em testes ou classificado como limitação.

### Achados e correções

| Área | Achado confirmado | Correção aplicada |
| --- | --- | --- |
| Registry | `value_name: None` era tratado pelo backend Windows como subchave inteira, e `remove` podia apagar um valor divergente do aplicativo. | O valor padrão é apagado com `delete_value("")`; o manager só remove valores que coincidem com a definição; chaves próprias vazias são podadas sem `delete_subkey_all`. |
| Estado | `RepairRequired` existia no modelo e na decisão arquitetural, mas nunca era retornado. | Valores divergentes agora resultam em `InstallationState::RepairRequired`; testes cobrem a preservação do valor estrangeiro. |
| Extração | Diretórios duplicados podiam passar pela extração, embora a validação rejeitasse duplicidades. | O caminho de extração mantém conjunto de nomes e rejeita qualquer duplicidade, inclusive diretórios. |
| Links e tipos | A análise seguia a raiz linkada e ignorava alguns links/tipos especiais dentro da árvore, enquanto a compactação os recusava. | A análise usa `symlink_metadata`, rejeita raiz, filhos, reparse points e tipos especiais; o container mantém a mesma política. |
| Amostragem | Seleções grandes podiam ser marcadas como totalmente comprimidas com base apenas nos primeiros arquivos. | A estimativa usa o tamanho analisado e perfis amostrados nunca afirmam que todo o conjunto já é comprimido. |
| Progresso | `Validando` era emitido antes da compactação real, e operações vazias não tinham conclusão visual coerente. | Callbacks de validação/finalização foram conectados ao container; `Concluído` de uma operação de zero bytes mostra 100%. |
| Publicação | `rename` podia substituir uma saída criada entre a checagem e a publicação. | ZIP e arquivos extraídos usam hard link no mesmo diretório e preservam a saída concorrente; staging usa criação exclusiva. |
| Caminhos | `to_string_lossy` era usado na normalização de nomes, e NUL não era rejeitado antecipadamente. | Nomes portáveis exigem Unicode válido, não contêm NUL, separadores perigosos ou componentes inseguros; nomes automáticos usam `OsString`. |
| Diagnóstico | Erros ZIP de I/O, formato e suporte eram convertidos para a mesma categoria. | O mapeamento preserva `CoreError::Io`, `InvalidInput` e `Unsupported`. |
| CLI | O launcher aceitava argumentos extras silenciosamente. | Argumentos extras e o primeiro argumento não Unicode resultam em código 2. |
| Metadados | `ArchiveEntry::data_offset` era sempre zero. | O resumo usa `ZipFile::data_start()` depois da leitura da entrada, com testes para compressão e extração. |
| Cancelamento | Só havia teste de cancelamento antes do trabalho. | Teste durante streaming confirma que o temporário é descartado e a saída final não é publicada. |

### Verificação funcional real

Os binários release foram executados em um E2E dedicado. Foram confirmados `--help` e `-h`, arquivos e diretórios com Unicode e espaços, arquivo vazio, diretório vazio, diretório recursivo, seleção múltipla, todos os cinco níveis, escolha efetiva de Store para ZIP, extração com comparação byte a byte, destino existente preservado, nomeação automática repetida, entrada inexistente rejeitada, argumentos extras rejeitados e código 1 do launcher em Linux. A integração visual do Explorer não foi simulada.

### Segunda auditoria independente e gates

Após as correções, uma segunda inspeção percorreu novamente os módulos, as APIs públicas, os padrões de risco, as conversões de caminho, o Registro, o container, o compressor, o launcher, as dependências, o README, as decisões arquiteturais, a checklist Windows e o workflow. Não foram encontrados novos problemas relevantes no código portátil.

| Verificação | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --workspace` | Aprovado |
| `cargo test --workspace` | Aprovado; 31 testes |
| `cargo test --workspace --release` | Aprovado; 31 testes |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Aprovado, sem warnings |
| `cargo build --workspace --release` | Aprovado |
| `cargo tree -d` | Nenhuma duplicação para imprimir |
| `cargo metadata --locked --no-deps --format-version 1` | Aprovado |
| `git diff --check` | Aprovado |
| E2E dos binários release | Aprovado |
| `cargo-audit` | Não executado; ferramenta indisponível |
| Target MSVC local | Não disponível; host não possui `rustup` nem target instalado |

### Limitações mantidas

O workflow [Windows release](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32413648267) foi concluído com sucesso em `2026-08-20`, no job `Build Windows release`. A validação Windows passou após a tag, e a release publicou [`Compactador-v0.1.6-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.6/Compactador-v0.1.6-windows-x86_64.zip) e [`Compactador-v0.1.6-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.6/Compactador-v0.1.6-windows-x86_64.zip.sha256). Permanecem pendentes a aparência do menu, reinicialização do Explorer, Windows 10/11 reais, seleções múltiplas extensas, UNC, caminhos longos e permissões reais. A proteção contra TOCTOU do diretório final de extração não é absoluta em Unix; arquivos individuais e o ZIP usam publicação sem sobrescrita. O cancelamento cooperativo está disponível na API e testado durante streaming, mas o CLI não instala handler próprio de Ctrl+C. O protocolo `launcher_protocol` e os estados operacionais públicos sem consumidores são contratos passivos reservados à evolução da ponte de shell; não são anunciados como funcionalidades ativas.

### Preparação do release

O código desta passagem foi commitado em `2ff0b03` com a mensagem `fix: complete audit pass 5 — safety and contracts`. A preparação do release foi commitada em `43d7e4b` com a mensagem `chore: prepare release v0.1.6`; a versão do workspace foi incrementada de `0.1.5` para `0.1.6`, o `Cargo.lock` foi regenerado e as notas públicas e o changelog foram atualizados. A tag anotada `v0.1.6` aponta para `43d7e4b`, está publicada no GitHub e a release está disponível em https://github.com/danilo-jesus-unifil/Compactador/releases/tag/v0.1.6. Após o workflow Windows, esta documentação foi sincronizada em um commit posterior para registrar os artefatos efetivamente publicados.
