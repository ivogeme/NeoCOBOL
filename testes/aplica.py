import sys
p = "src/main.rs"
s = open(p, encoding="utf-8").read()
T = []
def t(old, new): T.append((old, new))

t("assert_eq!(tokens[5].kind, TokenKind::Display);",
  "assert_eq!(tokens[4].kind, TokenKind::Display);")
t("assert_eq!(tokens[6].kind, TokenKind::Falso);",
  "assert_eq!(tokens[5].kind, TokenKind::Falso);")
t("assert_eq!(erro.posicao().coluna, 8);",
  "assert_eq!(erro.posicao().coluna, 9);")

t("let mut esquerda = self.comparacao()?;",
  "let mut esquerda = self.nao()?;")
t("let direita = self.comparacao()?;",
  "let direita = self.nao()?;")
t("fn and(&mut self) -> Result<Expr, NeoErro> {",
"""fn nao(&mut self) -> Result<Expr, NeoErro> {
        let token = self.token_atual_seguro();
        if token.kind == TokenKind::Not {
            self.avancar();
            let valor = self.nao()?;
            return Ok(Expr {
                kind: Expressao::Not(Box::new(valor)),
                pos: token.pos,
            });
        }
        self.comparacao()
    }

    fn and(&mut self) -> Result<Expr, NeoErro> {""")

t("Return {\n        valor: Option<Expr>,",
  "Expressao {\n        valor: Expr,\n        pos: Posicao,\n    },\n    Return {\n        valor: Option<Expr>,")
t("| Instrucao::Return { pos, .. } => *pos,",
  "| Instrucao::Return { pos, .. }\n            | Instrucao::Expressao { pos, .. } => *pos,")
t("TokenKind::Identificador => self.atribuicao(),",
"""TokenKind::Identificador => {
                let proximo = self.tokens.get(self.atual + 1).map(|t| t.kind.clone());
                if proximo == Some(TokenKind::AbrePar) {
                    let valor = self.primario()?;
                    Ok(Instrucao::Expressao { valor, pos: token.pos })
                } else {
                    self.atribuicao()
                }
            }""")
t("Instrucao::Display { valor, .. } => {",
"""Instrucao::Expressao { valor, .. } => {
                self.expressao(valor)?;
            }
            Instrucao::Display { valor, .. } => {""")
t("Instrucao::Funcao { .. } => {}",
"""Instrucao::Funcao { .. } => {}
            Instrucao::Expressao { valor, pos } => {
                self.avaliar(valor, *pos)?;
            }""")

falhas = [o for o, n in T if s.count(o) != 1]
if falhas:
    print("NADA FOI ALTERADO. Nao achei (ou achei varios):")
    for f in falhas:
        print(" -", f[:60].replace("\n", " "))
    sys.exit(1)
for o, n in T:
    s = s.replace(o, n)
open(p, "w", encoding="utf-8").write(s)
print("ok:", len(T), "trocas aplicadas")
