# Changelog

## [0.1.4] — 2026-08-20

### Auditoria e hardening

A revisão funcional corrigiu a propagação da estratégia efetiva para o container, distinguiu seleções totalmente comprimidas de seleções mistas, adicionou Store real, progresso incremental por bytes e fases explícitas de validação. O compressor passou a expor `--help`/`-h` e descompactação segura por `--decompress`.

O container agora rejeita colisões de saída, saídas dentro das entradas, temporários concorrentes, destinos de extração existentes, entradas ZIP duplicadas, traversal e razões de expansão extremas. A enumeração de diretórios foi convertida para streaming, symlinks e reparse points são rejeitados na raiz e a remoção de valores ausentes do Registro Windows tornou-se idempotente. A suíte de testes do container foi separada para um teste de integração dedicado.

### Validação

A versão foi validada com `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo test --workspace --release`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --release`, `cargo tree -d` e `git diff --check`. O fluxo real de compactação, Store, descompactação e comparação do conteúdo restaurado também foi executado no Linux.

### Limitações conhecidas

A validação efetiva do Registro, da aparência do menu, de seleções múltiplas extensas, de caminhos UNC e longos e da integração em Windows 10/11 continua dependente do workflow e de testes em máquinas Windows. `cargo-audit` não estava instalado no ambiente desta revisão.

## [0.1.3] — 2026-08-20

### Correções finais de CI Windows

O launcher agora mantém importações específicas de plataforma sob `cfg`, instancia corretamente o backend de registro unitário e o reporter nulo fica restrito aos testes. Esses ajustes permitem que o workflow Windows complete a validação com as regras Clippy atuais.

## [0.1.2] — 2026-08-20

### Correções finais

A saída de progresso agora usa divisão segura compatível com as verificações Clippy do runner Windows, e o compressor reporta o identificador e a estratégia da operação para manter o resultado usado e auditável. A versão foi validada localmente antes da publicação da tag.

## [0.1.1] — 2026-08-20

### Correções

O workflow de release do Windows agora compila o backend `winreg` com o tipo `std::io::Error` efetivamente exposto pela crate, e o cálculo da análise satisfaz as verificações Clippy mais novas usadas por `windows-latest`. Esta versão de correção resolve os problemas específicos encontrados pela primeira validação de CI.

### Validação

A quality gate Linux permanece verde, e o workflow Windows associado à tag é a verificação autoritativa dos executáveis e do empacotamento para Windows.

## [0.1.0] — 2026-08-20

### Recursos

Esta versão inicial estabelece o workspace Rust modular com crates separados para domínio, integração com o Windows, launcher e compressor. Ela inclui instalação, verificação, reparo e remoção idempotentes por uma definição centralizada de registro; menu estático em cascata com os níveis Rápida, Baixa, Normal, Alta e Máxima; parsing de seleção com Unicode, espaços e múltiplas entradas; análise amostral; seletor heurístico explicável; catálogo Deflate/Store; container ZIP padrão; compressão e extração em streaming; CRC; proteção contra traversal; escrita temporária; progresso por fases e cancelamento cooperativo.

### Validação

A suíte foi executada em Linux com Rust/Cargo 1.75 e passou por `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` e `git diff --check`.

### Limitações conhecidas

A integração efetiva com o Registro, a aparência do menu e o comportamento de seleções múltiplas pelo Explorer ainda precisam ser validados em Windows 10 e Windows 11. O ambiente que produziu este release não é Windows e não publicou binários `.exe` não validados. A checklist de execução está em [`docs/COMPATIBILIDADE_WINDOWS.md`](docs/COMPATIBILIDADE_WINDOWS.md).
