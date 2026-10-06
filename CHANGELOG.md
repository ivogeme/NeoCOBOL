# Changelog

Todas as mudanças importantes da NeoCOBOL serão documentadas neste arquivo.

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