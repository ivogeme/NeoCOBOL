
Este documento preserva o roteiro original da NeoCOBOL. Para consultar os planos atuais, veja ROADMAP.md.

0.3 → núcleo funcional
0.4 → funções e escopos
0.5 → biblioteca padrão
0.6 → módulos e pacotes
0.7 → concorrência e recursos modernos
0.8 → compilador nativo
0.9 → estabilização
1.0 → especificação + compilador + biblioteca + ferramentas


Eu faria o próximo teste
Antes de adicionarmos mais funcionalidades, vale testar string + concatenação + boolean + not, porque assim cobrimos praticamente todos os tipos atuais.
Depois disso, eu partiria para a próxima grande evolução: funções.
Por exemplo, queremos chegar a algo conceitualmente parecido com:
function saudacao(nome)
    display "Olá, {nome}!"
end

saudacao("NeoCOBOL")
Isso seria um salto enorme, porque começaria a transformar o NeoCOBOL de uma linguagem que executa instruções em uma linguagem capaz de organizar programas reais.
E eu manteria a versão como 0.3.0 por enquanto. Não há necessidade de correr para 1.0. É melhor deixar a versão 1.0 ser aquele momento em que você olha para o projeto e pensa: agora isso realmente é uma linguagem. 🛠️
Seu teste do while passou. O próximo pedaço da criatura pode crescer em cima dessa 

E tem uma coisa que eu mudaria no seu projeto
Eu não abandonaria o interpretador quando criarmos o compilador.
Ele pode continuar existindo como ferramenta de desenvolvimento:
neocobol run programa.neo
para testar rapidamente.
E:
neocobol build programa.neo
para gerar o executável.
Isso dá à linguagem dois modos:
Run
Código → interpretador
rápido para desenvolvimento.
Build
Código → compilador → executável nativo
para produção.
Isso é muito mais interessante do que simplesmente jogar fora o runtime.


O que eu faria daqui para frente
NeoCOBOL 0.3.0, agora
Você já tem uma base funcional:
lexer
parser
AST
tipos decimal, string e boolean
operações matemáticas
comparações
if/else
while
interpolação de strings
mensagens de erro
execução de arquivos .neo
decimal baseado em rust_decimal
gerenciamento automático de memória pelo Rust
Então ela já merece estar no GitHub.
Mas eu colocaria como:
NeoCOBOL 0.3.0 Experimental
e não como 1.0.
O caminho que eu sugiro
0.4.0: melhorar a linguagem
Primeiro eu fortaleceria o que já existe:
variáveis
constantes
funções
retorno
escopo
arrays
datas
mais operações
strings melhores
Também corrigiria algumas limitações do parser atual.
0.5.0: biblioteca padrão
Começaria a criar coisas realmente úteis:
arquivo
diretório
data/hora
texto
JSON
rede
ambiente
Por exemplo, futuramente algo como:
arquivo = ler("dados.txt")
display arquivo
em vez de o programador precisar lidar diretamente com APIs complicadas do sistema.
0.6.0: módulos e pacotes
Algo como:
import banco
import arquivos
import json
E começar a construir um sistema de pacotes.
0.7.0: concorrência
Aqui entraria uma das características importantes da linguagem:
task processar()
ou uma sintaxe semelhante que definirmos.
O objetivo seria esconder boa parte da complexidade de threads, sincronização e comunicação.
0.8.0: compilador
Aqui começa a grande mudança.
Hoje:
.neo
 ↓
Lexer
 ↓
Parser
 ↓
AST
 ↓
Runtime
 ↓
executa
Queremos:
.neo
 ↓
Lexer
 ↓
Parser
 ↓
AST
 ↓
verificação de tipos
 ↓
IR
 ↓
código nativo
 ↓
.exe / ELF
Podemos usar LLVM inicialmente em vez de escrever um backend inteiro do zero.
0.9.0: compilador utilizável
A linguagem já deveria conseguir produzir programas independentes:
neocobol programa.neo -o programa
e depois:
./programa
Sem precisar do interpretador para executar.
1.0.0: primeira versão estável
Eu só chamaria de 1.0 quando tivermos:
sintaxe relativamente estável
compilador funcional
mensagens de erro boas
sistema de módulos
biblioteca padrão básica
arquivos
strings
datas
decimal financeiro
JSON
HTTP
concorrência básica
Unicode
documentação
testes automatizados
exemplos
código aberto
executáveis para os principais sistemas
Aí sim seria uma linguagem que alguém poderia instalar e realmente usar em um projet


