# Decisão de container: ZIP com Deflate

## Escolha

O projeto usa **ZIP padrão** como container de primeira versão e **Deflate** como algoritmo inicial. A escolha evita criar um formato proprietário antes de haver uma necessidade técnica comprovada, permite múltiplas entradas e mantém o algoritmo atrás da abstração de compressão.

| Camada | Responsabilidade |
| --- | --- |
| Seleção | Escolher nível e estratégia explicável |
| Algoritmo | Aplicar Deflate em streaming |
| Container | Armazenar nomes, metadados e entradas múltiplas |
| Integridade | Validar CRC antes de concluir ou extrair |
| Filesystem | Escrever temporário, sincronizar, validar e publicar sem sobrescrita |

A crate `zip` é usada com os recursos mínimos necessários para Deflate, sem habilitar funcionalidades não utilizadas. O manifesto mantém `Cargo.lock` versionado para reprodutibilidade do binário.

## Pipeline

A operação valida a seleção, analisa amostras pequenas, escolhe uma estratégia, grava em arquivo temporário, finaliza o ZIP, sincroniza o arquivo, valida todas as entradas, e somente então publica o arquivo no destino por hard link no mesmo diretório, sem substituir uma saída criada concorrentemente. Uma falha remove o temporário conhecido.

## Determinismo e portabilidade de nomes

A compactação percorre filhos de diretórios em ordem lexical explícita, em vez de depender da ordem de `read_dir`, para que a ordem dos entries e a amostra da análise sejam reproduzíveis entre filesystems. A validação e a extração também rejeitam entries que diferem apenas por maiúsculas/minúsculas, pois podem representar o mesmo caminho em volumes Windows mesmo quando o ZIP foi criado em um filesystem case-sensitive.

A comparação case-insensitive é deliberadamente conservadora e complementa as regras já existentes para traversal, nomes reservados de dispositivos, caracteres proibidos e pontos ou espaços finais.

## Segurança de extração

A extração rejeita nomes que não possam ser convertidos em caminhos relativos seguros, incluindo caminhos absolutos, componentes pai e separadores perigosos. Como o pacote é destinado ao Windows, também são rejeitados caracteres de controle ou proibidos, componentes terminados em ponto ou espaço e nomes reservados de dispositivos como `CON`, `NUL`, `COM1` e `LPT1`, inclusive quando recebem uma extensão. O limite de entradas e o tamanho expandido máximo são verificados antes e durante a leitura. Cada arquivo regular é escrito em temporário no mesmo diretório de destino, validado por CRC e publicado por hard link sem sobrescrita apenas depois da escrita completa.

A política atual não segue links simbólicos ao compactar diretórios. Isso reduz ambiguidades de escopo e evita que uma seleção atravesse inesperadamente para fora da árvore fornecida. Permissões, timestamps e atributos avançados não fazem parte do contrato mínimo desta versão e não devem ser anunciados como preservados.

## Referências

[1]: https://docs.rs/zip/0.6.6/zip/ "Documentação da crate zip 0.6.6"
[2]: https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT "PKWARE: .ZIP File Format Specification"
