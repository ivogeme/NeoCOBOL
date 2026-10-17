// Testes do comportamento da linha de comando, com o executável de verdade.

use std::path::PathBuf;
use std::process::{Command, Output};

fn executavel() -> &'static str {
    option_env!("CARGO_BIN_EXE_neocobol")
        .or(option_env!("CARGO_BIN_EXE_NeoCOBOL"))
        .expect("executável não encontrado")
}

fn rodar(argumentos: &[&str]) -> Output {
    Command::new(executavel())
        .args(argumentos)
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .output()
        .expect("não foi possível executar o programa")
}

fn saida(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn erros(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

#[test]
fn check_verifica_sem_executar() {
    let o = rodar(&["check", "testes/basico.neo"]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(saida(&o), "testes/basico.neo: nenhum erro encontrado.\n");
    assert_eq!(erros(&o), "");
}

#[test]
fn check_mostra_erro_e_sai_com_1() {
    let o = rodar(&["check", "testes/erro_tipo.neo"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(saida(&o), "");
    assert!(erros(&o).contains("erro[NEO2001]"));
}

#[test]
fn execucao_normal_mostra_so_o_resultado() {
    let o = rodar(&["testes/basico.neo"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(saida(&o).starts_with("59.70\n"));
    assert!(!saida(&o).contains("Lexer"));
    assert_eq!(erros(&o), "");
}

#[test]
fn detalhes_mostra_as_etapas() {
    let o = rodar(&["--detalhes", "testes/basico.neo"]);
    assert_eq!(o.status.code(), Some(0));
    let s = saida(&o);
    assert!(s.contains("NeoCOBOL 0.6.0"));
    assert!(s.contains("Compilando: testes/basico.neo"));
    assert!(s.contains("Lexer: "));
    assert!(s.contains("Parser: AST construída."));
    assert!(s.contains("Semântica: programa válido."));
    assert!(s.contains("59.70"));
    assert!(s.contains("Programa executado com sucesso."));
}

#[test]
fn detalhes_e_check_em_qualquer_ordem() {
    let combinacoes: [[&str; 3]; 3] = [
        ["--detalhes", "check", "testes/basico.neo"],
        ["check", "--detalhes", "testes/basico.neo"],
        ["check", "testes/basico.neo", "--detalhes"],
    ];
    for args in combinacoes.iter() {
        let o = rodar(args);
        assert_eq!(o.status.code(), Some(0), "args: {:?}", args);
        let s = saida(&o);
        assert!(s.contains("Lexer: "), "args: {:?}", args);
        assert!(s.contains("nenhum erro encontrado"), "args: {:?}", args);
        assert!(!s.contains("59.70"), "args: {:?}", args);
    }
}

#[test]
fn detalhes_com_erro_nao_mostra_sucesso() {
    let o = rodar(&["--detalhes", "testes/erro_tipo.neo"]);
    assert_eq!(o.status.code(), Some(1));
    let s = saida(&o);
    assert!(s.contains("Parser: AST construída."));
    assert!(!s.contains("Semântica: programa válido."));
    assert!(!s.contains("Programa executado com sucesso."));
    assert!(erros(&o).contains("erro[NEO2001]"));
}

#[test]
fn arquivo_inexistente_gera_neo3009() {
    let o = rodar(&["nao_existe.neo"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(saida(&o), "");
    assert!(erros(&o).contains("NEO3009"));
}

#[test]
fn opcao_desconhecida_sai_com_2() {
    let o = rodar(&["--xyz"]);
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(saida(&o), "");
    assert!(erros(&o).contains("Opção desconhecida: --xyz"));
}

#[test]
fn argumento_extra_sai_com_2() {
    let o = rodar(&["a.neo", "b.neo"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(erros(&o).contains("Argumento inesperado"));
}

#[test]
fn check_sem_arquivo_sai_com_2() {
    let o = rodar(&["check"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(erros(&o).contains("check precisa de um arquivo"));
}

#[test]
fn versao_e_ajuda() {
    for arg in ["--versao", "-v"].iter() {
        let o = rodar(&[arg]);
        assert_eq!(o.status.code(), Some(0));
        assert_eq!(saida(&o), "NeoCOBOL 0.6.0\n");
    }
    let o = rodar(&["--ajuda"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(saida(&o).contains("Uso:"));
    assert!(saida(&o).contains("check"));
    let o = rodar(&[]);
    assert_eq!(o.status.code(), Some(0));
    assert!(saida(&o).contains("Uso:"));
}

#[test]
fn erros_vao_para_stderr() {
    let o = rodar(&["testes/erro_divisao_por_zero.neo"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(saida(&o), "");
    assert!(erros(&o).contains("erro[NEO3001]"));
}
