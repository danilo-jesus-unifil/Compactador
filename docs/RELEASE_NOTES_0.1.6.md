# Compactador v0.1.6

## Resumo

Esta versão resulta de uma revisão completa do projeto, com auditoria funcional, arquitetural, de segurança, desempenho, compatibilidade, organização e consistência entre código e documentação. A passagem foi além da compilação: os binários release foram executados em fluxos E2E com entradas Unicode, espaços, arquivos vazios, diretórios vazios, múltiplas entradas, cinco níveis, Store, extração, repetição e falhas esperadas.

## Correções principais

| Área | Resultado |
| --- | --- |
| Registro Windows | A remoção verifica o valor atual antes de apagar, preserva valores divergentes e trata o comando sem nome como valor padrão. Chaves próprias vazias são podadas somente quando continuam vazias. |
| Estado de instalação | Divergências nos valores declarados retornam `RepairRequired`, alinhando o estado público ao modelo arquitetural documentado. |
| Container ZIP | Diretórios duplicados são rejeitados na extração; NUL e nomes não Unicode/portáveis não são aceitos; `data_offset` usa o offset real da crate ZIP; erros distinguem I/O, formato inválido e recurso não suportado. |
| Publicação segura | O arquivo compactado e os arquivos extraídos são publicados por hard link no mesmo diretório, sem substituir uma saída criada concorrentemente. O staging de extração é criado com operação exclusiva. |
| Análise | Links, reparse points e tipos especiais são rejeitados na raiz e durante a travessia. Uma seleção amostrada não é declarada como completamente comprimida, e a estimativa usa somente o tamanho analisado. |
| Progresso | Os eventos `Validando` e `Finalizando` são emitidos nos pontos reais do pipeline. Operações vazias mostram 100% somente em `Concluído`. |
| Unicode e CLI | Nomes automáticos usam `OsString`; o launcher rejeita argumentos extras; nomes de entrada com NUL ou conversões lossy não são aceitos silenciosamente. |
| Cancelamento | O cancelamento durante streaming remove o temporário e não publica um arquivo final parcial. |

## Validação local

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo test --workspace --release`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --release`, `cargo tree -d`, `cargo metadata --locked` e `git diff --check`. O ciclo final possui 31 testes: 14 testes unitários do core, 9 testes de integração do container, 4 testes do compressor e 4 testes da integração Windows em memória.

O E2E dos binários release confirmou `--help`/`-h`, compressão de arquivo e diretório com Unicode e espaços, arquivo vazio, diretório vazio, múltiplas entradas, todos os níveis `fast`, `low`, `normal`, `high` e `maximum`, Store para uma entrada ZIP, extração com comparação byte a byte, preservação de destino existente, nomeação automática repetida, erros para entrada inexistente, argumentos extras e código 1 do launcher fora do Windows.

## Artefatos e compatibilidade

A tag anotada `v0.1.6` acionou o workflow [Windows release](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32413648267), que terminou com sucesso em `windows-latest`, validou o backend `winreg`, compilou os executáveis MSVC e publicou os artefatos. O pacote é [`Compactador-v0.1.6-windows-x86_64.zip`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.6/Compactador-v0.1.6-windows-x86_64.zip), acompanhado de [`Compactador-v0.1.6-windows-x86_64.zip.sha256`](https://github.com/danilo-jesus-unifil/Compactador/releases/download/v0.1.6/Compactador-v0.1.6-windows-x86_64.zip.sha256). O host de desenvolvimento é Linux e não substitui a validação visual do Explorer.

## Limitações conhecidas

A confirmação manual do Registro, da aparência do menu estático em cascata, da reinicialização do Explorer, de seleções múltiplas que excedam o limite do verbo estático, de caminhos UNC e longos e da compatibilidade efetiva com Windows 10 e Windows 11 permanece pendente. A proteção contra TOCTOU do diretório final de extração não é absoluta em Unix; a proteção dos arquivos individuais e dos arquivos ZIP usa publicação sem sobrescrita. O cancelamento cooperativo existe na API e foi testado durante streaming, mas o CLI não instala um handler próprio de Ctrl+C. `cargo-audit` não foi executado porque a ferramenta não está instalada no ambiente.

A auditoria detalhada está em [`docs/AUDITORIA_REVISAO_2026-08.md`](AUDITORIA_REVISAO_2026-08.md), a decisão do container está em [`docs/DECISAO_CONTAINER_ZIP.md`](DECISAO_CONTAINER_ZIP.md), e a checklist Windows está em [`docs/COMPATIBILIDADE_WINDOWS.md`](COMPATIBILIDADE_WINDOWS.md).
