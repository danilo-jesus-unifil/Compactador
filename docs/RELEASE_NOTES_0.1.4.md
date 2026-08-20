# Compactador v0.1.4

## Resumo

Esta versão é o resultado da auditoria funcional, arquitetural, de segurança, desempenho e UX do projeto. O objetivo foi substituir comportamentos apenas anunciados por caminhos efetivamente conectados e reduzir riscos de sobrescrita, traversal, links, duplicidade e expansão abusiva.

## Principais melhorias

| Área | Resultado |
| --- | --- |
| Estratégia | O seletor agora encaminha `store` ou `deflate` ao container, e seleções mistas não são classificadas como totalmente comprimidas. |
| Progresso | O callback de streaming acumula bytes reais e a operação expõe análise, preparação, compactação, validação, finalização e conclusão. |
| Descompactação | O compressor oferece `--decompress`, valida CRC e limites, rejeita traversal e duplicidade, usa staging e exige destino novo. |
| Segurança de saída | Saída existente, saída dentro da entrada e temporários concorrentes são rejeitados. |
| Escala | Enumeração de diretórios em análise e compactação não acumula todos os filhos em memória. |
| Windows | Symlinks e reparse points são recusados na raiz; remoção de valores ausentes no Registro é idempotente. |
| Manutenção | Testes comportamentais do container foram separados em `crates/core/tests/container.rs`. |
| UX | `--help` e `-h` são suportados pelo launcher e compressor. |

## Validação executada

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo test --workspace --release`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --release`, `cargo tree -d` e `git diff --check`. Também foram executados fluxos reais de compactação, seleção Store para ZIP, descompactação e comparação byte a byte do resultado restaurado.

## Limitações

O ambiente de desenvolvimento desta revisão é Linux. A confirmação final do backend `winreg`, do menu estático no Explorer, da aparência do cascaded menu, de seleções múltiplas que excedem o limite de linha de comando, de caminhos UNC/longos e da compatibilidade efetiva com Windows 10 e Windows 11 depende do workflow Windows e de testes em máquinas Windows. O paralelismo de arquivos independentes continua desativado, pois não deve ser anunciado sem implementação e benchmark. A proteção contra TOCTOU entre validação e leitura posterior é reduzida, mas não eliminada. `cargo-audit` não estava disponível neste ambiente.

A auditoria detalhada está em [`docs/AUDITORIA_REVISAO_2026-08.md`](AUDITORIA_REVISAO_2026-08.md), e o procedimento de compatibilidade Windows está em [`docs/COMPATIBILIDADE_WINDOWS.md`](COMPATIBILIDADE_WINDOWS.md).
