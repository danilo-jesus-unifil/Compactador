# Changelog

## [0.1.12] — 2026-08-20

### Instalação segura e reprodutibilidade

A ação `install` agora preserva valores divergentes no Registry e retorna `RepairRequired`, sem sobrescrever recursos que podem pertencer a outro aplicativo. A ação explícita `repair` continua capaz de restaurar a definição declarada, usando rollback de melhor esforço quando uma escrita falha. O comportamento foi coberto por regressão de conflito e reparo.

O workflow Windows passou a exigir `--locked` em check, testes, Clippy e build de release. O README foi alinhado aos mesmos comandos reproduzíveis, e a checklist Windows passou a documentar o resultado esperado para conflitos.

### Validação

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release. A suíte passou a ter 38 testes: 18 do core, 9 do container, 5 do compressor e 6 da integração Windows em memória. `cargo-audit` permanece indisponível no ambiente.

### Limitações conhecidas

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C; o cancelamento exposto pelo pipeline permanece cooperativo pela API. O rollback do Registry é de melhor esforço quando o próprio backend falha durante a restauração. O host Linux não possui target MSVC local, e a proteção contra TOCTOU do diretório final de extração não é absoluta em Unix.

## [0.1.11] — 2026-08-20

### Segurança, rollback e cancelamento

A validação de caminhos ZIP passou a rejeitar caracteres de controle e proibidos pelo Windows, componentes terminados em ponto ou espaço e nomes reservados de dispositivos, inclusive com extensão. Foram ampliados os testes unitários e de integração de extração para cobrir esses casos.

A instalação da integração com o Registry agora captura os valores anteriores e tenta restaurar em ordem reversa as entradas já alteradas quando uma escrita intermediária falha. O erro de rollback é preservado quando a restauração também falha, evitando ocultar a causa do estado parcial.

O container passou a verificar cancelamento nas transições de validação e finalização, antes de publicar o arquivo. Foi adicionado teste que cancela no evento `Validating` e confirma que a saída não é publicada.

A auditoria também corrigiu a contagem histórica das notas v0.1.10, que dizia 36 testes enquanto a suíte comprovada possuía 35 antes desta nova passagem.

### Validação

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release. A suíte passou a ter 37 testes: 18 do core, 9 do container, 5 do compressor e 5 da integração Windows em memória. `cargo-audit` permanece indisponível no ambiente.

### Limitações conhecidas

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C; o cancelamento exposto pelo pipeline permanece cooperativo pela API. O rollback do Registry é de melhor esforço quando o próprio backend falha durante a restauração. O host Linux não possui target MSVC local, e a proteção contra TOCTOU do diretório final de extração não é absoluta em Unix.

## [0.1.10] — 2026-08-20

### Correções de análise e cancelamento

A análise de diretórios grandes agora preserva a contagem total de arquivos descobertos mesmo quando a classificação utiliza a amostra limitada de 4.096 arquivos. O tamanho total continua sendo acumulado, o campo `sampled` permanece explícito e a heurística não anuncia que todo o conteúdo está comprimido quando a análise foi parcial.

O pipeline operacional agora emite um evento de progresso na fase `Cancelled` tanto para cancelamentos antes do início da próxima etapa quanto para cancelamentos ocorridos durante o streaming. O evento preserva o progresso conhecido e continua acompanhado do erro `Cancelled`; temporários parciais não são publicados.

### Validação

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E dos binários release. A suíte passou a ter 35 testes: 18 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória. `cargo-audit` permanece indisponível no ambiente.

### Limitações conhecidas

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI ainda não instala handler próprio de Ctrl+C; o cancelamento exposto pelo pipeline permanece cooperativo pela API. O host Linux não possui target MSVC local, e a proteção contra TOCTOU do diretório final de extração não é absoluta em Unix.

## [0.1.9] — 2026-08-20

### Consistência da documentação de extração

A decisão arquitetural do container foi alinhada ao código real: arquivos extraídos são publicados por hard link no mesmo diretório, sem sobrescrita, depois da escrita, sincronização e validação por CRC. A redação antiga dizia `rename`, o que poderia sugerir uma política de substituição diferente da implementada.

Nenhum comportamento funcional foi alterado nesta passagem. A correção torna explícita a política de publicação segura já usada pelo código.

### Validação

A auditoria repetida aprovou `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, build release locked, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e E2E. A suíte possui 34 testes: 17 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória. `cargo-audit` não estava disponível.

### Limitações conhecidas

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI ainda não instala handler próprio de Ctrl+C; o cancelamento validado é cooperativo pela API. O host Linux não possui target MSVC local.

## [0.1.8] — 2026-08-20

### Correções de análise e robustez

A análise de seleção agora rejeita explicitamente uma seleção vazia. Também deixou de ignorar silenciosamente um `InputEntry` declarado como arquivo quando o caminho real é diretório, link/reparse point ou outro tipo não regular. A falha é classificada como `InvalidInput` ou `Unsupported`, preservando a distinção para o chamador e evitando estratégia/progresso baseados em totais incompletos.

Foram adicionados testes para seleção vazia e divergência entre o tipo declarado e o tipo real. Nenhuma funcionalidade correta da v0.1.7 foi removida.

### Validação

A passagem repetida aprovou `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, build release locked, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e E2E. A suíte possui 34 testes: 17 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória. `cargo-audit` não estava disponível.

### Limitações conhecidas

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI ainda não instala handler próprio de Ctrl+C; o cancelamento validado é cooperativo pela API. O host Linux não possui target MSVC local.

## [0.1.7] — 2026-08-20

### Auditoria de contratos e consistência

A nova revisão removeu da heurística uma condição que alterava apenas a justificativa conforme memória disponível, sem alterar o algoritmo efetivo. O seletor agora mantém comportamento determinístico e documenta que o pipeline permanece single-threaded e sem tuning de workers nesta versão.

Os campos públicos de recursos, paralelismo, estado operacional e protocolo launcher/compressor foram documentados com seus limites reais. O `launcher_protocol` continua disponível como contrato passivo para evolução futura; o CLI atual usa argumentos diretamente e não há IPC ou serialização desse tipo em produção.

O README foi alinhado à publicação sem sobrescrita e à ausência de paralelismo operacional. Foi acrescentada regressão que confirma que perfis de recursos diferentes não simulam ganho ou comportamento de paralelismo.

### Validação

A nova passagem aprovou `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, build release locked, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e E2E. A suíte possui 32 testes: 15 do core, 9 do container, 4 do compressor e 4 da integração Windows em memória. `cargo-audit` não estava disponível.

### Limitações conhecidas

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas permanecem dependentes de testes manuais Windows. O CLI ainda não instala handler próprio de Ctrl+C; o cancelamento testado é cooperativo pela API. O host Linux não possui target MSVC local.

## [0.1.6] — 2026-08-20

### Auditoria completa, segurança e consistência

A integração do Registro passou a remover somente valores que ainda correspondem à definição do aplicativo. O valor padrão dos comandos não é mais interpretado como uma subchave inteira; após a remoção, chaves próprias vazias são podadas sem apagar valores ou subchaves de terceiros. Divergências agora retornam `RepairRequired` e são preservadas para diagnóstico e reparo.

O container rejeita diretórios ZIP duplicados, nomes com NUL ou nomes não Unicode/portáveis, publica arquivos sem sobrescrita concorrente e cria staging de extração exclusivamente. O resumo passa a expor o offset real da entrada ZIP, e erros de I/O, formato inválido e recursos não suportados preservam categorias distintas.

A análise rejeita links, reparse points e tipos especiais na raiz e durante a travessia; perfis amostrados não são classificados como totalmente comprimidos; somas são saturantes; e o fluxo de progresso emite `Validando` e `Finalizando` na ordem real. Nomes automáticos preservam Unicode com `OsString`, o launcher rejeita argumentos extras e o cancelamento durante streaming confirma a remoção do temporário sem publicar saída parcial.

### Validação

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo test --workspace --release`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --release`, `cargo tree -d`, `cargo metadata --locked` e `git diff --check`. O ciclo final totaliza 31 testes e passou em debug e release. O E2E dos binários release confirmou Unicode, espaços, arquivo e diretório vazios, múltiplas entradas, cinco níveis, Store, extração byte a byte, destinos existentes, repetição, nomeação automática, erros e launcher fora do Windows.

### Limitações conhecidas

O workflow Windows específico desta passagem terminou com sucesso após a tag, compilando e empacotando os executáveis MSVC. O host Linux não possui `rustup` nem target MSVC instalado. A validação visual do Explorer, Windows 10/11, seleção múltipla extensa, UNC e caminhos longos continua manual. A proteção contra TOCTOU do diretório final de extração não é absoluta em Unix, embora arquivos individuais usem publicação sem sobrescrita. `cargo-audit` não está instalado no ambiente.

## [0.1.5] — 2026-08-20

### Correções e hardening

A integração do Explorer agora registra os nomes textuais aceitos pelo compressor (`fast`, `low`, `normal`, `high` e `maximum`), corrigindo o contrato entre o Registro e o parser CLI. O launcher também retorna falha explícita quando executado fora do Windows, em vez de reportar uma instalação não realizada como sucesso.

Seleções compostas somente por diretórios, inclusive diretórios vazios, passaram a ser aceitas pelo seletor e pelo fluxo de compactação. A política de segurança contra symlinks e reparse points foi unificada entre análise, compactação e validação de entradas; a razão máxima de expansão também é verificada durante o streaming da extração, reduzindo consumo desnecessário antes do bloqueio.

### Validação

Foram executados `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo test --workspace --release`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --release`, `cargo tree -d` e `git diff --check`. A suíte atual totaliza 26 testes e passou no ambiente Linux. A verificação RustSec permanece pendente porque `cargo-audit` não está instalado neste ambiente.

### Limitações conhecidas

A validação real do Registro, da aparência do menu, de seleções múltiplas extensas, de caminhos UNC e longos e da integração em Windows 10/11 continua dependente do workflow e de testes manuais em Windows. A proteção contra TOCTOU entre validação e leitura não é absoluta, e o paralelismo de arquivos independentes continua deliberadamente desativado.

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
