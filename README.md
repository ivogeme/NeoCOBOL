# NeoCOBOL

Uma linguagem de programação experimental, inspirada na clareza do COBOL, feita do zero em Rust.

****Versão atual: 0.6.0.** Esta versão muda a saída do programa e os códigos de erro e **quebra compatibilidade** com a 0.5.1.1. Veja o (CHANGELOG.md).

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

### Comandos

~~~
neocobol programa.neo               executa o programa
neocobol check programa.neo         verifica o programa sem executar
neocobol --detalhes programa.neo    mostra também as etapas (Lexer, Parser, Semântica)
neocobol --versao
neocobol --ajuda
~~~

Por padrão o programa mostra só o resultado ou o erro. Com o Cargo, use `cargo run -- check programa.neo`. Depois de `cargo build --release`, o executável fica em `target/release/neocobol`. O código de saída é 0 (sucesso), 1 (erro no programa) ou 2 (opção inválida).

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

Cada erro tem um código estável, o arquivo, a linha e a coluna, o trecho do código com um indicador e uma ajuda:

~~~
erro[NEO2101]: Variável não declarada
 --> erro1.neo:2:9
  |
2 | display total
  |         ^^^^^
  = A variável 'total' não foi declarada. Declare-a antes de usá-la. ...
  = ajuda: Declare a variável antes de usá-la, por exemplo: decimal nome = 0
~~~

Os erros vão para a saída de erros. A tabela completa está em [ERROS.md](ERROS.md). As famílias são NEO1xxx (léxico e sintaxe), NEO2xxx (semântica e tipos), NEO3xxx (execução) e NEO4xxx (avisos, planejados).

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

#### Testes

- `cargo test` roda mais de 75 testes: os unitários (código, posição e formato de cada erro, entradas inválidas, escopo e interpolação), os da linha de comando (`tests/cli.rs`) e o dos programas completos (`tests/exemplos.rs`).
- A pasta `testes/` tem programas `.neo`. Para cada `nome.neo`, o arquivo `nome.saida` guarda a saída esperada e `nome.erro` guarda os erros esperados. Os programas que começam com `erro_` devem terminar com código de saída 1.
- Para rodar um programa à mão: `cargo run -- testes/basico.neo`.






## Mudanças da 0.6.0 (quebram compatibilidade)

- A saída é limpa por padrão. `--detalhes` mostra as etapas.
- Cada tipo de erro tem um código novo, com significado diferente dos códigos antigos (veja [ERROS.md](ERROS.md)).
- O formato do erro mudou: agora mostra o trecho do código e um indicador.
- Programas muito aninhados ou com instruções enormes geram o erro NEO1107.
- O executável se chama `neocobol`.





Detalhes no (CHANGELOG.md).

## - Só o primeiro erro de cada programa é mostrado.
- Ainda não há avisos nem saída em JSON.
- Uma instrução pode ter no máximo 1000 operadores, cerca de 50 níveis de parênteses e cerca de 150 blocos aninhados.

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

NeoCOBOL é distribuído sob a licença Apache License 2.0. Consulte o arquivo [LICENSE](LICENSE).perimental. As versões 0.x podem quebrar compatibilidade, e cada quebra é registrada no [CHANGELOG](CHANGELOG.md).

## Licença

NeoCOBOL é distribuído sob a licença Apache License 2.0. Consulte o arquivo [LICENSE](LICENSE).