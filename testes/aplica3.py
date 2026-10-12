p = "src/main.rs"
s = open(p, encoding="utf-8").read()
T = []
def t(old, new): T.append((old, new))

t("self.locais.push(novo_escopo);",
  "let locais_do_chamador = std::mem::take(&mut self.locais);\n        self.locais.push(novo_escopo);")
t("self.profundidade_funcoes = self.profundidade_funcoes.saturating_sub(1);",
  "self.profundidade_funcoes = self.profundidade_funcoes.saturating_sub(1);\n        self.locais = locais_do_chamador;")
t("fn escopos_de_blocos_nao_vazam() {",
"""fn funcao_nao_enxerga_local_do_chamador_no_runtime() {
        let codigo = r#"
decimal x = 10
func f() -> decimal
    return x
end
decimal r = 0
if true
    decimal x = 99
    r = f()
end
"#;
        let tokens = lex(codigo).expect("lexer");
        let mut parser = Parser::novo(tokens);
        let programa = parser.analisar().expect("parser");
        let mut sem = AnalisadorSemantico::novo();
        sem.analisar(&programa).expect("semantica");
        let mut runtime = Runtime::novo();
        runtime.executar(&programa).expect("runtime");
        assert_eq!(
            runtime.globais.get("r"),
            Some(&NeoValor::Decimal(Decimal::from(10)))
        );
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
