p = "src/main.rs"
s = open(p, encoding="utf-8").read()
T = []
def t(old, new): T.append((old, new))

t("não existe neste escopo. Funções não podem acessar variáveis locais de quem as chamou.",
  "não foi declarada. Declare-a antes de usá-la. Dentro de funções, só estão disponíveis os parâmetros, as variáveis da própria função e as globais.")

t("fn escopos_de_blocos_nao_vazam() {",
r"""fn variavel_nao_declarada_tem_mensagem_clara() {
        let erro = analisar("display total").expect_err("deve falhar");
        assert_eq!(erro.codigo(), CodigoErro::Semantico);
        assert!(erro.to_string().contains("não foi declarada"));
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
    if "NeoCOBOL 0.5.1.1" not in s:
        s = s.replace("NeoCOBOL 0.5.1", "NeoCOBOL 0.5.1.1")
    open(p, "w", encoding="utf-8").write(s)
    print("ok: mensagem, teste e versao atualizados")
