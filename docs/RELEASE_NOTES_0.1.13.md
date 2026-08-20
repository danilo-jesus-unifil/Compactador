# Compactador v0.1.13

## Resumo

Esta versão registra uma nova auditoria completa após a v0.1.12. A revisão confrontou novamente funcionamento real, arquitetura, segurança, desempenho, compatibilidade Windows, organização, dependências, documentação, workflow, integração com o Explorer e regressões.

## Validação adicional do release

O workflow Windows agora executa `cargo test --workspace --release --locked` antes do build dos binários. Isso alinha o CI Windows à validação local e confirma a suíte no mesmo perfil otimizado usado para gerar o pacote.

O README foi atualizado com a mesma matriz de comandos. Nenhuma dependência ou comportamento funcional foi alterado nesta passagem.

## Resultado da auditoria

A linha de base e a validação pós-correção não encontraram novos bugs funcionais relevantes, placeholders operacionais, dependências duplicadas, perda de Unicode, traversal, publicação sem validação, remoção indevida de valores externos, uso de dados falsos ou divergências adicionais relevantes entre código e documentação.

## Validação local

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked --format-version 1`, `git diff --check` e o E2E dos binários release.

A suíte possui 38 testes: 18 testes do core, 9 testes de integração do container, 5 testes do compressor e 6 testes da integração Windows em memória. O E2E confirmou ajuda, Unicode, espaços, arquivos e diretórios vazios, múltiplas entradas, cinco níveis, Store, extração, colisões, repetição, erros e launcher fora do Windows. `cargo-audit` permanece indisponível no ambiente.

## Compatibilidade, artefatos e limitações

A tag `v0.1.13` acionará o [workflow Windows](https://github.com/danilo-jesus-unifil/Compactador/actions) em `windows-latest`. O resultado do workflow, os links dos artefatos e o checksum serão registrados nesta seção somente após a conclusão real do CI e a verificação local do SHA-256.

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C; o cancelamento do pipeline é cooperativo pela API. O rollback do Registry é de melhor esforço quando o próprio backend falha durante a restauração. O host Linux não possui target MSVC local, e a proteção contra TOCTOU do diretório final de extração não é absoluta em Unix. O workflow ainda informa um aviso não bloqueante sobre o runtime Node.js 20 de actions atuais.

Os contratos públicos reservados continuam não sendo anunciados como funcionalidades ativas: não há IPC baseado em `CompressionRequest`, store global de `OperationStatus` nem paralelismo de arquivos independentes nesta versão.

## Referências

[1]: https://github.com/danilo-jesus-unifil/Compactador/actions "GitHub Actions do Compactador"
[2]: https://github.com/danilo-jesus-unifil/Compactador "Repositório do Compactador"

Sem artefatos Windows verificados, esta nota não afirma que a compilação MSVC ou o pacote final foram publicados.

---

**Versão:** `0.1.13`
**Status local antes do CI:** validações Linux concluídas; workflow Windows pendente.
