# Boas práticas de Git e de projeto

## Finalidade

Este documento estabelece as regras de trabalho para o desenvolvimento do **Compactador Inteligente para Windows em Rust**. Ele deve ser lido antes de cada categoria do prompt mestre e revisitado durante a implementação, os testes, a revisão e a preparação do release.

> Princípio central: trabalhar uma categoria por vez, com análise, implementação, testes e revisão/refatoração antes de avançar.

## Estado inicial e escopo

Antes de modificar qualquer arquivo, registrar o estado do repositório com `git status`, identificar a branch atual, ler a documentação existente e listar os arquivos relevantes. O escopo de cada mudança deve ser explícito. Alterações não relacionadas à categoria atual devem ser adiadas, salvo quando forem indispensáveis para manter o workspace compilável ou corrigir uma falha descoberta pela validação.

A arquitetura deve manter separadas as responsabilidades de domínio, infraestrutura, integração com o Windows e pontos de entrada. O domínio não deve conhecer detalhes do Registro do Windows, do Explorer ou de caminhos específicos de instalação. Interfaces, tipos e contratos devem ser pequenos e estáveis, e `pub` deve ser usado apenas quando a API realmente precisar atravessar o limite do módulo ou do crate.

## Fluxo por categoria

Cada categoria deve seguir o fluxo abaixo:

| Etapa | Pergunta de controle | Evidência esperada |
| --- | --- | --- |
| Análise | Quais módulos, contratos, dependências e riscos são afetados? | Nota no plano, issue ou documentação da categoria |
| Implementação | A mudança atende somente ao objetivo atual? | Código coeso e revisão do diff |
| Testes | O comportamento novo e os casos de falha foram exercitados? | Comandos de validação e resultados registrados |
| Revisão/refatoração | Há duplicação, API pública excessiva, risco de segurança ou dívida evitável? | Diff final, clippy/fmt e decisão documentada |
| Conclusão | A categoria está realmente pronta, além de apenas compilar? | Commit ou marco identificável e checklist concluído |

Não marcar uma categoria como concluída apenas porque o código compila. Se um teste exigir Windows e o ambiente atual não for Windows, registrar a limitação, manter testes portáveis para o domínio e preparar uma validação explícita em Windows 10/11.

## Regras de Git

O repositório deve permanecer limpo e reproduzível. Não versionar segredos, credenciais, arquivos temporários, binários de build, diretórios `target/`, logs locais ou artefatos gerados que não sejam parte de um release deliberado. O `.gitignore` deve refletir essa regra.

As mudanças devem ser organizadas em commits pequenos, coesos e verificáveis. Cada commit deve representar uma unidade lógica, como a documentação de práticas, a fundação do workspace, o launcher ou o motor de compactação. Evitar commits que misturem refatoração ampla, formatação automática e comportamento novo sem necessidade.

As mensagens de commit devem ser claras, no imperativo e preferencialmente seguir uma convenção consistente, por exemplo:

```text
feat(core): add compression level domain model
fix(launcher): make integration removal idempotent
test(core): cover quoted multi-path parsing
docs: document Windows integration assumptions
chore: prepare release 0.1.0
```

Antes de cada commit, revisar `git diff`, `git diff --check`, os arquivos alterados e os testes relacionados. Depois do commit, confirmar `git status` e registrar o hash quando ele for relevante para o histórico do release.

Não reescrever o histórico compartilhado, não usar `git push --force` e não remover alterações de terceiros sem compreender sua origem. O branch principal deve receber apenas alterações que tenham passado pela validação disponível.

## Qualidade de Rust

Usar `cargo fmt --all -- --check` como verificação de formatação e `cargo clippy --workspace --all-targets --all-features -- -D warnings` quando a configuração e as dependências permitirem. Executar `cargo test --workspace` e `cargo check --workspace`; quando houver código condicional de Windows, validar também com o alvo apropriado ou em um ambiente Windows real.

Erros devem preservar contexto e distinção sem converter tudo para `String` prematuramente. Tipos próprios devem representar conceitos como nível de compactação, tipo de entrada, classificação, estratégia, estado de instalação, status de operação e identificador de operação. Caminhos devem ser tratados com `Path` e `PathBuf`, preservando Unicode e evitando parsing ingênuo por espaços.

Dependências novas precisam de justificativa técnica, licença compatível, manutenção razoável, superfície de segurança aceitável e uso mínimo. Preferir abstrações da biblioteca padrão quando forem suficientes. Atualizações de dependência devem ser separadas de mudanças funcionais, salvo quando a atualização for necessária para corrigir uma vulnerabilidade ou viabilizar a categoria atual.

## Segurança e integração com Windows

A integração com o Explorer deve ser pequena, explícita e idempotente. A definição das entradas criadas pelo aplicativo deve ser centralizada para que instalação, verificação, reparo e remoção compartilhem a mesma fonte de verdade. Nenhuma operação de registro deve ocorrer sem validação do estado existente e verificação posterior.

Toda entrada recebida do Explorer deve ser tratada como dado não confiável. Validar existência, tipo, permissões, caminhos especiais, links, reanálises, UNC, Unicode, caminhos longos quando suportados e possíveis colisões de saída antes de iniciar uma operação. A criação do arquivo de saída deve usar arquivo temporário, flush/sync quando apropriado, validação de integridade e renomeação final atômica ou o fallback documentado.

Não executar operações privilegiadas silenciosamente. O launcher deve informar claramente quando uma ação exigir elevação, evitar apagar chaves que não pertençam ao aplicativo e manter a remoção restrita aos recursos declarados pelo próprio projeto.

## Testes e revisão

Cada categoria deve acrescentar testes de unidade, integração ou contrato na camada correta. Testes do domínio devem ser executáveis sem Explorer ou Registro. Testes de filesystem devem usar diretórios temporários e nomes com espaços, Unicode e caracteres especiais. Testes de integração do Windows devem ser isolados por plataforma e documentar pré-requisitos.

Os casos felizes não são suficientes. Incluir entradas vazias, arquivos já comprimidos, arquivos grandes, múltiplas seleções, diretórios heterogêneos, caminhos inválidos, permissões insuficientes, interrupção, cancelamento, corrupção do container, colisão de saída e falhas parciais de instalação quando esses comportamentos fizerem parte da categoria.

A revisão deve procurar especialmente: dependências circulares; módulos monolíticos; `unwrap` ou `expect` em caminhos operacionais; perda de informação Unicode; comandos construídos por concatenação insegura; acesso desnecessário ao Registro; ausência de validação pós-escrita; relatórios de progresso inconsistentes; e APIs públicas maiores do que o necessário.

## Documentação e decisões

Toda decisão que altere o formato do container, a localização das chaves de registro, o protocolo entre launcher e compressor, a política de compatibilidade ou a estratégia de segurança deve ser registrada em `docs/`, com motivação, alternativas consideradas e impacto de migração.

O README deve explicar o objetivo do produto, o estado atual, como compilar, como testar, quais partes exigem Windows e como instalar ou remover a integração quando essa funcionalidade existir. A documentação não deve prometer recursos ainda não implementados.

## Release

Um release só deve ser criado após a auditoria final, a execução das validações disponíveis, a atualização do changelog e a confirmação de que o working tree está limpo. O número da versão deve seguir SemVer e ser consistente entre `Cargo.toml`, metadados de runtime, documentação e tag quando aplicável.

A tag deve ser anotada e seguir o formato `vMAJOR.MINOR.PATCH`. As notas do release devem separar recursos, correções, limitações conhecidas, requisitos do Windows, instruções de validação e artefatos publicados. Se o ambiente não puder produzir binários Windows, isso deve ser declarado em vez de publicar artefatos não validados.

## Checklist operacional

Antes de iniciar uma categoria:

- [ ] Ler este documento e o trecho correspondente do prompt mestre.
- [ ] Confirmar `git status`, branch e escopo.
- [ ] Identificar módulos afetados e riscos.
- [ ] Definir testes e evidências de conclusão.

Antes de concluir uma categoria:

- [ ] Executar os testes diretamente relacionados.
- [ ] Executar `cargo fmt`, `cargo check` e `cargo clippy` quando aplicável.
- [ ] Revisar segurança, erros, caminhos e compatibilidade.
- [ ] Atualizar documentação e registrar limitações.
- [ ] Revisar o diff e criar um commit coeso.

Antes do release:

- [ ] Executar a suíte completa disponível.
- [ ] Confirmar que não há segredos ou artefatos indevidos.
- [ ] Atualizar README, changelog e versão.
- [ ] Confirmar working tree limpo.
- [ ] Criar tag anotada e publicar release com notas honestas.

## Referências

[1]: https://git-scm.com/docs/git-diff "Git diff documentation"
[2]: https://doc.rust-lang.org/cargo/commands/cargo-test.html "The Cargo Book: cargo test"
[3]: https://doc.rust-lang.org/cargo/commands/cargo-clippy.html "The Cargo Book: cargo clippy"
[4]: https://doc.rust-lang.org/cargo/reference/workspaces.html "The Cargo Book: Workspaces"
[5]: https://rust-lang.github.io/api-guidelines/ "Rust API Guidelines"
[6]: https://learn.microsoft.com/windows/win32/shell/context-menu-handlers "Microsoft Learn: Context Menu Handlers"
[7]: https://learn.microsoft.com/windows/win32/shell/handlers "Microsoft Learn: Shell Handlers"
