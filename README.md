# NeoCOBOL

Uma linguagem de programação experimental, inspirada na clareza do COBOL, feita do zero em Rust.

**Versão atual: 0.5.1.1.** Esta versão corrige falhas da 0.5.1 e **quebra compatibilidade** com ela. Veja o [CHANGELOG](CHANGELOG.md).

## O que é

O NeoCOBOL foi pensado para ser simples de ler e de aprender:

- Sintaxe pequena e clara
- Tipos explícitos, sem conversão automática
- Cálculos decimais exatos (`decimal`, usando `rust_decimal`)
- Erros com código, linha, coluna e mensagem em português

Hoje a linguagem funciona por meio de um interpretador escrito em Rust, que roda inclusive no celular (Termux/Android). O projeto não busca reproduzir o COBOL original: a proposta é explorar uma linguagem moderna que preserve as ideias de legibilidade e de programas administrativos das linguagens clássicas.

## Início rápido

É preciso ter Rust e Cargo instalados.

~~~
git clone https://github.com/ivogeme/NeoCOBOL
cd NeoCOBOL
~~~

Crie um arquivo `ola.neo`:

~~~
display "Olá, mundo"
~~~

E execute:

~~~
cargo run -- ola.neo
~~~

O programa mostra algumas linhas de diagnóstico (Lexer, Parser, Semântica) antes do resultado.

## A linguagem

### Variáveis e tipos

~~~
decimal preco = 19.90
string nome = "Ana"
boolean ativo = true
~~~

Toda variável é declarada com tipo e valor inicial. Os tipos são `decimal`, `string` e `boolean`. O `vazio` é usado como tipo de retorno de funções que não devolvem valor. Não há conversão automática: `decimal x = "10"` é um erro. Comentários começam com `#`.

### Operadores

- Aritméticos: `+ - * / %` (o `+` também junta strings)
- Comparação: `== != > < >= <=`
- Lógicos: `and or not`
- Atribuição composta: `+= -= *= /= %=`

O `not` vale sobre a comparação inteira: `not x > 5` é o mesmo que `not (x > 5)`. O `and` e o `or` não calculam o lado direito quando o resultado já está decidido.

### Condicionais e repetição

~~~
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
~~~

As condições precisam ser `boolean`.

### Funções

~~~
func dobro(numero: decimal) -> decimal
    return numero * 2
end

func saudar(nome: string) -> vazio
    display "Olá, {nome}!"
end

display dobro(10)
saudar("Ana")
~~~

- Parâmetros e retorno são tipados.
- Uma função enxerga apenas seus parâmetros, suas variáveis locais e as variáveis globais, nunca as variáveis locais de quem a chamou.
- Uma função com retorno precisa devolver valor em todos os caminhos.
- Funções só podem ser definidas no nível principal do programa.
- A recursão é limitada a 256 chamadas.

### Texto e interpolação

~~~
decimal preco = 250.75
decimal quantidade = 4
display "Total: {preco * quantidade}"
display "Ela disse \"oi\""
~~~

- O `{expressão}` é calculado quando o texto é criado e verificado antes da execução. Pode conter chamadas de função.
- Escapes disponíveis: `\"`, `\\`, `\n` e `\t`.
- `{}` vazio e `{` sem fechamento ficam como texto. Ainda não existe forma de escrever um `{x}` literal.

### Erros

Cada erro informa código, posição e mensagem:

~~~
NEO2002 - Erro semântico na linha 1, coluna 9: A variável 'total' não foi declarada. Declare-a antes de usá-la. ...
~~~

| Código | Tipo |
| --- | --- |
| NEO1001 | Erro léxico |
| NEO1002 | Erro de sintaxe |
| NEO2001 | Erro de tipo |
| NEO2002 | Erro semântico |
| NEO3001 | Erro de execução |

## Arquitetura

~~~
Programa .neo
     ↓
Lexer → Tokens
     ↓
Parser → AST
     ↓
Análise semântica
     ↓
Runtime → Execução
~~~

## Testes

- `cargo test` roda 42 testes unitários (lexer, parser, semântica e runtime).
- Os programas `.neo` do repositório (`teste.neo`, `teste3.neo`, `teste5.neo`, `teste_completo_0.5.1.neo`, `megatest_0.5.1.neo` e `MEGATEST_FINAL_0.5.1.neo`) exercitam os recursos da linguagem. Rode-os e confira a saída.

## Mudanças da 0.5.1.1 (quebram compatibilidade)

- Funções não enxergam mais variáveis locais de quem as chamou.
- `not` vale sobre a comparação inteira.
- Variável global usada antes de ser declarada agora é erro.
- `\"`, `\\`, `\n` e `\t` em textos passam a ter significado.
- A interpolação é calculada ao criar o texto, e não no `display`.

Detalhes no [CHANGELOG](CHANGELOG.md).

## Limitações atuais

- Não há biblioteca padrão, arrays nem registros.
- Não há entrada de dados pelo teclado nem leitura de arquivos.
- Só existe um tipo numérico (`decimal`).
- Só existe a repetição `while`, e não há `else if`.
- Não há módulos nem compilação para código nativo.
- Não existe forma de escrever um `{x}` literal em um texto.
- O programa sempre mostra linhas de diagnóstico antes do resultado.
- O projeto ainda não tem a estabilidade de uma linguagem de produção.

## Próximos objetivos

Sem ordem fixa, e podem mudar:

- Definir o idioma das palavras da linguagem
- Arredondamento e formatação de decimais e moeda
- Registros (dados organizados em campos)
- Entrada de dados e arquivos
- Arrays, `else if` e `for`
- Saída limpa por padrão
- Biblioteca padrão e módulos
- Ferramenta de linha de comando e testes automáticos mais abrangentes

## Filosofia

O NeoCOBOL nasceu de uma observação: a ideia por trás do COBOL era muito boa, mas a linguagem envelheceu. Em vez de tentar reproduzi-la, este projeto a refaz do zero, como uma linguagem nova, mantendo a filosofia original e usando tecnologia moderna.

**O que o COBOL acertou e o NeoCOBOL quer preservar**

- **Legibilidade.** Um programa deve poder ser lido quase como um texto, inclusive por quem não é programador.
- **Dinheiro exato.** Cálculos decimais não podem esconder erros de arredondamento.
- **Dados organizados.** Informações de negócio (clientes, pedidos, notas) devem ser fáceis de representar e processar. *(planejado)*

**O que o NeoCOBOL faz de diferente**

- **Simples para quem está começando.** Sintaxe pequena, tipos explícitos e erros com código, linha, coluna e mensagem clara.
- **Tecnologia moderna.** Interpretador escrito em Rust, que já roda no celular (Termux).
- **Pequena de propósito.**

> Uma linguagem não precisa ser enorme para ser útil.

O projeto é experimental. As versões 0.x podem quebrar compatibilidade, e cada quebra é registrada no [CHANGELOG](CHANGELOG.md).

## Licença

NeoCOBOL é distribuído sob a licença Apache License 2.0. Consulte o arquivo [LICENSE](LICENSE).