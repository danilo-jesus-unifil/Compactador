# Compactador v0.1.17

## Resumo

Esta versão registra a aplicação de um prompt estruturado de auditoria de projeto por IA e corrige um conflito hierárquico no container ZIP encontrado durante a revisão. O prompt, a pesquisa de fontes e os resultados completos estão em [`docs/PESQUISA_PROMPTS_REVISAO_IA.md`](PESQUISA_PROMPTS_REVISAO_IA.md). O texto reproduzível usado na auditoria está em [`docs/PROMPT_AUDITORIA_IA_APLICADO_2026-08.md`](PROMPT_AUDITORIA_IA_APLICADO_2026-08.md).

## Correção

Archives ZIP que continham um arquivo-pai, como `Folder`, e uma entrada descendente, como `folder/child.txt`, podiam ser aceitos pela validação e falhar somente durante a extração. O core agora rejeita o conflito hierárquico durante `validate_archive` e antes da publicação em `extract_archive`, usando a política case-insensitive já adotada para compatibilidade Windows.

O teste de regressão `rejects_file_path_that_is_an_ancestor_of_another_entry` confirma o caso com capitalização diferente, verifica a falha na validação e na extração e garante que o destino final não seja criado.

## Documentação

A auditoria foi registrada em [`docs/AUDITORIA_REVISAO_2026-08.md`](AUDITORIA_REVISAO_2026-08.md). O novo documento de pesquisa explica termos para encontrar prompts, compara modelos de revisão de arquivo, diff, codebase, segurança e release, e registra as fontes oficiais consultadas.

## Validação

A validação local aprovou:

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo test --workspace --locked`
- `cargo test --workspace --release --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo build --workspace --release --locked`
- `cargo tree -d`
- `cargo metadata --locked --no-deps --format-version 1`
- `git diff --check`
- `bash /home/ubuntu/full_audit_e2e.sh`

A suíte local possui 41 testes executáveis: 18 do core, 12 do container, 5 do compressor e 6 da integração Windows em memória. A confirmação Windows real e a publicação dos artefatos dependem do workflow acionado pela tag.

## Limitações conhecidas

A janela TOCTOU entre verificar e publicar o diretório final de extração permanece como risco residual dependente de plataforma. A validação visual do Explorer, o Registro real, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam pendentes de execução Windows. O CLI ainda não instala handler próprio de Ctrl+C, e `cargo-audit` não estava disponível no ambiente local.

## Artefatos esperados

Após o workflow Windows concluir com sucesso, a release deve conter:

- `Compactador-v0.1.17-windows-x86_64.zip`
- `Compactador-v0.1.17-windows-x86_64.zip.sha256`

A publicação só deve ser considerada concluída após confirmar o status do workflow, o conteúdo do ZIP e o checksum SHA-256.
