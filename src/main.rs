use rust_decimal::Decimal;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::str::FromStr;

// ================================================================
// NeoCOBOL 0.5.1
//
// Foco desta versão:
// - funções com parâmetros tipados e retorno tipado
// - análise semântica completa dos corpos das funções
// - escopo lexical: funções só enxergam parâmetros e globais
// - limite de profundidade de chamadas
// - linha + coluna em tokens, AST e erros
// - true/false separados do tipo boolean
// - parser defensivo, sem unwrap/unreachable em entrada do usuário
// - mensagens de erro mais detalhadas
// - suíte de testes para erros e casos extremos
// ================================================================

const MAX_PROFUNDIDADE_FUNCAO: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Posicao {
    linha: usize,
    coluna: usize,
}

impl Posicao {
    fn nova(linha: usize, coluna: usize) -> Self {
        Self { linha, coluna }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tipo {
    Decimal,
    String,
    Boolean,
    Vazio,
}

impl Tipo {
    fn nome(&self) -> &'static str {
        match self {
            Tipo::Decimal => "decimal",
            Tipo::String => "string",
            Tipo::Boolean => "boolean",
            Tipo::Vazio => "vazio",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum NeoValor {
    Decimal(Decimal),
    String(String),
    Boolean(bool),
    Vazio,
}

impl NeoValor {
    fn tipo(&self) -> Tipo {
        match self {
            NeoValor::Decimal(_) => Tipo::Decimal,
            NeoValor::String(_) => Tipo::String,
            NeoValor::Boolean(_) => Tipo::Boolean,
            NeoValor::Vazio => Tipo::Vazio,
        }
    }

    fn texto(&self) -> String {
        match self {
            NeoValor::Decimal(v) => v.to_string(),
            NeoValor::String(v) => v.clone(),
            NeoValor::Boolean(v) => v.to_string(),
            NeoValor::Vazio => String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CodigoErro {
    Lexico,
    Sintaxe,
    Tipo,
    Semantico,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NeoErro {
    Lexico { pos: Posicao, mensagem: String },
    Sintaxe { pos: Posicao, mensagem: String },
    Tipo {
        pos: Posicao,
        esperado: String,
        recebido: String,
        contexto: String,
    },
    Semantico { pos: Posicao, mensagem: String },
    Runtime { pos: Posicao, mensagem: String },
}

impl NeoErro {
    fn codigo(&self) -> CodigoErro {
        match self {
            NeoErro::Lexico { .. } => CodigoErro::Lexico,
            NeoErro::Sintaxe { .. } => CodigoErro::Sintaxe,
            NeoErro::Tipo { .. } => CodigoErro::Tipo,
            NeoErro::Semantico { .. } => CodigoErro::Semantico,
            NeoErro::Runtime { .. } => CodigoErro::Runtime,
        }
    }

    fn posicao(&self) -> Posicao {
        match self {
            NeoErro::Lexico { pos, .. }
            | NeoErro::Sintaxe { pos, .. }
            | NeoErro::Tipo { pos, .. }
            | NeoErro::Semantico { pos, .. }
            | NeoErro::Runtime { pos, .. } => *pos,
        }
    }
}

impl fmt::Display for NeoErro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeoErro::Lexico { pos, mensagem } => write!(
                f,
                "NEO1001 - Erro léxico na linha {}, coluna {}: {}",
                pos.linha, pos.coluna, mensagem
            ),
            NeoErro::Sintaxe { pos, mensagem } => write!(
                f,
                "NEO1002 - Erro de sintaxe na linha {}, coluna {}: {}",
                pos.linha, pos.coluna, mensagem
            ),
            NeoErro::Tipo {
                pos,
                esperado,
                recebido,
                contexto,
            } => write!(
                f,
                "NEO2001 - Erro de tipo na linha {}, coluna {}: esperado {}, recebido {}. {}",
                pos.linha, pos.coluna, esperado, recebido, contexto
            ),
            NeoErro::Semantico { pos, mensagem } => write!(
                f,
                "NEO2002 - Erro semântico na linha {}, coluna {}: {}",
                pos.linha, pos.coluna, mensagem
            ),
            NeoErro::Runtime { pos, mensagem } => write!(
                f,
                "NEO3001 - Erro de execução na linha {}, coluna {}: {}",
                pos.linha, pos.coluna, mensagem
            ),
        }
    }
}

// ================================================================
// TOKENS
// ================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    DecimalTipo,
    StringTipo,
    BooleanTipo,
    VazioTipo,

    Verdadeiro,
    Falso,

    Display,
    If,
    Else,
    End,
    While,
    Func,
    Return,

    And,
    Or,
    Not,

    Identificador,
    Numero,
    Texto,

    Mais,
    Menos,
    Multiplica,
    Divide,
    Modulo,

    Igual,
    IgualIgual,
    Diferente,
    Maior,
    Menor,
    MaiorIgual,
    MenorIgual,

    MaisIgual,
    MenosIgual,
    MultiplicaIgual,
    DivideIgual,
    ModuloIgual,

    AbrePar,
    FechaPar,
    Virgula,
    DoisPontos,
    Seta,

    Fim,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    texto: String,
    pos: Posicao,
}

fn lex(codigo: &str) -> Result<Vec<Token>, NeoErro> {
    let mut tokens = Vec::new();

    for (indice_linha, linha) in codigo.lines().enumerate() {
        let linha_numero = indice_linha + 1;
        let chars: Vec<char> = linha.chars().collect();
        let mut i = 0usize;

        while i < chars.len() {
            let c = chars[i];
            let coluna = i + 1;
            let pos = Posicao::nova(linha_numero, coluna);

            if c.is_whitespace() {
                i += 1;
                continue;
            }

            if c == '#' {
                break;
            }

            if c == '"' {
                i += 1;
                let inicio = i;
                let mut texto = String::new();
                let mut fechou = false;

                while i < chars.len() {
                    let ch = chars[i];
                    if ch == '"' {
                        fechou = true;
                        i += 1;
                        break;
                    }
                    texto.push(ch);
                    i += 1;
                }

                if !fechou {
                    return Err(NeoErro::Lexico {
                        pos,
                        mensagem: format!(
                            "String não terminada. A string começou na coluna {} e deveria terminar com '\"'.",
                            coluna
                        ),
                    });
                }

                if i == inicio {
                    // String vazia é válida. Mantido explicitamente para deixar
                    // a intenção clara e evitar qualquer caminho implícito.
                }

                tokens.push(Token {
                    kind: TokenKind::Texto,
                    texto,
                    pos,
                });
                continue;
            }

            if c.is_ascii_digit() {
                let inicio = i;
                let mut pontos = 0usize;

                while i < chars.len() {
                    let ch = chars[i];
                    if ch.is_ascii_digit() {
                        i += 1;
                    } else if ch == '.' {
                        pontos += 1;
                        if pontos > 1 {
                            return Err(NeoErro::Lexico {
                                pos: Posicao::nova(linha_numero, i + 1),
                                mensagem: "Número possui mais de um ponto decimal. Exemplo válido: 12.50.".into(),
                            });
                        }
                        i += 1;
                    } else {
                        break;
                    }
                }

                let texto: String = chars[inicio..i].iter().collect();
                if texto == "." {
                    return Err(NeoErro::Lexico {
                        pos,
                        mensagem: "Número decimal inválido.".into(),
                    });
                }

                if Decimal::from_str(&texto).is_err() {
                    return Err(NeoErro::Lexico {
                        pos,
                        mensagem: format!("Número decimal inválido: '{}'.", texto),
                    });
                }

                tokens.push(Token {
                    kind: TokenKind::Numero,
                    texto,
                    pos,
                });
                continue;
            }

            if c.is_alphabetic() || c == '_' {
                let inicio = i;
                i += 1;
                while i < chars.len() {
                    let ch = chars[i];
                    if ch.is_alphanumeric() || ch == '_' {
                        i += 1;
                    } else {
                        break;
                    }
                }

                let palavra: String = chars[inicio..i].iter().collect();
                let kind = match palavra.as_str() {
                    "decimal" => TokenKind::DecimalTipo,
                    "string" => TokenKind::StringTipo,
                    "boolean" => TokenKind::BooleanTipo,
                    "vazio" => TokenKind::VazioTipo,
                    "true" => TokenKind::Verdadeiro,
                    "false" => TokenKind::Falso,
                    "display" => TokenKind::Display,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "end" => TokenKind::End,
                    "while" => TokenKind::While,
                    "func" => TokenKind::Func,
                    "return" => TokenKind::Return,
                    "and" => TokenKind::And,
                    "or" => TokenKind::Or,
                    "not" => TokenKind::Not,
                    _ => TokenKind::Identificador,
                };

                tokens.push(Token {
                    kind,
                    texto: palavra,
                    pos,
                });
                continue;
            }

            let mut token = None;
            let mut consumo = 1usize;

            match c {
                '+' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        token = Some((TokenKind::MaisIgual, "+="));
                        consumo = 2;
                    } else {
                        token = Some((TokenKind::Mais, "+"));
                    }
                }
                '-' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        token = Some((TokenKind::MenosIgual, "-="));
                        consumo = 2;
                    } else if i + 1 < chars.len() && chars[i + 1] == '>' {
                        token = Some((TokenKind::Seta, "->"));
                        consumo = 2;
                    } else {
                        token = Some((TokenKind::Menos, "-"));
                    }
                }
                '*' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        token = Some((TokenKind::MultiplicaIgual, "*="));
                        consumo = 2;
                    } else {
                        token = Some((TokenKind::Multiplica, "*"));
                    }
                }
                '/' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        token = Some((TokenKind::DivideIgual, "/="));
                        consumo = 2;
                    } else {
                        token = Some((TokenKind::Divide, "/"));
                    }
                }
                '%' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        token = Some((TokenKind::ModuloIgual, "%="));
                        consumo = 2;
                    } else {
                        token = Some((TokenKind::Modulo, "%"));
                    }
                }
                '=' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        token = Some((TokenKind::IgualIgual, "=="));
                        consumo = 2;
                    } else {
                        token = Some((TokenKind::Igual, "="));
                    }
                }
                '!' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        token = Some((TokenKind::Diferente, "!="));
                        consumo = 2;
                    } else {
                        return Err(NeoErro::Lexico {
                            pos,
                            mensagem: "Caractere '!' isolado. Use '!=' para diferença.".into(),
                        });
                    }
                }
                '>' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        token = Some((TokenKind::MaiorIgual, ">="));
                        consumo = 2;
                    } else {
                        token = Some((TokenKind::Maior, ">"));
                    }
                }
                '<' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        token = Some((TokenKind::MenorIgual, "<="));
                        consumo = 2;
                    } else {
                        token = Some((TokenKind::Menor, "<"));
                    }
                }
                '(' => token = Some((TokenKind::AbrePar, "(")),
                ')' => token = Some((TokenKind::FechaPar, ")")),
                ',' => token = Some((TokenKind::Virgula, ",")),
                ':' => token = Some((TokenKind::DoisPontos, ":")),
                _ => {}
            }

            if let Some((kind, texto)) = token {
                tokens.push(Token {
                    kind,
                    texto: texto.into(),
                    pos,
                });
                i += consumo;
            } else {
                return Err(NeoErro::Lexico {
                    pos,
                    mensagem: format!("Caractere desconhecido '{}'.", c),
                });
            }
        }
    }

    let ultima_linha = codigo.lines().count().max(1) + 1;
    let ultima_coluna = codigo
        .lines()
        .last()
        .map(|l| l.chars().count() + 1)
        .unwrap_or(1);

    tokens.push(Token {
        kind: TokenKind::Fim,
        texto: String::new(),
        pos: Posicao::nova(ultima_linha, ultima_coluna),
    });

    Ok(tokens)
}

// ================================================================
// AST
// ================================================================

#[derive(Debug, Clone)]
struct Expr {
    kind: Expressao,
    pos: Posicao,
}

#[derive(Debug, Clone)]
enum Expressao {
    Decimal(Decimal),
    String(String),
    Boolean(bool),
    Variavel(String),
    Soma(Box<Expr>, Box<Expr>),
    Subtracao(Box<Expr>, Box<Expr>),
    Multiplicacao(Box<Expr>, Box<Expr>),
    Divisao(Box<Expr>, Box<Expr>),
    Modulo(Box<Expr>, Box<Expr>),
    Negativo(Box<Expr>),
    Maior(Box<Expr>, Box<Expr>),
    Menor(Box<Expr>, Box<Expr>),
    MaiorIgual(Box<Expr>, Box<Expr>),
    MenorIgual(Box<Expr>, Box<Expr>),
    Igual(Box<Expr>, Box<Expr>),
    Diferente(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Chamada { nome: String, argumentos: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OperadorAtribuicao {
    Igual,
    Mais,
    Menos,
    Multiplica,
    Divide,
    Modulo,
}

#[derive(Debug, Clone)]
enum Instrucao {
    Declarar {
        tipo: Tipo,
        nome: String,
        valor: Expr,
        pos: Posicao,
    },
    Atribuir {
        nome: String,
        operador: OperadorAtribuicao,
        valor: Expr,
        pos: Posicao,
    },
    Display {
        valor: Expr,
        pos: Posicao,
    },
    If {
        condicao: Expr,
        verdadeiro: Vec<Instrucao>,
        falso: Vec<Instrucao>,
        pos: Posicao,
    },
    While {
        condicao: Expr,
        bloco: Vec<Instrucao>,
        pos: Posicao,
    },
    Funcao {
        nome: String,
        parametros: Vec<Parametro>,
        retorno: Tipo,
        bloco: Vec<Instrucao>,
        pos: Posicao,
    },
    Return {
        valor: Option<Expr>,
        pos: Posicao,
    },
}

#[derive(Debug, Clone)]
struct Parametro {
    nome: String,
    tipo: Tipo,
    pos: Posicao,
}

impl Instrucao {
    fn pos(&self) -> Posicao {
        match self {
            Instrucao::Declarar { pos, .. }
            | Instrucao::Atribuir { pos, .. }
            | Instrucao::Display { pos, .. }
            | Instrucao::If { pos, .. }
            | Instrucao::While { pos, .. }
            | Instrucao::Funcao { pos, .. }
            | Instrucao::Return { pos, .. } => *pos,
        }
    }
}

// ================================================================
// PARSER
// ================================================================

struct Parser {
    tokens: Vec<Token>,
    atual: usize,
}

impl Parser {
    fn novo(tokens: Vec<Token>) -> Self {
        Self { tokens, atual: 0 }
    }

    fn atual(&self) -> Option<&Token> {
        self.tokens.get(self.atual)
    }

    fn token_atual_seguro(&self) -> Token {
        self.atual().cloned().unwrap_or(Token {
            kind: TokenKind::Fim,
            texto: String::new(),
            pos: Posicao::nova(1, 1),
        })
    }

    fn avancar(&mut self) {
        if self.atual + 1 < self.tokens.len() {
            self.atual += 1;
        } else if self.tokens.is_empty() {
            self.atual = 0;
        }
    }

    fn consumir(&mut self, esperado: TokenKind) -> Result<Token, NeoErro> {
        let token = self.token_atual_seguro();
        if token.kind == esperado {
            self.avancar();
            Ok(token)
        } else {
            Err(NeoErro::Sintaxe {
                pos: token.pos,
                mensagem: format!(
                    "Esperado {}, mas encontrado '{}'.",
                    nome_token(&esperado),
                    token.texto
                ),
            })
        }
    }

    fn analisar(&mut self) -> Result<Vec<Instrucao>, NeoErro> {
        if self.tokens.is_empty() {
            return Err(NeoErro::Sintaxe {
                pos: Posicao::nova(1, 1),
                mensagem: "Fluxo de tokens vazio. Era esperado pelo menos o marcador de fim.".into(),
            });
        }

        let mut resultado = Vec::new();
        loop {
            let token = self.token_atual_seguro();
            if token.kind == TokenKind::Fim {
                break;
            }
            resultado.push(self.instrucao()?);
        }
        Ok(resultado)
    }

    fn instrucao(&mut self) -> Result<Instrucao, NeoErro> {
        let token = self.token_atual_seguro();
        match token.kind {
            TokenKind::DecimalTipo
            | TokenKind::StringTipo
            | TokenKind::BooleanTipo => self.declaracao(),
            TokenKind::Display => self.display(),
            TokenKind::If => self.if_instrucao(),
            TokenKind::While => self.while_instrucao(),
            TokenKind::Func => self.funcao(),
            TokenKind::Return => self.retorno(),
            TokenKind::Identificador => self.atribuicao(),
            _ => Err(NeoErro::Sintaxe {
                pos: token.pos,
                mensagem: format!(
                    "Instrução inesperada '{}'. Esperado início de declaração, atribuição, display, if, while, func ou return.",
                    token.texto
                ),
            }),
        }
    }

    fn declaracao(&mut self) -> Result<Instrucao, NeoErro> {
        let token = self.token_atual_seguro();
        let tipo = match token.kind {
            TokenKind::DecimalTipo => Tipo::Decimal,
            TokenKind::StringTipo => Tipo::String,
            TokenKind::BooleanTipo => Tipo::Boolean,
            _ => {
                return Err(NeoErro::Sintaxe {
                    pos: token.pos,
                    mensagem: "Tipo inválido em declaração.".into(),
                })
            }
        };
        self.avancar();

        let nome = self.consumir(TokenKind::Identificador)?;
        self.consumir(TokenKind::Igual)?;
        let valor = self.expressao()?;

        Ok(Instrucao::Declarar {
            tipo,
            nome: nome.texto,
            valor,
            pos: token.pos,
        })
    }

    fn atribuicao(&mut self) -> Result<Instrucao, NeoErro> {
        let nome = self.consumir(TokenKind::Identificador)?;
        let pos = nome.pos;
        let operador = match self.token_atual_seguro().kind {
            TokenKind::Igual => OperadorAtribuicao::Igual,
            TokenKind::MaisIgual => OperadorAtribuicao::Mais,
            TokenKind::MenosIgual => OperadorAtribuicao::Menos,
            TokenKind::MultiplicaIgual => OperadorAtribuicao::Multiplica,
            TokenKind::DivideIgual => OperadorAtribuicao::Divide,
            TokenKind::ModuloIgual => OperadorAtribuicao::Modulo,
            _ => {
                let t = self.token_atual_seguro();
                return Err(NeoErro::Sintaxe {
                    pos: t.pos,
                    mensagem: "Esperado '=', '+=', '-=', '*=', '/=' ou '%=' após o identificador.".into(),
                });
            }
        };
        self.avancar();
        let valor = self.expressao()?;
        Ok(Instrucao::Atribuir {
            nome: nome.texto,
            operador,
            valor,
            pos,
        })
    }

    fn display(&mut self) -> Result<Instrucao, NeoErro> {
        let token = self.consumir(TokenKind::Display)?;
        let valor = self.expressao()?;
        Ok(Instrucao::Display { valor, pos: token.pos })
    }

    fn if_instrucao(&mut self) -> Result<Instrucao, NeoErro> {
        let token = self.consumir(TokenKind::If)?;
        let condicao = self.expressao()?;
        let verdadeiro = self.bloco_ate(&[TokenKind::Else, TokenKind::End, TokenKind::Fim])?;
        let falso = if self.token_atual_seguro().kind == TokenKind::Else {
            self.avancar();
            self.bloco_ate(&[TokenKind::End, TokenKind::Fim])?
        } else {
            Vec::new()
        };
        self.consumir(TokenKind::End)?;
        Ok(Instrucao::If {
            condicao,
            verdadeiro,
            falso,
            pos: token.pos,
        })
    }

    fn while_instrucao(&mut self) -> Result<Instrucao, NeoErro> {
        let token = self.consumir(TokenKind::While)?;
        let condicao = self.expressao()?;
        let bloco = self.bloco_ate(&[TokenKind::End, TokenKind::Fim])?;
        self.consumir(TokenKind::End)?;
        Ok(Instrucao::While {
            condicao,
            bloco,
            pos: token.pos,
        })
    }

    fn bloco_ate(&mut self, finais: &[TokenKind]) -> Result<Vec<Instrucao>, NeoErro> {
        let mut bloco = Vec::new();
        loop {
            let token = self.token_atual_seguro();
            if finais.iter().any(|k| *k == token.kind) {
                break;
            }
            bloco.push(self.instrucao()?);
        }
        Ok(bloco)
    }

    fn funcao(&mut self) -> Result<Instrucao, NeoErro> {
        let inicio = self.consumir(TokenKind::Func)?;
        let nome = self.consumir(TokenKind::Identificador)?;
        self.consumir(TokenKind::AbrePar)?;

        let mut parametros = Vec::new();
        if self.token_atual_seguro().kind != TokenKind::FechaPar {
            loop {
                let parametro = self.consumir(TokenKind::Identificador)?;
                self.consumir(TokenKind::DoisPontos)?;
                let tipo = self.tipo_funcao()?;
                parametros.push(Parametro {
                    nome: parametro.texto,
                    tipo,
                    pos: parametro.pos,
                });

                if self.token_atual_seguro().kind == TokenKind::Virgula {
                    self.avancar();
                } else {
                    break;
                }
            }
        }

        self.consumir(TokenKind::FechaPar)?;
        self.consumir(TokenKind::Seta)?;
        let retorno = self.tipo_funcao()?;
        let bloco = self.bloco_ate(&[TokenKind::End, TokenKind::Fim])?;
        self.consumir(TokenKind::End)?;

        Ok(Instrucao::Funcao {
            nome: nome.texto,
            parametros,
            retorno,
            bloco,
            pos: inicio.pos,
        })
    }

    fn tipo_funcao(&mut self) -> Result<Tipo, NeoErro> {
        let token = self.token_atual_seguro();
        let tipo = match token.kind {
            TokenKind::DecimalTipo => Tipo::Decimal,
            TokenKind::StringTipo => Tipo::String,
            TokenKind::BooleanTipo => Tipo::Boolean,
            TokenKind::VazioTipo => Tipo::Vazio,
            _ => {
                return Err(NeoErro::Sintaxe {
                    pos: token.pos,
                    mensagem: "Esperado um tipo de função: decimal, string, boolean ou vazio.".into(),
                })
            }
        };
        self.avancar();
        Ok(tipo)
    }

    fn retorno(&mut self) -> Result<Instrucao, NeoErro> {
        let token = self.consumir(TokenKind::Return)?;
        let valor = match self.token_atual_seguro().kind {
            TokenKind::End | TokenKind::Else | TokenKind::Fim => None,
            _ => Some(self.expressao()?),
        };
        Ok(Instrucao::Return {
            valor,
            pos: token.pos,
        })
    }

    fn expressao(&mut self) -> Result<Expr, NeoErro> {
        self.or()
    }

    fn or(&mut self) -> Result<Expr, NeoErro> {
        let mut esquerda = self.and()?;
        while self.token_atual_seguro().kind == TokenKind::Or {
            self.avancar();
            let direita = self.and()?;
            let pos = esquerda.pos;
            esquerda = Expr {
                kind: Expressao::Or(Box::new(esquerda), Box::new(direita)),
                pos,
            };
        }
        Ok(esquerda)
    }

    fn and(&mut self) -> Result<Expr, NeoErro> {
        let mut esquerda = self.comparacao()?;
        while self.token_atual_seguro().kind == TokenKind::And {
            self.avancar();
            let direita = self.comparacao()?;
            let pos = esquerda.pos;
            esquerda = Expr {
                kind: Expressao::And(Box::new(esquerda), Box::new(direita)),
                pos,
            };
        }
        Ok(esquerda)
    }

    fn comparacao(&mut self) -> Result<Expr, NeoErro> {
        let mut esquerda = self.termo()?;
        loop {
            let kind = self.token_atual_seguro().kind.clone();
            let op = match kind {
                TokenKind::Maior => 0,
                TokenKind::Menor => 1,
                TokenKind::MaiorIgual => 2,
                TokenKind::MenorIgual => 3,
                TokenKind::IgualIgual => 4,
                TokenKind::Diferente => 5,
                _ => break,
            };
            self.avancar();
            let direita = self.termo()?;
            let pos = esquerda.pos;
            esquerda = Expr {
                kind: match op {
                    0 => Expressao::Maior(Box::new(esquerda), Box::new(direita)),
                    1 => Expressao::Menor(Box::new(esquerda), Box::new(direita)),
                    2 => Expressao::MaiorIgual(Box::new(esquerda), Box::new(direita)),
                    3 => Expressao::MenorIgual(Box::new(esquerda), Box::new(direita)),
                    4 => Expressao::Igual(Box::new(esquerda), Box::new(direita)),
                    _ => Expressao::Diferente(Box::new(esquerda), Box::new(direita)),
                },
                pos,
            };
        }
        Ok(esquerda)
    }

    fn termo(&mut self) -> Result<Expr, NeoErro> {
        let mut esquerda = self.fator()?;
        loop {
            let kind = self.token_atual_seguro().kind.clone();
            if kind != TokenKind::Mais && kind != TokenKind::Menos {
                break;
            }
            self.avancar();
            let direita = self.fator()?;
            let pos = esquerda.pos;
            esquerda = Expr {
                kind: if kind == TokenKind::Mais {
                    Expressao::Soma(Box::new(esquerda), Box::new(direita))
                } else {
                    Expressao::Subtracao(Box::new(esquerda), Box::new(direita))
                },
                pos,
            };
        }
        Ok(esquerda)
    }

    fn fator(&mut self) -> Result<Expr, NeoErro> {
        let mut esquerda = self.unario()?;
        loop {
            let kind = self.token_atual_seguro().kind.clone();
            if !matches!(kind, TokenKind::Multiplica | TokenKind::Divide | TokenKind::Modulo) {
                break;
            }
            self.avancar();
            let direita = self.unario()?;
            let pos = esquerda.pos;
            esquerda = Expr {
                kind: match kind {
                    TokenKind::Multiplica => Expressao::Multiplicacao(Box::new(esquerda), Box::new(direita)),
                    TokenKind::Divide => Expressao::Divisao(Box::new(esquerda), Box::new(direita)),
                    _ => Expressao::Modulo(Box::new(esquerda), Box::new(direita)),
                },
                pos,
            };
        }
        Ok(esquerda)
    }

    fn unario(&mut self) -> Result<Expr, NeoErro> {
        let token = self.token_atual_seguro();
        if token.kind == TokenKind::Not {
            self.avancar();
            let valor = self.unario()?;
            return Ok(Expr {
                kind: Expressao::Not(Box::new(valor)),
                pos: token.pos,
            });
        }
        if token.kind == TokenKind::Menos {
            self.avancar();
            let valor = self.unario()?;
            return Ok(Expr {
                kind: Expressao::Negativo(Box::new(valor)),
                pos: token.pos,
            });
        }
        self.primario()
    }

    fn primario(&mut self) -> Result<Expr, NeoErro> {
        let token = self.token_atual_seguro();
        match token.kind {
            TokenKind::Numero => {
                self.avancar();
                let valor = Decimal::from_str(&token.texto).map_err(|_| NeoErro::Sintaxe {
                    pos: token.pos,
                    mensagem: format!("Número decimal inválido: '{}'.", token.texto),
                })?;
                Ok(Expr {
                    kind: Expressao::Decimal(valor),
                    pos: token.pos,
                })
            }
            TokenKind::Texto => {
                self.avancar();
                Ok(Expr {
                    kind: Expressao::String(token.texto),
                    pos: token.pos,
                })
            }
            TokenKind::Verdadeiro | TokenKind::Falso => {
                self.avancar();
                Ok(Expr {
                    kind: Expressao::Boolean(token.kind == TokenKind::Verdadeiro),
                    pos: token.pos,
                })
            }
            TokenKind::Identificador => {
                self.avancar();
                if self.token_atual_seguro().kind == TokenKind::AbrePar {
                    self.avancar();
                    let mut argumentos = Vec::new();
                    if self.token_atual_seguro().kind != TokenKind::FechaPar {
                        loop {
                            argumentos.push(self.expressao()?);
                            if self.token_atual_seguro().kind == TokenKind::Virgula {
                                self.avancar();
                            } else {
                                break;
                            }
                        }
                    }
                    self.consumir(TokenKind::FechaPar)?;
                    return Ok(Expr {
                        kind: Expressao::Chamada {
                            nome: token.texto,
                            argumentos,
                        },
                        pos: token.pos,
                    });
                }
                Ok(Expr {
                    kind: Expressao::Variavel(token.texto),
                    pos: token.pos,
                })
            }
            TokenKind::AbrePar => {
                self.avancar();
                let resultado = self.expressao()?;
                self.consumir(TokenKind::FechaPar)?;
                Ok(resultado)
            }
            _ => Err(NeoErro::Sintaxe {
                pos: token.pos,
                mensagem: format!(
                    "Expressão inesperada '{}'. Esperado número, string, true, false, variável, chamada de função ou expressão entre parênteses.",
                    token.texto
                ),
            }),
        }
    }
}

fn nome_token(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::DecimalTipo => "decimal",
        TokenKind::StringTipo => "string",
        TokenKind::BooleanTipo => "boolean",
        TokenKind::VazioTipo => "vazio",
        TokenKind::Verdadeiro => "true",
        TokenKind::Falso => "false",
        TokenKind::Display => "display",
        TokenKind::If => "if",
        TokenKind::Else => "else",
        TokenKind::End => "end",
        TokenKind::While => "while",
        TokenKind::Func => "func",
        TokenKind::Return => "return",
        TokenKind::And => "and",
        TokenKind::Or => "or",
        TokenKind::Not => "not",
        TokenKind::Identificador => "identificador",
        TokenKind::Numero => "número",
        TokenKind::Texto => "string literal",
        TokenKind::Mais => "+",
        TokenKind::Menos => "-",
        TokenKind::Multiplica => "*",
        TokenKind::Divide => "/",
        TokenKind::Modulo => "%",
        TokenKind::Igual => "=",
        TokenKind::IgualIgual => "==",
        TokenKind::Diferente => "!=",
        TokenKind::Maior => ">",
        TokenKind::Menor => "<",
        TokenKind::MaiorIgual => ">=",
        TokenKind::MenorIgual => "<=",
        TokenKind::MaisIgual => "+=",
        TokenKind::MenosIgual => "-=",
        TokenKind::MultiplicaIgual => "*=",
        TokenKind::DivideIgual => "/=",
        TokenKind::ModuloIgual => "%=",
        TokenKind::AbrePar => "(",
        TokenKind::FechaPar => ")",
        TokenKind::Virgula => ",",
        TokenKind::DoisPontos => ":",
        TokenKind::Seta => "->",
        TokenKind::Fim => "fim do arquivo",
    }
}

// ================================================================
// FUNÇÕES / SÍMBOLOS
// ================================================================

#[derive(Debug, Clone)]
struct Funcao {
    parametros: Vec<Parametro>,
    retorno: Tipo,
    bloco: Vec<Instrucao>,
    pos: Posicao,
}

#[derive(Debug, Clone)]
struct Simbolo {
    tipo: Tipo,
    pos: Posicao,
}

#[derive(Debug, Clone)]
struct AssinaturaFuncao {
    parametros: Vec<Tipo>,
    retorno: Tipo,
    pos: Posicao,
}

// ================================================================
// ANALISADOR SEMÂNTICO
// ================================================================

struct AnalisadorSemantico {
    globais: HashMap<String, Simbolo>,
    escopos_locais: Vec<HashMap<String, Simbolo>>,
    funcoes: HashMap<String, AssinaturaFuncao>,
    funcao_atual: Option<String>,
    retorno_atual: Option<Tipo>,
}

impl AnalisadorSemantico {
    fn novo() -> Self {
        Self {
            globais: HashMap::new(),
            escopos_locais: Vec::new(),
            funcoes: HashMap::new(),
            funcao_atual: None,
            retorno_atual: None,
        }
    }

    fn analisar(&mut self, programa: &[Instrucao]) -> Result<(), NeoErro> {
        // Passo 1: registra assinaturas de todas as funções.
        for instrucao in programa {
            if let Instrucao::Funcao {
                nome,
                parametros,
                retorno,
                pos,
                ..
            } = instrucao
            {
                if self.funcoes.contains_key(nome) {
                    return Err(NeoErro::Semantico {
                        pos: *pos,
                        mensagem: format!(
                            "A função '{}' já foi declarada. Cada função precisa ter um nome único.",
                            nome
                        ),
                    });
                }
                self.funcoes.insert(
                    nome.clone(),
                    AssinaturaFuncao {
                        parametros: parametros.iter().map(|p| p.tipo.clone()).collect(),
                        retorno: retorno.clone(),
                        pos: *pos,
                    },
                );
            }
        }

        // Passo 2: registra os tipos das variáveis globais.
        // Isso permite que funções sejam analisadas independentemente da
        // posição textual da declaração global.
        for instrucao in programa {
            if let Instrucao::Declarar { tipo, nome, pos, .. } = instrucao {
                if self.globais.contains_key(nome) {
                    return Err(NeoErro::Semantico {
                        pos: *pos,
                        mensagem: format!(
                            "A variável global '{}' já foi declarada neste nível.",
                            nome
                        ),
                    });
                }
                self.globais.insert(
                    nome.clone(),
                    Simbolo {
                        tipo: tipo.clone(),
                        pos: *pos,
                    },
                );
            }
        }

        // Passo 3: analisa cada função inteira.
        for instrucao in programa {
            if let Instrucao::Funcao {
                nome,
                parametros,
                retorno,
                bloco,
                pos,
            } = instrucao
            {
                self.analisar_funcao(nome, parametros, retorno, bloco, *pos)?;
            }
        }

        // Passo 4: analisa o código global. Funções são declarações, não execução.
        for instrucao in programa {
            if !matches!(instrucao, Instrucao::Funcao { .. }) {
                self.instrucao(instrucao)?;
            }
        }

        Ok(())
    }

    fn analisar_funcao(
        &mut self,
        nome: &str,
        parametros: &[Parametro],
        retorno: &Tipo,
        bloco: &[Instrucao],
        pos: Posicao,
    ) -> Result<(), NeoErro> {
        self.escopos_locais.clear();
        self.escopos_locais.push(HashMap::new());
        self.funcao_atual = Some(nome.to_string());
        self.retorno_atual = Some(retorno.clone());

        for parametro in parametros {
            let escopo = match self.escopos_locais.last_mut() {
                Some(e) => e,
                None => {
                    return Err(NeoErro::Semantico {
                        pos: parametro.pos,
                        mensagem: "Falha interna ao criar o escopo dos parâmetros.".into(),
                    })
                }
            };
            if escopo.contains_key(&parametro.nome) {
                return Err(NeoErro::Semantico {
                    pos: parametro.pos,
                    mensagem: format!(
                        "O parâmetro '{}' foi declarado mais de uma vez na função '{}'.",
                        parametro.nome, nome
                    ),
                });
            }
            escopo.insert(
                parametro.nome.clone(),
                Simbolo {
                    tipo: parametro.tipo.clone(),
                    pos: parametro.pos,
                },
            );
        }

        let resultado = self.analisar_bloco_funcao(bloco);
        if resultado.is_ok() && *retorno != Tipo::Vazio && !bloco_garante_retorno(bloco) {
            resultado.map(|_| ())?;
            return Err(NeoErro::Semantico {
                pos,
                mensagem: format!(
                    "A função '{}' declara retorno '{}' mas existe um caminho que pode terminar sem 'return'.",
                    nome,
                    retorno.nome()
                ),
            });
        }
        resultado?;

        self.escopos_locais.clear();
        self.funcao_atual = None;
        self.retorno_atual = None;
        Ok(())
    }

    fn analisar_bloco_funcao(&mut self, bloco: &[Instrucao]) -> Result<(), NeoErro> {
        for instrucao in bloco {
            self.instrucao(instrucao)?;
        }
        Ok(())
    }

    fn instrucao(&mut self, instrucao: &Instrucao) -> Result<(), NeoErro> {
        match instrucao {
            Instrucao::Declarar { tipo, nome, valor, pos } => {
                let ja_existe = self
                    .escopos_locais
                    .last()
                    .map(|escopo| escopo.contains_key(nome))
                    .unwrap_or(false);
                if ja_existe {
                    return Err(NeoErro::Semantico {
                        pos: *pos,
                        mensagem: format!(
                            "A variável '{}' já foi declarada neste escopo.",
                            nome
                        ),
                    });
                }

                let tipo_valor = self.expressao(valor)?;
                if tipo_valor != *tipo {
                    return Err(NeoErro::Tipo {
                        pos: *pos,
                        esperado: tipo.nome().into(),
                        recebido: tipo_valor.nome().into(),
                        contexto: format!(
                            "A declaração de '{}' exige que o valor inicial seja {}.",
                            nome,
                            tipo.nome()
                        ),
                    });
                }

                if let Some(escopo) = self.escopos_locais.last_mut() {
                    escopo.insert(
                        nome.clone(),
                        Simbolo {
                            tipo: tipo.clone(),
                            pos: *pos,
                        },
                    );
                }
                // No nível global, o símbolo já foi registrado no primeiro passe.

            }
            Instrucao::Atribuir { nome, operador, valor, pos } => {
                let tipo_variavel = self.buscar_variavel(nome).ok_or_else(|| NeoErro::Semantico {
                    pos: *pos,
                    mensagem: format!(
                        "A variável '{}' não existe neste escopo. Declare-a antes de usá-la.",
                        nome
                    ),
                })?;
                let tipo_valor = self.expressao(valor)?;

                match operador {
                    OperadorAtribuicao::Igual => {
                        if tipo_variavel != tipo_valor {
                            return Err(NeoErro::Tipo {
                                pos: *pos,
                                esperado: tipo_variavel.nome().into(),
                                recebido: tipo_valor.nome().into(),
                                contexto: format!(
                                    "A variável '{}' foi declarada como {} e não pode receber {}.",
                                    nome,
                                    tipo_variavel.nome(),
                                    tipo_valor.nome()
                                ),
                            });
                        }
                    }
                    OperadorAtribuicao::Mais => {
                        let valido =
                            (tipo_variavel == Tipo::Decimal && tipo_valor == Tipo::Decimal)
                                || (tipo_variavel == Tipo::String && tipo_valor == Tipo::String);
                        if !valido {
                            return Err(NeoErro::Tipo {
                                pos: *pos,
                                esperado: "decimal + decimal ou string + string".into(),
                                recebido: format!("{} + {}", tipo_variavel.nome(), tipo_valor.nome()),
                                contexto: "O operador '+=' exige dois operandos do mesmo tipo compatível.".into(),
                            });
                        }
                    }
                    OperadorAtribuicao::Menos
                    | OperadorAtribuicao::Multiplica
                    | OperadorAtribuicao::Divide
                    | OperadorAtribuicao::Modulo => {
                        if tipo_variavel != Tipo::Decimal || tipo_valor != Tipo::Decimal {
                            return Err(NeoErro::Tipo {
                                pos: *pos,
                                esperado: "decimal".into(),
                                recebido: format!("{} e {}", tipo_variavel.nome(), tipo_valor.nome()),
                                contexto: "Esta atribuição composta trabalha apenas com valores decimal.".into(),
                            });
                        }
                    }
                }
            }
            Instrucao::Display { valor, .. } => {
                self.expressao(valor)?;
            }
            Instrucao::If { condicao, verdadeiro, falso, pos } => {
                let tipo = self.expressao(condicao)?;
                if tipo != Tipo::Boolean {
                    return Err(NeoErro::Tipo {
                        pos: *pos,
                        esperado: "boolean".into(),
                        recebido: tipo.nome().into(),
                        contexto: "A condição de 'if' precisa produzir true ou false.".into(),
                    });
                }
                self.entrar_escopo();
                let r1 = verdadeiro.iter().try_for_each(|i| self.instrucao(i));
                self.sair_escopo();
                r1?;
                self.entrar_escopo();
                let r2 = falso.iter().try_for_each(|i| self.instrucao(i));
                self.sair_escopo();
                r2?;
            }
            Instrucao::While { condicao, bloco, pos } => {
                let tipo = self.expressao(condicao)?;
                if tipo != Tipo::Boolean {
                    return Err(NeoErro::Tipo {
                        pos: *pos,
                        esperado: "boolean".into(),
                        recebido: tipo.nome().into(),
                        contexto: "A condição de 'while' precisa produzir true ou false.".into(),
                    });
                }
                self.entrar_escopo();
                let r = bloco.iter().try_for_each(|i| self.instrucao(i));
                self.sair_escopo();
                r?;
            }
            Instrucao::Funcao { .. } => {
                return Err(NeoErro::Semantico {
                    pos: instrucao.pos(),
                    mensagem: "Uma definição de função não pode aparecer dentro de outra função ou bloco nesta versão.".into(),
                });
            }
            Instrucao::Return { valor, pos } => {
                let retorno = match &self.retorno_atual {
                    Some(t) => t.clone(),
                    None => {
                        return Err(NeoErro::Semantico {
                            pos: *pos,
                            mensagem: "'return' só pode ser usado dentro de uma função.".into(),
                        })
                    }
                };

                match (retorno, valor) {
                    (Tipo::Vazio, None) => {}
                    (Tipo::Vazio, Some(expr)) => {
                        let tipo = self.expressao(expr)?;
                        return Err(NeoErro::Tipo {
                            pos: *pos,
                            esperado: "vazio".into(),
                            recebido: tipo.nome().into(),
                            contexto: "Uma função '-> vazio' deve usar apenas 'return' sem valor, ou simplesmente terminar.".into(),
                        });
                    }
                    (esperado, None) => {
                        return Err(NeoErro::Tipo {
                            pos: *pos,
                            esperado: esperado.nome().into(),
                            recebido: "vazio".into(),
                            contexto: format!(
                                "A função atual exige um valor de retorno {}.",
                                esperado.nome()
                            ),
                        });
                    }
                    (esperado, Some(expr)) => {
                        let recebido = self.expressao(expr)?;
                        if esperado != recebido {
                            return Err(NeoErro::Tipo {
                                pos: *pos,
                                esperado: esperado.nome().into(),
                                recebido: recebido.nome().into(),
                                contexto: "O valor retornado precisa ter exatamente o tipo declarado na assinatura da função.".into(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn expressao(&self, expr: &Expr) -> Result<Tipo, NeoErro> {
        match &expr.kind {
            Expressao::Decimal(_) => Ok(Tipo::Decimal),
            Expressao::String(_) => Ok(Tipo::String),
            Expressao::Boolean(_) => Ok(Tipo::Boolean),
            Expressao::Variavel(nome) => self.buscar_variavel(nome).ok_or_else(|| NeoErro::Semantico {
                pos: expr.pos,
                mensagem: format!(
                    "A variável '{}' não existe neste escopo. Funções não podem acessar variáveis locais de quem as chamou.",
                    nome
                ),
            }),
            Expressao::Negativo(valor) => {
                let tipo = self.expressao(valor)?;
                if tipo != Tipo::Decimal {
                    return Err(NeoErro::Tipo {
                        pos: expr.pos,
                        esperado: "decimal".into(),
                        recebido: tipo.nome().into(),
                        contexto: "O operador unário '-' só pode ser aplicado a decimal.".into(),
                    });
                }
                Ok(Tipo::Decimal)
            }
            Expressao::Soma(a, b) => {
                let ta = self.expressao(a)?;
                let tb = self.expressao(b)?;
                match (&ta, &tb) {
                    (Tipo::Decimal, Tipo::Decimal) | (Tipo::String, Tipo::String) => Ok(ta),
                    _ => Err(NeoErro::Tipo {
                        pos: expr.pos,
                        esperado: "decimal + decimal ou string + string".into(),
                        recebido: format!("{} + {}", ta.nome(), tb.nome()),
                        contexto: "Não há conversão implícita entre os tipos.".into(),
                    }),
                }
            }
            Expressao::Subtracao(a, b)
            | Expressao::Multiplicacao(a, b)
            | Expressao::Divisao(a, b)
            | Expressao::Modulo(a, b) => {
                let ta = self.expressao(a)?;
                let tb = self.expressao(b)?;
                if ta != Tipo::Decimal || tb != Tipo::Decimal {
                    return Err(NeoErro::Tipo {
                        pos: expr.pos,
                        esperado: "decimal e decimal".into(),
                        recebido: format!("{} e {}", ta.nome(), tb.nome()),
                        contexto: "Este operador aritmético trabalha somente com decimal.".into(),
                    });
                }
                Ok(Tipo::Decimal)
            }
            Expressao::Maior(a, b)
            | Expressao::Menor(a, b)
            | Expressao::MaiorIgual(a, b)
            | Expressao::MenorIgual(a, b) => {
                let ta = self.expressao(a)?;
                let tb = self.expressao(b)?;
                if ta != Tipo::Decimal || tb != Tipo::Decimal {
                    return Err(NeoErro::Tipo {
                        pos: expr.pos,
                        esperado: "decimal e decimal".into(),
                        recebido: format!("{} e {}", ta.nome(), tb.nome()),
                        contexto: "Comparações ordenadas não fazem conversão automática de string ou boolean.".into(),
                    });
                }
                Ok(Tipo::Boolean)
            }
            Expressao::Igual(a, b) | Expressao::Diferente(a, b) => {
                let ta = self.expressao(a)?;
                let tb = self.expressao(b)?;
                if ta != tb {
                    return Err(NeoErro::Tipo {
                        pos: expr.pos,
                        esperado: ta.nome().into(),
                        recebido: tb.nome().into(),
                        contexto: "'==' e '!=' exigem operandos do mesmo tipo.".into(),
                    });
                }
                Ok(Tipo::Boolean)
            }
            Expressao::And(a, b) | Expressao::Or(a, b) => {
                let ta = self.expressao(a)?;
                let tb = self.expressao(b)?;
                if ta != Tipo::Boolean || tb != Tipo::Boolean {
                    return Err(NeoErro::Tipo {
                        pos: expr.pos,
                        esperado: "boolean e boolean".into(),
                        recebido: format!("{} e {}", ta.nome(), tb.nome()),
                        contexto: "Use 'and' e 'or' somente com condições boolean.".into(),
                    });
                }
                Ok(Tipo::Boolean)
            }
            Expressao::Not(a) => {
                let tipo = self.expressao(a)?;
                if tipo != Tipo::Boolean {
                    return Err(NeoErro::Tipo {
                        pos: expr.pos,
                        esperado: "boolean".into(),
                        recebido: tipo.nome().into(),
                        contexto: "'not' só pode ser aplicado a boolean.".into(),
                    });
                }
                Ok(Tipo::Boolean)
            }
            Expressao::Chamada { nome, argumentos } => {
                let assinatura = self.funcoes.get(nome).ok_or_else(|| NeoErro::Semantico {
                    pos: expr.pos,
                    mensagem: format!(
                        "A função '{}' não existe. Verifique o nome ou declare a função antes de executar o programa.",
                        nome
                    ),
                })?;
                if argumentos.len() != assinatura.parametros.len() {
                    return Err(NeoErro::Semantico {
                        pos: expr.pos,
                        mensagem: format!(
                            "A função '{}' espera {} argumento(s), mas recebeu {}.",
                            nome,
                            assinatura.parametros.len(),
                            argumentos.len()
                        ),
                    });
                }
                for (indice, (argumento, esperado)) in argumentos
                    .iter()
                    .zip(assinatura.parametros.iter())
                    .enumerate()
                {
                    let recebido = self.expressao(argumento)?;
                    if recebido != *esperado {
                        return Err(NeoErro::Tipo {
                            pos: argumento.pos,
                            esperado: esperado.nome().into(),
                            recebido: recebido.nome().into(),
                            contexto: format!(
                                "O argumento {} da função '{}' deveria ser {}.",
                                indice + 1,
                                nome,
                                esperado.nome()
                            ),
                        });
                    }
                }
                Ok(assinatura.retorno.clone())
            }
        }
    }

    fn buscar_variavel(&self, nome: &str) -> Option<Tipo> {
        for escopo in self.escopos_locais.iter().rev() {
            if let Some(simbolo) = escopo.get(nome) {
                return Some(simbolo.tipo.clone());
            }
        }
        self.globais.get(nome).map(|s| s.tipo.clone())
    }

    fn entrar_escopo(&mut self) {
        self.escopos_locais.push(HashMap::new());
    }

    fn sair_escopo(&mut self) {
        let _ = self.escopos_locais.pop();
    }
}

fn bloco_garante_retorno(bloco: &[Instrucao]) -> bool {
    for instrucao in bloco {
        match instrucao {
            Instrucao::Return { .. } => return true,
            Instrucao::If { verdadeiro, falso, .. } => {
                if !falso.is_empty() && bloco_garante_retorno(verdadeiro) && bloco_garante_retorno(falso) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

// ================================================================
// RUNTIME
// ================================================================

struct Runtime {
    globais: HashMap<String, NeoValor>,
    locais: Vec<HashMap<String, NeoValor>>,
    funcoes: HashMap<String, Funcao>,
    profundidade_funcoes: usize,
}

impl Runtime {
    fn novo() -> Self {
        Self {
            globais: HashMap::new(),
            locais: Vec::new(),
            funcoes: HashMap::new(),
            profundidade_funcoes: 0,
        }
    }

    fn executar(&mut self, programa: &[Instrucao]) -> Result<(), NeoErro> {
        for instrucao in programa {
            if let Instrucao::Funcao {
                nome,
                parametros,
                retorno,
                bloco,
                pos,
            } = instrucao
            {
                self.funcoes.insert(
                    nome.clone(),
                    Funcao {
                        parametros: parametros.clone(),
                        retorno: retorno.clone(),
                        bloco: bloco.clone(),
                        pos: *pos,
                    },
                );
            }
        }

        for instrucao in programa {
            if matches!(instrucao, Instrucao::Funcao { .. }) {
                continue;
            }
            if self.executar_instrucao(instrucao)?.is_some() {
                return Err(NeoErro::Semantico {
                    pos: instrucao.pos(),
                    mensagem: "'return' não pode escapar de uma função.".into(),
                });
            }
        }
        Ok(())
    }

    fn executar_instrucao(&mut self, instrucao: &Instrucao) -> Result<Option<NeoValor>, NeoErro> {
        match instrucao {
            Instrucao::Declarar { nome, valor, pos, .. } => {
                let resultado = self.avaliar(valor, *pos)?;
                if let Some(escopo) = self.locais.last_mut() {
                    escopo.insert(nome.clone(), resultado);
                } else {
                    self.globais.insert(nome.clone(), resultado);
                }
            }
            Instrucao::Atribuir { nome, operador, valor, pos } => {
                let novo = self.avaliar(valor, *pos)?;
                let antigo = self.buscar_valor(nome).ok_or_else(|| NeoErro::Runtime {
                    pos: *pos,
                    mensagem: format!("A variável '{}' não existe em nenhum escopo acessível.", nome),
                })?;
                let resultado = self.aplicar_atribuicao(antigo, novo, operador, *pos)?;
                if let Some(indice) = self.encontrar_escopo_local(nome) {
                    if let Some(escopo) = self.locais.get_mut(indice) {
                        escopo.insert(nome.clone(), resultado);
                    } else {
                        return Err(NeoErro::Runtime {
                            pos: *pos,
                            mensagem: "Falha ao acessar o escopo local durante atribuição.".into(),
                        });
                    }
                } else if self.globais.contains_key(nome) {
                    self.globais.insert(nome.clone(), resultado);
                } else {
                    return Err(NeoErro::Runtime {
                        pos: *pos,
                        mensagem: format!("A variável '{}' não existe em nenhum escopo acessível.", nome),
                    });
                }
            }
            Instrucao::Display { valor, pos } => {
                let resultado = self.avaliar(valor, *pos)?;
                let texto = self.interpolar(&resultado.texto(), *pos)?;
                println!("{}", texto);
            }
            Instrucao::If { condicao, verdadeiro, falso, pos } => {
                let resultado = self.avaliar(condicao, *pos)?;
                match resultado {
                    NeoValor::Boolean(true) => {
                        if let Some(valor) = self.executar_bloco(verdadeiro)? {
                            return Ok(Some(valor));
                        }
                    }
                    NeoValor::Boolean(false) => {
                        if let Some(valor) = self.executar_bloco(falso)? {
                            return Ok(Some(valor));
                        }
                    }
                    outro => {
                        return Err(NeoErro::Tipo {
                            pos: *pos,
                            esperado: "boolean".into(),
                            recebido: outro.tipo().nome().into(),
                            contexto: "A condição de 'if' precisa ser boolean.".into(),
                        });
                    }
                }
            }
            Instrucao::While { condicao, bloco, pos } => loop {
                let resultado = self.avaliar(condicao, *pos)?;
                match resultado {
                    NeoValor::Boolean(true) => {
                        if let Some(valor) = self.executar_bloco(bloco)? {
                            return Ok(Some(valor));
                        }
                    }
                    NeoValor::Boolean(false) => break,
                    outro => {
                        return Err(NeoErro::Tipo {
                            pos: *pos,
                            esperado: "boolean".into(),
                            recebido: outro.tipo().nome().into(),
                            contexto: "A condição de 'while' precisa ser boolean.".into(),
                        });
                    }
                }
            },
            Instrucao::Funcao { .. } => {}
            Instrucao::Return { valor, pos } => {
                if self.locais.is_empty() {
                    return Err(NeoErro::Semantico {
                        pos: *pos,
                        mensagem: "'return' só pode ser usado dentro de uma função.".into(),
                    });
                }
                let resultado = match valor {
                    Some(expr) => self.avaliar(expr, *pos)?,
                    None => NeoValor::Vazio,
                };
                return Ok(Some(resultado));
            }
        }
        Ok(None)
    }

    fn executar_bloco(&mut self, bloco: &[Instrucao]) -> Result<Option<NeoValor>, NeoErro> {
        self.locais.push(HashMap::new());
        for instrucao in bloco {
            match self.executar_instrucao(instrucao) {
                Ok(Some(valor)) => {
                    let _ = self.locais.pop();
                    return Ok(Some(valor));
                }
                Ok(None) => {}
                Err(erro) => {
                    let _ = self.locais.pop();
                    return Err(erro);
                }
            }
        }
        let _ = self.locais.pop();
        Ok(None)
    }

    fn avaliar(&mut self, expr: &Expr, pos_fallback: Posicao) -> Result<NeoValor, NeoErro> {
        let pos = expr.pos;
        match &expr.kind {
            Expressao::Decimal(v) => Ok(NeoValor::Decimal(*v)),
            Expressao::String(v) => Ok(NeoValor::String(v.clone())),
            Expressao::Boolean(v) => Ok(NeoValor::Boolean(*v)),
            Expressao::Variavel(nome) => self.buscar_valor(nome).ok_or_else(|| NeoErro::Runtime {
                pos,
                mensagem: format!("A variável '{}' não existe em nenhum escopo acessível.", nome),
            }),
            Expressao::Negativo(valor) => match self.avaliar(valor, pos_fallback)? {
                NeoValor::Decimal(v) => Ok(NeoValor::Decimal(-v)),
                outro => Err(NeoErro::Tipo {
                    pos,
                    esperado: "decimal".into(),
                    recebido: outro.tipo().nome().into(),
                    contexto: "O operador '-' exige decimal.".into(),
                }),
            },
            Expressao::Soma(a, b) => {
                let va = self.avaliar(a, pos_fallback)?;
                let vb = self.avaliar(b, pos_fallback)?;
                match (va, vb) {
                    (NeoValor::Decimal(a), NeoValor::Decimal(b)) => a
                    .checked_add(b)
                    .map(NeoValor::Decimal)
                    .ok_or_else(|| NeoErro::Runtime {
                        pos,
                        mensagem: "Overflow aritmético em '+'.".into(),
                    }),
                    (NeoValor::String(a), NeoValor::String(b)) => Ok(NeoValor::String(format!("{}{}", a, b))),
                    (a, b) => Err(NeoErro::Tipo {
                        pos,
                        esperado: "decimal + decimal ou string + string".into(),
                        recebido: format!("{} + {}", a.tipo().nome(), b.tipo().nome()),
                        contexto: "Os tipos precisam ser compatíveis com '+'.".into(),
                    }),
                }
            }
            Expressao::Subtracao(a, b) => self.operacao_decimal(a, b, pos, |x, y| x.checked_sub(y), "decimal - decimal", pos_fallback),
            Expressao::Multiplicacao(a, b) => self.operacao_decimal(a, b, pos, |x, y| x.checked_mul(y), "decimal * decimal", pos_fallback),
            Expressao::Divisao(a, b) => {
                let va = self.avaliar(a, pos_fallback)?;
                let vb = self.avaliar(b, pos_fallback)?;
                match (va, vb) {
                    (NeoValor::Decimal(a), NeoValor::Decimal(b)) => {
                        if b.is_zero() {
                            Err(NeoErro::Runtime {
                                pos,
                                mensagem: "Divisão por zero não é permitida.".into(),
                            })
                        } else {
                            a.checked_div(b)
                                .map(NeoValor::Decimal)
                                .ok_or_else(|| NeoErro::Runtime {
                                    pos,
                                    mensagem: "Overflow aritmético em '/'.".into(),
                                })
                        }
                    }
                    (a, b) => Err(NeoErro::Tipo {
                        pos,
                        esperado: "decimal / decimal".into(),
                        recebido: format!("{} / {}", a.tipo().nome(), b.tipo().nome()),
                        contexto: "A divisão exige dois valores decimal.".into(),
                    }),
                }
            }
            Expressao::Modulo(a, b) => {
                let va = self.avaliar(a, pos_fallback)?;
                let vb = self.avaliar(b, pos_fallback)?;
                match (va, vb) {
                    (NeoValor::Decimal(a), NeoValor::Decimal(b)) => {
                        if b.is_zero() {
                            Err(NeoErro::Runtime {
                                pos,
                                mensagem: "Módulo por zero não é permitido.".into(),
                            })
                        } else {
                            a.checked_rem(b)
                                .map(NeoValor::Decimal)
                                .ok_or_else(|| NeoErro::Runtime {
                                    pos,
                                    mensagem: "Overflow aritmético em '%'.".into(),
                                })
                        }
                    }
                    (a, b) => Err(NeoErro::Tipo {
                        pos,
                        esperado: "decimal % decimal".into(),
                        recebido: format!("{} % {}", a.tipo().nome(), b.tipo().nome()),
                        contexto: "O módulo exige dois valores decimal.".into(),
                    }),
                }
            }
            Expressao::Maior(a, b) => self.comparar(a, b, pos, |x, y| x > y, pos_fallback),
            Expressao::Menor(a, b) => self.comparar(a, b, pos, |x, y| x < y, pos_fallback),
            Expressao::MaiorIgual(a, b) => self.comparar(a, b, pos, |x, y| x >= y, pos_fallback),
            Expressao::MenorIgual(a, b) => self.comparar(a, b, pos, |x, y| x <= y, pos_fallback),
            Expressao::Igual(a, b) => {
                let va = self.avaliar(a, pos_fallback)?;
                let vb = self.avaliar(b, pos_fallback)?;
                if va.tipo() != vb.tipo() {
                    return Err(NeoErro::Tipo {
                        pos,
                        esperado: va.tipo().nome().into(),
                        recebido: vb.tipo().nome().into(),
                        contexto: "'==' exige valores do mesmo tipo.".into(),
                    });
                }
                Ok(NeoValor::Boolean(va == vb))
            }
            Expressao::Diferente(a, b) => {
                let va = self.avaliar(a, pos_fallback)?;
                let vb = self.avaliar(b, pos_fallback)?;
                if va.tipo() != vb.tipo() {
                    return Err(NeoErro::Tipo {
                        pos,
                        esperado: va.tipo().nome().into(),
                        recebido: vb.tipo().nome().into(),
                        contexto: "'!=' exige valores do mesmo tipo.".into(),
                    });
                }
                Ok(NeoValor::Boolean(va != vb))
            }
            Expressao::And(a, b) => {
                let esquerda = self.obter_booleano(a, pos_fallback)?;
                if !esquerda {
                    return Ok(NeoValor::Boolean(false));
                }
                let direita = self.obter_booleano(b, pos_fallback)?;
                Ok(NeoValor::Boolean(direita))
            }
            Expressao::Or(a, b) => {
                let esquerda = self.obter_booleano(a, pos_fallback)?;
                if esquerda {
                    return Ok(NeoValor::Boolean(true));
                }
                let direita = self.obter_booleano(b, pos_fallback)?;
                Ok(NeoValor::Boolean(direita))
            }
            Expressao::Not(a) => Ok(NeoValor::Boolean(!self.obter_booleano(a, pos_fallback)?)),
            Expressao::Chamada { nome, argumentos } => self.chamar_funcao(nome, argumentos, pos),
        }
    }

    fn operacao_decimal<F>(
        &mut self,
        a: &Expr,
        b: &Expr,
        pos: Posicao,
        operacao: F,
        esperado: &str,
        pos_fallback: Posicao,
    ) -> Result<NeoValor, NeoErro>
    where
        F: Fn(Decimal, Decimal) -> Option<Decimal>,
    {
        let va = self.avaliar(a, pos_fallback)?;
        let vb = self.avaliar(b, pos_fallback)?;
        match (va, vb) {
            (NeoValor::Decimal(a), NeoValor::Decimal(b)) => match operacao(a, b) {
                Some(resultado) => Ok(NeoValor::Decimal(resultado)),
                None => Err(NeoErro::Runtime {
                    pos,
                    mensagem: "Overflow aritmético em operação decimal.".into(),
                }),
            },
            (a, b) => Err(NeoErro::Tipo {
                pos,
                esperado: esperado.into(),
                recebido: format!("{} e {}", a.tipo().nome(), b.tipo().nome()),
                contexto: "A operação aritmética exige valores decimal.".into(),
            }),
        }
    }

    fn comparar<F>(
        &mut self,
        a: &Expr,
        b: &Expr,
        pos: Posicao,
        operacao: F,
        pos_fallback: Posicao,
    ) -> Result<NeoValor, NeoErro>
    where
        F: Fn(Decimal, Decimal) -> bool,
    {
        let va = self.avaliar(a, pos_fallback)?;
        let vb = self.avaliar(b, pos_fallback)?;
        match (va, vb) {
            (NeoValor::Decimal(a), NeoValor::Decimal(b)) => Ok(NeoValor::Boolean(operacao(a, b))),
            (a, b) => Err(NeoErro::Tipo {
                pos,
                esperado: "decimal e decimal".into(),
                recebido: format!("{} e {}", a.tipo().nome(), b.tipo().nome()),
                contexto: "Comparações ordenadas exigem decimal.".into(),
            }),
        }
    }

    fn obter_booleano(&mut self, expr: &Expr, pos_fallback: Posicao) -> Result<bool, NeoErro> {
        match self.avaliar(expr, pos_fallback)? {
            NeoValor::Boolean(v) => Ok(v),
            outro => Err(NeoErro::Tipo {
                pos: expr.pos,
                esperado: "boolean".into(),
                recebido: outro.tipo().nome().into(),
                contexto: "Era esperado true ou false.".into(),
            }),
        }
    }

    fn chamar_funcao(&mut self, nome: &str, argumentos: &[Expr], pos: Posicao) -> Result<NeoValor, NeoErro> {
        let funcao = match self.funcoes.get(nome) {
            Some(f) => f.clone(),
            None => {
                return Err(NeoErro::Runtime {
                    pos,
                    mensagem: format!("A função '{}' não existe.", nome),
                })
            }
        };

        if argumentos.len() != funcao.parametros.len() {
            return Err(NeoErro::Runtime {
                pos,
                mensagem: format!(
                    "A função '{}' espera {} argumento(s), mas recebeu {}.",
                    nome,
                    funcao.parametros.len(),
                    argumentos.len()
                ),
            });
        }

        if self.profundidade_funcoes >= MAX_PROFUNDIDADE_FUNCAO {
            return Err(NeoErro::Runtime {
                pos,
                mensagem: format!(
                    "profundidade máxima de chamadas de função excedida. A função '{}' atingiu o limite de {} chamadas aninhadas. Isso normalmente indica uma recursão sem condição de parada.",
                    nome, MAX_PROFUNDIDADE_FUNCAO
                ),
            });
        }

        // Os argumentos são avaliados no ambiente do chamador.
        let mut novo_escopo = HashMap::new();
        for (indice, (parametro, argumento)) in funcao.parametros.iter().zip(argumentos.iter()).enumerate() {
            let valor = self.avaliar(argumento, pos)?;
            if valor.tipo() != parametro.tipo {
                return Err(NeoErro::Tipo {
                    pos: argumento.pos,
                    esperado: parametro.tipo.nome().into(),
                    recebido: valor.tipo().nome().into(),
                    contexto: format!(
                        "O argumento {} de '{}' não corresponde ao parâmetro '{}'.",
                        indice + 1,
                        nome,
                        parametro.nome
                    ),
                });
            }
            novo_escopo.insert(parametro.nome.clone(), valor);
        }

        self.profundidade_funcoes += 1;
        self.locais.push(novo_escopo);

        let resultado = self.executar_corpo_funcao(&funcao.bloco, pos);

        let _ = self.locais.pop();
        self.profundidade_funcoes = self.profundidade_funcoes.saturating_sub(1);

        let valor = resultado?;
        if valor.tipo() != funcao.retorno {
            return Err(NeoErro::Runtime {
                pos,
                mensagem: format!(
                    "A função '{}' declarou retorno '{}' mas produziu '{}'.",
                    nome,
                    funcao.retorno.nome(),
                    valor.tipo().nome()
                ),
            });
        }
        Ok(valor)
    }

    fn executar_corpo_funcao(&mut self, bloco: &[Instrucao], pos: Posicao) -> Result<NeoValor, NeoErro> {
        for instrucao in bloco {
            if let Some(valor) = self.executar_instrucao(instrucao)? {
                return Ok(valor);
            }
        }
        // Funções void podem terminar sem return.
        Ok(NeoValor::Vazio)
    }

    fn aplicar_atribuicao(
        &self,
        antigo: NeoValor,
        novo: NeoValor,
        operador: &OperadorAtribuicao,
        pos: Posicao,
    ) -> Result<NeoValor, NeoErro> {
        match operador {
            OperadorAtribuicao::Igual => {
                if antigo.tipo() != novo.tipo() {
                    return Err(NeoErro::Tipo {
                        pos,
                        esperado: antigo.tipo().nome().into(),
                        recebido: novo.tipo().nome().into(),
                        contexto: "A atribuição simples não faz conversão implícita.".into(),
                    });
                }
                Ok(novo)
            }
            OperadorAtribuicao::Mais => match (antigo, novo) {
                (NeoValor::Decimal(a), NeoValor::Decimal(b)) => a
                    .checked_add(b)
                    .map(NeoValor::Decimal)
                    .ok_or_else(|| NeoErro::Runtime {
                        pos,
                        mensagem: "Overflow aritmético em '+='.".into(),
                    }),
                (NeoValor::String(a), NeoValor::String(b)) => Ok(NeoValor::String(format!("{}{}", a, b))),
                (a, b) => Err(NeoErro::Tipo {
                    pos,
                    esperado: "decimal + decimal ou string + string".into(),
                    recebido: format!("{} e {}", a.tipo().nome(), b.tipo().nome()),
                    contexto: "'+=' exige tipos compatíveis.".into(),
                }),
            },
            OperadorAtribuicao::Menos => self.operacao_decimal_valores(antigo, novo, pos, |a, b| a.checked_sub(b), "-="),
            OperadorAtribuicao::Multiplica => self.operacao_decimal_valores(antigo, novo, pos, |a, b| a.checked_mul(b), "*="),
            OperadorAtribuicao::Divide => match (antigo, novo) {
                (NeoValor::Decimal(a), NeoValor::Decimal(b)) => {
                    if b.is_zero() {
                        Err(NeoErro::Runtime {
                            pos,
                            mensagem: "Divisão por zero em '/=' não é permitida.".into(),
                        })
                    } else {
                        a.checked_div(b)
                            .map(NeoValor::Decimal)
                            .ok_or_else(|| NeoErro::Runtime {
                                pos,
                                mensagem: "Overflow aritmético em '/='.".into(),
                            })
                    }
                }
                (a, b) => Err(NeoErro::Tipo {
                    pos,
                    esperado: "decimal /= decimal".into(),
                    recebido: format!("{} e {}", a.tipo().nome(), b.tipo().nome()),
                    contexto: "'/=' exige decimal.".into(),
                }),
            },
            OperadorAtribuicao::Modulo => match (antigo, novo) {
                (NeoValor::Decimal(a), NeoValor::Decimal(b)) => {
                    if b.is_zero() {
                        Err(NeoErro::Runtime {
                            pos,
                            mensagem: "Módulo por zero em '%=' não é permitido.".into(),
                        })
                    } else {
                        a.checked_rem(b)
                            .map(NeoValor::Decimal)
                            .ok_or_else(|| NeoErro::Runtime {
                                pos,
                                mensagem: "Overflow aritmético em '%='.".into(),
                            })
                    }
                }
                (a, b) => Err(NeoErro::Tipo {
                    pos,
                    esperado: "decimal %= decimal".into(),
                    recebido: format!("{} e {}", a.tipo().nome(), b.tipo().nome()),
                    contexto: "'%=' exige decimal.".into(),
                }),
            },
        }
    }

    fn operacao_decimal_valores<F>(
        &self,
        a: NeoValor,
        b: NeoValor,
        pos: Posicao,
        operacao: F,
        nome: &str,
    ) -> Result<NeoValor, NeoErro>
    where
        F: Fn(Decimal, Decimal) -> Option<Decimal>,
    {
        match (a, b) {
            (NeoValor::Decimal(a), NeoValor::Decimal(b)) => match operacao(a, b) {
                Some(resultado) => Ok(NeoValor::Decimal(resultado)),
                None => Err(NeoErro::Runtime {
                    pos,
                    mensagem: format!("Overflow aritmético em '{}'.", nome),
                }),
            },
            (a, b) => Err(NeoErro::Tipo {
                pos,
                esperado: format!("decimal {} decimal", nome),
                recebido: format!("{} e {}", a.tipo().nome(), b.tipo().nome()),
                contexto: "A operação composta exige decimal.".into(),
            }),
        }
    }

    fn buscar_valor(&self, nome: &str) -> Option<NeoValor> {
        for escopo in self.locais.iter().rev() {
            if let Some(valor) = escopo.get(nome) {
                return Some(valor.clone());
            }
        }
        self.globais.get(nome).cloned()
    }

    fn encontrar_escopo_local(&self, nome: &str) -> Option<usize> {
        for indice in (0..self.locais.len()).rev() {
            if self.locais.get(indice).map(|e| e.contains_key(nome)).unwrap_or(false) {
                return Some(indice);
            }
        }
        None
    }

    fn interpolar(
        &mut self,
        texto: &str,
        pos: Posicao,
    ) -> Result<String, NeoErro> {
        let caracteres: Vec<char> = texto.chars().collect();

        let mut resultado = String::new();
        let mut indice = 0;

        while indice < caracteres.len() {
            if caracteres[indice] != '{' {
                resultado.push(caracteres[indice]);
                indice += 1;
                continue;
            }

            indice += 1;

            let mut expressao = String::new();
            let mut profundidade = 0;

            while indice < caracteres.len() {
                let caractere = caracteres[indice];

                if caractere == '(' {
                    profundidade += 1;
                } else if caractere == ')' {
                    if profundidade > 0 {
                        profundidade -= 1;
                    }
                } else if caractere == '}' && profundidade == 0 {
                    break;
                }

                expressao.push(caractere);
                indice += 1;
            }

            if indice >= caracteres.len() {
                resultado.push('{');
                resultado.push_str(&expressao);
                break;
            }

            indice += 1;

            let expressao = expressao.trim();

            if expressao.is_empty() {
                resultado.push_str("{}");
                continue;
            }

            let tokens = lex(expressao)?;

            let mut parser = Parser::novo(tokens);
            let arvore = parser.expressao()?;

            if parser.token_atual_seguro().kind != TokenKind::Fim {
                return Err(NeoErro::Sintaxe {
                    pos,
                    mensagem: format!(
                        "Expressão inválida dentro da interpolação: '{}'.",
                        expressao
                    ),
                });
            }

            let valor = self.avaliar(&arvore, pos)?;
            resultado.push_str(&valor.texto());
        }

        Ok(resultado)
    }
}

// ================================================================
// PIPELINE / MAIN
// ================================================================

fn compilar_e_executar(codigo: &str) -> Result<(), NeoErro> {
    let tokens = lex(codigo)?;
    let mut parser = Parser::novo(tokens);
    let programa = parser.analisar()?;

    let mut semantico = AnalisadorSemantico::novo();
    semantico.analisar(&programa)?;

    let mut runtime = Runtime::novo();
    runtime.executar(&programa)?;
    Ok(())
}

fn executar_arquivo(caminho: &str) -> Result<(), NeoErro> {
    let codigo = fs::read_to_string(caminho).map_err(|erro| NeoErro::Runtime {
        pos: Posicao::nova(1, 1),
        mensagem: format!("Não foi possível abrir '{}': {}", caminho, erro),
    })?;

    println!("NeoCOBOL 0.5.1");
    println!("Compilando: {}", caminho);

    let tokens = lex(&codigo)?;
    println!("Lexer: {} tokens.", tokens.len());

    let mut parser = Parser::novo(tokens);
    let programa = parser.analisar()?;
    println!("Parser: AST construída.");

    let mut semantico = AnalisadorSemantico::novo();
    semantico.analisar(&programa)?;
    println!("Semântica: programa válido.");

    let mut runtime = Runtime::novo();
    runtime.executar(&programa)?;
    println!("\nPrograma executado com sucesso.");
    Ok(())
}

fn main() {
    let argumentos: Vec<String> = env::args().collect();

    if argumentos.len() < 2 {
        println!("NeoCOBOL 0.5.1");
        println!("Uso:");
        println!("  cargo run -- programa.neo");
        return;
    }

    if let Err(erro) = executar_arquivo(&argumentos[1]) {
        eprintln!("\n{}", erro);
        std::process::exit(1);
    }
}

// ================================================================
// TESTES
// ================================================================

#[cfg(test)]
mod testes {
    use super::*;

    fn analisar(codigo: &str) -> Result<(), NeoErro> {
        let tokens = lex(codigo)?;
        let mut parser = Parser::novo(tokens);
        let programa = parser.analisar()?;
        let mut sem = AnalisadorSemantico::novo();
        sem.analisar(&programa)
    }

    fn executar(codigo: &str) -> Result<(), NeoErro> {
        let tokens = lex(codigo)?;
        let mut parser = Parser::novo(tokens);
        let programa = parser.analisar()?;
        let mut sem = AnalisadorSemantico::novo();
        sem.analisar(&programa)?;
        let mut runtime = Runtime::novo();
        runtime.executar(&programa)
    }

    #[test]
    fn booleanos_sao_tokens_separados() {
        let tokens = lex("boolean ativo = true\ndisplay false").expect("lexer deve funcionar");
        assert_eq!(tokens[0].kind, TokenKind::BooleanTipo);
        assert_eq!(tokens[3].kind, TokenKind::Verdadeiro);
        assert_eq!(tokens[5].kind, TokenKind::Display);
        assert_eq!(tokens[6].kind, TokenKind::Falso);
    }

    #[test]
    fn funcao_tipada_valida() {
        let codigo = r#"
func soma(a: decimal, b: decimal) -> decimal
    return a + b
end

decimal resultado = soma(10, 20)
display resultado
"#;
        assert!(analisar(codigo).is_ok());
    }

    #[test]
    fn parametro_com_tipo_errado_e_rejeitado() {
        let codigo = r#"
func soma(a: decimal, b: decimal) -> decimal
    return a + b
end

decimal resultado = soma(10, "20")
"#;
        let erro = analisar(codigo).expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Tipo);
        assert!(erro.to_string().contains("argumento 2"));
        assert!(erro.to_string().contains("decimal"));
        assert!(erro.to_string().contains("string"));
    }

    #[test]
    fn corpo_da_funcao_e_analisado() {
        let codigo = r#"
func teste(x: decimal) -> decimal
    string texto = x
    return x
end
"#;
        assert!(analisar(codigo).is_err());
    }

    #[test]
    fn retorno_errado_e_rejeitado() {
        let codigo = r#"
func teste() -> decimal
    return "abc"
end
"#;
        let erro = analisar(codigo).expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Tipo);
        assert!(erro.to_string().contains("retornado"));
    }

    #[test]
    fn retorno_em_funcao_vazio_com_valor_e_rejeitado() {
        let codigo = r#"
func teste() -> vazio
    return 10
end
"#;
        assert!(analisar(codigo).is_err());
    }

    #[test]
    fn retorno_fora_de_funcao_e_rejeitado() {
        let erro = analisar("return 10").expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Semantico);
        assert!(erro.to_string().contains("só pode ser usado dentro de uma função"));
    }

    #[test]
    fn funcao_void_sem_return_e_valida() {
        let codigo = r#"
func mostrar(nome: string) -> vazio
    display nome
end
mostrar("Neo")
"#;
        assert!(analisar(codigo).is_ok());
    }

    #[test]
    fn retorno_vazio_sem_valor_e_valido() {
        let codigo = r#"
func mostrar() -> vazio
    return
end
"#;
        assert!(analisar(codigo).is_ok());
    }

    #[test]
    fn funcao_decimal_precisa_de_retorno() {
        let codigo = r#"
func teste() -> decimal
    display "sem retorno"
end
"#;
        assert!(analisar(codigo).is_err());
    }

    #[test]
    fn funcao_nao_enxerga_local_do_chamador() {
        let codigo = r#"
func teste() -> decimal
    return x
end

if true
    decimal x = 10
    display teste()
end
"#;
        let erro = analisar(codigo).expect_err("deve falhar por escopo lexical");
        assert_eq!(erro.codigo(), CodigoErro::Semantico);
        assert!(erro.to_string().contains("x"));
    }

    #[test]
    fn funcao_pode_acessar_global() {
        let codigo = r#"
decimal x = 10
func teste() -> decimal
    return x
end

decimal y = teste()
"#;
        assert!(analisar(codigo).is_ok());
    }

    #[test]
    fn funcoes_duplicadas_sao_rejeitadas() {
        let codigo = r#"
func a() -> vazio
end
func a() -> vazio
end
"#;
        assert!(analisar(codigo).is_err());
    }

    #[test]
    fn parametros_duplicados_sao_rejeitados() {
        let codigo = r#"
func a(x: decimal, x: string) -> vazio
end
"#;
        assert!(analisar(codigo).is_err());
    }

    #[test]
    fn quantidade_de_argumentos_e_verificada() {
        let codigo = r#"
func a(x: decimal) -> decimal
    return x
end

decimal y = a()
"#;
        assert!(analisar(codigo).is_err());
    }

    #[test]
    fn divisao_por_zero_e_detectada_no_runtime() {
        let codigo = "decimal x = 10 / 0";
        let erro = executar(codigo).expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Runtime);
        assert!(erro.to_string().contains("Divisão por zero"));
    }

    #[test]
    fn modulo_por_zero_e_detectado_no_runtime() {
        let codigo = "decimal x = 10 % 0";
        let erro = executar(codigo).expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Runtime);
        assert!(erro.to_string().contains("Módulo por zero"));
    }

    #[test]
    fn curto_circuito_and_e_preservado() {
        let codigo = r#"
boolean x = false and (10 / 0 > 1)
"#;
        assert!(executar(codigo).is_ok());
    }

    #[test]
    fn curto_circuito_or_e_preservado() {
        let codigo = r#"
boolean x = true or (10 / 0 > 1)
"#;
        assert!(executar(codigo).is_ok());
    }

    #[test]
    fn string_e_decimal_nao_podem_ser_somados() {
        assert!(analisar("decimal x = 10 + \"abc\"").is_err());
    }

    #[test]
    fn atribuicao_com_tipo_errado_e_rejeitada() {
        assert!(analisar("decimal x = 10\nx = \"abc\"").is_err());
    }

    #[test]
    fn variavel_inexistente_e_rejeitada() {
        assert!(analisar("display desconhecida").is_err());
    }

    #[test]
    fn funcao_inexistente_e_rejeitada() {
        assert!(analisar("decimal x = inexistente(1)").is_err());
    }

    #[test]
    fn string_nao_terminada_e_rejeitada() {
        let erro = lex("display \"abc").expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Lexico);
        assert_eq!(erro.posicao().linha, 1);
        assert_eq!(erro.posicao().coluna, 8);
    }

    #[test]
    fn numero_com_dois_pontos_decimais_e_rejeitado() {
        let erro = lex("decimal x = 1.2.3").expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Lexico);
    }

    #[test]
    fn caractere_desconhecido_e_rejeitado() {
        let erro = lex("decimal x = 10 @ 2").expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Lexico);
        assert!(erro.to_string().contains("@"));
    }

    #[test]
    fn sintaxe_truncada_nao_pode_panicar() {
        let resultado = std::panic::catch_unwind(|| {
            let tokens = lex("if true").expect("lexer");
            let mut parser = Parser::novo(tokens);
            parser.analisar()
        });
        assert!(resultado.is_ok());
        assert!(resultado.unwrap().is_err());
    }

    #[test]
    fn parser_com_vetor_de_tokens_vazio_nao_pode_panicar() {
        let resultado = std::panic::catch_unwind(|| {
            let mut parser = Parser::novo(Vec::new());
            parser.analisar()
        });
        assert!(resultado.is_ok());
        assert!(resultado.unwrap().is_err());
    }

    #[test]
    fn return_no_nivel_global_nao_pode_chegar_ao_runtime() {
        let resultado = executar("return 10");
        assert!(resultado.is_err());
    }

    #[test]
    fn recursao_e_limitada() {
        let codigo = r#"
func loop(x: decimal) -> decimal
    return loop(x + 1)
end

decimal resultado = loop(0)
"#;
        let erro = executar(codigo).expect_err("recursão deve atingir limite");
        assert_eq!(erro.codigo(), CodigoErro::Runtime);
        assert!(erro.to_string().contains("profundidade máxima"));
        assert!(erro.to_string().contains("256"));
    }

    #[test]
    fn operadores_compostos_funcionam() {
        let codigo = r#"
decimal x = 10
x += 5
x -= 2
x *= 2
x /= 2
x %= 5
"#;
        assert!(executar(codigo).is_ok());
    }

    #[test]
    fn retorno_em_if_com_else_garante_retorno() {
        let codigo = r#"
func sinal(x: decimal) -> string
    if x > 0
        return "positivo"
    else
        return "nao positivo"
    end
end
"#;
        assert!(analisar(codigo).is_ok());
    }

    #[test]
    fn retorno_em_if_sem_else_nao_garante_retorno() {
        let codigo = r#"
func sinal(x: decimal) -> string
    if x > 0
        return "positivo"
    end
end
"#;
        assert!(analisar(codigo).is_err());
    }

    #[test]
    fn coluna_de_erro_semantico_e_preservada() {
        let erro = analisar("decimal x = 10\nx = \"texto\"").expect_err("deve falhar");
        assert_eq!(erro.posicao().linha, 2);
        assert!(erro.posicao().coluna >= 1);
    }

    #[test]
    fn escopos_de_blocos_nao_vazam() {
        let codigo = r#"
if true
    decimal interno = 10
end
display interno
"#;
        assert!(analisar(codigo).is_err());
    }

}
