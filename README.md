# Compactador Inteligente para Windows

O **Compactador** é um projeto Rust-first para compactação iniciada pelo menu de contexto do Windows Explorer. A solução será composta por dois pontos de entrada independentes: um **Launcher/Manager**, responsável por instalar, verificar, reparar e remover a integração; e um **compressor operacional**, chamado pelo Explorer para validar a seleção e executar a operação.

O projeto está sendo desenvolvido por categorias, conforme o prompt mestre, com separação entre domínio, infraestrutura, integração com o Windows e executáveis. A lógica de domínio deve permanecer testável sem depender do Explorer ou do Registro do Windows.

## Arquitetura

```text
crates/
├── core/                    # modelos, erros e contratos independentes de plataforma
├── windows-integration/    # definição da integração, registro e protocolo
├── launcher/               # executável de instalação e manutenção
└── compressor/             # executável acionado pelo Explorer
```

As regras de trabalho estão em [`docs/BOAS_PRATICAS_GIT_E_PROJETO.md`](docs/BOAS_PRATICAS_GIT_E_PROJETO.md). Esse documento é parte do processo de desenvolvimento e deve ser consultado antes de cada categoria.

## Estado atual

A fundação da Categoria 1 está implementada. O workspace compila, os crates possuem responsabilidades separadas, os modelos de domínio são tipados, existe um contrato inicial para algoritmos e seleção de estratégias, e a validação de caminhos relativos perigosos já está coberta por testes. A integração efetiva com o Registro do Windows, o container e o motor de compactação ainda serão implementados nas categorias seguintes.

Como o ambiente de desenvolvimento atual é Linux, o comportamento dependente do Windows será validado de forma portável nesta etapa e deverá receber validação final em Windows 10 e Windows 11 antes de uma versão de produção.

## Desenvolvimento

É necessário ter Rust e Cargo instalados. Os comandos básicos são:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Para consultar os pontos de entrada durante a fundação:

```bash
cargo run -p compactador-launcher -- --help
cargo run -p compactador-compressor -- --help
```

## Princípios técnicos

O projeto evita parsing ingênuo de caminhos, mantém tipos de domínio separados de `String` genéricas, centraliza a definição dos recursos criados no sistema e prioriza escrita temporária, validação e renomeação final para operações de arquivo. A integração com o Explorer deverá permanecer pequena e fora do caminho de construção do menu.

## Licença

Este projeto é distribuído sob a licença MIT. Consulte [`LICENSE`](LICENSE).
