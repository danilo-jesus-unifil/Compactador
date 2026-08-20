# Decisão de integração com o Windows Explorer

## Decisão

A primeira implementação utilizará **verbos estáticos e menu em cascata por Registro**, sem uma Shell Extension COM carregada no processo do Explorer. A escolha atende ao objetivo de manter o caminho de construção do menu pequeno e reduzir o risco de instabilidade no Explorer. A documentação da Microsoft informa que o Windows 7 e posteriores suportam menus em cascata estáticos e recomenda soluções COM apenas quando os métodos estáticos forem insuficientes [1] [2].

O registro será feito no escopo do usuário, em `HKEY_CURRENT_USER\Software\Classes`, que participa da visão combinada de `HKEY_CLASSES_ROOT` e não exige elevação para a instalação do conjunto de verbos quando esse escopo é suficiente [1]. O launcher manterá a definição centralizada das entradas criadas e verificará cada entrada após escrever ou remover. A ação `install` não sobrescreverá valores divergentes: ela retornará `RepairRequired`; a substituição será reservada à ação explícita `repair`, com rollback de melhor esforço se uma escrita falhar.

## Estrutura escolhida

O menu principal terá um verbo exclusivo do aplicativo em `*\\shell` e `Directory\\shell`, com `MUIVerb` para o rótulo visível e `SubCommands` contendo os verbos de nível em ordem explícita. O conjunto de comandos filhos é compartilhado por meio do `CommandStore`, enquanto cada alvo mantém sua própria entrada de menu. Os subcomandos serão definidos em um conjunto reutilizável no escopo do usuário. Cada subcomando apontará para o executável operacional e receberá o nível solicitado.

| Elemento | Decisão | Motivo |
| --- | --- | --- |
| Menu principal | Verbo estático em cascata | Evita análise durante a construção do menu |
| Rótulo | `MUIVerb` | Separa o identificador estável do texto exibido |
| Filhos | `SubCommands` com ordem explícita | Permite apresentar Rápida, Baixa, Normal, Alta e Máxima |
| Comando | Executável externo | Mantém compressão fora do processo do Explorer |
| Escopo inicial | `HKCU\\Software\\Classes` | Instalação por usuário e menor privilégio |
| Alvos de seleção | `*` e `Directory` | Abrange arquivos e pastas selecionados sem depender de uma única associação |
| Fallback | Diagnóstico e estado `RepairRequired` | `install` preserva divergências; `repair` substitui somente após ação explícita |

## Seleção múltipla

A execução inicial pela linha de comando será implementada com argumentos individuais e parsing baseado em limites de argumentos do sistema, não por divisão ingênua de uma string única. A documentação da Microsoft alerta para limitações de buffer em protocolos antigos e recomenda abordagens de dados de shell quando a integração exigir preservação completa de namespace e múltiplas seleções [2]. Nesta fase, o contrato interno aceitará múltiplas entradas estáveis; a camada de entrada será mantida isolada para permitir evoluir para uma ponte de shell apropriada se o limite de argumentos se tornar insuficiente.

## Compatibilidade e limitações

A implementação estática por comando possui uma limitação importante: a documentação da Microsoft registra um limite de aproximadamente 2.000 caracteres para a linha de comando, o que restringe a quantidade de itens em uma seleção múltipla [4]. Por isso, o contrato de entrada será projetado desde já para aceitar vários argumentos, mas a validação em Windows deverá medir esse limite. Se a experiência real exigir preservar seleções maiores ou namespace de shell sem perda, a integração futura deverá migrar o ponto de invocação para uma solução de shell apropriada, como `IExplorerCommand` ou `IDropTarget`, sem colocar análise ou compressão no processo do Explorer [2] [4].

A documentação consultada cobre Windows 7 e posteriores, portanto a estratégia é compatível em princípio com Windows 10 e 11. Ainda assim, a aparência do menu compacto do Windows 11 e a posição de verbos de terceiros precisam de validação visual em máquinas reais. A implementação não deve prometer integração final antes de os testes de instalação, reinicialização, reparação e remoção serem executados em Windows 10 e Windows 11.

## Referências

[1]: https://learn.microsoft.com/en-us/windows/win32/shell/how-to--create-cascading-menus-with-the-subcommands-registry-entry "Microsoft Learn: Create Cascading Menus with the SubCommands Registry Entry"
[2]: https://learn.microsoft.com/en-us/windows/win32/shell/context-menu-handlers "Microsoft Learn: Creating Shortcut Menu Handlers"
[3]: https://learn.microsoft.com/en-us/windows/win32/shell/context-menu "Microsoft Learn: Shortcut Menus and Shortcut Menu Handlers"
[4]: https://learn.microsoft.com/en-us/previous-versions/windows/desktop/legacy/dd758091(v=vs.85) "Microsoft Learn: Choosing a Static or Dynamic Shortcut Menu Method"
