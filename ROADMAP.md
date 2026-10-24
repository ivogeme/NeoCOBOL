# Roteiro

Este roteiro é um plano e pode mudar. Cada versão tem um foco, e as quebras de compatibilidade são registradas no [CHANGELOG.md).



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
9. Especificação oficial da sintaxe e da semântica, escrita aos poucos e concluída antes da versão 1.0.

