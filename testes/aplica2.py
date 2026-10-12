p = "src/main.rs"
s = open(p, encoding="utf-8").read()
old = "fn recursao_e_limitada() {"
new = """fn recursao_e_limitada() {
        let t = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(recursao_e_limitada_corpo)
            .expect("nao foi possivel criar a thread");
        t.join().expect("o teste da recursao falhou");
    }

    fn recursao_e_limitada_corpo() {"""
if s.count(old) != 1:
    print("NADA FOI ALTERADO: trecho nao encontrado")
else:
    open(p, "w", encoding="utf-8").write(s.replace(old, new))
    print("ok")
