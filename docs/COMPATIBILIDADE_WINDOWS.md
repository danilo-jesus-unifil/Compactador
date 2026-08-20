# Checklist de compatibilidade Windows 10/11

## Pré-requisitos

Executar os testes em uma máquina de teste limpa, com uma conta de usuário sem alterações manuais prévias nas chaves usadas pelo aplicativo. Manter `compactador-launcher.exe` e `compactador-compressor.exe` no mesmo diretório. Registrar a versão do Windows, arquitetura, idioma do sistema e se o Explorer está em modo de 32 ou 64 bits.

## Instalação e estado

| Cenário | Comando ou ação | Resultado esperado |
| --- | --- | --- |
| Estado inicial | `compactador-launcher.exe verify` | Relata `NotInstalled` ou estado equivalente |
| Instalação | `compactador-launcher.exe install` | Cria somente entradas declaradas e confirma verificação |
| Idempotência | Executar `install` novamente | Não duplica entradas nem altera recursos externos |
| Menu | Reiniciar ou atualizar o Explorer e clicar com o botão direito | Exibe `Compactar` com cinco níveis |
| Reparo | Remover ou alterar uma entrada declarada e executar `repair` | Restaura a definição e confirma o estado |
| Remoção | `compactador-launcher.exe remove` | Remove apenas os recursos do aplicativo e verifica o resultado |
| Repetição | Executar `remove` novamente | Permanece removido sem falha destrutiva |

## Operação

Além da compactação iniciada pelo Explorer, validar o fluxo operacional direto `compactador-compressor.exe --decompress --output destino -- arquivo.zip`. O destino deve ser novo; uma tentativa sobre pasta existente deve falhar sem modificar seu conteúdo. Confirmar que CRC inválido, traversal e entradas duplicadas falham sem deixar staging ou arquivos parciais.

Testar `--help` e `-h` nos dois executáveis, um arquivo vazio, um arquivo de um byte, nomes com espaços, acentos, Unicode e parênteses, uma seleção de vários arquivos, uma pasta vazia, uma pasta com subpastas e arquivos heterogêneos, uma unidade UNC e um caminho longo quando o sistema estiver configurado para suportá-lo. Repetir cada cenário em Rápida, Baixa, Normal, Alta e Máxima, verificando que Normal permanece a opção recomendada e que todas as saídas podem ser validadas e extraídas.

Medir o comportamento com mais de quinze arquivos selecionados e com uma seleção cuja linha de comando exceda o limite documentado para verbos estáticos. Se a seleção for truncada, duplicada ou invocada item a item, registrar a evidência e não anunciar suporte completo; nesse caso, migrar a camada de invocação para uma abordagem de shell capaz de preservar a seleção inteira, mantendo o compressor fora do processo do Explorer.

## Segurança e recuperação

Interromper uma operação durante uma entrada grande e confirmar que o arquivo temporário é descartado e que o destino final não fica parcialmente escrito. Tentar extrair um ZIP com nomes `..\\..\\arquivo`, `C:\\arquivo`, `\\\\servidor\\share\\arquivo` e separadores mistos; todos devem ser rejeitados. Testar CRC inválido, tamanho declarado incompatível, excesso de entradas, falta de espaço e destino sem permissão de escrita.

## Reinicialização e atualização

Após instalar, reiniciar o Explorer e o Windows, verificar novamente o menu e executar uma operação. Substituir os binários por uma build da mesma versão, executar `verify` e `repair`, e confirmar que os caminhos registrados continuam apontando para o executável atual. Antes de remover a instalação, confirmar no Registro que nenhuma chave de outro aplicativo é tocada.

## Limitação desta entrega

Esta checklist foi preparada e os testes portáveis foram executados no ambiente Linux. A validação visual e os testes dependentes do Registro do Windows devem ser executados em máquinas Windows 10 e Windows 11 antes de distribuir binários de produção.
