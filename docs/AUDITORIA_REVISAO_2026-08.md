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

Com os gates locais aprovados e a documentação atualizada, a versão compartilhada do workspace foi incrementada para `0.1.5`. A publicação dos executáveis Windows continua condicionada ao workflow `windows-latest`, acionado pela tag anotada `v0.1.5`; a aprovação desse workflow será registrada após sua conclusão.
