# Prompt de auditoria de projeto aplicado ao Compactador

> Este prompt é uma adaptação para auditoria de um workspace Rust completo do modelo oficial **Review code** publicado na documentação do GitHub. A adaptação mantém as áreas de segurança, desempenho, qualidade, arquitetura, testes e documentação, mas acrescenta contexto do repositório, evidência executável, distinção entre tipos de achado e segunda revisão.

## Instrução

Atue como um engenheiro de software sênior conduzindo uma auditoria completa, conservadora e baseada em evidências de um workspace Rust. Não presuma que documentação histórica, compilação ou afirmações do usuário provem que um comportamento funciona. Leia primeiro o guia de boas práticas do projeto e os documentos de contexto; depois inventarie crates, entradas, dependências, testes, workflow e contratos públicos.

Analise o projeto nas seguintes áreas:

1. **Correção funcional e contratos**: confirme se os fluxos anunciados existem no código, se estados vazios, erros, entradas múltiplas, Unicode, diretórios, cancelamento, corrupção e repetição têm comportamento definido e se os contratos públicos correspondem ao comportamento real.
2. **Segurança**: procure validação insuficiente, traversal, links/reparse points, colisões, sobrescrita, publicação concorrente, TOCTOU, limites de tamanho/razão, parsing de argumentos, comandos inseguros, Registro e ownership de recursos externos.
3. **Desempenho e recursos**: procure leitura integral desnecessária, acumulação de diretórios, complexidade evitável, limites ausentes, progresso enganoso, paralelismo anunciado sem implementação e uso real de perfis de recursos.
4. **Arquitetura e qualidade**: avalie separação de responsabilidades, APIs públicas, coesão, duplicação, dependências, tratamento de erros, conversões de caminho, `unwrap`/`expect` operacionais e consistência entre crates.
5. **Testes e verificação**: examine se casos felizes e de falha são cobertos, se há testes de regressão, se os testes realmente exercitam comportamento e se as limitações de plataforma estão explicitamente separadas dos resultados confirmados.
6. **Windows e integração Explorer/Registro**: confronte comandos registrados, quoting, verbos, múltipla seleção, idempotência, reparo, rollback, remoção restrita, estado divergente e diferenças entre backend em memória e Windows real.
7. **Documentação, CI e release**: confronte README, decisões arquiteturais, changelog, notas de release, versão, workflow, propagação de falhas, artefatos e declarações públicas.

Para cada achado, produza:

- **ID**, área, severidade (`Crítica`, `Alta`, `Média`, `Baixa` ou `Informativa`) e confiança (`Confirmado`, `Provável` ou `Não verificado`).
- Arquivo e linha ou símbolo afetado.
- Comportamento observado e impacto concreto.
- Evidência: teste, comando, log, trecho de código ou documentação que sustenta a conclusão.
- Correção recomendada, sem afirmar que foi implementada antes de verificar o diff e os testes.
- Se o item não for bug, classifique-o como risco residual, dívida técnica, limitação de plataforma ou sugestão.

Separe explicitamente: (a) achados confirmados por reprodução ou inspeção direta; (b) riscos plausíveis que exigem validação específica; (c) limitações que não podem ser testadas no ambiente atual; e (d) pontos que estão corretos e não devem ser removidos apenas por preferência.

Execute, quando disponíveis, os gates de qualidade do projeto: `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked`, `git diff --check` e o E2E documentado. Registre resultados e limitações, incluindo ferramentas ausentes e validações específicas de Windows que não possam ser executadas.

Depois da primeira análise, faça uma segunda passagem independente concentrada em regressões, divergências entre documentação e código, APIs passivas que pareçam ativas, publicação sem sobrescrita, cancelamento, nomes de arquivo Windows, estados de Registro, arquivos temporários e propagação de erros do CI. Não faça mudanças cosméticas. Se uma correção for justificada, altere somente o escopo necessário, acrescente teste de regressão, revise o diff e repita os gates.

Finalize com uma tabela de achados, uma tabela de cobertura, limitações, decisão sobre correções e recomendação de release. Não invente resultados, não anuncie funcionalidade sem prova e não trate a resposta da IA como substituta de revisão humana ou validação real.

## Contexto específico fornecido

- Projeto: Compactador Inteligente para Windows em Rust.
- Crates: `compactador-core`, `compactador-windows-integration`, `compactador-launcher` e `compactador-compressor`.
- Versão inicial da auditoria: `0.1.16`, release `v0.1.16` publicada e working tree limpo.
- Limitação ambiental: auditoria local em Linux; validação Windows real ocorre pelo workflow do GitHub Actions.
- Guia obrigatório: `docs/BOAS_PRATICAS_GIT_E_PROJETO.md`.
- Histórico: `docs/AUDITORIA_REVISAO_2026-08.md`.
