# NeoCOBOL

Uma linguagem de programação experimental, inspirada na clareza do COBOL e desenvolvida do zero em Rust.

**Versão atual: 0.6.0.** Esta versão aprimora os diagnósticos e a interface de linha de comando. **Há mudanças incompatíveis com a 0.5.1.1**, especialmente no formato da saída e nos códigos de erro. Consulte o [CHANGELOG](CHANGELOG.md).

## O que é

A NeoCOBOL foi pensada para ser simples de ler e aprender:

- Sintaxe pequena e clara.
- Tipos explícitos, sem conversão automática.
- Cálculos com números decimais usando `rust_decimal`.
- Diagnósticos em português, com código, localização, trecho do programa e orientação para correção.

Atualmente, a linguagem é executada por um interpretador escrito em Rust, que também funciona no Termux/Android. A proposta não é reproduzir o COBOL original, mas explorar uma linguagem nova que preserve sua preocupação com legibilidade e aplicações administrativas.

## Início rápido

É necessário ter Rust e Cargo instalados.

```sh
git clone https://github.com/ivogeme/NeoCOBOL.git
cd NeoCOBOL
cargo build
```

Crie um arquivo chamado `ola.neo`:

```text
display "Olá, mundo"
```

Execute com:

```sh
cargo run -- ola.neo
```

Saída esperada:

```text
Olá, mundo
```

### Comandos

| Comando | Função |
| --- | --- |
| `neocobol programa.neo` | Analisa e executa o programa. |
| `neocobol check programa.neo` | Verifica o programa sem executá-lo. |
| `neocobol --detalhes programa.neo` | Mostra também as etapas de processamento. |
| `neocobol --versao` ou `neocobol -v` | Mostra a versão. |
| `neocobol --ajuda` ou `neocobol -h` | Mostra a ajuda. |

Para usar os mesmos comandos por meio do Cargo, acrescente `cargo run --` antes dos argumentos. Por exemplo:

```sh
cargo run -- check ola.neo
cargo run -- --detalhes ola.neo
```

Para gerar o executável otimizado:

```sh
cargo build --release
```

O executável fica em `target/release/neocobol` (ou `neocobol.exe` no Windows).

**Saída na versão 0.6.0:** a execução normal mostra apenas a saída produzida pelo programa. Mensagens como `Lexer`, `Parser`, `Semântica` e a confirmação de execução aparecem somente com `--detalhes`. O comando `check` confirma que o arquivo foi verificado, sem executar suas instruções.

Os códigos de saída são:

- `0`: sucesso.
- `1`: erro no programa ou falha ao abrir o arquivo.
- `2`: uso incorreto da linha de comando.

Os diagnósticos de erro são enviados para a saída de erros (`stderr`); a saída normal do programa vai para `stdout`.

## A linguagem

### Variáveis e tipos

```text
decimal preco = 19.90
string nome = "Ana"
boolean ativo = true
```

As variáveis são declaradas com tipo e valor inicial. Os tipos disponíveis são `decimal`, `string` e `boolean`. `vazio` é utilizado como tipo de retorno de funções que não devolvem um valor.

Não há conversão automática: `decimal x = "10"` é um erro. Comentários começam com `#`.

### Operadores

- Aritméticos: `+`, `-`, `*`, `/`, `%`. O `+` também concatena strings.
- Comparação: `==`, `!=`, `>`, `<`, `>=`, `<=`.
- Lógicos: `and`, `or`, `not`.
- Atribuição composta: `+=`, `-=`, `*=`, `/=`, `%=`.

A expressão `not x > 5` equivale a `not (x > 5)`. Os operadores `and` e `or` utilizam avaliação de curto-circuito: o lado direito não é calculado quando o resultado já está determinado.

### Condicionais e repetição

```text
decimal total = 100

if total >= 100
    display "Valor alto"
else
    display "Valor baixo"
end

decimal contador = 1
while contador <= 5
    display contador
    contador += 1
end
```

As condições de `if` e `while` precisam produzir um valor `boolean`.

### Funções

```text
func dobro(numero: decimal) -> decimal
    return numero * 2
end

func saudar(nome: string) -> vazio
    display "Olá, {nome}!"
end

display dobro(10)
saudar("Ana")
```

- Parâmetros e retornos são tipados.
- Uma função acessa seus parâmetros, suas variáveis locais e as variáveis globais, mas não as variáveis locais de quem a chamou.
- Funções com retorno não vazio precisam devolver um valor em todos os caminhos possíveis.
- Funções só podem ser declaradas no nível principal do programa.
- O limite de chamadas de função é de 256.

### Strings e interpolação

```text
decimal preco = 250.75
decimal quantidade = 4
display "Total: {preco * quantidade}"
display "Ela disse \"oi\""
```

- A expressão entre `{` e `}` é avaliada quando a string é criada e passa pela análise semântica antes da execução.
- A interpolação pode conter operações e chamadas de funções.
- Os escapes disponíveis são `\"`, `\\`, `\n` e `\t`.
- `{}` vazio e uma chave `{` sem fechamento são tratados como texto. Ainda não existe um escape próprio para produzir literalmente uma interpolação como `{x}`.

## Diagnósticos na versão 0.6.0

Os erros possuem um código específico, título, nome do arquivo, linha, coluna, trecho do código e indicador de localização. Quando aplicável, o diagnóstico também mostra os tipos esperado e recebido e uma sugestão de correção.

Por exemplo, ao tentar usar uma variável não declarada:

```text
decimal x = 10
display total
```

O diagnóstico indica o código `NEO2101` e destaca o nome `total` na segunda linha. Os códigos são agrupados por família:

- **NEO1xxx:** erros léxicos e sintáticos.
- **NEO2xxx:** erros de tipo e de análise semântica.
- **NEO3xxx:** erros de execução e de acesso ao arquivo.
- **NEO4xxx:** reservados para avisos futuros; ainda não implementados.

O catálogo dos códigos e suas mensagens está definido em [`src/diagnostico.rs`](src/diagnostico.rs). Alguns exemplos:

| Código | Significado |
| --- | --- |
| `NEO1002` | String não terminada. |
| `NEO1107` | Aninhamento ou tamanho excessivo. |
| `NEO2002` | Tipo incompatível em declaração ou atribuição. |
| `NEO2101` | Variável não declarada. |
| `NEO3001` | Divisão por zero. |
| `NEO3004` | Limite de chamadas excedido. |
| `NEO3009` | Arquivo não pode ser aberto. |

**Importante:** o significado de alguns códigos mudou em relação às versões anteriores. Não considere os códigos da 0.5.1.1 equivalentes aos da 0.6.0.

## Arquitetura

```text
Arquivo .neo
    |
    v
Lexer -> Tokens
    |
    v
Parser -> AST
    |
    v
Análise semântica
    |
    v
Runtime -> Execução
```

A organização atual inclui:

- `src/main.rs`: componentes principais da linguagem e do interpretador.
- `src/diagnostico.rs`: tipos de erro, catálogo de códigos e formatação dos diagnósticos.
- `src/cli.rs`: argumentos e execução pela linha de comando.
- `tests/cli.rs`: testes de integração da interface de linha de comando.
- `tests/exemplos.rs`: testes dos programas completos.
- `testes/`: programas `.neo` e arquivos de saída esperada.

### Testes

Execute:

```sh
cargo fmt -- --check
cargo test
cargo build
```

A suíte cobre análise léxica, sintaxe, tipos, escopos, funções, interpolação, diagnósticos, tratamento de entradas inválidas, limites de profundidade, interface de linha de comando e execução de exemplos.

Na última execução local informada durante a preparação desta documentação, **76 testes passaram**: 63 unitários, 12 de CLI e 1 de exemplos. A quantidade pode mudar com novas alterações no repositório.

A pasta `testes/` contém programas `.neo` e arquivos de referência `.saida` e `.erro`. Os programas cujo nome começa com `erro_` devem encerrar com código de saída `1`.

Para executar um exemplo manualmente:

```sh
cargo run -- testes/basico.neo
```

## O que mudou na 0.6.0

Esta versão concentra-se em **diagnósticos mais precisos, robustez e uma CLI mais previsível**.

- Saída limpa por padrão, com detalhes opcionais por meio de `--detalhes`.
- Comando `check` para validar programas sem executá-los.
- Diagnósticos com código específico, localização, indicação visual do trecho e ajuda.
- Melhor identificação de operadores, valores e nomes relacionados ao erro.
- Proteções contra entradas malformadas, profundidade excessiva e instruções muito grandes.
- Testes automatizados da CLI e dos programas de exemplo.
- Nome do executável padronizado como `neocobol`.

**Atenção à compatibilidade:** o formato da saída e o significado de códigos de erro anteriores mudaram. Para o histórico detalhado, consulte o [CHANGELOG](CHANGELOG.md).

## Limitações conhecidas

- Apenas o primeiro erro encontrado em cada programa é apresentado.
- Ainda não há avisos NEO4xxx, saída estruturada em JSON nem sugestões automáticas do tipo "você quis dizer...?".
- Uma instrução admite no máximo 1000 operadores; há limites de aproximadamente 50 níveis de parênteses ou chamadas aninhadas e 150 níveis de blocos.
- Não há arrays, registros, `for`, `else if`, biblioteca padrão, módulos ou acesso a arquivos pela própria linguagem.
- Ainda não existe compilação de programas NeoCOBOL para código nativo.

## Próximos objetivos

Os objetivos futuros podem mudar e não estão em ordem fixa:

- Definir a política de idioma das palavras-chave.
- Aprimorar arredondamento e formatação de decimais e valores monetários.
- Adicionar registros e outras estruturas de dados.
- Implementar entrada de dados e operações com arquivos.
- Explorar arrays, `else if` e `for`.
- Criar uma biblioteca padrão e um sistema de módulos.
- Ampliar as ferramentas de desenvolvimento, os testes e os diagnósticos.

## Filosofia

A NeoCOBOL nasceu da ideia de que a legibilidade do COBOL continua valiosa, mesmo que sua sintaxe e seu ecossistema pertençam a outra época. Em vez de reproduzir a linguagem original, o projeto constrói uma alternativa experimental com ferramentas modernas.

**O que preservar:**

- **Legibilidade:** programas fáceis de acompanhar, inclusive para iniciantes.
- **Cálculos decimais:** operações adequadas a valores que exigem representação decimal, respeitados os limites do tipo utilizado.
- **Dados organizados:** uma direção para versões futuras, especialmente em aplicações administrativas.

**O que fazer de diferente:**

- Uma sintaxe pequena, com tipos explícitos e mensagens de erro em português.
- Um interpretador em Rust que pode ser utilizado também no Android via Termux.
- Um projeto experimental que evolui em etapas, sem adicionar complexidade desnecessária.

> Uma linguagem não precisa ser enorme para ser útil.

As versões `0.x` podem introduzir mudanças incompatíveis. Essas alterações são documentadas no [CHANGELOG](CHANGELOG.md).

## Licença

A NeoCOBOL é distribuída sob a **Apache License 2.0**. Consulte o arquivo [LICENSE](LICENSE).
