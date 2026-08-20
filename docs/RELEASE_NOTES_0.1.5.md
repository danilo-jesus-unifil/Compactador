# Compactador v0.1.5

## Resumo

Esta versão conclui uma nova passagem de auditoria sobre o contrato entre a integração do Explorer e o compressor, os estados de seleção vazia, a política de links e reparse points e a proteção contra expansão abusiva durante a extração. O objetivo foi corrigir comportamentos residuais identificados por testes e inspeção do código, sem anunciar capacidades que ainda dependem de validação no Windows.

## Principais correções

| Área | Resultado |
| --- | --- |
| Protocolo Explorer → CLI | O Registro agora usa os nomes textuais aceitos pelo parser (`fast`, `low`, `normal`, `high` e `maximum`), com teste de regressão para todos os níveis. |
| Launcher | Em plataforma não suportada, `install`, `verify`, `repair` e `remove` retornam falha explícita após informar a limitação; o Linux não é tratado como instalação bem-sucedida. |
| Diretórios vazios | A seleção e o fluxo de compactação aceitam diretórios sem arquivos, preservando o diretório no container ZIP. |
| Symlinks e reparse points | A análise, a compactação e a validação usam a mesma política: links simbólicos e reparse points não são seguidos. |
| Extração | A razão máxima de expansão é verificada durante o streaming, além da validação final, reduzindo trabalho antes de rejeitar conteúdo potencialmente abusivo. |
| API e manutenção | O reporter público sem uso foi removido, e a detecção de reparse points foi centralizada para evitar políticas divergentes entre módulos. |

## Validação executada

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo test --workspace --release`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --release`, `cargo tree -d` e `git diff --check`. A suíte passou com 26 testes no ambiente Linux, incluindo o teste de protocolo dos níveis CLI e o caso de diretório vazio. O binário release do launcher foi executado em Linux com `install` e retornou código 1, conforme esperado para uma integração dependente do Windows. `cargo-audit` não foi executado porque não está instalado neste ambiente.

## Artefatos

O workflow de release do GitHub Actions será acionado pela tag `v0.1.5`, validará o workspace em `windows-latest` e publicará o pacote `Compactador-v0.1.5-windows-x86_64.zip` acompanhado de seu arquivo SHA-256. A conclusão do workflow Windows é a evidência autoritativa para os executáveis `.exe` deste release; a validação Linux não substitui a execução do Registro e do Explorer no Windows.

## Limitações conhecidas

A confirmação real do backend `winreg`, da aparência do menu estático em cascata, de seleções múltiplas extensas, de caminhos UNC e longos e da compatibilidade efetiva com Windows 10 e Windows 11 continua dependente do workflow e de testes em máquinas Windows. A proteção contra TOCTOU entre a validação de um caminho e sua leitura posterior é reduzida por revalidações e pela rejeição de links, mas não é absoluta. O paralelismo de arquivos independentes permanece desativado porque não deve ser anunciado sem implementação e benchmark.

A auditoria detalhada está em [`docs/AUDITORIA_REVISAO_2026-08.md`](AUDITORIA_REVISAO_2026-08.md), e o procedimento de compatibilidade Windows está em [`docs/COMPATIBILIDADE_WINDOWS.md`](COMPATIBILIDADE_WINDOWS.md).
