# Pesquisa de prompts para revisão de projetos por IA

**Autor:** Manus AI  
**Data:** 20 de agosto de 2026  
**Projeto aplicado:** Compactador Inteligente para Windows em Rust, versão inicial `0.1.16`

## 1. Objetivo e escopo

Esta pesquisa responde a três perguntas práticas: **quais prompts ajudam uma IA a revisar um projeto de software**, **como encontrar prompts de qualidade** e **como aplicar um deles com evidência no repositório Compactador**. O foco não foi reunir frases genéricas para “melhorar o código”, mas identificar estruturas que tornem a revisão verificável, reproduzível e útil para um projeto Rust com integração ao Windows Explorer, Registro, CLI, container ZIP, extração e workflow de release.

A pesquisa combinou documentação oficial de ferramentas de desenvolvimento com a aplicação direta ao workspace. A documentação do GitHub fornece um prompt completo de revisão de código; o VS Code descreve princípios gerais para especificidade, contexto, decomposição e verificação; e as documentações de GitHub Copilot e Claude Code mostram como revisões integradas usam o diff em contexto do repositório, severidade e instruções específicas do projeto [1] [2] [3] [4].

> **Conclusão executiva:** o melhor prompt para uma auditoria de projeto não é o mais longo. Ele deve combinar papel, escopo, contexto disponível, áreas de risco, formato dos achados, evidência exigida, comandos de validação, limites do ambiente e uma segunda passagem independente.

## 2. Como encontrar prompts de revisão de projetos

A busca funciona melhor quando o termo descreve **a atividade**, **o tipo de análise**, **a tecnologia ou domínio** e **o formato esperado**. Pesquisar apenas “prompt para IA revisar projeto” tende a retornar listas genéricas; acrescentar expressões como `structured feedback`, `repository context`, `security audit`, `regression`, `acceptance criteria`, `prompt file` ou `checklist` aumenta a probabilidade de encontrar modelos operacionais.

| Intenção | Termos recomendados | Refinamento útil |
| --- | --- | --- |
| Revisão geral de código | `AI code review prompt`, `LLM code review template`, `comprehensive code review prompt` | `structured feedback`, `line references`, `actionable findings` |
| Auditoria de projeto inteiro | `AI project audit prompt`, `LLM repository audit prompt`, `codebase review prompt` | `full repository context`, `architecture`, `regression` |
| Segurança | `AI security code review prompt`, `LLM secure code audit`, `software security audit prompt` | `threat model`, `OWASP`, `input validation`, `path traversal` |
| Arquitetura | `AI software architecture review prompt`, `LLM architecture audit template` | `separation of concerns`, `dependencies`, `failure modes` |
| Qualidade e testes | `AI code quality review prompt`, `AI test coverage review prompt` | `edge cases`, `acceptance criteria`, `run tests` |
| Pull request e diff | `AI pull request review prompt`, `code review prompt diff regression` | `changed files`, `full codebase`, `severity` |
| Rust | `Rust code review prompt`, `Rust security audit prompt`, `Cargo workspace review prompt` | `unsafe`, `clippy`, `cargo audit`, `error handling` |
| Windows e filesystem | `Windows path security code review prompt`, `Rust Windows Explorer integration audit` | `Registry`, `reparse point`, `UNC`, `TOCTOU`, `shell context menu` |
| Formato reutilizável | `GitHub Copilot review-code prompt file`, `review-code.prompt.md`, `AI code review checklist` | `official documentation`, `repository custom instructions` |

A recomendação é pesquisar primeiro em fontes primárias, usando `site:docs.github.com`, `site:code.visualstudio.com`, `site:docs.anthropic.com` ou a documentação da ferramenta utilizada. Depois, exemplos comunitários podem ser usados para ampliar a cobertura, mas devem ser comparados com a documentação oficial e não tratados como prova de eficácia.

## 3. Fontes e prompts encontrados

| Fonte | Tipo | Estrutura encontrada | Valor para o Compactador |
| --- | --- | --- | --- |
| VS Code, **Best practices for using AI in VS Code** [1] | Guia oficial de prompting | Exige especificidade sobre entradas, saídas, restrições e comportamento; recomenda decompor tarefas, incluir critérios de aceitação, iterar e verificar a saída | Fundamentou a separação entre análise, correção, testes e revisão, além da exigência de evidências |
| Claude Code, **Code Review** [2] | Fluxo oficial de revisão | Analisa diff em contexto do repositório completo, procura erros lógicos, vulnerabilidades, edge cases e regressões, e classifica achados por severidade | Fundamentou a leitura do workspace inteiro, a severidade e a segunda passagem focada em regressões |
| GitHub Docs, **About GitHub Copilot code review** [3] | Documentação oficial de revisão agentic | Destaca que contexto completo do projeto aumenta a especificidade e que falhas da infraestrutura podem produzir revisão mais limitada | Fundamentou o registro explícito do contexto disponível e das limitações de CI/Windows |
| GitHub Docs, **Review code prompt** [4] | Prompt oficial reutilizável | Define papel de engenheiro sênior e cobre segurança, desempenho, qualidade, arquitetura, testes e documentação; exige referências, explicação, solução e justificativa | Foi a base escolhida para aplicação, adaptada de arquivo selecionado para workspace Rust completo |

A fonte mais diretamente reutilizável foi o prompt oficial do GitHub. Ela especifica áreas, formato e tom, mas foi escrita para revisar código selecionado no Copilot Chat. Para este projeto, foi necessário convertê-la em um protocolo de auditoria de repositório, com inventário de crates, leitura da documentação interna, comandos reais, distinção entre achado confirmado e risco não reproduzido e decisão explícita sobre release.

> A documentação do GitHub descreve o modelo como um prompt que “conducts thorough code reviews and provides structured, actionable feedback as a single comprehensive report” [4]. Essa característica foi preservada, mas a adaptação acrescentou evidência executável e limites de plataforma.

## 4. Comparação dos padrões de prompt

Um prompt de **diff de pull request** é adequado para mudanças pequenas e para impedir regressões no conjunto alterado. Um prompt de **auditoria de projeto** precisa ir além do diff, porque deve encontrar contratos históricos que já estavam incorretos, divergências entre README e código, dependências sem uso real, falhas de instalação e limites de plataforma. Um prompt de **segurança** deve ser mais restrito e pedir modelo de ameaça, entradas não confiáveis, impacto e reprodução; caso contrário, tende a produzir listas de vulnerabilidades sem priorização.

| Padrão | Entrada principal | Saída esperada | Risco se usado sozinho |
| --- | --- | --- | --- |
| Revisão de arquivo | Um arquivo ou trecho | Comentários por linha | Não enxerga contratos entre crates nem fluxo completo |
| Revisão de diff | Alterações de uma branch ou PR | Regressões e problemas nas linhas modificadas | Pode ignorar dívida preexistente fora do diff |
| Auditoria de codebase | Repositório e documentação | Achados por área, evidência e prioridade | Pode ficar amplo demais sem escopo e critérios de aceitação |
| Auditoria de segurança | Código, entradas e modelo de ameaça | Riscos reproduzíveis, impacto e mitigação | Pode gerar falsos positivos se não exigir prova |
| Revisão de arquitetura | Estrutura e dependências | Limites, acoplamento e decisões | Pode sugerir refatoração cosmética sem benefício funcional |
| Gate de release | Código, testes, CI e artefatos | Decisão de publicar ou bloquear | Não substitui revisão profunda de comportamento |

A melhor prática é encadear os padrões: primeiro uma auditoria orientada ao contexto, depois uma revisão de diff das correções, em seguida os gates automatizados e, por fim, uma inspeção independente de regressão e release. Essa ordem segue a recomendação de decompor tarefas complexas e verificar a saída em etapas [1].

## 5. Prompt selecionado e adaptação aplicada

O prompt de referência foi o **Review code prompt** oficial do GitHub [4]. O texto adaptado foi salvo no repositório em [`docs/PROMPT_AUDITORIA_IA_APLICADO_2026-08.md`](PROMPT_AUDITORIA_IA_APLICADO_2026-08.md). Ele preserva seis elementos do modelo original: papel de engenheiro sênior, segurança, desempenho, qualidade, arquitetura, testes e documentação, além de feedback acionável.

A adaptação para o Compactador acrescenta os seguintes controles:

| Controle acrescentado | Motivo |
| --- | --- |
| Leitura obrigatória de `docs/BOAS_PRATICAS_GIT_E_PROJETO.md` e do histórico de auditoria | Evita que a IA repita decisões já tomadas ou desfaça políticas de segurança intencionais |
| Inventário dos quatro crates, workflow e contratos públicos | A falha pode estar na fronteira entre launcher, compressor, core e integração Windows |
| Classificação `Confirmado`, `Provável` ou `Não verificado` | Separa evidência de hipótese e reduz afirmações indevidas |
| Referência a símbolo, arquivo, linha e comando | Torna o achado auditável por outra pessoa |
| Execução de fmt, check, test, clippy, build, metadata e E2E | Compilação isolada não prova funcionamento |
| Segunda passagem independente | Procura regressões, divergências documentais, cancelamento, publicação e propagação de falhas |
| Limitações explícitas de Linux versus Windows real | Evita declarar que Explorer, Registry ou caminhos UNC foram validados sem execução Windows |

O prompt instrui a IA a não modificar código durante a primeira análise, a propor apenas correções justificadas, a adicionar regressão antes de marcar um bug como corrigido e a repetir os gates após a alteração. Assim, o prompt funciona como **procedimento de revisão**, não apenas como uma pergunta longa.

## 6. Aplicação ao Compactador

### 6.1 Linha de base

A aplicação começou com a tag `v0.1.16` publicada, branch `main` sincronizado e working tree limpo. O projeto possui `compactador-core`, `compactador-windows-integration`, `compactador-launcher` e `compactador-compressor`. O ambiente local é Linux; o workflow Windows é a verificação autoritativa para os binários MSVC e para os testes condicionais de integração.

A primeira execução dos gates passou em `v0.1.16`, com 40 testes não documentais distribuídos entre core, container, compressor e integração Windows em memória. O teste E2E confirmou ajuda, níveis, arquivos e diretórios, Unicode, Store, extração, colisões, repetição, erros e comportamento do launcher fora do Windows.

### 6.2 Achado confirmado

A auditoria encontrou um conflito hierárquico que não era coberto pelas verificações anteriores. Um arquivo ZIP chamado `Folder` podia coexistir com uma entrada descendente chamada `folder/child.txt`. A validação existente rejeitava entradas duplicadas e colisões case-insensitive de nomes iguais, mas não verificava que um caminho-pai usado como **arquivo** não poderia também ser tratado como diretório para receber descendentes.

O comportamento era inconsistente: `validate_archive` podia aceitar o archive, enquanto `extract_archive` falharia somente ao tentar criar `folder/child.txt` depois de ter processado o arquivo-pai no staging. A falha era limpa antes da publicação final, mas o archive inválido atravessava a etapa pública de validação e o problema só aparecia mais tarde.

| Campo | Resultado |
| --- | --- |
| ID | `CONT-HIER-001` |
| Área | Segurança e integridade do container ZIP |
| Severidade | Alta |
| Confiança | Confirmado por teste de regressão |
| Evidência inicial | `crates/core/src/container/mod.rs`, funções `validate_archive` e `extract_archive` |
| Reprodução | Archive com `Folder` como arquivo e `folder/child.txt` como segundo arquivo |
| Impacto | Validação e extração discordavam; consumidores podiam aceitar um container que não era materializável como árvore de arquivos |

### 6.3 Correção aplicada

Foi criada a função compartilhada `reject_hierarchical_conflicts`, chamada tanto ao final de `validate_archive` quanto antes da publicação em `extract_archive`. Ela coleta nomes de arquivos e, para cada entrada descendente, verifica os ancestrais normalizados com a política case-insensitive já usada para portabilidade Windows. Diretórios legítimos continuam permitidos; somente um arquivo que também seja ancestral de outra entrada é rejeitado.

O teste de integração `rejects_file_path_that_is_an_ancestor_of_another_entry` cria o conflito com capitalização diferente, confirma que `validate_archive` falha, confirma que a extração falha e verifica que o destino final não é publicado. O teste reproduziu a falha antes da correção e passou depois dela, fornecendo evidência causal em vez de apenas cobertura nominal.

### 6.4 Segunda passagem independente

A segunda passagem reexaminou a correção, o diff, a política de nomes, o fluxo de staging, o cancelamento, a publicação, os parsers de CLI, o manager de Registro, o workflow e as declarações do README. A correção é limitada ao container e não remove funcionalidades corretas. A análise não encontrou novo bug confirmado no escopo portátil após a mudança.

Permanece um risco residual de concorrência na publicação do diretório de extração: o código verifica `destination.exists()` antes de trabalhar e finaliza com `fs::rename(&staging, destination)`. Uma criação concorrente do destino entre esses pontos tem semântica dependente da plataforma; em sistemas onde `rename` substitui o destino, pode haver substituição concorrente. O risco não foi reproduzido neste ambiente e foi mantido como limitação documentada, não como correção improvisada. A evolução recomendada é uma primitiva de publicação sem sobrescrita específica para cada plataforma, acompanhada de testes Windows e Unix.

Também permanecem as limitações já conhecidas: validação visual do Explorer e do Registro real, Windows 10/11, UNC, caminhos longos e seleções extensas; handler próprio de Ctrl+C no CLI; e auditoria RustSec com `cargo-audit`, indisponível no ambiente. Esses itens não foram apresentados como resolvidos.

## 7. Evidências de validação após a correção

Após a implementação, foram executados os gates locais em debug e release, além do E2E. A contagem passou de 40 para **41 testes executáveis não documentais**, porque foi acrescentada uma regressão de integração do container.

| Verificação | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | Aprovado |
| `cargo check --workspace --locked` | Aprovado |
| `cargo test --workspace --locked` | Aprovado; 41 testes executáveis, sem falhas |
| `cargo test --workspace --release --locked` | Aprovado; 41 testes executáveis, sem falhas |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Aprovado, sem warnings |
| `cargo build --workspace --release --locked` | Aprovado |
| `cargo tree -d` | Nenhuma duplicação reportada |
| `cargo metadata --locked --no-deps --format-version 1` | Aprovado |
| `git diff --check` | Aprovado |
| `/home/ubuntu/full_audit_e2e.sh` | Aprovado; terminou com `E2E_OK` |
| `cargo-audit` | Não executado; ferramenta indisponível |
| Validação Windows real | Deve ser confirmada pelo workflow de release; não é substituída pelos testes Linux |

O resultado do E2E também preservou os sinais esperados de erro de entrada e do launcher fora do Windows. A saída textual de progresso mostra validação, finalização e seleção de estratégia, mas não foi interpretada como prova de integração visual do Explorer.

## 8. Recomendações para uso futuro

Para uma revisão pontual, comece com um prompt de diff que nomeie a branch, os arquivos alterados, o comportamento esperado e os testes de aceitação. Para uma auditoria completa, use o prompt adaptado do repositório e forneça primeiro os documentos de arquitetura, boas práticas, decisões de segurança, workflow e histórico de auditorias. Não entregue apenas um arquivo isolado quando o problema puder estar na fronteira entre módulos.

O prompt deve sempre solicitar referências precisas e distinguir severidade de confiança. Um achado de alta severidade e baixa confiança deve virar investigação ou teste, não correção automática. A resposta deve indicar também o que foi revisado e não apresentou problema, pois isso evita remover contratos corretos em nome de uma refatoração estética.

A IA deve ser instruída a executar testes, mas o operador deve conferir os comandos e os logs. Ferramentas integradas podem revisar com contexto limitado quando Actions, agentes ou arquivos de instrução não estão disponíveis [3]. Em segurança, não copie código sugerido sem verificar a ameaça, a semântica da plataforma e o comportamento após falhas. Em releases Windows, o gate deve bloquear falhas reais e a existência do artefato deve ser verificada por checksum e conteúdo.

Finalmente, o prompt deve ser versionado junto com o projeto quando ele codificar políticas específicas. O arquivo [`docs/PROMPT_AUDITORIA_IA_APLICADO_2026-08.md`](PROMPT_AUDITORIA_IA_APLICADO_2026-08.md) registra a versão usada nesta auditoria; futuras revisões podem compará-lo com o resultado anterior e medir quais categorias realmente produziram achados.

## 9. Referências

[1]: https://code.visualstudio.com/docs/agents/best-practices "VS Code — Best practices for using AI in VS Code"
[2]: https://code.claude.com/docs/en/code-review "Claude Code — Code Review"
[3]: https://docs.github.com/en/copilot/concepts/agents/code-review "GitHub Docs — About GitHub Copilot code review"
[4]: https://docs.github.com/en/copilot/tutorials/customization-library/prompt-files/review-code "GitHub Docs — Review code prompt"
