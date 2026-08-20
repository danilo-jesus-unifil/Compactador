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
