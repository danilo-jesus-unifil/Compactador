# Compactador Inteligente para Windows

O **Compactador** é um projeto Rust-first para compactação iniciada pelo menu de contexto do Windows Explorer. A solução possui dois pontos de entrada independentes: um **Launcher/Manager**, responsável por instalar, verificar, reparar e remover a integração; e um **compressor operacional**, chamado pelo Explorer para validar a seleção e executar a operação.

O desenvolvimento segue as categorias do prompt mestre, com análise, implementação, testes e revisão antes de cada avanço. A arquitetura separa domínio, filesystem, container, análise, seleção de estratégia, integração com o Windows e executáveis.

## Arquitetura

```text
crates/
├── core/                    # modelos, erros, análise, seleção e container ZIP
├── windows-integration/    # definição do menu, registro e manager idempotente
├── launcher/               # executável de instalação e manutenção
└── compressor/             # executável acionado pelo Explorer e operação
```

As regras de trabalho estão em [`docs/BOAS_PRATICAS_GIT_E_PROJETO.md`](docs/BOAS_PRATICAS_GIT_E_PROJETO.md). As decisões de registro estão em [`docs/DECISAO_INTEGRACAO_WINDOWS.md`](docs/DECISAO_INTEGRACAO_WINDOWS.md).

## Recursos implementados

A base atual já possui seleção de um ou vários arquivos, diretórios recursivos sem seguir links simbólicos, suporte a Unicode e espaços, análise amostral limitada, classificação por extensão e conteúdo, seletor heurístico explicável, níveis Rápida/Baixa/Normal/Alta/Máxima, compressão Deflate em streaming, container ZIP padrão, CRC, validação, extração com proteção contra traversal, escrita temporária, renomeação final, progresso por fases e cancelamento cooperativo.

O Launcher possui fluxo explícito de detecção, instalação idempotente, verificação, reparo e remoção restrita às entradas declaradas pelo próprio aplicativo. A integração foi modelada para verbos estáticos e menu em cascata no escopo do usuário, sem carregar uma extensão COM no Explorer.

## Estado e limitações conhecidas

A implementação do Registro está compilada somente em Windows por meio de um adaptador `winreg`; em Linux, os contratos e o backend em memória são usados para testes. A seleção múltipla via verbo estático ainda precisa ser validada no Windows quanto ao limite de linha de comando documentado pela Microsoft. Caso seleções muito grandes exijam preservação completa do namespace do Shell, a evolução indicada está registrada em [`docs/DECISAO_INTEGRACAO_WINDOWS.md`](docs/DECISAO_INTEGRACAO_WINDOWS.md).

A análise de diretórios já evita carregar todo o conteúdo em memória durante a compactação, mas a política de amostragem por tamanho e o controle de workers para pastas gigantescas ainda são pontos de evolução. O container usado é ZIP padrão com Deflate, não um formato proprietário; algoritmos adicionais poderão ser adaptados atrás do contrato comum sem alterar o fluxo de operação.

Como o ambiente de desenvolvimento atual é Linux, o comportamento dependente do Windows ainda requer validação final em Windows 10 e Windows 11, incluindo aparência do menu, reinicialização do Explorer, instalação, reparo, remoção, caminhos UNC, caminhos longos e seleções extensas.

## Desenvolvimento

É necessário ter Rust e Cargo instalados. Os comandos de qualidade são:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Para executar os pontos de entrada localmente:

```bash
cargo run -p compactador-launcher -- --help
cargo run -p compactador-compressor -- --compress normal -- caminho/para/arquivo.txt
```

O compressor também aceita uma saída explícita:

```bash
cargo run -p compactador-compressor -- --compress high --output resultado.zip -- "Meu arquivo.txt" "dados.csv"
```

No Windows, o launcher deve estar ao lado de `compactador-compressor.exe` para que a definição registrada aponte para o executável operacional correto. Tags no formato `vMAJOR.MINOR.PATCH` acionam o workflow de release, que valida o workspace em Windows e empacota os dois executáveis com checksum.

## Princípios técnicos

Toda entrada externa é tratada como não confiável. Caminhos armazenados no ZIP são validados como relativos antes da extração; arquivos são escritos em temporários e renomeados apenas após validação de integridade. O domínio não conhece o Registro, e o carregamento do menu não inicia análise ou compactação.

## Licença

Este projeto é distribuído sob a licença MIT. Consulte [`LICENSE`](LICENSE).
