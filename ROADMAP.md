# Roteiro

Este roteiro é um plano e pode mudar. Cada versão tem um foco, e as quebras de compatibilidade são registradas no [CHANGELOG](CHANGELOG.md).



## 0.6.1 - Avisos e vários erros

- Avisos NEO4xxx: variável declarada e não utilizada; código inalcançável; valor calculado e descartado (a decidir, porque hoje só chamadas de função podem ser instruções).
- Vários diagnósticos de uma vez, começando pela análise semântica e depois pelo parser.
- Resumo dos erros encontrados.
- Saída estruturada em JSON, preparando a integração com ferramentas de IDE.
- Sugestões do tipo "você quis dizer...?" para nomes parecidos.

## 0.6.2

1. Definir o idioma das palavras da linguagem.
2. Arredondamento e formatação de decimais e moeda.
3. Conversões explícitas e seguras de tipos.
4. Entrada de dados e arquivos (leitura e gravação).
5. Registros (dados organizados em campos), arrays, `else if`, `for` e chaves literais em textos.
6. Biblioteca padrão.
7. Módulos, com visibilidade pública e privada, e pacotes.
8. Bibliotecas externas, em caráter experimental, depois que módulos e pacotes estiverem estáveis.
9. Especificação oficial da sintaxe e da semântica, escrita aos poucos e concluída antes da versão 1.0.a maioria já existe e será reclassificada): caractere desconhecido, string não terminada, número decimal inválido, expressão incompleta, delimitador ausente, tipos incompatíveis, variável não declarada, função não encontrada, quantidade incorreta de argumentos, tipo incorreto de argumento, tipo de retorno incompatível, `return` fora de função, retorno obrigatório ausente, declaração duplicada, divisão por zero, módulo por zero e limite de chamadas excedido.

### 2. Ferramentas

- Comando `neocobol check`, que verifica o código sem executá-lo.
- Saída limpa por padrão: só o resultado do programa ou os erros. O resultado vai para a saída normal e os diagnósticos para a saída de erros.
- As linhas de etapas (Lexer, Parser, Semântica) aparecem apenas com `--detalhes`.
- O comando se chama `neocobol`, em minúsculas.
- Entradas inválidas nunca devem causar panic em Rust.

### 3. Organização do código

- Primeiro passo: mover os tipos de erro e de posição para um módulo próprio.
- Todo código novo já nasce em módulos separados. O código antigo continua em um arquivo único por enquanto.

### 4. Testes

- Verificar código, mensagem, arquivo, linha, coluna, trecho e sugestão de cada diagnóstico.
- Pasta `testes/` com programas `.neo` completos e a saída esperada de cada um, comparados automaticamente (inclusive os `erro_*.neo`).
- Testar entradas inválidas e casos extremos.
- Garantir que as correções não quebrem o que já funciona.

## 0.6.1 - Avisos e vários erros

- Avisos NEO4xxx: variável declarada e não utilizada; código inalcançável; valor calculado e descartado (a decidir, porque hoje só chamadas de função podem ser instruções).
- Vários diagnósticos de uma vez, começando pela análise semântica e depois pelo parser.
- Resumo dos erros encontrados.
- Saída estruturada em JSON, preparando a integração com ferramentas de IDE.
- Sugestões do tipo "você quis dizer...?" para nomes parecidos.

## Próximas versões (ordem proposta)

1. Definir o idioma das palavras da linguagem.
2. Arredondamento e formatação de decimais e moeda.
3. Conversões explícitas e seguras de tipos.
4. Entrada de dados e arquivos (leitura e gravação).
5. Registros (dados organizados em campos), arrays, `else if`, `for` e chaves literais em textos.
6. Biblioteca padrão.
7. Módulos, com visibilidade pública e privada, e pacotes.
8. Bibliotecas externas, em caráter experimental, depois que módulos e pacotes estiverem estáveis.
9. Especificação oficial da sintaxe e da semântica, escrita aos poucos e concluída antes da versão 1.0.