// ================================================================
// Linha de comando da NeoCOBOL.
// ================================================================

use std::fs;

use crate::diagnostico::{self, NeoErro, Posicao};
use crate::{AnalisadorSemantico, Parser, Runtime, lex};

pub const VERSAO: &str = "0.6.0";

// O interpretador roda numa thread com pilha grande, para que programas
// profundos (recursão, expressões aninhadas) não estourem a pilha comum.
const PILHA_DO_INTERPRETADOR: usize = 256 * 1024 * 1024;

pub struct Opcoes {
    pub arquivo: Option<String>,
    pub executar: bool,
    pub detalhes: bool,
    pub ajuda: bool,
    pub versao: bool,
}

pub fn ler_opcoes(argumentos: &[String]) -> Result<Opcoes, String> {
    let mut opcoes = Opcoes {
        arquivo: None,
        executar: true,
        detalhes: false,
        ajuda: false,
        versao: false,
    };
    let mut comando_lido = false;
    for argumento in argumentos.iter().skip(1) {
        match argumento.as_str() {
            "check" if !comando_lido && opcoes.arquivo.is_none() => {
                opcoes.executar = false;
                comando_lido = true;
            }
            "--detalhes" => opcoes.detalhes = true,
            "--ajuda" | "-h" => opcoes.ajuda = true,
            "--versao" | "-v" => opcoes.versao = true,
            outro if outro.starts_with('-') => {
                return Err(format!("Opção desconhecida: {}", outro));
            }
            outro => {
                if opcoes.arquivo.is_some() {
                    return Err(format!("Argumento inesperado: {}", outro));
                }
                opcoes.arquivo = Some(outro.to_string());
            }
        }
    }
    Ok(opcoes)
}

fn executar_arquivo(caminho: &str, executar: bool, detalhes: bool) -> Result<(), NeoErro> {
    let codigo = fs::read_to_string(caminho).map_err(|erro| NeoErro::Runtime {
        pos: Posicao::nova(1, 1),
        codigo: "NEO3009",
        mensagem: format!("Não foi possível abrir '{}': {}", caminho, erro),
    })?;

    if detalhes {
        println!("NeoCOBOL {}", VERSAO);
        println!("Compilando: {}", caminho);
    }

    let tokens = lex(&codigo)?;
    if detalhes {
        println!("Lexer: {} tokens.", tokens.len());
    }

    let mut parser = Parser::novo(tokens);
    let programa = parser.analisar()?;
    if detalhes {
        println!("Parser: AST construída.");
    }

    let mut semantico = AnalisadorSemantico::novo();
    semantico.analisar(&programa)?;
    if detalhes {
        println!("Semântica: programa válido.");
    }

    if !executar {
        println!("{}: nenhum erro encontrado.", caminho);
        return Ok(());
    }

    let mut runtime = Runtime::novo();
    runtime.executar(&programa)?;
    if detalhes {
        println!("\nPrograma executado com sucesso.");
    }
    Ok(())
}

fn mostrar_ajuda() {
    println!("NeoCOBOL {}", VERSAO);
    println!("Uso:");
    println!("  neocobol programa.neo          executa o programa");
    println!("  neocobol check programa.neo    verifica o programa sem executar");
    println!("Opções:");
    println!("  --detalhes   mostra as etapas (Lexer, Parser, Semântica)");
    println!("  --versao     mostra a versão");
    println!("  --ajuda      mostra esta ajuda");
    println!("Códigos de saída: 0 (sucesso), 1 (erro no programa), 2 (opção inválida).");
}

fn processar(argumentos: Vec<String>) -> i32 {
    let opcoes = match ler_opcoes(&argumentos) {
        Ok(opcoes) => opcoes,
        Err(mensagem) => {
            eprintln!("{}", mensagem);
            eprintln!("Use --ajuda para ver as opções.");
            return 2;
        }
    };

    if opcoes.versao {
        println!("NeoCOBOL {}", VERSAO);
        return 0;
    }
    if opcoes.ajuda {
        mostrar_ajuda();
        return 0;
    }
    let caminho = match opcoes.arquivo.clone() {
        Some(caminho) => caminho,
        None => {
            if !opcoes.executar {
                eprintln!(
                    "O comando check precisa de um arquivo. Exemplo: neocobol check programa.neo"
                );
                return 2;
            }
            mostrar_ajuda();
            return 0;
        }
    };

    match executar_arquivo(&caminho, opcoes.executar, opcoes.detalhes) {
        Ok(()) => 0,
        Err(erro) => {
            let fonte = fs::read_to_string(&caminho).unwrap_or_default();
            eprint!("{}", diagnostico::formatar(&erro, &caminho, &fonte));
            1
        }
    }
}

pub fn principal(argumentos: Vec<String>) -> i32 {
    let copia = argumentos.clone();
    let tentativa = std::thread::Builder::new()
        .stack_size(PILHA_DO_INTERPRETADOR)
        .spawn(move || processar(copia));
    match tentativa {
        Ok(thread) => thread.join().unwrap_or(101),
        Err(_) => processar(argumentos),
    }
}
