# Changelog

Todas as mudanças importantes da NeoCOBOL serão documentadas neste arquivo.
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