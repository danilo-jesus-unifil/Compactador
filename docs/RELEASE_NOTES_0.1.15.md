# Compactador v0.1.15

## Resumo

Esta versão incorpora uma investigação exploratória adicional, voltada a problemas não cobertos pela auditoria anterior e a comportamentos que poderiam divergir entre Linux e Windows.

## Correções

A compactação e a análise de diretórios agora percorrem filhos em ordem lexical explícita, eliminando a dependência de `read_dir` e tornando reproduzíveis a amostra heurística e a ordem dos entries ZIP.

A validação e a extração de ZIP agora rejeitam colisões que diferem apenas por maiúsculas/minúsculas, incluindo conflitos entre arquivo e diretório com o mesmo nome lógico. A comparação de sobreposição entre entrada e saída usa a política case-insensitive apropriada para Windows e respeita limites de componentes, evitando confundir prefixos como `dados` e `dados-antigos`.

A validação de nomes Windows passou a rejeitar também os dispositivos reservados `COM¹`, `COM²`, `COM³`, `LPT¹`, `LPT²` e `LPT³`, inclusive quando recebem extensões.

## Testes e validação

Foram aprovados `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, `cargo test --workspace --release --locked`, Clippy estrito locked, `cargo build --workspace --release --locked`, `cargo tree -d`, `cargo metadata --locked --format-version 1`, `git diff --check` e o E2E dos binários release.

A suíte Linux possui 40 testes: 18 do core, 11 do container, 5 do compressor e 6 da integração Windows em memória. O CI Windows executará adicionalmente o teste específico de sobreposição case-insensitive, totalizando 41 testes nessa plataforma.

## Riscos e limitações

A investigação confirmou que ainda existem janelas TOCTOU entre validar e abrir entradas, entre verificar e publicar a extração e entre ler e remover valores do Registry. Os temporários, a validação CRC, o hard link sem sobrescrita e a política de não seguir links reduzem a superfície de risco, mas não constituem proteção absoluta contra concorrência adversarial sem APIs de handles específicas de cada plataforma. Esses riscos permanecem explicitamente documentados.

A política de colisão case-insensitive é conservadora para o comportamento padrão do Windows. O Windows também suporta diretórios configurados como case-sensitive; nesses diretórios, o Compactador poderá rejeitar um archive que seria tecnicamente distinguível. A escolha prioriza a portabilidade segura com o Explorer e o comportamento Windows padrão.

A validação visual do Explorer, instalação e remoção reais do Registry, Windows 10/11, UNC, caminhos longos e seleções múltiplas extensas continuam dependentes de testes manuais Windows. O CLI não instala handler próprio de Ctrl+C. `cargo-audit` permanece indisponível no ambiente, e o workflow informa um aviso não bloqueante sobre o runtime Node.js 20 de actions atuais.

## Resultado pós-CI e supersession

O workflow [Windows release #32421723076](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32421723076) não deve ser tratado como validação funcional da v0.1.15. O log registrou falha em `directory_entries_are_emitted_in_sorted_order` e `supports_directory_and_multiple_selection_without_following_symlinks`, causada pela rejeição indevida de backslashes nativos no Windows. O job terminou verde porque o shell padrão não interrompeu a etapa após o código de saída não zero de um comando nativo.

Os artefatos publicados naquela execução existem, mas são **superseded** e não devem ser usados como evidência de uma release Windows validada. A correção está na v0.1.16, cujo workflow usa `bash` com `set -euo pipefail` e trata falhas de validação como bloqueadoras.

## Compatibilidade, artefatos e referências

A tag `v0.1.15` acionou o [workflow Windows #32421723076](https://github.com/danilo-jesus-unifil/Compactador/actions/runs/32421723076) em `windows-latest`. Embora o job tenha terminado verde, o log registrou dois testes falhando; por isso os artefatos dessa execução são superseded e a versão não é evidência de uma release Windows funcionalmente validada.

As regras de nomes reservados e caracteres proibidos foram conferidas na documentação oficial do Windows [1]. A distinção entre comportamento case-insensitive padrão e diretórios case-sensitive foi conferida na documentação oficial do Windows/WSL [2].

[1]: https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file "Microsoft Learn: Naming Files, Paths, and Namespaces"
[2]: https://learn.microsoft.com/en-us/windows/wsl/case-sensitivity "Microsoft Learn: Adjust case sensitivity"

A v0.1.15 foi superseded pela v0.1.16 após a falha funcional descoberta no log do CI; consulte as notas da v0.1.16 para o resultado corrigido e verificado.

---

**Versão:** `0.1.15`
**Status:** superseded; CI Windows registrou falhas funcionais e a validação foi corrigida na v0.1.16.
