# Auditoria e aprimoramento — achados iniciais

Este documento registra as passagens da auditoria solicitada no prompt de revisão. Os achados foram obtidos por leitura do código, inspeção da estrutura, execução dos testes e comparação com os requisitos anteriores; cada nova passagem é acrescentada ao final, preservando o histórico das decisões.

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

## Nova passagem após v0.1.6 — contratos e consistência

A revisão seguinte foi iniciada sobre `main` limpo, com `v0.1.6` publicado e o workflow Windows anterior concluído com sucesso. O prompt completo e as boas práticas internas foram lidos antes da inspeção. Foram novamente confrontados os requisitos funcionais, CLI/UX, arquitetura, segurança, desempenho, compatibilidade, modularização, dependências, regressão, documentação e limitações previamente registradas.

### Achados confirmados

| Área | Achado | Ação |
| --- | --- | --- |
| Heurística | `available_memory_bytes` alterava somente a justificativa de nível Máxima, embora o algoritmo e o pipeline permanecessem iguais; isso aparentava tuning de recursos sem efeito operacional. | Removida a condição. O seletor é determinístico e o pipeline continua single-threaded, com `parallel = false`. |
| API pública | `ResourceProfile`, `OperationStatus` e `CompressionStrategy.parallel` poderiam sugerir recursos já implementados. | Comentários documentam o alcance real: hints de recursos e status são contratos reservados; não há workers, store global ou paralelismo atual. |
| Protocolo | `launcher_protocol` era exportado, mas não era IPC nem era consumido pelo CLI. | O módulo agora informa explicitamente que é contrato passivo para futura ponte; o CLI atual usa argumentos diretamente. |
| README | A lista de recursos ainda dizia “renomeação final” e repetia a política de não sobrescrita. | Texto atualizado para “publicação sem sobrescrita”, sem duplicação. |

Nenhuma funcionalidade correta foi removida. A única mudança comportamental é retirar uma justificativa enganosa que não correspondia a uma diferença real do pipeline. Foi adicionado o teste `resource_hints_do_not_fake_parallel_or_memory_tuning`, comparando perfis de recursos baixo e alto.

### Regressão e gates

Após as alterações, passaram `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E. A suíte conta 32 testes: 15 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória. `cargo-audit` permanece indisponível.

### Preparação do release 0.1.7

As correções de código foram separadas no commit `cca6815`, com a mensagem `fix: clarify resource and protocol contracts`. A preparação do release foi commitada em `1e10e7c`, com a mensagem `chore: prepare release v0.1.7`; a versão do workspace foi incrementada de `0.1.6` para `0.1.7`, o lockfile foi atualizado e changelog, README e notas públicas foram atualizados. A tag anotada `v0.1.7` aponta para `1e10e7c` e está publicada em https://github.com/danilo-jesus-unifil/Compactador/releases/tag/v0.1.7.

O workflow [Windows release](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32414585227) terminou com sucesso no job `Build Windows release`. A release publicou [`Compactador-v0.1.7-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.7/Compactador-v0.1.7-windows-x86_64.zip) e [`Compactador-v0.1.7-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.7/Compactador-v0.1.7-windows-x86_64.zip.sha256). Os dois artefatos foram baixados; o checksum retornou `OK`, e o ZIP contém `compactador-launcher.exe` e `compactador-compressor.exe`. Esta atualização registra o resultado pós-CI; após o commit documental, o working tree deverá permanecer limpo e sincronizado com `origin/main`.

## Nova passagem repetida após v0.1.7 — análise estrita de entradas

A auditoria foi reiniciada sobre `main` limpo, com `v0.1.7` publicada e o workflow Windows anterior concluído. O prompt completo e as boas práticas internas foram lidos antes da inspeção. Foram novamente confrontados os requisitos funcionais, CLI/UX, arquitetura, segurança, desempenho, compatibilidade, modularização, dependências, regressão e documentação.

### Achado confirmado

A API pública `analyze_selection` aceitava uma seleção vazia. Além disso, `AnalysisAccumulator::add_file` retornava sucesso quando recebia um `InputEntry` declarado como arquivo cujo caminho real havia virado diretório, link/reparse point ou outro tipo não regular. Isso permitia uma análise parcialmente silenciosa, com totais e justificativa de estratégia incompletos, antes de uma eventual revalidação posterior do container.

A correção agora rejeita seleção vazia com `InvalidInput`, links/reparse points com `Unsupported` e caminhos não regulares com `InvalidInput`. Foram adicionados os testes `rejects_empty_selection_analysis` e `rejects_file_entry_that_is_not_a_regular_file`. Arquivos e diretórios válidos continuam usando o mesmo fluxo.

### Segunda auditoria e gates

Após a correção, a segunda revisão reexaminou análise, container, segurança, CLI, Registry, contratos públicos, documentação, dependências e workflow. Passaram `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E. A suíte conta 34 testes: 17 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória. `cargo-audit` permanece indisponível.

### Preparação do release 0.1.8

A correção de código foi separada no commit `442e08a`, com a mensagem `fix(core): reject invalid analysis selections`. A preparação do release foi commitada em `d94b156`, com a mensagem `chore: prepare release v0.1.8`; a versão do workspace foi incrementada de `0.1.7` para `0.1.8`, o lockfile foi atualizado, e changelog e notas públicas foram revisados. A tag anotada `v0.1.8` aponta para `d94b156` e está publicada em https://github.com/danilo-jesus-unifil/Compactador/releases/tag/v0.1.8.

O workflow [Windows release](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32415522642) terminou com sucesso no job `Build Windows release`. A release publicou [`Compactador-v0.1.8-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.8/Compactador-v0.1.8-windows-x86_64.zip) e [`Compactador-v0.1.8-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.8/Compactador-v0.1.8-windows-x86_64.zip.sha256). Os artefatos foram baixados; o checksum retornou `OK`, e o ZIP contém `compactador-launcher.exe` e `compactador-compressor.exe`. O resultado pós-CI foi sincronizado no commit `ad5040e`, mantendo o branch principal alinhado.

## Nova passagem após v0.1.8 — consistência da publicação de arquivos extraídos

A auditoria repetida foi iniciada sobre `main` limpo, com `v0.1.8` publicada e os artefatos Windows disponíveis. O prompt completo e as boas práticas internas foram lidos antes da inspeção. Foram reavaliados requisitos funcionais, CLI/UX, arquitetura, segurança, desempenho, compatibilidade, modularização, dependências, documentação e regressões.

### Achado confirmado

A implementação e o README já descreviam publicação sem sobrescrita por hard link, mas `docs/DECISAO_CONTAINER_ZIP.md` ainda dizia que cada arquivo extraído era “renomeado”. Essa diferença era documentalmente relevante porque poderia sugerir uma política de substituição diferente da efetivamente implementada. O texto foi corrigido para descrever hard link no mesmo diretório, sem sobrescrita, após escrita, sincronização e validação por CRC.

Nenhum comportamento funcional foi alterado. A inspeção não encontrou novo bug funcional, placeholder operacional, perda Unicode, leitura integral de arquivos grandes, uso de `delete_subkey_all`, paralelismo anunciado indevidamente ou divergência adicional entre contratos e implementação.

### Segunda auditoria e gates

Após a correção documental, passaram `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E. A suíte permanece com 34 testes: 17 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória. `cargo-audit` permanece indisponível.

### Preparação do release 0.1.9

A correção documental foi preparada com bump de `0.1.8` para `0.1.9`, changelog, notas públicas e lockfile atualizados. A preparação do release foi commitada em `e29e6d7`, com a mensagem `chore: prepare release v0.1.9`; a tag anotada `v0.1.9` aponta para `e29e6d7` e está publicada em https://github.com/danilo-jesus-unifil/Compactador/releases/tag/v0.1.9.

O workflow [Windows release](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32416149877) terminou com sucesso no job `Build Windows release`. A release publicou [`Compactador-v0.1.9-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.9/Compactador-v0.1.9-windows-x86_64.zip) e [`Compactador-v0.1.9-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.9/Compactador-v0.1.9-windows-x86_64.zip.sha256). Os artefatos foram baixados; o checksum retornou `OK`, e o ZIP contém `compactador-launcher.exe` e `compactador-compressor.exe`. Esta atualização registra o resultado pós-CI e será incluída no commit documental desta passagem.

## Nova passagem após v0.1.9 — análise amostrada e cancelamento operacional

A nova auditoria foi iniciada sobre o branch `main` sincronizado com `origin/main`, após a leitura integral do prompt de revisão e do documento `docs/BOAS_PRATICAS_GIT_E_PROJETO.md`. Foram reavaliados os requisitos funcionais, a interface CLI, a integração Explorer→CLI, a arquitetura dos quatro crates, segurança de caminhos e extração, desempenho de streaming e amostragem, compatibilidade Windows, tratamento de erros, dependências, documentação e regressões.

### Achados confirmados

Foi reproduzida uma inconsistência funcional na análise de diretórios grandes. O acumulador interrompia a classificação após 4.096 arquivos, mas reutilizava a mesma contagem para o campo público `SelectionAnalysis.files`. Assim, o tamanho total incluía todos os arquivos, enquanto a quantidade reportada representava somente a amostra. Essa divergência podia fornecer ao seletor e aos consumidores da análise um perfil de seleção incompleto.

Também foi confirmado que `OperationPhase::Cancelled` existia no modelo público e era reconhecido pelo formatador do CLI, mas o pipeline operacional não emitia esse evento: cancelamentos eram apenas retornados como `CoreError::Cancelled`. Isso deixava o contrato de progresso menos observável para consumidores de `ProgressReporter`, especialmente em integrações futuras.

A revisão não encontrou placeholders operacionais, perda de Unicode, processamento integral de arquivos grandes, remoção indevida de valores externos do Registry, paralelismo anunciado sem implementação, dependências duplicadas ou divergências adicionais relevantes entre documentação e código. `cargo-audit` não está disponível neste ambiente.

### Correções implementadas

O acumulador passou a manter `files` como contagem total descoberta e `analyzed_files` como contador interno da amostra. O limite de 4.096 arquivos continua controlando somente o trabalho de classificação; o campo `sampled` e a política conservadora de `already_compressed` permanecem preservados.

O pipeline operacional passou a emitir `OperationPhase::Cancelled` antes da próxima fase quando o token já está cancelado e quando o container retorna cancelamento durante o streaming. O evento inclui o progresso conhecido por meio de um contador atômico, e a função continua retornando `CoreError::Cancelled`, sem publicar temporários parciais.

### Cobertura adicionada

Foi adicionado o teste `reports_total_file_count_when_analysis_is_sampled`, que cria 4.097 arquivos, confirma a contagem e o tamanho totais, verifica `sampled` e garante que a análise parcial não anuncie todo o conteúdo como comprimido. Os testes de cancelamento pré-início e durante streaming agora verificam explicitamente que o último evento emitido é `OperationPhase::Cancelled`.

### Validação local pós-correção

Passaram `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release. A suíte passou a totalizar 35 testes: 18 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória. O E2E confirmou ajuda, Unicode, espaços, arquivos e diretórios vazios, múltiplas entradas, cinco níveis, Store, extração, colisões, repetição, erros e launcher fora do Windows.

### Preparação do release 0.1.10

A versão do workspace e dos quatro pacotes locais foi incrementada de `0.1.9` para `0.1.10`; `Cargo.lock`, `CHANGELOG.md` e `docs/RELEASE_NOTES_0.1.10.md` foram atualizados. A preparação foi commitada em `8bd931c`, com a mensagem `chore: prepare release v0.1.10`; a tag anotada `v0.1.10` aponta para esse commit e está publicada em https://github.com/danilo-jesus-unifil/Compactador/releases/tag/v0.1.10.

O workflow [Windows release](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32417679053) terminou com sucesso no job `Build Windows release` em `windows-latest`. A release publicou [`Compactador-v0.1.10-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.10/Compactador-v0.1.10-windows-x86_64.zip), com 314.906 bytes, e [`Compactador-v0.1.10-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.10/Compactador-v0.1.10-windows-x86_64.zip.sha256), com 106 bytes. O pacote baixado contém `compactador-launcher.exe` e `compactador-compressor.exe`; `sha256sum -c` retornou `OK`.

As limitações remanescentes são a validação visual do Explorer, Registry real, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas em Windows; a ausência de handler próprio de Ctrl+C no CLI; a proteção TOCTOU não absoluta do diretório final de extração em Unix; a indisponibilidade de `cargo-audit`; e o caráter reservado, não ativo, de `CompressionRequest`, `OperationStatus` e `parallel`.

## Nova passagem após v0.1.10 — compatibilidade Windows, rollback e cancelamento entre fases

A nova auditoria foi iniciada sobre `main` limpo, com a release `v0.1.10` publicada e seus artefatos Windows verificados. O prompt completo e `docs/BOAS_PRATICAS_GIT_E_PROJETO.md` foram lidos antes da inspeção. Foram reavaliados requisitos funcionais, CLI/UX, arquitetura, segurança, desempenho, compatibilidade Windows, modularização, dependências, documentação, Registry e regressões.

### Achados confirmados

A validação de caminhos ZIP rejeitava traversal, caminhos absolutos, barras invertidas, NUL e nomes não Unicode, mas ainda aceitava nomes incompatíveis com o filesystem Windows, como `CON.txt`, `NUL`, `arquivo?.txt` e componentes terminados em ponto ou espaço. A aceitação poderia fazer uma extração segura no sentido lexical falhar ou colidir no Windows.

A instalação do Registry escrevia todas as entradas sequencialmente, mas uma falha intermediária retornava o erro sem restaurar valores já alterados. Isso podia deixar a integração em estado parcial após uma falha de permissão, I/O ou backend.

O pipeline já cancelava durante o streaming e antes das fases posteriores, mas não verificava o token nas transições de validação e finalização. Um cancelamento observado pelo reporter no início da validação podia, portanto, permitir trabalho adicional ou publicação indevida.

A revisão documental também encontrou uma inconsistência histórica: as notas da v0.1.10 informavam 36 testes, enquanto a suíte real antes desta passagem possuía 35 — 18 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória. As notas, o changelog e o registro de auditoria foram corrigidos para refletir o estado comprovado.

### Correções implementadas

A validação de componentes relativos passou a rejeitar caracteres de controle e caracteres proibidos pelo Windows, pontos ou espaços finais e nomes reservados de dispositivos (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9` e `LPT1`–`LPT9`), inclusive quando recebem extensão. O teste unitário e o teste de integração de extração foram ampliados com esses casos.

A instalação do Registry agora captura os valores anteriores de todas as entradas, restaura em ordem reversa as entradas já processadas quando uma escrita falha e retorna o erro de rollback caso a restauração também falhe. A estratégia é de rollback de melhor esforço, preservando o diagnóstico do erro e evitando deixar alterações conhecidas quando o backend permite a restauração.

O container verifica cancelamento antes da validação, depois do callback de início da validação, após a validação, depois do callback de finalização e antes da publicação. O compressor ganhou um teste que cancela pelo evento `Validating` e confirma que a saída não é publicada.

### Validação pós-correção

Passaram `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release. A suíte passou a totalizar 37 testes: 18 do core, 9 do container, 5 do compressor e 5 da integração Windows em memória. O E2E confirmou novamente ajuda, Unicode, espaços, arquivos e diretórios vazios, múltiplas entradas, cinco níveis, Store, extração, colisões, repetição, erros e launcher fora do Windows. `cargo-audit` permanece indisponível.

### Preparação do release 0.1.11

A versão do workspace e dos quatro pacotes locais foi incrementada de `0.1.10` para `0.1.11`; `Cargo.lock`, `CHANGELOG.md`, as notas públicas e esta auditoria foram atualizados. A preparação foi commitada em `00319e6`, com a mensagem `chore: prepare release v0.1.11`; a tag anotada `v0.1.11` aponta para esse commit e está publicada em https://github.com/danilo-jesus-unifil/Compactador/releases/tag/v0.1.11.

Permanecem como limitações a validação visual do Explorer, Registry real, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas em Windows; a ausência de handler próprio de Ctrl+C no CLI; a proteção TOCTOU não absoluta do diretório final de extração em Unix; a indisponibilidade de `cargo-audit`; e o caráter reservado, não ativo, de `CompressionRequest`, `OperationStatus` e `parallel`.
