p = "src/main.rs"
s = open(p, encoding="utf-8").read()
T = []
def t(old, new): T.append((old, new))

t("enum Expressao {",
r"""enum ParteTexto {
    Literal(String),
    Valor(Expr),
}

#[derive(Debug, Clone)]
enum Expressao {
    Interpolada(Vec<ParteTexto>),""")

t("kind: Expressao::String(token.texto),",
  "kind: dividir_interpolacao(&token.texto, token.pos)?,")

t("fn nome_token(kind: &TokenKind) -> &'static str {",
r"""fn erro_interpolacao(trecho: &str, pos: Posicao, erro: &NeoErro) -> NeoErro {
    let detalhe = match erro {
        NeoErro::Lexico { mensagem, .. }
        | NeoErro::Sintaxe { mensagem, .. }
        | NeoErro::Semantico { mensagem, .. }
        | NeoErro::Runtime { mensagem, .. } => mensagem.clone(),
        NeoErro::Tipo { contexto, .. } => contexto.clone(),
    };
    NeoErro::Sintaxe {
        pos,
        mensagem: format!("Interpolação inválida em '{{{}}}': {}", trecho, detalhe),
    }
}

fn dividir_interpolacao(texto: &str, pos: Posicao) -> Result<Expressao, NeoErro> {
    let c: Vec<char> = texto.chars().collect();
    let mut partes: Vec<ParteTexto> = Vec::new();
    let mut literal = String::new();
    let mut i = 0usize;
    let mut tem_valor = false;

    while i < c.len() {
        if c[i] != '{' {
            literal.push(c[i]);
            i += 1;
            continue;
        }

        let inicio = i + 1;
        let mut j = inicio;
        let mut profundidade = 0usize;
        let mut achou = false;
        while j < c.len() {
            if c[j] == '(' {
                profundidade += 1;
            } else if c[j] == ')' {
                profundidade = profundidade.saturating_sub(1);
            } else if c[j] == '}' && profundidade == 0 {
                achou = true;
                break;
            }
            j += 1;
        }

        if !achou {
            literal.push('{');
            i += 1;
            continue;
        }

        let trecho: String = c[inicio..j].iter().collect();
        let corte = trecho.trim();
        if corte.is_empty() {
            literal.push_str("{}");
            i = j + 1;
            continue;
        }

        let esquerda = trecho.chars().take_while(|ch| ch.is_whitespace()).count();
        let deslocamento = inicio + esquerda;
        let mut tokens = lex(corte).map_err(|e| erro_interpolacao(corte, pos, &e))?;
        for token in tokens.iter_mut() {
            token.pos = Posicao::nova(pos.linha, pos.coluna + deslocamento + token.pos.coluna);
        }
        let mut parser = Parser::novo(tokens);
        let valor = parser.expressao()?;
        if parser.token_atual_seguro().kind != TokenKind::Fim {
            return Err(NeoErro::Sintaxe {
                pos,
                mensagem: format!("Expressão inválida dentro da interpolação: '{}'.", corte),
            });
        }

        if !literal.is_empty() {
            partes.push(ParteTexto::Literal(std::mem::take(&mut literal)));
        }
        partes.push(ParteTexto::Valor(valor));
        tem_valor = true;
        i = j + 1;
    }

    if !tem_valor {
        return Ok(Expressao::String(literal));
    }
    if !literal.is_empty() {
        partes.push(ParteTexto::Literal(literal));
    }
    Ok(Expressao::Interpolada(partes))
}

fn nome_token(kind: &TokenKind) -> &'static str {""")

t("Expressao::Decimal(_) => Ok(Tipo::Decimal),",
r"""Expressao::Interpolada(partes) => {
                for parte in partes {
                    if let ParteTexto::Valor(e) = parte {
                        let tipo = self.expressao(e)?;
                        if tipo == Tipo::Vazio {
                            return Err(NeoErro::Tipo {
                                pos: e.pos,
                                esperado: "decimal, string ou boolean".into(),
                                recebido: "vazio".into(),
                                contexto: "Uma função 'vazio' não produz valor para a interpolação.".into(),
                            });
                        }
                    }
                }
                Ok(Tipo::String)
            }
            Expressao::Decimal(_) => Ok(Tipo::Decimal),""")

t("Expressao::Decimal(v) => Ok(NeoValor::Decimal(*v)),",
r"""Expressao::Interpolada(partes) => {
                let mut resultado = String::new();
                for parte in partes {
                    match parte {
                        ParteTexto::Literal(texto) => resultado.push_str(texto),
                        ParteTexto::Valor(e) => {
                            let valor = self.avaliar(e, pos_fallback)?;
                            resultado.push_str(&valor.texto());
                        }
                    }
                }
                Ok(NeoValor::String(resultado))
            }
            Expressao::Decimal(v) => Ok(NeoValor::Decimal(*v)),""")

t("let texto = self.interpolar(&resultado.texto(), *pos)?;",
  "let texto = resultado.texto();")

t("fn escopos_de_blocos_nao_vazam() {",
r"""fn interpolacao_calcula_expressoes() {
        let g = globais_apos(
            r#"
decimal x = 2
string s = "valor {x + 1}!"
"#,
        );
        assert_eq!(g.get("s"), Some(&NeoValor::String("valor 3!".to_string())));
    }

    #[test]
    fn interpolacao_com_variavel_inexistente_e_rejeitada() {
        let erro = analisar(
            r#"
string s = "{inexistente}"
"#,
        )
        .expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Semantico);
    }

    #[test]
    fn erro_dentro_da_interpolacao_aponta_a_linha_certa() {
        let erro = analisar(
            r#"
decimal a = 1
string s = "x {a +}"
"#,
        )
        .expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Sintaxe);
        assert_eq!(erro.posicao().linha, 3);
    }

    fn globais_apos(codigo: &str) -> std::collections::HashMap<String, NeoValor> {
        let tokens = lex(codigo).expect("lexer");
        let mut parser = Parser::novo(tokens);
        let programa = parser.analisar().expect("parser");
        let mut sem = AnalisadorSemantico::novo();
        sem.analisar(&programa).expect("semantica");
        let mut runtime = Runtime::novo();
        runtime.executar(&programa).expect("runtime");
        runtime.globais
    }

    #[test]
    fn escopos_de_blocos_nao_vazam() {""")

falhas = [o for o, n in T if s.count(o) != 1]
if s.count("    fn interpolar(") != 1:
    falhas.append("fn interpolar")
if falhas:
    print("NADA FOI ALTERADO. Nao achei (ou achei varios):")
    for f in falhas:
        print(" -", f[:60].replace("\n", " "))
else:
    for o, n in T:
        s = s.replace(o, n)
    a = s.find("    fn interpolar(")
    b = s.find("\n}\n", a)
    s = s[:a].rstrip() + s[b:]
    open(p, "w", encoding="utf-8").write(s)
    print("ok:", len(T), "trocas e funcao interpolar antiga removida")
