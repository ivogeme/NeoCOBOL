// ================================================================
// Diagnósticos da NeoCOBOL: posição, erros, códigos e formatação.
// 0.6.0
// ================================================================

use std::fmt;

/// Trecho de uma linha do código: linha, coluna inicial e coluna final.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Posicao {
    pub linha: usize,
    pub coluna: usize,
    /// Coluna logo depois do último caractere do trecho (exclusiva).
    /// Igual a `coluna` quando o fim do trecho não é conhecido.
    pub fim: usize,
}

impl Posicao {
    pub fn nova(linha: usize, coluna: usize) -> Self {
        Self {
            linha,
            coluna,
            fim: coluna,
        }
    }

    pub fn ate(self, fim: usize) -> Self {
        Self {
            fim: fim.max(self.coluna),
            ..self
        }
    }

    pub fn largura(&self) -> usize {
        self.fim.saturating_sub(self.coluna)
    }
}

// Usado só pelos testes.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodigoErro {
    Lexico,
    Sintaxe,
    Tipo,
    Semantico,
    Runtime,
}

// Todo erro nasce com o seu código (campo `codigo`).
// O título e a ajuda de cada código ficam na TABELA, mais abaixo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeoErro {
    Lexico {
        pos: Posicao,
        codigo: &'static str,
        mensagem: String,
    },
    Sintaxe {
        pos: Posicao,
        codigo: &'static str,
        mensagem: String,
    },
    Tipo {
        pos: Posicao,
        codigo: &'static str,
        esperado: String,
        recebido: String,
        contexto: String,
    },
    Semantico {
        pos: Posicao,
        codigo: &'static str,
        mensagem: String,
    },
    Runtime {
        pos: Posicao,
        codigo: &'static str,
        mensagem: String,
    },
}

#[derive(Debug, Clone)]
pub struct Diagnostico {
    pub codigo: &'static str,
    pub titulo: &'static str,
    pub pos: Posicao,
    pub mensagem: String,
    pub esperado: Option<String>,
    pub recebido: Option<String>,
    pub ajuda: Option<&'static str>,
}

impl Diagnostico {
    fn detalhe(&self) -> String {
        match (&self.esperado, &self.recebido) {
            (Some(e), Some(r)) => format!("esperado {}, recebido {}. {}", e, r, self.mensagem),
            _ => self.mensagem.clone(),
        }
    }
}

// Usados só pelos testes.
#[cfg(test)]
impl NeoErro {
    pub fn codigo(&self) -> CodigoErro {
        match self {
            NeoErro::Lexico { .. } => CodigoErro::Lexico,
            NeoErro::Sintaxe { .. } => CodigoErro::Sintaxe,
            NeoErro::Tipo { .. } => CodigoErro::Tipo,
            NeoErro::Semantico { .. } => CodigoErro::Semantico,
            NeoErro::Runtime { .. } => CodigoErro::Runtime,
        }
    }

    pub fn posicao(&self) -> Posicao {
        match self {
            NeoErro::Lexico { pos, .. }
            | NeoErro::Sintaxe { pos, .. }
            | NeoErro::Tipo { pos, .. }
            | NeoErro::Semantico { pos, .. }
            | NeoErro::Runtime { pos, .. } => *pos,
        }
    }
}

impl NeoErro {
    pub fn para_diagnostico(&self) -> Diagnostico {
        let (codigo, pos, mensagem, esperado, recebido) = match self {
            NeoErro::Lexico {
                pos,
                codigo,
                mensagem,
            }
            | NeoErro::Sintaxe {
                pos,
                codigo,
                mensagem,
            }
            | NeoErro::Semantico {
                pos,
                codigo,
                mensagem,
            }
            | NeoErro::Runtime {
                pos,
                codigo,
                mensagem,
            } => (*codigo, *pos, mensagem.clone(), None, None),
            NeoErro::Tipo {
                pos,
                codigo,
                esperado,
                recebido,
                contexto,
            } => (
                *codigo,
                *pos,
                contexto.clone(),
                Some(esperado.clone()),
                Some(recebido.clone()),
            ),
        };
        let (titulo, ajuda) = info(codigo);
        Diagnostico {
            codigo,
            titulo,
            pos,
            mensagem,
            esperado,
            recebido,
            ajuda,
        }
    }
}

impl fmt::Display for NeoErro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.para_diagnostico();
        write!(
            f,
            "{} - {} na linha {}, coluna {}: {}",
            d.codigo,
            d.titulo,
            d.pos.linha,
            d.pos.coluna,
            d.detalhe()
        )
    }
}

// ================================================================
// TABELA DE CÓDIGOS
// Cada código identifica um tipo de erro e não muda de significado.
// ================================================================

pub struct Entrada {
    pub codigo: &'static str,
    pub titulo: &'static str,
    pub ajuda: Option<&'static str>,
}

pub const TABELA: &[Entrada] = &[
    Entrada {
        codigo: "NEO1001",
        titulo: "Caractere desconhecido",
        ajuda: Some("Remova o caractere ou coloque-o dentro de um texto entre aspas."),
    },
    Entrada {
        codigo: "NEO1002",
        titulo: "String não terminada",
        ajuda: Some("Feche o texto com aspas duplas."),
    },
    Entrada {
        codigo: "NEO1003",
        titulo: "Número decimal inválido",
        ajuda: Some("Use apenas um ponto decimal, por exemplo 12.50."),
    },
    Entrada {
        codigo: "NEO1004",
        titulo: "Uso inválido de '!'",
        ajuda: Some("Para comparar diferença, use '!='."),
    },
    Entrada {
        codigo: "NEO1101",
        titulo: "Instrução inesperada",
        ajuda: Some(
            "Uma instrução começa com um tipo (decimal, string ou boolean), um nome, display, if, while, func ou return.",
        ),
    },
    Entrada {
        codigo: "NEO1102",
        titulo: "Expressão inesperada ou incompleta",
        ajuda: Some("Complete a expressão: falta um valor, uma variável ou uma chamada de função."),
    },
    Entrada {
        codigo: "NEO1103",
        titulo: "Elemento esperado ausente",
        ajuda: Some("Confira se não falta algo, como ')' ou 'end'."),
    },
    Entrada {
        codigo: "NEO1104",
        titulo: "Tipo esperado",
        ajuda: Some("Use decimal, string ou boolean (ou vazio no retorno de função)."),
    },
    Entrada {
        codigo: "NEO1105",
        titulo: "Atribuição incompleta",
        ajuda: Some("Depois do nome, use '=' ou um operador como '+='."),
    },
    Entrada {
        codigo: "NEO1106",
        titulo: "Interpolação inválida",
        ajuda: Some("Confira a expressão entre chaves."),
    },
    Entrada {
        codigo: "NEO1107",
        titulo: "Aninhamento ou tamanho excessivo",
        ajuda: Some("Divida a expressão ou o bloco em partes menores."),
    },
    Entrada {
        codigo: "NEO1108",
        titulo: "Fluxo de tokens vazio",
        ajuda: None,
    },
    Entrada {
        codigo: "NEO2001",
        titulo: "Tipos incompatíveis",
        ajuda: Some(
            "Os dois lados da operação precisam ter tipos compatíveis. Não há conversão automática.",
        ),
    },
    Entrada {
        codigo: "NEO2002",
        titulo: "Tipo incompatível em declaração ou atribuição",
        ajuda: Some("O valor precisa ter o mesmo tipo da variável. Não há conversão automática."),
    },
    Entrada {
        codigo: "NEO2003",
        titulo: "Condição que não é boolean",
        ajuda: Some("A condição precisa resultar em true ou false, por exemplo: x > 0"),
    },
    Entrada {
        codigo: "NEO2004",
        titulo: "Tipo incorreto de argumento",
        ajuda: Some("Passe o argumento com o tipo declarado no parâmetro da função."),
    },
    Entrada {
        codigo: "NEO2005",
        titulo: "Tipo de retorno incompatível",
        ajuda: Some("O valor devolvido precisa ter o tipo declarado depois de '->'."),
    },
    Entrada {
        codigo: "NEO2006",
        titulo: "Valor vazio usado como valor",
        ajuda: Some("Uma função 'vazio' não devolve valor; use-a como instrução."),
    },
    Entrada {
        codigo: "NEO2101",
        titulo: "Variável não declarada",
        ajuda: Some("Declare a variável antes de usá-la, por exemplo: decimal nome = 0"),
    },
    Entrada {
        codigo: "NEO2102",
        titulo: "Função não encontrada",
        ajuda: Some("Confira o nome da função e se ela foi declarada com func."),
    },
    Entrada {
        codigo: "NEO2103",
        titulo: "Quantidade incorreta de argumentos",
        ajuda: Some("Confira quantos parâmetros a função declara."),
    },
    Entrada {
        codigo: "NEO2104",
        titulo: "Declaração duplicada",
        ajuda: Some("Cada nome pode ser declarado uma única vez no mesmo escopo."),
    },
    Entrada {
        codigo: "NEO2105",
        titulo: "return fora de função",
        ajuda: Some("O return só pode ser usado dentro de uma função."),
    },
    Entrada {
        codigo: "NEO2106",
        titulo: "Retorno obrigatório ausente",
        ajuda: Some("Inclua um return em todos os caminhos da função."),
    },
    Entrada {
        codigo: "NEO2107",
        titulo: "Função em local inválido",
        ajuda: Some("Declare funções apenas no nível principal do programa."),
    },
    Entrada {
        codigo: "NEO2108",
        titulo: "Falha interna da análise",
        ajuda: None,
    },
    Entrada {
        codigo: "NEO3001",
        titulo: "Divisão por zero",
        ajuda: Some("Confira se o divisor pode ser zero antes de dividir."),
    },
    Entrada {
        codigo: "NEO3002",
        titulo: "Módulo por zero",
        ajuda: Some("Confira se o divisor pode ser zero antes de usar %."),
    },
    Entrada {
        codigo: "NEO3003",
        titulo: "Overflow aritmético",
        ajuda: Some("O resultado ficou grande demais para o tipo decimal."),
    },
    Entrada {
        codigo: "NEO3004",
        titulo: "Limite de chamadas excedido",
        ajuda: Some("Isso costuma indicar uma recursão sem condição de parada."),
    },
    Entrada {
        codigo: "NEO3005",
        titulo: "Variável não encontrada na execução",
        ajuda: None,
    },
    Entrada {
        codigo: "NEO3006",
        titulo: "Função não encontrada na execução",
        ajuda: None,
    },
    Entrada {
        codigo: "NEO3007",
        titulo: "Quantidade incorreta de argumentos (execução)",
        ajuda: None,
    },
    Entrada {
        codigo: "NEO3008",
        titulo: "Retorno diferente do declarado",
        ajuda: None,
    },
    Entrada {
        codigo: "NEO3009",
        titulo: "Arquivo não pode ser aberto",
        ajuda: Some("Confira o caminho e o nome do arquivo."),
    },
    Entrada {
        codigo: "NEO3010",
        titulo: "Falha interna de execução",
        ajuda: None,
    },
];

fn info(codigo: &str) -> (&'static str, Option<&'static str>) {
    for e in TABELA {
        if e.codigo == codigo {
            return (e.titulo, e.ajuda);
        }
    }
    ("Erro", None)
}

// ================================================================
// FORMATAÇÃO
// ================================================================

pub fn formatar(erro: &NeoErro, arquivo: &str, fonte: &str) -> String {
    let d = erro.para_diagnostico();
    let numero = d.pos.linha.to_string();
    let margem = " ".repeat(numero.len());
    let mut saida = String::new();

    saida.push_str(&format!("erro[{}]: {}\n", d.codigo, d.titulo));
    saida.push_str(&format!(
        "{}--> {}:{}:{}\n",
        margem, arquivo, d.pos.linha, d.pos.coluna
    ));

    let linha = if d.pos.linha >= 1 {
        fonte.lines().nth(d.pos.linha - 1)
    } else {
        None
    };
    if let Some(texto) = linha {
        let coluna = d.pos.coluna.max(1);
        let prefixo: String = texto
            .chars()
            .take(coluna - 1)
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        let restante = texto.chars().count().saturating_sub(coluna - 1).max(1);
        let largura = if d.pos.largura() > 0 {
            d.pos.largura()
        } else {
            tamanho_trecho(texto, coluna)
        };
        let marcas = "^".repeat(largura.min(restante).max(1));
        saida.push_str(&format!("{} |\n", margem));
        saida.push_str(&format!("{} | {}\n", numero, texto));
        saida.push_str(&format!("{} | {}{}\n", margem, prefixo, marcas));
    }

    if let (Some(e), Some(r)) = (&d.esperado, &d.recebido) {
        saida.push_str(&format!("{} = esperado: {}\n", margem, e));
        saida.push_str(&format!("{} = recebido: {}\n", margem, r));
    }
    saida.push_str(&format!("{} = {}\n", margem, d.mensagem));
    if let Some(ajuda) = d.ajuda {
        saida.push_str(&format!("{} = ajuda: {}\n", margem, ajuda));
    }
    saida
}

// Plano B quando o fim do trecho não é conhecido.
fn tamanho_trecho(linha: &str, coluna: usize) -> usize {
    let chars: Vec<char> = linha.chars().collect();
    let i = coluna.saturating_sub(1);
    if i >= chars.len() {
        return 1;
    }
    let c = chars[i];
    if c.is_alphanumeric() || c == '_' {
        let numero = c.is_ascii_digit();
        let mut j = i;
        while j < chars.len()
            && (chars[j].is_alphanumeric() || chars[j] == '_' || (numero && chars[j] == '.'))
        {
            j += 1;
        }
        return j - i;
    }
    if c == '"' {
        let mut j = i + 1;
        while j < chars.len() {
            if chars[j] == '\\' {
                j += 2;
                continue;
            }
            if chars[j] == '"' {
                return j + 1 - i;
            }
            j += 1;
        }
        return chars.len() - i;
    }
    if i + 1 < chars.len() {
        let par: String = chars[i..i + 2].iter().collect();
        if matches!(
            par.as_str(),
            "==" | "!=" | ">=" | "<=" | "+=" | "-=" | "*=" | "/=" | "%=" | "->"
        ) {
            return 2;
        }
    }
    1
}
