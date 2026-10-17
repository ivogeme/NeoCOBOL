// Testes de programas .neo completos.
// Cada testes/NOME.neo tem a saída esperada em NOME.saida e os erros em NOME.erro.
// Programas que começam com "erro_" devem terminar com código de saída 1.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn executavel() -> &'static str {
    option_env!("CARGO_BIN_EXE_neocobol")
        .or(option_env!("CARGO_BIN_EXE_NeoCOBOL"))
        .expect("executável não encontrado")
}

#[test]
fn programas_da_pasta_testes_produzem_a_saida_esperada() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut arquivos: Vec<PathBuf> = fs::read_dir(raiz.join("testes"))
        .expect("pasta testes não encontrada")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "neo").unwrap_or(false))
        .collect();
    arquivos.sort();
    assert!(!arquivos.is_empty(), "nenhum programa .neo em testes/");

    for arquivo in arquivos {
        let nome = arquivo.file_stem().unwrap().to_string_lossy().to_string();
        let relativo = format!("testes/{}.neo", nome);
        let resultado = Command::new(executavel())
            .arg(&relativo)
            .current_dir(&raiz)
            .output()
            .expect("não foi possível executar o programa");
        let esperado_saida =
            fs::read_to_string(raiz.join(format!("testes/{}.saida", nome))).unwrap_or_default();
        let esperado_erro =
            fs::read_to_string(raiz.join(format!("testes/{}.erro", nome))).unwrap_or_default();
        assert_eq!(
            String::from_utf8_lossy(&resultado.stdout).to_string(),
            esperado_saida,
            "saída diferente em {}",
            relativo
        );
        assert_eq!(
            String::from_utf8_lossy(&resultado.stderr).to_string(),
            esperado_erro,
            "erros diferentes em {}",
            relativo
        );
        let codigo = if nome.starts_with("erro_") { 1 } else { 0 };
        assert_eq!(
            resultado.status.code(),
            Some(codigo),
            "código de saída em {}",
            relativo
        );
    }
}
