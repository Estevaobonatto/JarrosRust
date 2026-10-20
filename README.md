# Problema dos Jarros — busca em espaço de estados

Atividade de **busca em espaço de estados** aplicada ao Problema dos Jarros: encontrar uma
sequência de ações — encher, esvaziar e transferir água entre dois jarros — que meça uma
quantidade exata.

A entrega foi implementada em **Rust**, com duas formas de uso:

- **Interface web local** (foco do trabalho): jarros animados, árvore de busca, borda,
  ordem de expansão e reprodução passo a passo da solução e da busca;
- **Modo terminal**: imprime a ordem de expansão, o caminho e as estatísticas, no formato
  clássico do relatório.

## Como executar

### Requisitos

- **Rust 1.70 ou superior** com `cargo` — confira com `cargo --version`;
- sistema operacional Windows, Linux ou macOS e um navegador atualizado.

O projeto **não usa nenhuma dependência externa**: apenas a biblioteca padrão do Rust.
Não é necessário internet para compilar nem para usar a interface.

### Interface web (recomendado)

```bash
cd "Entrega Rust"
cargo run --release
```

O terminal exibe o endereço local (por padrão `http://127.0.0.1:8080`) e o navegador abre
automaticamente. A interface é servida pelo próprio executável; para encerrar, pressione
`Ctrl+C` no terminal.

Opções úteis:

```bash
cargo run --release -- --no-open        # não abre o navegador automaticamente
cargo run --release -- --port 9000      # usa outra porta
```

> **Sem Rust instalado?** No Windows, execute o `jarros.exe` que acompanha a entrega.
> Ele contém a interface embutida e abre exatamente igual.

### Linux

O `jarros.exe` incluído é **somente para Windows**. No Linux, o projeto compila normalmente
com o Rust do sistema e gera um binário Linux equivalente.

```bash
# 1) instalar o Rust (se ainda não tiver) — sem precisar de sudo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo --version

# 2) entrar na pasta do projeto e executar
cd "Entrega Rust"
cargo run --release
```

Na primeira execução o Cargo compila tudo (não há dependências externas) e sobe a
interface em `http://127.0.0.1:8080`; se o navegador não abrir, acesse a URL mostrada no
terminal. `Ctrl+C` encerra o servidor.

Se a compilação falhar por falta de linker (`cc: not found`), instale as ferramentas de
compilação da sua distro:

```bash
# Debian/Ubuntu
sudo apt install build-essential
# Fedora
sudo dnf group install "Development Tools"
# Arch
sudo pacman -S base-devel
```

Para gerar o executável Linux e rodar sem o Cargo:

```bash
cargo build --release
./target/release/jarros
```

Em um servidor **sem interface gráfica**, use `--no-open`; para acessar de outra máquina,
faça um túnel SSH (o servidor escuta só em `127.0.0.1`):

```bash
cargo run --release -- --no-open --port 8080
# em outra máquina: ssh -L 8080:127.0.0.1:8080 usuario@servidor
# depois abra http://127.0.0.1:8080 no navegador local
```

O modo terminal também funciona igual:

```bash
cargo run --release -- --cli
cargo run --release -- --cli --cap-a 4 --cap-b 5 --meta 3
```


### Modo terminal

```bash
cargo run --release -- --cli
```

O modo terminal aceita os mesmos parâmetros do problema:

```bash
cargo run --release -- --cli --cap-a 5 --cap-b 3 --meta 4
cargo run --release -- --cli --algoritmo dfs --cap-a 4 --cap-b 3 --meta 2
cargo run --release -- --cli --cap-a 5 --cap-b 3 --meta 4 --alvo b
```

| Opção | Padrão | Descrição |
|---|---|---|
| `--cli` | desligado | Executa no terminal em vez de subir o servidor |
| `--port N` | `8080` | Porta do servidor local (tenta as 20 seguintes se estiver ocupada) |
| `--no-open` | desligado | Não abre o navegador automaticamente |
| `--cap-a N` | `5` | Capacidade do jarro A, em litros |
| `--cap-b N` | `3` | Capacidade do jarro B, em litros |
| `--meta N` | `4` | Quantidade a medir, em litros |
| `--alvo qualquer\|a\|b` | `qualquer` | Onde a meta deve ser medida |
| `--algoritmo bfs\|dfs` | `bfs` | Estratégia de busca |
| `--help` | — | Mostra a ajuda |

---

## O que a interface mostra

A tela foi mantida enxuta: configuração, jarros e árvore.

- **Configuração**: capacidades dos dois jarros (1 a 12 L), quantidade da meta, jarro onde
  a meta deve ser medida e algoritmo (BFS ou DFS).
- **Estação de medição**: os dois jarros desenhados com escala em litros, linha tracejada
  na altura da meta e água animada conforme o estado. Abaixo, a leitura exata de cada
  jarro e o estado atual (passo/expansão, estado `(a, b)` e ação executada).
- **Controles**: início, anterior, reproduzir/pausar, próximo e fim, alternando entre:
  - **Solução**: percorre o caminho encontrado, passo a passo;
  - **Exploração**: percorre a ordem real de expansão da busca (a árvore vai sendo
    revelada conforme os nós são gerados).
  Atalhos de teclado: `←` / `→` navegam, `espaço` reproduz, `Home` / `End` vão aos extremos.
- **Árvore de busca**: nós em cápsula, com:
  - **azul** = estado expandido; **tracejado azul** = ainda na borda;
  - **verde** = caminho da solução; **verde preenchido** = objetivo;
  - **anel escuro** = estado atual;
  - números pretos = **ordem de expansão**; rótulos nas arestas = ação executada;
  - zoom (+/−) e “Ajustar” para enquadrar a árvore.
- **Resumo discreto** abaixo da árvore: profundidade da solução, nós gerados e
  transições repetidas evitadas.
- **Copiar relatório**: gera a saída em texto no mesmo formato do modo CLI (ordem de
  expansão, caminho e estatísticas).
- **Mensagens de erro** para entradas inválidas e para metas impossíveis, explicando o
  motivo (por exemplo, com jarros de 4 L e 6 L só é possível medir múltiplos de 2 L).

---

## Como o problema foi modelado

### Estado

Um estado é o par `(a, b)` com a quantidade de água, em litros, em cada jarro.
O estado inicial é sempre `(0, 0)` (`Estado::INICIAL` em `src/solver.rs`).

### Ações (sucessores)

As seis ações clássicas são aplicadas a cada estado, respeitando as capacidades:

| Ação | Efeito |
|---|---|
| Encher jarro A | `(a, b) → (capA, b)` |
| Encher jarro B | `(a, b) → (a, capB)` |
| Esvaziar jarro A | `(a, b) → (0, b)` |
| Esvaziar jarro B | `(a, b) → (a, 0)` |
| Transferir A → B | passa `min(a, capB − b)` litros de A para B |
| Transferir B → A | passa `min(b, capA − a)` litros de B para A |

Ações que não mudam o estado (por exemplo, transferir de um jarro vazio) não geram nós.

### Objetivo

O estado é objetivo quando a meta aparece no jarro escolhido (`qualquer`, `A` ou `B`).

### Busca

- **BFS (busca em largura)**: usa uma **fila** (`VecDeque`, FIFO). Expande primeiro os
  estados mais rasos e, por isso, **garante o caminho mínimo** em número de ações.
- **DFS (busca em profundidade)**: usa uma **pilha** (`Vec`, LIFO). Atinge a solução
  seguindo um ramo por vez, mas **não garante** o caminho mais curto.

Em ambas, um `HashSet` de estados visitados evita ciclos. Para cada expansão, o programa
registra:

- qual nó foi expandido;
- os sucessores gerados naquela expansão, com a situação de cada um (`novo` ou
  `repetido`, descartado);
- a **borda** resultante (fila ou pilha, conforme o algoritmo);
- o total de nós já expandidos.

O caminho da solução é reconstruído seguindo os **ponteiros de pai** de cada nó, do
objetivo de volta até o estado inicial. A interface usa exatamente esses dados.

### Entradas impossíveis

Antes de buscar, o programa valida a entrada e evita buscas inúteis:

- as capacidades devem ser maiores que zero (limite de visualização: 12 L);
- a meta precisa caber no jarro escolhido;
- pelo teorema do MDC, com dois jarros só é possível medir múltiplos do MDC das
  capacidades. Se `meta % mdc(capA, capB) != 0`, a meta é impossível.

O MDC é calculado com o algoritmo de Euclides (`mdc` em `src/solver.rs`).

### Complexidade

O espaço de estados tem no máximo `(capA + 1) × (capB + 1)` estados, cada um com até 6
arestas. A busca visita cada estado no máximo uma vez: `O(V + E)`, com `V ≤ 169` e
`E ≤ 6V` para o limite da interface. O tempo medido fica na casa de centésimos de
milissegundo.

---

## Estrutura do projeto

```
Entrega Rust/
├── Cargo.toml                # projeto Rust (sem dependências externas)
├── jarros.exe                # build pronto para Windows (interface embutida)
├── README.md                 # este arquivo
├── saida-cli-exemplo.txt     # exemplo de saída do modo terminal
├── src/
│   ├── main.rs               # modo CLI + servidor HTTP local + API JSON
│   ├── solver.rs             # busca BFS/DFS, validações, rastro e testes
│   └── web/
│       ├── index.html        # estrutura da interface
│       ├── style.css         # stylesheet (sem frameworks)
│       └── app.js            # interação: fetch da API, jarros, árvore e controles
└── screenshots/              # capturas usadas neste README
```

Detalhes de implementação:

- O servidor HTTP foi escrito com `std::net::TcpListener`, sem frameworks e sem
  dependências. Ele responde à página, aos arquivos estáticos e ao endpoint
  `GET /api/solve`;
- os arquivos da interface são embutidos no binário com `include_str!`, então o
  executável funciona sozinho;
- a API devolve um JSON com a configuração, os nós da árvore, a ordem de expansão, a
  borda, os sucessores de cada expansão, o caminho da solução e as estatísticas. O
  front-end usa esses dados para desenhar tudo sem recalcular a busca.

---

## Testes

```bash
cd "Entrega Rust"
cargo test
```

O projeto tem **9 testes automatizados** (`src/solver.rs`), todos passando:

- BFS 5 L / 3 L com meta 4 L encontra o caminho mínimo de 6 passos;
- nenhum estado é expandido duas vezes;
- DFS encontra uma solução válida;
- meta 0 é satisfeita no estado inicial;
- meta igual à capacidade de um jarro é alcançada em 1 passo;
- metas impossíveis pelo MDC e acima da capacidade são rejeitadas;
- capacidades inválidas são rejeitadas;
- todos os estados do caminho respeitam as capacidades;
- as transições do caminho são legais para todas as metas testadas (cada ação aplicada
  ao estado de origem leva ao estado seguinte).

---

## Exemplo de saída no modo terminal

Saída de `cargo run --release -- --cli` (também disponível em
[`saida-cli-exemplo.txt`](saida-cli-exemplo.txt)):

```
============================================================
 PROBLEMA DOS JARROS — BUSCA EM ESPAÇO DE ESTADOS
 Algoritmo: Busca em largura (BFS)
 Jarro A: 5 L · Jarro B: 3 L · Meta: 4 L (qualquer jarro)
============================================================

ORDEM DE EXPANSÃO:
  0. ( 0, 0)  Início                 borda: 2
  1. ( 5, 0)  Encher jarro A         borda: 3
  2. ( 0, 3)  Encher jarro B         borda: 3
  3. ( 5, 3)  Encher jarro B         borda: 2
  4. ( 2, 3)  Transferir A → B       borda: 2
  5. ( 3, 0)  Transferir B → A       borda: 2
  6. ( 2, 0)  Esvaziar jarro B       borda: 2
  7. ( 3, 3)  Encher jarro B         borda: 2
  8. ( 0, 2)  Transferir A → B       borda: 2
  9. ( 5, 1)  Transferir B → A       borda: 2
 10. ( 5, 2)  Encher jarro A         borda: 2
 11. ( 0, 1)  Esvaziar jarro A       borda: 2
 12. ( 4, 3)  Transferir A → B       borda: 1

OBJETIVO ENCONTRADO

CAMINHO DA SOLUÇÃO (6 passos):
  0. ( 0, 0)  Início
  1. ( 5, 0)  Encher jarro A
  2. ( 2, 3)  Transferir A → B
  3. ( 2, 0)  Esvaziar jarro B
  4. ( 0, 2)  Transferir A → B
  5. ( 5, 2)  Encher jarro A
  6. ( 4, 3)  Transferir A → B

ESTATÍSTICAS
  nós gerados:        14
  estados visitados:  14
  nós expandidos:     13
  repetidos evitados: 29
  maior borda:        3
  profundidade:       6
  tempo:              ~0,04 ms
```

---

## Roteiro rápido para apresentar

1. Abra a interface e mostre o estado final da solução (4 L no jarro A com 5 L e 3 L).
2. Volte ao início com `⏮` e reproduza o caminho no modo **Solução**, explicando cada ação.
3. Troque para **Exploração** e mostre a fila (BFS) crescendo: borda, ordem de expansão e
   nós repetidos descartados.
4. Troque o algoritmo para **DFS** e compare: o caminho encontrado não é garantidamente o
   mínimo; a quantidade de nós gerados também muda.
5. Teste uma meta impossível (por exemplo, 4 L e 6 L para medir 3 L) e mostre a mensagem.
6. Rode o modo CLI para apresentar a saída em texto, útil para o relatório.

---

## Checklist antes de enviar

- [ ] Rodar `cargo test` e conferir os 9 testes passando;
- [ ] Rodar `cargo run --release` e navegar pela interface;
- [ ] (Opcional) Compactar a pasta `Entrega Rust` em um `.zip` para entregar.
