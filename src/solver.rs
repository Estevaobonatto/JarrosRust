//! Núcleo do trabalho: busca em espaço de estados para o Problema dos Jarros.
//!
//! O estado é representado pelo par `(litros no jarro A, litros no jarro B)`.
//! As seis ações clássicas geram os sucessores. A busca em largura (BFS) usa
//! uma fila (FIFO) e garante o caminho mínimo; a busca em profundidade (DFS)
//! usa uma pilha (LIFO). Em ambas, um conjunto de visitados evita ciclos.
//!
//! O resultado inclui o rastro completo da busca (ordem de expansão, borda,
//! sucessores descartados por repetição) para alimentar a interface e o
//! relatório em modo CLI.

use std::collections::{HashSet, VecDeque};
use std::time::Instant;

/// Estado dos dois jarros. `a` = litros no jarro A, `b` = litros no jarro B.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Estado {
    pub a: u32,
    pub b: u32,
}

impl Estado {
    pub const INICIAL: Estado = Estado { a: 0, b: 0 };
}

/// As seis ações possíveis do problema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acao {
    EncherA,
    EncherB,
    EsvaziarA,
    EsvaziarB,
    TransferirAparaB,
    TransferirBparaA,
}

impl Acao {
    /// Ordem determinística de geração dos sucessores (mesma usada no CLI).
    pub const TODAS: [Acao; 6] = [
        Acao::EncherA,
        Acao::EncherB,
        Acao::EsvaziarA,
        Acao::EsvaziarB,
        Acao::TransferirAparaB,
        Acao::TransferirBparaA,
    ];

    pub fn rotulo(self) -> &'static str {
        match self {
            Acao::EncherA => "Encher jarro A",
            Acao::EncherB => "Encher jarro B",
            Acao::EsvaziarA => "Esvaziar jarro A",
            Acao::EsvaziarB => "Esvaziar jarro B",
            Acao::TransferirAparaB => "Transferir A → B",
            Acao::TransferirBparaA => "Transferir B → A",
        }
    }

    pub fn curto(self) -> &'static str {
        match self {
            Acao::EncherA => "encher A",
            Acao::EncherB => "encher B",
            Acao::EsvaziarA => "esvaziar A",
            Acao::EsvaziarB => "esvaziar B",
            Acao::TransferirAparaB => "A → B",
            Acao::TransferirBparaA => "B → A",
        }
    }

    /// Aplica a ação a um estado. A quantidade transferida é limitada pelo
    /// espaço livre do destino e pelo conteúdo da origem.
    pub fn aplicar(self, e: Estado, cap_a: u32, cap_b: u32) -> Estado {
        let de_a_para_b = e.a.min(cap_b.saturating_sub(e.b));
        let de_b_para_a = e.b.min(cap_a.saturating_sub(e.a));
        match self {
            Acao::EncherA => Estado { a: cap_a, b: e.b },
            Acao::EncherB => Estado { a: e.a, b: cap_b },
            Acao::EsvaziarA => Estado { a: 0, b: e.b },
            Acao::EsvaziarB => Estado { a: e.a, b: 0 },
            Acao::TransferirAparaB => Estado {
                a: e.a - de_a_para_b,
                b: e.b + de_a_para_b,
            },
            Acao::TransferirBparaA => Estado {
                a: e.a + de_b_para_a,
                b: e.b - de_b_para_a,
            },
        }
    }
}

/// Onde a meta deve ser medida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alvo {
    Qualquer,
    JarroA,
    JarroB,
}

impl Alvo {
    pub fn rotulo(self) -> &'static str {
        match self {
            Alvo::Qualquer => "Qualquer jarro",
            Alvo::JarroA => "Jarro A",
            Alvo::JarroB => "Jarro B",
        }
    }
}

/// Estratégia de busca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algoritmo {
    Bfs,
    Dfs,
}

impl Algoritmo {
    pub fn rotulo(self) -> &'static str {
        match self {
            Algoritmo::Bfs => "Busca em largura (BFS)",
            Algoritmo::Dfs => "Busca em profundidade (DFS)",
        }
    }
}

/// Configuração do problema escolhida pelo usuário.
#[derive(Clone, Debug)]
pub struct Configuracao {
    pub capacidade_a: u32,
    pub capacidade_b: u32,
    pub meta: u32,
    pub alvo: Alvo,
    pub algoritmo: Algoritmo,
}

impl Configuracao {
    pub fn nova(capacidade_a: u32, capacidade_b: u32, meta: u32, alvo: Alvo, algoritmo: Algoritmo) -> Self {
        Configuracao {
            capacidade_a,
            capacidade_b,
            meta,
            alvo,
            algoritmo,
        }
    }

    /// Um estado é objetivo quando a quantidade pedida aparece no alvo.
    pub fn objetivo(&self, e: &Estado) -> bool {
        match self.alvo {
            Alvo::Qualquer => e.a == self.meta || e.b == self.meta,
            Alvo::JarroA => e.a == self.meta,
            Alvo::JarroB => e.b == self.meta,
        }
    }

    /// Valida a entrada e detecta metas impossíveis antes de buscar.
    ///
    /// Com dois jarros, só é possível medir múltiplos do MDC das capacidades;
    /// e a meta precisa caber no jarro escolhido.
    pub fn validar(&self) -> Result<(), String> {
        if self.capacidade_a == 0 || self.capacidade_b == 0 {
            return Err("As capacidades dos jarros precisam ser maiores que zero.".into());
        }
        if self.capacidade_a > 12 || self.capacidade_b > 12 {
            return Err("Use capacidades de no máximo 12 litros (limite da interface).".into());
        }

        let maximo = match self.alvo {
            Alvo::Qualquer => self.capacidade_a.max(self.capacidade_b),
            Alvo::JarroA => self.capacidade_a,
            Alvo::JarroB => self.capacidade_b,
        };
        if self.meta > maximo {
            return Err(format!(
                "A meta de {} L não cabe no jarro escolhido (limite de {} L).",
                self.meta, maximo
            ));
        }

        let divisor = mdc(self.capacidade_a, self.capacidade_b);
        if self.meta % divisor != 0 {
            return Err(format!(
                "Meta impossível: com jarros de {} L e {} L só é possível medir múltiplos de {} L.",
                self.capacidade_a, self.capacidade_b, divisor
            ));
        }
        Ok(())
    }
}

/// Máximo divisor comum (algoritmo de Euclides).
pub fn mdc(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// Nó da árvore de busca.
#[derive(Clone, Debug)]
pub struct No {
    pub id: usize,
    pub estado: Estado,
    pub pai: Option<usize>,
    pub acao: Option<Acao>,
    pub profundidade: u32,
    pub expandido: bool,
}

/// Situação de um sucessor gerado durante a expansão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Situacao {
    Novo,
    Repetido,
}

/// Sucessor gerado em uma expansão (usado no rastro da busca).
#[derive(Clone, Debug)]
pub struct SucessorTrace {
    pub estado: Estado,
    pub acao: Acao,
    pub situacao: Situacao,
    pub no: Option<usize>,
}

/// Um passo do rastro: qual nó foi expandido, o que gerou e a borda restante.
#[derive(Clone, Debug)]
pub struct Passo {
    pub expandido: usize,
    pub sucessores: Vec<SucessorTrace>,
    pub borda: Vec<usize>,
    pub total_expandidos: usize,
    pub objetivo: bool,
}

/// Estatísticas para o painel de desempenho.
#[derive(Clone, Debug)]
pub struct Estatisticas {
    pub nos_gerados: usize,
    pub estados_visitados: usize,
    pub estados_expandidos: usize,
    pub repetidos_evitados: usize,
    pub maior_borda: usize,
    pub profundidade_solucao: Option<usize>,
    pub duracao_micros: u128,
}

/// Resultado completo de uma execução da busca.
#[derive(Clone, Debug)]
pub struct Resultado {
    pub configuracao: Configuracao,
    pub nos: Vec<No>,
    pub passos: Vec<Passo>,
    /// Ids dos nós do estado inicial até o objetivo (vazio se não achou).
    pub caminho: Vec<usize>,
    pub encontrado: bool,
    pub expandidos: Vec<usize>,
    pub borda_final: Vec<usize>,
    pub estatisticas: Estatisticas,
}

impl Resultado {
    /// Sequência de ações do caminho (sem o estado inicial). Usada nos testes.
    #[cfg(test)]
    pub fn acoes_do_caminho(&self) -> Vec<Acao> {
        self.caminho
            .iter()
            .filter_map(|id| self.nos[*id].acao)
            .collect()
    }

    /// Profundidade da solução (número de ações).
    pub fn profundidade(&self) -> Option<usize> {
        if self.encontrado {
            Some(self.caminho.len().saturating_sub(1))
        } else {
            None
        }
    }
}

/// Executa a busca (BFS ou DFS) sobre o espaço de estados do problema.
pub fn resolver(config: &Configuracao) -> Resultado {
    let inicio = Instant::now();
    let cap_a = config.capacidade_a;
    let cap_b = config.capacidade_b;

    let mut nos: Vec<No> = vec![No {
        id: 0,
        estado: Estado::INICIAL,
        pai: None,
        acao: None,
        profundidade: 0,
        expandido: false,
    }];
    let mut visitados: HashSet<Estado> = HashSet::new();
    visitados.insert(Estado::INICIAL);

    // BFS: fila (FIFO). DFS: pilha (LIFO).
    let mut fila: VecDeque<usize> = VecDeque::new();
    let mut pilha: Vec<usize> = Vec::new();
    match config.algoritmo {
        Algoritmo::Bfs => fila.push_back(0),
        Algoritmo::Dfs => pilha.push(0),
    }

    let mut passos: Vec<Passo> = Vec::new();
    let mut expandidos: Vec<usize> = Vec::new();
    let mut repetidos_evitados: usize = 0;
    let mut maior_borda: usize = 1;
    let mut objetivo: Option<usize> = None;

    loop {
        let candidato = match config.algoritmo {
            Algoritmo::Bfs => fila.pop_front(),
            Algoritmo::Dfs => pilha.pop(),
        };
        let Some(id) = candidato else {
            break;
        };

        nos[id].expandido = true;
        expandidos.push(id);

        let estado = nos[id].estado;
        let eh_objetivo = config.objetivo(&estado);
        let mut sucessores: Vec<SucessorTrace> = Vec::new();

        if !eh_objetivo {
            let profundidade_pai = nos[id].profundidade;
            let mut novos: Vec<usize> = Vec::new();

            for acao in Acao::TODAS {
                let proximo = acao.aplicar(estado, cap_a, cap_b);

                // Ação sem efeito (ex.: transferir de jarro vazio): não gera nó.
                if proximo == estado {
                    continue;
                }

                if visitados.contains(&proximo) {
                    repetidos_evitados += 1;
                    sucessores.push(SucessorTrace {
                        estado: proximo,
                        acao,
                        situacao: Situacao::Repetido,
                        no: None,
                    });
                    continue;
                }

                visitados.insert(proximo);
                let filho_id = nos.len();
                nos.push(No {
                    id: filho_id,
                    estado: proximo,
                    pai: Some(id),
                    acao: Some(acao),
                    profundidade: profundidade_pai + 1,
                    expandido: false,
                });
                novos.push(filho_id);
                sucessores.push(SucessorTrace {
                    estado: proximo,
                    acao,
                    situacao: Situacao::Novo,
                    no: Some(filho_id),
                });
            }

            match config.algoritmo {
                Algoritmo::Bfs => {
                    for filho in novos {
                        fila.push_back(filho);
                    }
                }
                Algoritmo::Dfs => {
                    // Empilha em ordem inversa para que a primeira ação
                    // (encher A) seja a primeira a sair da pilha.
                    for filho in novos.into_iter().rev() {
                        pilha.push(filho);
                    }
                }
            }
        } else {
            objetivo = Some(id);
        }

        let borda: Vec<usize> = match config.algoritmo {
            Algoritmo::Bfs => fila.iter().copied().collect(),
            Algoritmo::Dfs => pilha.iter().copied().collect(),
        };
        maior_borda = maior_borda.max(borda.len());

        passos.push(Passo {
            expandido: id,
            sucessores,
            borda,
            total_expandidos: expandidos.len(),
            objetivo: eh_objetivo,
        });

        if eh_objetivo {
            break;
        }
    }

    // Reconstrói o caminho seguindo os ponteiros de "pai".
    let caminho = match objetivo {
        Some(mut id) => {
            let mut caminho = vec![id];
            while let Some(pai) = nos[id].pai {
                id = pai;
                caminho.push(id);
            }
            caminho.reverse();
            caminho
        }
        None => Vec::new(),
    };

    let borda_final: Vec<usize> = match config.algoritmo {
        Algoritmo::Bfs => fila.iter().copied().collect(),
        Algoritmo::Dfs => pilha.iter().copied().collect(),
    };

    let profundidade = if objetivo.is_some() {
        Some(caminho.len().saturating_sub(1))
    } else {
        None
    };

    Resultado {
        configuracao: config.clone(),
        estatisticas: Estatisticas {
            nos_gerados: nos.len(),
            estados_visitados: visitados.len(),
            estados_expandidos: expandidos.len(),
            repetidos_evitados,
            maior_borda,
            profundidade_solucao: profundidade,
            duracao_micros: inicio.elapsed().as_micros(),
        },
        nos,
        passos,
        caminho,
        encontrado: objetivo.is_some(),
        expandidos,
        borda_final,
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn configuracao_padrao() -> Configuracao {
        Configuracao::nova(5, 3, 4, Alvo::Qualquer, Algoritmo::Bfs)
    }

    /// Confere que cada ação do caminho leva de fato de um estado ao seguinte.
    fn caminho_e_legal(resultado: &Resultado) {
        let acoes = resultado.acoes_do_caminho();
        for (i, acao) in acoes.iter().enumerate() {
            let origem = resultado.nos[resultado.caminho[i]].estado;
            let destino = resultado.nos[resultado.caminho[i + 1]].estado;
            let calculado = acao.aplicar(
                origem,
                resultado.configuracao.capacidade_a,
                resultado.configuracao.capacidade_b,
            );
            assert_eq!(calculado, destino, "transição {i} inválida para {:?}", acao);
        }
    }

    #[test]
    fn bfs_resolve_5_3_para_4() {
        let r = resolver(&configuracao_padrao());
        assert!(r.encontrado);
        assert_eq!(r.profundidade(), Some(6), "BFS deve achar o caminho mínimo (6 passos)");
        assert_eq!(r.nos[r.caminho[0]].estado, Estado::INICIAL);
        let ultimo = r.nos[*r.caminho.last().unwrap()].estado;
        assert_eq!(ultimo.a, 4);
        assert_eq!(r.nos.last().unwrap().id, r.estatisticas.nos_gerados - 1);
        caminho_e_legal(&r);
    }

    #[test]
    fn bfs_nao_expande_nos_ja_visitados() {
        let r = resolver(&configuracao_padrao());
        let mut vistos = HashSet::new();
        for id in &r.expandidos {
            let estado = r.nos[*id].estado;
            assert!(vistos.insert(estado), "estado expandido duas vezes: {:?}", estado);
        }
        // Toda transição do caminho existe no rastro como sucessor "novo".
        assert!(r.estatisticas.repetidos_evitados > 0);
    }

    #[test]
    fn dfs_encontra_solucao_valida() {
        let mut config = configuracao_padrao();
        config.algoritmo = Algoritmo::Dfs;
        let r = resolver(&config);
        assert!(r.encontrado);
        assert!(r.profundidade().unwrap() >= 6);
        caminho_e_legal(&r);
    }

    #[test]
    fn meta_ja_satisfeita_no_inicio() {
        let config = Configuracao::nova(5, 3, 0, Alvo::Qualquer, Algoritmo::Bfs);
        let r = resolver(&config);
        assert!(r.encontrado);
        assert_eq!(r.profundidade(), Some(0));
        assert_eq!(r.caminho.len(), 1);
        assert_eq!(r.passos.len(), 1);
    }

    #[test]
    fn jarro_unico_para_meta() {
        // Com jarros 3 e 3, medir 3 L: basta encher um jarro (1 passo).
        let config = Configuracao::nova(3, 3, 3, Alvo::JarroB, Algoritmo::Bfs);
        let r = resolver(&config);
        assert!(r.encontrado);
        assert_eq!(r.profundidade(), Some(1));
        assert_eq!(r.acoes_do_caminho(), vec![Acao::EncherB]);
    }

    #[test]
    fn meta_impossivel_por_mdc_e_rejeitada() {
        let config = Configuracao::nova(4, 6, 3, Alvo::Qualquer, Algoritmo::Bfs);
        let erro = config.validar().unwrap_err();
        assert!(erro.contains("múltiplos de 2"));
    }

    #[test]
    fn meta_maior_que_capacidade_e_rejeitada() {
        let config = Configuracao::nova(4, 3, 8, Alvo::Qualquer, Algoritmo::Bfs);
        assert!(config.validar().is_err());
    }

    #[test]
    fn capacidades_invalidas_sao_rejeitadas() {
        for (a, b) in [(0, 3), (5, 0), (13, 3)] {
            let config = Configuracao::nova(a, b, 1, Alvo::Qualquer, Algoritmo::Bfs);
            assert!(config.validar().is_err(), "{a}/{b} deveria ser inválido");
        }
    }

    #[test]
    fn todos_os_estados_do_caminho_cabem_nas_capacidades() {
        for meta in 1..=4 {
            let config = Configuracao::nova(5, 3, meta, Alvo::Qualquer, Algoritmo::Bfs);
            let r = resolver(&config);
            assert!(r.encontrado);
            for id in &r.caminho {
                let e = r.nos[*id].estado;
                assert!(e.a <= config.capacidade_a && e.b <= config.capacidade_b);
            }
        }
    }
}
