//! Problema dos Jarros — entrega com interface.
//!
//! O binário tem dois modos:
//!   * `cargo run --release`          inicia o servidor local e abre a interface;
//!   * `cargo run --release -- --cli` imprime o relatório da busca no terminal.
//!
//! A interface web (HTML/CSS/JS) é embutida no binário com `include_str!`, então
//! a entrega é um executável único, sem dependências externas e sem instalar
//! nada além do compilador Rust.

mod solver;

use solver::{Algoritmo, Alvo, Configuracao, Resultado};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{self, Command};
use std::thread;

const HTML: &str = include_str!("web/index.html");
const CSS: &str = include_str!("web/style.css");
const JS: &str = include_str!("web/app.js");

const HELP: &str = "\
Problema dos Jarros — busca em espaço de estados (BFS/DFS)

Uso:
  cargo run --release                    inicia a interface no navegador
  cargo run --release -- --cli           mostra o relatório da busca no terminal

Opções:
  --cli                   modo terminal (não sobe o servidor)
  --port N                porta do servidor local (padrão: 8080)
  --no-open               não abre o navegador automaticamente
  --cap-a N               capacidade do jarro A (padrão: 5)
  --cap-b N               capacidade do jarro B (padrão: 3)
  --meta N                quantidade de litros a medir (padrão: 4)
  --alvo qualquer|a|b     jarro onde a meta é medida (padrão: qualquer)
  --algoritmo bfs|dfs     estratégia de busca (padrão: bfs)
  --help                  mostra esta ajuda

Exemplo:
  cargo run --release -- --cli --cap-a 5 --cap-b 3 --meta 4
";

struct Argumentos {
    cli: bool,
    porta: u16,
    abrir_navegador: bool,
    capacidade_a: u32,
    capacidade_b: u32,
    meta: u32,
    alvo: Alvo,
    algoritmo: Algoritmo,
}

impl Default for Argumentos {
    fn default() -> Self {
        Argumentos {
            cli: false,
            porta: 8080,
            abrir_navegador: true,
            capacidade_a: 5,
            capacidade_b: 3,
            meta: 4,
            alvo: Alvo::Qualquer,
            algoritmo: Algoritmo::Bfs,
        }
    }
}

fn proximo_valor(iterador: &mut impl Iterator<Item = String>, opcao: &str) -> String {
    match iterador.next() {
        Some(valor) => valor,
        None => {
            eprintln!("Faltou o valor de {opcao}.");
            process::exit(2);
        }
    }
}

fn ler_numero(iterador: &mut impl Iterator<Item = String>, opcao: &str) -> u32 {
    let valor = proximo_valor(iterador, opcao);
    valor.parse::<u32>().unwrap_or_else(|_| {
        eprintln!("{opcao}: número inválido ({valor}).");
        process::exit(2);
    })
}

fn interpretar_argumentos() -> Argumentos {
    let mut args = Argumentos::default();
    let mut iterador = std::env::args().skip(1);

    while let Some(arg) = iterador.next() {
        match arg.as_str() {
            "--cli" => args.cli = true,
            "--no-open" => args.abrir_navegador = false,
            "--port" => {
                let valor = proximo_valor(&mut iterador, "--port");
                args.porta = valor.parse::<u16>().unwrap_or_else(|_| {
                    eprintln!("--port: porta inválida ({valor}).");
                    process::exit(2);
                });
            }
            "--cap-a" => args.capacidade_a = ler_numero(&mut iterador, "--cap-a"),
            "--cap-b" => args.capacidade_b = ler_numero(&mut iterador, "--cap-b"),
            "--meta" => args.meta = ler_numero(&mut iterador, "--meta"),
            "--alvo" => {
                let valor = proximo_valor(&mut iterador, "--alvo");
                args.alvo = match valor.as_str() {
                    "qualquer" | "any" => Alvo::Qualquer,
                    "a" | "A" => Alvo::JarroA,
                    "b" | "B" => Alvo::JarroB,
                    outro => {
                        eprintln!("--alvo: valor inválido ({outro}); use qualquer, a ou b.");
                        process::exit(2);
                    }
                };
            }
            "--algoritmo" => {
                let valor = proximo_valor(&mut iterador, "--algoritmo");
                args.algoritmo = match valor.as_str() {
                    "bfs" | "BFS" => Algoritmo::Bfs,
                    "dfs" | "DFS" => Algoritmo::Dfs,
                    outro => {
                        eprintln!("--algoritmo: valor inválido ({outro}); use bfs ou dfs.");
                        process::exit(2);
                    }
                };
            }
            "--help" | "-h" => {
                println!("{HELP}");
                process::exit(0);
            }
            outro => {
                eprintln!("Argumento desconhecido: {outro}\n");
                eprintln!("{HELP}");
                process::exit(2);
            }
        }
    }

    args
}

fn main() {
    let args = interpretar_argumentos();
    let configuracao = Configuracao::nova(
        args.capacidade_a,
        args.capacidade_b,
        args.meta,
        args.alvo,
        args.algoritmo,
    );

    if args.cli {
        executar_cli(&configuracao);
    } else {
        iniciar_servidor(args.porta, args.abrir_navegador);
    }
}

// ---------------------------------------------------------------------------
// Modo CLI: relatório textual da busca (para o relatório da disciplina).
// ---------------------------------------------------------------------------

fn executar_cli(configuracao: &Configuracao) {
    println!();
    println!("============================================================");
    println!(" PROBLEMA DOS JARROS — BUSCA EM ESPAÇO DE ESTADOS");
    println!(" Algoritmo: {}", configuracao.algoritmo.rotulo());
    println!(
        " Jarro A: {} L · Jarro B: {} L · Meta: {} L ({})",
        configuracao.capacidade_a,
        configuracao.capacidade_b,
        configuracao.meta,
        configuracao.alvo.rotulo().to_lowercase()
    );
    println!("============================================================");
    println!();

    if let Err(mensagem) = configuracao.validar() {
        println!("ENTRADA INVÁLIDA: {mensagem}");
        println!();
        return;
    }

    let resultado = solver::resolver(configuracao);

    println!("ORDEM DE EXPANSÃO:");
    for (indice, passo) in resultado.passos.iter().enumerate() {
        let no = &resultado.nos[passo.expandido];
        let acao = no.acao.map(|a| a.rotulo()).unwrap_or("Início");
        println!(
            "{:>3}. ({:>2},{:>2})  {:<22} borda: {}",
            indice,
            no.estado.a,
            no.estado.b,
            acao,
            passo.borda.len()
        );
    }

    println!();
    if resultado.encontrado {
        let profundidade = resultado.profundidade().unwrap_or(0);
        println!("OBJETIVO ENCONTRADO");
        println!();
        println!("CAMINHO DA SOLUÇÃO ({profundidade} passos):");
        for (indice, id) in resultado.caminho.iter().enumerate() {
            let no = &resultado.nos[*id];
            let acao = match no.acao {
                Some(acao) => acao.rotulo().to_string(),
                None => "Início".to_string(),
            };
            println!(
                "{:>3}. ({:>2},{:>2})  {}",
                indice, no.estado.a, no.estado.b, acao
            );
        }
    } else {
        println!("NENHUMA SOLUÇÃO ENCONTRADA para esta configuração.");
    }

    let e = &resultado.estatisticas;
    let tempo = format!("{:.3}", e.duracao_micros as f64 / 1000.0).replace('.', ",");
    println!();
    println!("ESTATÍSTICAS");
    println!("  nós gerados:        {}", e.nos_gerados);
    println!("  estados visitados:  {}", e.estados_visitados);
    println!("  nós expandidos:     {}", e.estados_expandidos);
    println!("  repetidos evitados: {}", e.repetidos_evitados);
    println!("  maior borda:        {}", e.maior_borda);
    println!(
        "  profundidade:       {}",
        e.profundidade_solucao
            .map(|p| p.to_string())
            .unwrap_or_else(|| "—".to_string())
    );
    println!("  tempo:              {tempo} ms");
    println!();
    println!("Dica: rode `cargo run --release` para ver a mesma busca na interface.");
    println!();
}

// ---------------------------------------------------------------------------
// Utilitários de JSON (sem serde: o formato de resposta é fixo e pequeno).
// ---------------------------------------------------------------------------

fn escapar_json(texto: &str) -> String {
    let mut saida = String::with_capacity(texto.len() + 8);
    for caractere in texto.chars() {
        match caractere {
            '"' => saida.push_str("\\\""),
            '\\' => saida.push_str("\\\\"),
            '\n' => saida.push_str("\\n"),
            '\r' => saida.push_str("\\r"),
            '\t' => saida.push_str("\\t"),
            c if (c as u32) < 0x20 => saida.push_str(&format!("\\u{:04x}", c as u32)),
            c => saida.push(c),
        }
    }
    saida
}

fn texto_json(valor: &str) -> String {
    format!("\"{}\"", escapar_json(valor))
}

fn opcional_json<T: ToString>(valor: Option<T>) -> String {
    valor
        .map(|v| v.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn ids_json(ids: &[usize]) -> String {
    format!(
        "[{}]",
        ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn configuracao_json(configuracao: &Configuracao) -> String {
    let alvo = match configuracao.alvo {
        Alvo::Qualquer => "qualquer",
        Alvo::JarroA => "a",
        Alvo::JarroB => "b",
    };
    let algoritmo = match configuracao.algoritmo {
        Algoritmo::Bfs => "bfs",
        Algoritmo::Dfs => "dfs",
    };
    format!(
        "{{\"capacidadeA\":{},\"capacidadeB\":{},\"meta\":{},\"alvo\":{},\"alvoRotulo\":{},\"algoritmo\":{},\"algoritmoRotulo\":{}}}",
        configuracao.capacidade_a,
        configuracao.capacidade_b,
        configuracao.meta,
        texto_json(alvo),
        texto_json(configuracao.alvo.rotulo()),
        texto_json(algoritmo),
        texto_json(configuracao.algoritmo.rotulo())
    )
}

fn resultado_json(resultado: &Resultado) -> String {
    let nos = resultado
        .nos
        .iter()
        .map(|no| {
            let acao_rotulo = no
                .acao
                .map(|a| a.rotulo())
                .map(texto_json)
                .unwrap_or_else(|| "null".to_string());
            let acao_curta = no
                .acao
                .map(|a| a.curto())
                .map(texto_json)
                .unwrap_or_else(|| "null".to_string());
            format!(
                "{{\"id\":{},\"a\":{},\"b\":{},\"pai\":{},\"profundidade\":{},\"expandido\":{},\"acaoRotulo\":{},\"acaoCurta\":{}}}",
                no.id,
                no.estado.a,
                no.estado.b,
                opcional_json(no.pai),
                no.profundidade,
                no.expandido,
                acao_rotulo,
                acao_curta
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let passos = resultado
        .passos
        .iter()
        .map(|passo| {
            let sucessores = passo
                .sucessores
                .iter()
                .map(|sucessor| {
                    let situacao = match sucessor.situacao {
                        solver::Situacao::Novo => "novo",
                        solver::Situacao::Repetido => "repetido",
                    };
                    format!(
                        "{{\"a\":{},\"b\":{},\"acaoRotulo\":{},\"acaoCurta\":{},\"situacao\":{},\"no\":{}}}",
                        sucessor.estado.a,
                        sucessor.estado.b,
                        texto_json(sucessor.acao.rotulo()),
                        texto_json(sucessor.acao.curto()),
                        texto_json(situacao),
                        opcional_json(sucessor.no)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"expandido\":{},\"totalExpandidos\":{},\"objetivo\":{},\"borda\":{},\"sucessores\":[{}]}}",
                passo.expandido,
                passo.total_expandidos,
                passo.objetivo,
                ids_json(&passo.borda),
                sucessores
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let estatisticas = &resultado.estatisticas;
    let estatisticas_json = format!(
        "{{\"nosGerados\":{},\"estadosVisitados\":{},\"estadosExpandidos\":{},\"repetidosEvitados\":{},\"maiorBorda\":{},\"profundidade\":{},\"duracaoMicros\":{}}}",
        estatisticas.nos_gerados,
        estatisticas.estados_visitados,
        estatisticas.estados_expandidos,
        estatisticas.repetidos_evitados,
        estatisticas.maior_borda,
        opcional_json(estatisticas.profundidade_solucao),
        estatisticas.duracao_micros
    );

    let mensagem = if resultado.encontrado {
        let passos = resultado.profundidade().unwrap_or(0);
        format!(
            "Solução encontrada em {} {}.{}",
            passos,
            if passos == 1 { "passo" } else { "passos" },
            if resultado.configuracao.algoritmo == Algoritmo::Bfs {
                " A BFS garante que esse é o caminho mínimo."
            } else {
                ""
            }
        )
    } else {
        "Nenhuma solução encontrada para esta configuração.".to_string()
    };

    format!(
        "{{\"ok\":true,\"config\":{},\"encontrado\":{},\"mensagem\":{},\"nos\":[{}],\"caminho\":{},\"expandidos\":{},\"bordaFinal\":{},\"passos\":[{}],\"estatisticas\":{}}}",
        configuracao_json(&resultado.configuracao),
        resultado.encontrado,
        texto_json(&mensagem),
        nos,
        ids_json(&resultado.caminho),
        ids_json(&resultado.expandidos),
        ids_json(&resultado.borda_final),
        passos,
        estatisticas_json
    )
}

fn erro_json(mensagem: &str, configuracao: Option<&Configuracao>) -> String {
    let config = configuracao
        .map(configuracao_json)
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{\"ok\":false,\"mensagem\":{},\"config\":{}}}",
        texto_json(mensagem),
        config
    )
}

// ---------------------------------------------------------------------------
// Servidor HTTP local (somente biblioteca padrão).
// ---------------------------------------------------------------------------

struct Resposta {
    status: &'static str,
    content_type: &'static str,
    corpo: String,
}

impl Resposta {
    fn html(corpo: String) -> Self {
        Resposta {
            status: "200 OK",
            content_type: "text/html; charset=utf-8",
            corpo,
        }
    }

    fn js(corpo: String) -> Self {
        Resposta {
            status: "200 OK",
            content_type: "application/javascript; charset=utf-8",
            corpo,
        }
    }

    fn css(corpo: String) -> Self {
        Resposta {
            status: "200 OK",
            content_type: "text/css; charset=utf-8",
            corpo,
        }
    }

    fn json(corpo: String) -> Self {
        Resposta {
            status: "200 OK",
            content_type: "application/json; charset=utf-8",
            corpo,
        }
    }

    fn nao_encontrado() -> Self {
        Resposta {
            status: "404 Not Found",
            content_type: "text/plain; charset=utf-8",
            corpo: "Não encontrado".to_string(),
        }
    }

    fn sem_conteudo() -> Self {
        Resposta {
            status: "204 No Content",
            content_type: "text/plain; charset=utf-8",
            corpo: String::new(),
        }
    }
}

fn escrever_resposta(mut stream: &TcpStream, resposta: &Resposta) {
    let cabecalho = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        resposta.status,
        resposta.content_type,
        resposta.corpo.as_bytes().len()
    );
    let _ = stream.write_all(cabecalho.as_bytes());
    let _ = stream.write_all(resposta.corpo.as_bytes());
    let _ = stream.flush();
}

/// Lê pares `chave=valor` da query string, sem dependências externas.
fn parametros(consulta: Option<&str>) -> Vec<(String, String)> {
    let mut lista = Vec::new();
    if let Some(consulta) = consulta {
        for par in consulta.split('&') {
            if par.is_empty() {
                continue;
            }
            let (chave, valor) = match par.split_once('=') {
                Some((chave, valor)) => (chave, valor),
                None => (par, ""),
            };
            lista.push((decodificar(chave), decodificar(valor)));
        }
    }
    lista
}

/// Decodifica %XX e '+' de uma query string.
fn decodificar(texto: &str) -> String {
    let mut bytes = Vec::with_capacity(texto.len());
    let mut iterador = texto.bytes();
    while let Some(byte) = iterador.next() {
        if byte == b'%' {
            let alto = iterador.next().unwrap_or(b'0');
            let baixo = iterador.next().unwrap_or(b'0');
            let digitos = [alto, baixo];
            if let Ok(digitos) = std::str::from_utf8(&digitos) {
                if let Ok(valor) = u8::from_str_radix(digitos, 16) {
                    bytes.push(valor);
                    continue;
                }
            }
            bytes.push(byte);
        } else if byte == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

fn obter<'a>(parametros: &'a [(String, String)], chave: &str) -> Option<&'a str> {
    parametros
        .iter()
        .find(|(nome, _)| nome == chave)
        .map(|(_, valor)| valor.as_str())
}

fn api_resolver(consulta: Option<&str>) -> String {
    let parametros = parametros(consulta);

    let capacidade_a = match obter(&parametros, "capA").and_then(|v| v.parse::<u32>().ok()) {
        Some(valor) => valor,
        None => return erro_json("Parâmetro capA ausente ou inválido.", None),
    };
    let capacidade_b = match obter(&parametros, "capB").and_then(|v| v.parse::<u32>().ok()) {
        Some(valor) => valor,
        None => return erro_json("Parâmetro capB ausente ou inválido.", None),
    };
    let meta = match obter(&parametros, "meta").and_then(|v| v.parse::<u32>().ok()) {
        Some(valor) => valor,
        None => return erro_json("Parâmetro meta ausente ou inválido.", None),
    };
    let alvo = match obter(&parametros, "alvo").unwrap_or("qualquer") {
        "a" => Alvo::JarroA,
        "b" => Alvo::JarroB,
        "qualquer" | "any" => Alvo::Qualquer,
        outro => return erro_json(&format!("Parâmetro alvo inválido: {outro}."), None),
    };
    let algoritmo = match obter(&parametros, "algoritmo").unwrap_or("bfs") {
        "bfs" => Algoritmo::Bfs,
        "dfs" => Algoritmo::Dfs,
        outro => return erro_json(&format!("Parâmetro algoritmo inválido: {outro}."), None),
    };

    let configuracao = Configuracao::nova(capacidade_a, capacidade_b, meta, alvo, algoritmo);
    match configuracao.validar() {
        Err(mensagem) => erro_json(&mensagem, Some(&configuracao)),
        Ok(()) => {
            let resultado = solver::resolver(&configuracao);
            resultado_json(&resultado)
        }
    }
}

fn responder_requisicao(stream: &TcpStream) {
    let mut buffer = [0u8; 8192];
    let mut leitura = match stream.try_clone() {
        Ok(copia) => copia,
        Err(_) => return,
    };
    let _ = leitura.set_read_timeout(Some(std::time::Duration::from_secs(10)));
    let lidos = match leitura.read(&mut buffer) {
        Ok(0) => return,
        Ok(n) => n,
        Err(_) => return,
    };
    let requisicao = String::from_utf8_lossy(&buffer[..lidos]);
    let primeira_linha = requisicao.lines().next().unwrap_or("").trim();
    let mut partes = primeira_linha.split_whitespace();
    let metodo = partes.next().unwrap_or("");
    let alvo_url = partes.next().unwrap_or("/");
    let (rota, consulta) = match alvo_url.split_once('?') {
        Some((rota, consulta)) => (rota, Some(consulta)),
        None => (alvo_url, None),
    };

    let resposta = match (metodo, rota) {
        ("GET", "/") | ("GET", "/index.html") => Resposta::html(HTML.to_string()),
        ("GET", "/style.css") => Resposta::css(CSS.to_string()),
        ("GET", "/app.js") => Resposta::js(JS.to_string()),
        ("GET", "/api/solve") => Resposta::json(api_resolver(consulta)),
        ("GET", "/favicon.ico") => Resposta::sem_conteudo(),
        _ => Resposta::nao_encontrado(),
    };
    escrever_resposta(stream, &resposta);
}

fn abrir_navegador(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(url).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

fn iniciar_servidor(porta_inicial: u16, abrir: bool) {
    let mut escolha = None;
    for deslocamento in 0..20 {
        let porta = porta_inicial.saturating_add(deslocamento);
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", porta)) {
            escolha = Some((listener, porta));
            break;
        }
    }

    let Some((listener, porta)) = escolha else {
        eprintln!(
            "Não foi possível abrir nenhuma porta entre {porta_inicial} e {}.",
            porta_inicial.saturating_add(19)
        );
        process::exit(1);
    };

    let url = format!("http://127.0.0.1:{porta}");
    println!();
    println!("  Problema dos Jarros — interface de busca");
    println!("  Servidor local: {url}");
    println!("  Encerre com Ctrl+C.");
    println!();

    if abrir {
        abrir_navegador(&url);
    }

    for conexao in listener.incoming() {
        match conexao {
            Ok(stream) => {
                thread::spawn(move || responder_requisicao(&stream));
            }
            Err(erro) => eprintln!("Falha ao aceitar conexão: {erro}"),
        }
    }
}
