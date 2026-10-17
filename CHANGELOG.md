# Changelog

Todas as mudanças importantes da NeoCOBOL serão documentadas neste arquivo.
## [0.6.0] - 2026-10-17

Foco: diagnósticos. Esta versão muda a saída do programa e os códigos de erro e **quebra compatibilidade com a 0.5.1.1**.

### Alterado (quebra compatibilidade)

- A saída agora é limpa por padrão: o programa mostra só o resultado ou o erro. As linhas "NeoCOBOL", "Compilando", "Lexer", "Parser", "Semântica" e "Programa executado com sucesso." só aparecem com `--detalhes`.
- Cada tipo de erro passou a ter o seu código (de NEO1001 a NEO3010). Os cinco códigos antigos mudaram de significado: NEO1001 era "erro léxico" e agora é "caractere desconhecido"; NEO1002 era "erro de sintaxe" e agora é "string não terminada"; NEO2001 era "erro de tipo" e agora é "tipos incompatíveis"; NEO2002 era "erro semântico" e agora é "tipo incompatível em declaração ou atribuição"; NEO3001 era "erro de execução" e agora é "divisão por zero".
- O formato do erro mudou: código, arquivo, linha e coluna, trecho do código com indicador que cobre o trecho inteiro do erro, esperado e recebido (nos erros de tipo) e ajuda. Os erros vão para a saída de erros. A tabela completa está em [ERROS.md](ERROS.md).
- Novos limites, que geram o erro NEO1107: cerca de 50 níveis de parênteses ou de chamadas aninhadas, cerca de 150 blocos, `not` ou `-` seguidos, e 1000 operadores em uma única instrução.
- O executável passou a se chamar `neocobol`.

### Adicionado

- Comando `check`: `neocobol check programa.neo` verifica o programa sem executá-lo.
- Opções `--detalhes`, `--versao` (`-v`) e `--ajuda` (`-h`). O `--detalhes` e o `check` podem vir em qualquer ordem.
- Códigos de saída: 0 (sucesso), 1 (erro no programa) e 2 (opção inválida).
- Novos códigos de erro: NEO1107 (aninhamento ou tamanho excessivo), NEO1108, NEO2108 e NEO3010 (falhas internas).

### Melhorias

- O erro agora guarda o início e o fim do trecho, e o indicador `^` cobre o trecho inteiro.
- Em erros de operação, o indicador aponta para o operador (`+`, `/`, `and`...). Em declaração, atribuição e condição, aponta para o valor ou a condição. Em função ou variável duplicada, aponta para o nome. Erros no fim do arquivo aparecem na última linha.
- A mensagem de declaração duplicada informa a linha da primeira declaração.
- O interpretador roda numa thread com pilha de 256 MB, para que recursão e programas profundos não estourem a pilha.
- Entradas inválidas não causaram panic nos testes: eles cobrem milhares de entradas geradas (cada programa de exemplo cortado, com um caractere removido ou trocado, e sequências aleatórias de tokens).
- Cada erro já nasce com o seu código. Um erro criado sem código não compila.

### Testes

- 64 testes unitários (eram 42), 12 testes do executável de verdade (`tests/cli.rs`) e o teste dos programas completos (`tests/exemplos.rs`).
- Cada código de erro que dá para provocar a partir do código-fonte tem um teste que confere título, posição, mensagem, trecho, indicador e ajuda, e o teste falha se um código novo for criado sem teste. O NEO3009 é testado na linha de comando. O NEO2108 e o NEO3010 são defesas internas e não têm teste.
- Pasta `testes/` com 11 programas `.neo`: para cada `nome.neo`, `nome.saida` guarda a saída esperada e `nome.erro` guarda os erros esperados. Os programas `erro_*` devem terminar com código de saída 1.

### Interno

- Tipos de erro, tabela de códigos e formatação em `src/diagnostico.rs`. Linha de comando em `src/cli.rs`.
- Zero avisos do compilador.

### Limitações conhecidas

- Só o primeiro erro de cada programa é mostrado.
- Ainda não há avisos (NEO4xxx), saída em JSON nem sugestões do tipo "você quis dizer".
- As limitações da 0.5.1.1 continuam.


## [0.5.1.1] - 2026-10-11

Versão focada em correções, fora do plano original.

**Esta versão quebra compatibilidade com a 0.5.1.**

### Alterado (quebra compatibilidade)

- Funções agora enxergam apenas seus parâmetros, suas variáveis locais e as variáveis globais. Antes, no runtime, também enxergavam as variáveis locais de quem as chamava.
- O operador `not` passou a valer sobre a comparação inteira: `not x > 5` agora equivale a `not (x > 5)`. Antes equivalia a `(not x) > 5`.
- No código principal, usar uma variável global antes de declará-la agora é erro semântico. As funções continuam enxergando todas as globais.
- Em textos, `\"`, `\\`, `\n` e `\t` passaram a ter significado (aspas, barra invertida, quebra de linha e tabulação).
- A interpolação `{expressão}` agora é calculada quando o texto é criado, e não no `display`. Textos que vêm de variáveis ou de funções não são mais reinterpretados.

### Adicionado

- Chamada de função como instrução, por exemplo `mostrar("Neo")`. Antes isso gerava erro de sintaxe.
- Análise semântica das expressões dentro de `{}`: variável inexistente ou erro de tipo na interpolação agora são detectados antes da execução.
- Erro de tipo ao interpolar uma chamada de função `vazio`.

### Melhorias

- - Erros dentro de `{}` agora informam a linha correta do arquivo (antes apontavam sempre para a linha 1). A coluna é aproximada quando o texto tem escapes, e erros léxicos dentro do `{}` apontam para o início do texto.).
- Mensagem de variável não declarada reescrita, para ser clara tanto dentro de funções quanto no código principal.
- A versão exibida pelo programa é 0.5.1.1.

### Correções

- Corrigido o runtime, em que uma função podia ler ou alterar variáveis locais de quem a chamava.
- Corrigidos três testes unitários que falhavam (dois com valores errados e um que dependia da chamada de função como instrução).
- Corrigido o estouro de pilha do teste de recursão, que passa a rodar numa thread com pilha maior.
- Removida a função antiga que reinterpretava textos durante o `display`.

### Testes

- 42 testes unitários (`cargo test`), contra 35 na 0.5.1.
- Novos testes: escopo no runtime, global usada antes de declarar, aspas escapadas, cálculo de interpolação, interpolação com variável inexistente, posição do erro dentro da interpolação e mensagem de variável não declarada.
- O megatest continua executando até o fim com sucesso.

### Limitações conhecidas

- Ainda não existe forma de escrever `{x}` literal em um texto.
- Dentro de `{}`, aspas precisam de barra invertida, por exemplo `{f(\"a\")}`.
- Só existe a repetição `while`, e não há `else if`.
- O programa ainda mostra linhas de diagnóstico antes do resultado.
- Continuam sem existir biblioteca padrão, arrays, registros, entrada de dados e acesso a arquivos.

## [0.5.1] - 2026-10-06

### Adicionado

- Interpolação de strings com avaliação de expressões.
- Suporte a variáveis dentro de interpolação.
- Suporte a operações matemáticas dentro de interpolação.
- Suporte a chamadas de funções dentro de interpolação.
- Suporte a expressões compostas e chamadas de funções aninhadas dentro de interpolação.
- Megatest abrangente para validação do interpretador.
- Testes integrados de tipos, operadores, atribuições, condicionais, loops, funções, escopo e cálculos decimais.

### Melhorias

- Avaliação de expressões interpoladas integrada ao runtime.
- Melhor suporte a expressões complexas em `display`.
- Maior cobertura de testes do interpretador.
- Maior estabilidade do runtime.
- Validação da execução de programas completos utilizando múltiplos recursos da linguagem.

### Correções

- Corrigida a interpolação que anteriormente tratava chamadas de funções como texto literal.
- Corrigida a avaliação de expressões dentro de `{}` em strings.
- Corrigido o processamento de expressões com parênteses dentro de interpolação.

### Testes

A versão 0.5.1 foi validada com testes envolvendo:

- Tipos `decimal`, `string` e `boolean`.
- Operações matemáticas.
- Comparações.
- Atribuições compostas.
- `if / else`.
- `while`.
- Funções com parâmetros.
- `return`.
- Escopo de variáveis.
- Interpolação simples.
- Interpolação com expressões.
- Interpolação com chamadas de funções.
- Interpolação com expressões compostas.
- Cálculos financeiros utilizando `rust_decimal`.

## [0.5.0] - 2026-09-02

### Adicionado

- Sistema básico de tipos com `decimal`, `string` e `boolean`.
- Operadores matemáticos.
- Operador módulo `%`.
- Operadores de comparação.
- Operadores lógicos `and`, `or` e `not`.
- Atribuições compostas:
  - `+=`
  - `-=`
  - `*=`
  - `/=`
  - `%=`
- Estruturas condicionais `if`, `else` e `end`.
- Estrutura de repetição `while`.
- Funções com parâmetros.
- `return`.
- Chamadas de funções.
- Strings com concatenação.
- Interpolação de strings usando `{variavel}`.
- Comentários usando `#`.
- Análise semântica.
- AST.
- Sistema de erros com códigos NEO.
- Runtime próprio.
- Operações decimais usando `rust_decimal`.

### Melhorias

- Separação mais clara entre lexer, parser, análise semântica e runtime.
- Mensagens de erro mais específicas.
- Verificação de tipos antes da execução.
- Melhor organização interna da linguagem.
- Suporte a programas maiores e mais estruturados.

### Limitações conhecidas

- Parâmetros de funções ainda não possuem tipos declarados.
- Tipos de retorno das funções ainda não são declarados.
- A análise semântica dos corpos das funções ainda precisa ser aprimorada.
- Ainda não existe compilação para código nativo.
- Ainda não existe biblioteca padrão.
- Ainda não existem arrays ou estruturas de dados complexas.
- Ainda não existe acesso a arquivos ou hardware.

## [0.4.0]

Versão anterior da linguagem.

## [0.3.0]

Versão anterior da linguagem.