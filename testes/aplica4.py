p = "src/main.rs"
s = open(p, encoding="utf-8").read()
T = []
def t(old, new): T.append((old, new))

t("retorno_atual: Option<Tipo>,",
  "retorno_atual: Option<Tipo>,\n    globais_declaradas: std::collections::HashSet<String>,")
t("retorno_atual: None,",
  "retorno_atual: None,\n            globais_declaradas: std::collections::HashSet::new(),")
t("registrado no primeiro passe.",
"""registrado no primeiro passe.
                if self.funcao_atual.is_none() && self.escopos_locais.is_empty() {
                    self.globais_declaradas.insert(nome.clone());
                }""")
t("self.globais.get(nome).map(|s| s.tipo.clone())",
"""if self.funcao_atual.is_none() && !self.globais_declaradas.contains(nome) {
            return None;
        }
        self.globais.get(nome).map(|s| s.tipo.clone())""")

t("""if ch == '"' {""",
r"""if ch == '\\' && i + 1 < chars.len() {
                        let traduzido = match chars[i + 1] {
                            '"' => Some('"'),
                            '\\' => Some('\\'),
                            'n' => Some('\n'),
                            't' => Some('\t'),
                            _ => None,
                        };
                        if let Some(c) = traduzido {
                            texto.push(c);
                            i += 2;
                            continue;
                        }
                    }
                    if ch == '"' {""")

t("fn escopos_de_blocos_nao_vazam() {",
r"""fn global_usada_antes_de_declarar_e_rejeitada() {
        let erro = analisar("display total\ndecimal total = 10").expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Semantico);
    }

    #[test]
    fn aspas_escapadas_em_string() {
        let tokens = lex(r#"display "a\"b""#).expect("lexer");
        assert_eq!(tokens[1].texto, "a\"b");
    }

    #[test]
    fn escopos_de_blocos_nao_vazam() {""")

falhas = [o for o, n in T if s.count(o) != 1]
if falhas:
    print("NADA FOI ALTERADO. Nao achei (ou achei varios):")
    for f in falhas:
        print(" -", f[:60].replace("\n", " "))
else:
    for o, n in T:
        s = s.replace(o, n)
    open(p, "w", encoding="utf-8").write(s)
    print("ok:", len(T), "trocas aplicadas")
