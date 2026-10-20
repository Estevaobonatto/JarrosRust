/* ---------------------------------------------------------------------------
   Problema dos Jarros — interface.
   Consome a API local (/api/solve) e desenha os jarros e a árvore de busca.
   Nenhuma dependência externa.
--------------------------------------------------------------------------- */

(() => {
  "use strict";

  // ----------------------------------------------------------------- refs
  const el = {
    form: document.getElementById("form-config"),
    capA: document.getElementById("cap-a"),
    capB: document.getElementById("cap-b"),
    meta: document.getElementById("meta"),
    alvo: document.getElementById("alvo"),
    botaoBuscar: document.getElementById("botao-buscar"),
    mensagemErro: document.getElementById("mensagem-erro"),
    jarroA: document.getElementById("jarro-a"),
    jarroB: document.getElementById("jarro-b"),
    leituras: {
      a: document.getElementById("leitura-a"),
      b: document.getElementById("leitura-b"),
    },
    estadoRotulo: document.getElementById("estado-rotulo"),
    estadoEstado: document.getElementById("estado-estado"),
    estadoAcao: document.getElementById("estado-acao"),
    btnInicio: document.getElementById("btn-inicio"),
    btnAnterior: document.getElementById("btn-anterior"),
    btnReproduzir: document.getElementById("btn-reproduzir"),
    textoReproduzir: document.getElementById("texto-reproduzir"),
    btnProximo: document.getElementById("btn-proximo"),
    btnFim: document.getElementById("btn-fim"),
    arvore: document.getElementById("arvore"),
    arvoreScroll: document.getElementById("arvore-scroll"),
    arvoreVazia: document.getElementById("arvore-vazia"),
    zoomNivel: document.getElementById("zoom-nivel"),
    btnAjustar: document.getElementById("btn-ajustar"),
    btnZoomMais: document.getElementById("btn-zoom-mais"),
    btnZoomMenos: document.getElementById("btn-zoom-menos"),
    resumoBusca: document.getElementById("resumo-busca"),
    btnRelatorio: document.getElementById("btn-relatorio"),
  };

  const DIMENSOES = {
    larguraNo: 66,
    alturaNo: 32,
    larguraSlot: 94,
    alturaNivel: 78,
    margemX: 34,
    margemTopo: 38,
    margemBase: 26,
    alturaInternaJarro: 154,
  };

  const estado = {
    resultado: null,
    modo: "solucao",
    indice: 0,
    reproduzindo: false,
    temporizador: null,
    velocidade: 850,
    zoom: 1,
    zoomManual: false,
    arvore: null,
    arvoreRefs: null,
    maxGerado: [],
    jarros: null,
    capacidades: { a: 5, b: 3 },
  };

  const movimentoReduzido = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  // -------------------------------------------------------------- utilitários
  function escaparHtml(texto) {
    return String(texto)
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;");
  }

  function limitar(valor, minimo, maximo) {
    return Math.min(Math.max(valor, minimo), maximo);
  }

  function numeroInteiro(valor, padrao) {
    const numero = Number.parseInt(valor, 10);
    return Number.isFinite(numero) ? numero : padrao;
  }

  // ------------------------------------------------------------- lógica de UI
  function tamanhoLinha() {
    if (!estado.resultado) return 0;
    return estado.modo === "solucao"
      ? estado.resultado.caminho.length
      : estado.resultado.passos.length;
  }

  function pausar() {
    if (estado.temporizador !== null) {
      window.clearInterval(estado.temporizador);
      estado.temporizador = null;
    }
    if (estado.reproduzindo) {
      estado.reproduzindo = false;
      renderizarControles();
    }
  }

  function tick() {
    if (!estado.reproduzindo) return;
    const total = tamanhoLinha();
    if (estado.indice >= total - 1) {
      pausar();
      return;
    }
    estado.indice += 1;
    renderizarTudo();
  }

  function iniciarTemporizador() {
    if (estado.temporizador !== null) window.clearInterval(estado.temporizador);
    estado.temporizador = window.setInterval(tick, estado.velocidade);
  }

  function alternarReproducao() {
    if (!estado.resultado) return;
    const total = tamanhoLinha();
    if (total <= 1) return;

    if (estado.reproduzindo) {
      pausar();
      return;
    }

    if (estado.indice >= total - 1) {
      estado.indice = 0;
      renderizarTudo();
    }

    estado.reproduzindo = true;
    renderizarControles();
    iniciarTemporizador();
  }

  function definirIndice(novoIndice) {
    if (!estado.resultado) return;
    const total = tamanhoLinha();
    if (total === 0) return;
    estado.indice = limitar(novoIndice, 0, total - 1);
    renderizarTudo();
  }

  function definirModo(modo) {
    if (!estado.resultado || estado.modo === modo) return;
    pausar();
    estado.modo = modo;
    const total = tamanhoLinha();
    estado.indice = modo === "solucao" ? Math.max(0, total - 1) : 0;
    renderizarTudo();
  }

  // --------------------------------------------------------------- mensagens
  function mostrarErro(mensagem) {
    el.mensagemErro.textContent = mensagem;
    el.mensagemErro.hidden = false;
  }

  function esconderErro() {
    el.mensagemErro.hidden = true;
    el.mensagemErro.textContent = "";
  }

  // ------------------------------------------------------------------ jarros
  function svgJarro(id, capacidade, metaVisivel, meta) {
    const interiorX = 38;
    const interiorLargura = 74;
    const interiorY = 26;
    const interiorAltura = DIMENSOES.alturaInternaJarro;
    const fundo = interiorY + interiorAltura;

    const marcacoes = [];
    for (let nivel = 1; nivel <= capacidade; nivel += 1) {
      const y = fundo - (nivel / capacidade) * interiorAltura;
      const comprimento = capacidade <= 8 ? 10 : 7;
      const principal = nivel % 5 === 0;
      marcacoes.push(
        `<line class="marcacao${principal ? " marcacao-principal" : ""}" x1="${interiorX}" x2="${
          interiorX + (principal ? comprimento + 4 : comprimento)
        }" y1="${y.toFixed(2)}" y2="${y.toFixed(2)}"/>`
      );
      if (capacidade <= 8 || nivel % 2 === 0 || nivel === capacidade) {
        marcacoes.push(
          `<text class="marcacao-texto" x="${interiorX - 8}" y="${(y + 3).toFixed(2)}" text-anchor="end">${nivel}</text>`
        );
      }
    }

    let linhaMeta = "";
    if (metaVisivel) {
      const y = fundo - (meta / capacidade) * interiorAltura;
      linhaMeta =
        `<line class="linha-meta" x1="${interiorX}" x2="${interiorX + interiorLargura}" y1="${y.toFixed(2)}" y2="${y.toFixed(2)}"/>` +
        `<text class="meta-texto" x="${interiorX + interiorLargura - 2}" y="${(y - 3).toFixed(2)}" text-anchor="end">meta ${meta} L</text>`;
    }

    return `
      <svg class="jarros-svg" viewBox="0 0 150 210" role="img" aria-label="Jarro ${id.toUpperCase()} de ${capacidade} litros">
        <defs>
          <clipPath id="recorte-${id}">
            <path d="M${interiorX} ${interiorY} h${interiorLargura} v${interiorAltura - 12} a12 12 0 0 1 -12 12 h-${interiorLargura - 24} a12 12 0 0 1 -12 -12 z"/>
          </clipPath>
          <linearGradient id="gradiente-${id}" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stop-color="#86c6d9"/>
            <stop offset="0.5" stop-color="#4694ae"/>
            <stop offset="1" stop-color="#1d617f"/>
          </linearGradient>
        </defs>
        <ellipse class="sombra" cx="75" cy="200" rx="42" ry="5"/>
        <path class="vidro" d="M32 16 h86 v152 a16 16 0 0 1 -16 16 h-54 a16 16 0 0 1 -16 -16 z"/>
        <g clip-path="url(#recorte-${id})">
          <rect class="agua" x="${interiorX}" y="${interiorY}" width="${interiorLargura}" height="${interiorAltura}"
            fill="url(#gradiente-${id})" style="transform: scaleY(0)"/>
          <line class="superficie" x1="${interiorX}" x2="${interiorX + interiorLargura}" y1="${fundo}" y2="${fundo}"
            style="transform: translateY(0)"/>
          <rect class="brilho" x="${interiorX + 5}" y="${interiorY + 6}" width="9" height="${
      interiorAltura - 22
    }" rx="4"/>
        </g>
        ${marcacoes.join("")}
        <path class="contorno" d="M32 16 h86 v152 a16 16 0 0 1 -16 16 h-54 a16 16 0 0 1 -16 -16 z"/>
        <line class="borda-superior" x1="24" y1="16" x2="126" y2="16"/>
        ${linhaMeta}
      </svg>`;
  }

  function montarJarros(config) {
    const metaA =
      config.meta > 0 &&
      config.meta <= config.capacidadeA &&
      (config.alvo === "qualquer" || config.alvo === "a");
    const metaB =
      config.meta > 0 &&
      config.meta <= config.capacidadeB &&
      (config.alvo === "qualquer" || config.alvo === "b");

    el.jarroA.innerHTML = svgJarro("a", config.capacidadeA, metaA, config.meta);
    el.jarroB.innerHTML = svgJarro("b", config.capacidadeB, metaB, config.meta);

    estado.capacidades = { a: config.capacidadeA, b: config.capacidadeB };
    estado.jarros = {
      a: {
        agua: el.jarroA.querySelector(".agua"),
        superficie: el.jarroA.querySelector(".superficie"),
      },
      b: {
        agua: el.jarroB.querySelector(".agua"),
        superficie: el.jarroB.querySelector(".superficie"),
      },
    };
    atualizarJarros({ a: 0, b: 0 });
  }

  function atualizarJarros(niveis) {
    if (!estado.jarros) return;
    for (const chave of ["a", "b"]) {
      const jarro = estado.jarros[chave];
      const capacidade = estado.capacidades[chave];
      const valor = niveis[chave] ?? 0;
      if (!jarro || !jarro.agua) continue;
      const fracao = capacidade > 0 ? limitar(valor / capacidade, 0, 1) : 0;
      jarro.agua.style.transform = `scaleY(${fracao.toFixed(4)})`;
      jarro.superficie.style.transform = `translateY(${(
        -fracao * DIMENSOES.alturaInternaJarro
      ).toFixed(2)}px)`;
      jarro.superficie.style.opacity = fracao > 0 ? "0.95" : "0";
      el.leituras[chave].innerHTML = `${valor}<small>/ ${capacidade} L</small>`;
    }
  }

  // ------------------------------------------------------------------ estado
  function renderizarEstadoEJarros() {
    const resultado = estado.resultado;
    if (!resultado) return;

    if (estado.modo === "solucao") {
      const id = resultado.caminho[estado.indice];
      if (id === undefined) return;
      const no = resultado.nos[id];
      atualizarJarros(no);
      const total = Math.max(0, resultado.caminho.length - 1);
      el.estadoRotulo.textContent =
        total === 0 ? "Estado inicial" : `Passo ${estado.indice} de ${total}`;
      el.estadoEstado.textContent = `(${no.a}, ${no.b})`;
      el.estadoAcao.textContent = no.acaoRotulo ?? "Estado inicial";
    } else {
      const passo = resultado.passos[estado.indice];
      if (!passo) return;
      const no = resultado.nos[passo.expandido];
      atualizarJarros(no);
      el.estadoRotulo.textContent = `Expansão ${estado.indice + 1} de ${resultado.passos.length}`;
      el.estadoEstado.textContent = `(${no.a}, ${no.b})`;
      el.estadoAcao.textContent = passo.objetivo
        ? "Objetivo encontrado"
        : no.acaoRotulo ?? "Estado inicial";
    }
  }

  // ------------------------------------------------------------------ árvore
  function calcularLayout(nos) {
    const filhos = new Map();
    for (const no of nos) {
      if (no.pai === null) continue;
      if (!filhos.has(no.pai)) filhos.set(no.pai, []);
      filhos.get(no.pai).push(no.id);
    }

    const posicoes = new Map();
    let proximoSlot = 0;
    let maiorProfundidade = 0;

    const visitar = (id) => {
      const no = nos[id];
      if (no.profundidade > maiorProfundidade) maiorProfundidade = no.profundidade;
      const lista = filhos.get(id) ?? [];
      if (lista.length === 0) {
        posicoes.set(id, proximoSlot);
        proximoSlot += 1;
        return;
      }
      for (const filho of lista) visitar(filho);
      const primeiro = posicoes.get(lista[0]);
      const ultimo = posicoes.get(lista[lista.length - 1]);
      posicoes.set(id, (primeiro + ultimo) / 2);
    };

    visitar(0);
    return { posicoes, maiorProfundidade, totalSlots: Math.max(1, proximoSlot) };
  }

  function montarArvore(resultado) {
    const svg = el.arvore;
    if (!resultado) {
      svg.innerHTML = "";
      return;
    }

    const layout = calcularLayout(resultado.nos);
    const largura =
      DIMENSOES.margemX + layout.totalSlots * DIMENSOES.larguraSlot + 18;
    const altura =
      DIMENSOES.margemTopo +
      layout.maiorProfundidade * DIMENSOES.alturaNivel +
      DIMENSOES.alturaNo +
      DIMENSOES.margemBase;

    const x = (id) =>
      DIMENSOES.margemX +
      layout.posicoes.get(id) * DIMENSOES.larguraSlot +
      DIMENSOES.larguraSlot / 2;
    const y = (id) => DIMENSOES.margemTopo + resultado.nos[id].profundidade * DIMENSOES.alturaNivel;

    // Ordem de expansão (1-based) para os badges pretos, como no design de referência.
    const ordemExpansao = new Map();
    resultado.expandidos.forEach((id, indice) => ordemExpansao.set(id, indice + 1));
    const objetivoId = resultado.caminho.length
      ? resultado.caminho[resultado.caminho.length - 1]
      : null;

    const partes = [];

    for (const no of resultado.nos) {
      if (no.pai === null) continue;
      const x1 = x(no.pai);
      const y1 = y(no.pai) + DIMENSOES.alturaNo;
      const x2 = x(no.id);
      const y2 = y(no.id);
      const meioX = (x1 + x2) / 2;
      const meioY = (y1 + y2) / 2;
      partes.push(
        `<g class="grupo-aresta" data-id="${no.id}">` +
          `<line class="aresta" x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}"/>` +
          `<g class="rotulo-aresta" transform="translate(${meioX}, ${meioY})">` +
          `<text text-anchor="middle" y="3">${escaparHtml(no.acaoCurta)}</text>` +
          `</g>` +
          `</g>`
      );
    }

    for (const no of resultado.nos) {
      const ehObjetivo = no.id === objetivoId;
      const ordem = ordemExpansao.get(no.id);
      const distintivo =
        ordem !== undefined && !ehObjetivo
          ? `<g class="distintivo" transform="translate(${-DIMENSOES.larguraNo / 2 + 1}, 1)">` +
            `<circle r="8"/>` +
            `<text y="3.4" text-anchor="middle">${ordem}</text>` +
            `</g>`
          : "";
      const dica = `(${no.a}, ${no.b})${no.acaoRotulo ? ` — ${escaparHtml(no.acaoRotulo)}` : ""} · profundidade ${
        no.profundidade
      }${ordem !== undefined ? ` · expansão ${ordem}` : " · na borda"}`;
      partes.push(
        `<g class="no" data-id="${no.id}" transform="translate(${x(no.id)}, ${y(no.id)})">` +
          `<title>${escaparHtml(dica)}</title>` +
          `<rect x="${-DIMENSOES.larguraNo / 2}" y="0" width="${DIMENSOES.larguraNo}" height="${
          DIMENSOES.alturaNo
        }" rx="16"/>` +
          `<rect class="selecao" x="${-DIMENSOES.larguraNo / 2 - 3}" y="-3" width="${
          DIMENSOES.larguraNo + 6
        }" height="${DIMENSOES.alturaNo + 6}" rx="19"/>` +
          `<text class="no-estado" x="0" y="${DIMENSOES.alturaNo / 2 + 4.3}">(${no.a},${no.b})</text>` +
          distintivo +
          `</g>`
      );
    }

    svg.innerHTML = partes.join("");
    svg.setAttribute("viewBox", `0 0 ${largura} ${altura}`);
    estado.arvore = { largura, altura };
    estado.arvoreRefs = {
      nos: [...svg.querySelectorAll("g.no")].map((elemento) => ({
        id: Number(elemento.dataset.id),
        elemento,
        distintivo: elemento.querySelector(".distintivo"),
      })),
      arestas: [...svg.querySelectorAll("g.grupo-aresta")].map((elemento) => ({
        id: Number(elemento.dataset.id),
        elemento,
      })),
    };

    el.arvoreVazia.hidden = true;
    aplicarZoomAutomatico();
  }

  function aplicarZoom() {
    if (!estado.arvore) return;
    el.arvore.setAttribute("width", String(Math.round(estado.arvore.largura * estado.zoom)));
    el.arvore.setAttribute("height", String(Math.round(estado.arvore.altura * estado.zoom)));
    el.zoomNivel.textContent = `${Math.round(estado.zoom * 100)}%`;
  }

  function aplicarZoomAutomatico() {
    if (!estado.arvore) return;
    const disponivelLargura = Math.max(220, el.arvoreScroll.clientWidth - 30);
    const disponivelAltura = Math.max(200, el.arvoreScroll.clientHeight - 26);
    const ajusteLargura = disponivelLargura / estado.arvore.largura;
    const ajusteAltura = disponivelAltura / estado.arvore.altura;
    const ajuste = Math.min(ajusteLargura, ajusteAltura);
    estado.zoom = limitar(Number(ajuste.toFixed(2)), 0.45, 1);
    estado.zoomManual = false;
    aplicarZoom();
  }

  function alterarZoom(fator) {
    if (!estado.arvore) return;
    estado.zoomManual = true;
    estado.zoom = limitar(Number((estado.zoom * fator).toFixed(2)), 0.25, 2.5);
    aplicarZoom();
  }

  function focarElementoNoMapa(elemento, suave = false) {
    const caixa = el.arvoreScroll;
    if (!elemento || !caixa) return;

    const margem = 22;
    const caixaRect = caixa.getBoundingClientRect();
    const alvoRect = elemento.getBoundingClientRect();
    let topo = caixa.scrollTop;
    let esquerda = caixa.scrollLeft;

    if (alvoRect.top < caixaRect.top + margem) {
      topo += alvoRect.top - caixaRect.top - margem;
    } else if (alvoRect.bottom > caixaRect.bottom - margem) {
      topo += alvoRect.bottom - caixaRect.bottom + margem;
    }

    if (alvoRect.left < caixaRect.left + margem) {
      esquerda += alvoRect.left - caixaRect.left - margem;
    } else if (alvoRect.right > caixaRect.right - margem) {
      esquerda += alvoRect.right - caixaRect.right + margem;
    }

    if (topo === caixa.scrollTop && esquerda === caixa.scrollLeft) return;

    caixa.scrollTo({
      top: topo,
      left: esquerda,
      behavior: suave && !movimentoReduzido ? "smooth" : "auto",
    });
  }

  function focarNoAtual() {
    if (!estado.resultado || !estado.arvoreRefs) return;
    const atualId =
      estado.modo === "solucao"
        ? estado.resultado.caminho[estado.indice]
        : estado.resultado.passos[estado.indice]?.expandido;
    const referencia = estado.arvoreRefs.nos.find(({ id }) => id === atualId);
    if (!referencia) return;
    focarElementoNoMapa(referencia.elemento);
  }

  function pintarArvore() {
    const resultado = estado.resultado;
    if (!resultado || !estado.arvoreRefs) return;

    const posCaminho = new Map();
    resultado.caminho.forEach((id, indice) => posCaminho.set(id, indice));
    const objetivoId = resultado.caminho.length
      ? resultado.caminho[resultado.caminho.length - 1]
      : null;
    const modoSolucao = estado.modo === "solucao";

    let expandidosVisiveis = null;
    let borda = null;
    let maxGerado = Infinity;
    let atualId = null;

    if (modoSolucao) {
      atualId = resultado.caminho[estado.indice] ?? null;
      borda = new Set(resultado.bordaFinal);
    } else {
      const passo = resultado.passos[estado.indice];
      if (passo) {
        expandidosVisiveis = new Set(resultado.expandidos.slice(0, estado.indice + 1));
        borda = new Set(passo.borda);
        maxGerado = estado.maxGerado[estado.indice] ?? 0;
        atualId = passo.expandido;
      }
    }

    for (const { id, elemento, distintivo } of estado.arvoreRefs.nos) {
      const no = resultado.nos[id];
      const oculto = !modoSolucao && id > maxGerado;
      const expandidoAgora = modoSolucao ? no.expandido : expandidosVisiveis?.has(id);
      const classes = ["no"];
      if (oculto) classes.push("oculto");
      if (expandidoAgora) classes.push("expandido");
      if (!oculto && borda?.has(id)) classes.push("borda");
      if (modoSolucao && posCaminho.has(id) && posCaminho.get(id) <= estado.indice) {
        classes.push("caminho");
      }
      if (modoSolucao && id === objetivoId) classes.push("objetivo");
      if (!oculto && id === atualId) classes.push("atual");
      if (distintivo) distintivo.classList.toggle("oculto", !expandidoAgora || id === objetivoId);
      elemento.setAttribute("class", classes.join(" "));
    }

    for (const { id, elemento } of estado.arvoreRefs.arestas) {
      const oculto = !modoSolucao && id > maxGerado;
      const noCaminho = modoSolucao && posCaminho.has(id) && posCaminho.get(id) <= estado.indice;
      const classes = ["grupo-aresta"];
      if (oculto) classes.push("oculto");
      if (noCaminho) classes.push("caminho");
      elemento.setAttribute("class", classes.join(" "));
    }

    if (estado.reproduzindo) {
      const atual = estado.arvoreRefs.nos.find(({ id }) => id === atualId);
      if (atual) focarElementoNoMapa(atual.elemento, true);
    }
  }

  // ------------------------------------------------------------------ resumo
  function renderizarResumo() {
    const resultado = estado.resultado;
    if (!resultado) {
      el.resumoBusca.textContent = "—";
      return;
    }

    const estatisticas = resultado.estatisticas;
    el.resumoBusca.textContent = resultado.encontrado
      ? `profundidade ${estatisticas.profundidade} · ${estatisticas.nosGerados} nós gerados · ${estatisticas.repetidosEvitados} repetidos evitados`
      : `sem solução · ${estatisticas.nosGerados} nós gerados`;
  }

  // --------------------------------------------------------------- controles
  function renderizarControles() {
    const total = tamanhoLinha();
    const semResultado = !estado.resultado || total === 0;
    const noInicio = estado.indice <= 0;
    const noFim = estado.indice >= total - 1;

    el.btnInicio.disabled = semResultado || noInicio;
    el.btnAnterior.disabled = semResultado || noInicio;
    el.btnProximo.disabled = semResultado || noFim;
    el.btnFim.disabled = semResultado || noFim;
    el.btnReproduzir.disabled = semResultado || total <= 1;
    el.btnRelatorio.disabled = !estado.resultado;

    el.btnReproduzir.dataset.reproduzindo = estado.reproduzindo ? "true" : "false";
    el.btnReproduzir.setAttribute("aria-label", estado.reproduzindo ? "Pausar" : "Reproduzir");
    el.textoReproduzir.textContent = estado.reproduzindo ? "Pausar" : "Reproduzir";

    document.getElementById(estado.modo === "solucao" ? "modo-solucao" : "modo-busca").checked = true;
  }

  // ------------------------------------------------------------------ geral
  function renderizarVazio() {
    el.estadoRotulo.textContent = "Sem busca";
    el.estadoEstado.textContent = "(0, 0)";
    el.estadoAcao.textContent = "Configure os jarros e execute uma busca";
    atualizarJarros({ a: 0, b: 0 });

    el.arvore.innerHTML = "";
    el.arvoreVazia.hidden = false;
    el.arvoreVazia.textContent = "Execute uma busca para desenhar a árvore de estados.";
    el.resumoBusca.textContent = "—";
    el.zoomNivel.textContent = "100%";

    estado.maxGerado = [];
    estado.arvore = null;
    estado.arvoreRefs = null;
    estado.zoom = 1;
    estado.zoomManual = false;
    renderizarControles();
  }

  function renderizarTudo() {
    if (!estado.resultado) {
      renderizarVazio();
      return;
    }
    renderizarEstadoEJarros();
    pintarArvore();
    renderizarResumo();
    renderizarControles();
  }

  function calcularMaxGerado(resultado) {
    let maximo = 0;
    estado.maxGerado = resultado.passos.map((passo) => {
      for (const sucessor of passo.sucessores) {
        if (sucessor.no !== null && sucessor.no > maximo) maximo = sucessor.no;
      }
      return maximo;
    });
  }

  function aplicarResultado(resultado) {
    estado.resultado = resultado;
    estado.modo = resultado.encontrado ? "solucao" : "busca";
    estado.reproduzindo = false;
    estado.indice = resultado.encontrado
      ? Math.max(0, resultado.caminho.length - 1)
      : Math.max(0, resultado.passos.length - 1);

    calcularMaxGerado(resultado);
    montarJarros(resultado.config);
    montarArvore(resultado);

    esconderErro();
    renderizarTudo();
    window.requestAnimationFrame(focarNoAtual);
  }

  function configuracaoDoFormulario() {
    const capacidadeA = numeroInteiro(el.capA.value, NaN);
    const capacidadeB = numeroInteiro(el.capB.value, NaN);
    const meta = numeroInteiro(el.meta.value, NaN);
    if (!Number.isFinite(capacidadeA) || !Number.isFinite(capacidadeB) || !Number.isFinite(meta)) {
      return null;
    }
    if (capacidadeA < 1 || capacidadeB < 1 || meta < 0) return null;
    return {
      capacidadeA,
      capacidadeB,
      meta,
      alvo: el.alvo.value,
      algoritmo: document.querySelector('input[name="algoritmo"]:checked')?.value ?? "bfs",
    };
  }

  async function executarBusca(evento) {
    if (evento) evento.preventDefault();
    pausar();

    const configuracao = configuracaoDoFormulario();
    if (!configuracao) {
      mostrarErro("Preencha as capacidades com valores entre 1 e 12 litros e a meta com um valor maior ou igual a zero.");
      return;
    }

    const parametros = new URLSearchParams({
      capA: String(configuracao.capacidadeA),
      capB: String(configuracao.capacidadeB),
      meta: String(configuracao.meta),
      alvo: configuracao.alvo,
      algoritmo: configuracao.algoritmo,
    });

    el.botaoBuscar.disabled = true;
    const rotuloOriginal = el.botaoBuscar.textContent;
    el.botaoBuscar.textContent = "Buscando…";

    try {
      const resposta = await fetch(`/api/solve?${parametros.toString()}`, {
        headers: { Accept: "application/json" },
      });
      const dados = await resposta.json();

      if (!dados.ok) {
        mostrarErro(dados.mensagem || "A busca não pôde ser executada com esses valores.");
        limparApresentacao();
        return;
      }

      aplicarResultado(dados);
    } catch (erro) {
      mostrarErro(
        "Não foi possível falar com o servidor local. Verifique se o terminal ainda está aberto e recarregue a página."
      );
      limparApresentacao();
    } finally {
      el.botaoBuscar.disabled = false;
      el.botaoBuscar.textContent = rotuloOriginal;
    }
  }

  function limparApresentacao() {
    pausar();
    estado.resultado = null;
    estado.modo = "solucao";
    estado.indice = 0;
    estado.maxGerado = [];

    const configuracao = configuracaoDoFormulario();
    if (configuracao) {
      montarJarros(configuracao);
    }
    renderizarVazio();
  }

  // --------------------------------------------------------------- relatório
  function gerarRelatorio() {
    const resultado = estado.resultado;
    if (!resultado) return "Nenhuma busca executada.";

    const config = resultado.config;
    const linhas = [];
    linhas.push("PROBLEMA DOS JARROS — RELATÓRIO DE BUSCA");
    linhas.push("");
    linhas.push(
      `Configuração: jarro A = ${config.capacidadeA} L · jarro B = ${config.capacidadeB} L · meta = ${config.meta} L (${config.alvoRotulo.toLowerCase()})`
    );
    linhas.push(`Algoritmo: ${config.algoritmoRotulo}`);
    linhas.push("");
    linhas.push("ORDEM DE EXPANSÃO");
    resultado.passos.forEach((passo, indice) => {
      const no = resultado.nos[passo.expandido];
      linhas.push(
        `${String(indice).padStart(2, "0")}. (${no.a},${no.b}) — ${no.acaoRotulo ?? "Início"} — borda: ${passo.borda.length}`
      );
    });
    linhas.push("");

    if (resultado.encontrado) {
      linhas.push(`CAMINHO DA SOLUÇÃO (${resultado.caminho.length - 1} passos)`);
      resultado.caminho.forEach((id, indice) => {
        const no = resultado.nos[id];
        linhas.push(
          `${String(indice).padStart(2, "0")}. (${no.a},${no.b}) — ${no.acaoRotulo ?? "Início"}`
        );
      });
    } else {
      linhas.push("NENHUMA SOLUÇÃO ENCONTRADA para esta configuração.");
    }

    const estatisticas = resultado.estatisticas;
    linhas.push("");
    linhas.push("ESTATÍSTICAS");
    linhas.push(`  nós gerados:        ${estatisticas.nosGerados}`);
    linhas.push(`  estados visitados:  ${estatisticas.estadosVisitados}`);
    linhas.push(`  nós expandidos:     ${estatisticas.estadosExpandidos}`);
    linhas.push(`  repetidos evitados: ${estatisticas.repetidosEvitados}`);
    linhas.push(`  maior borda:        ${estatisticas.maiorBorda}`);
    linhas.push(
      `  profundidade:       ${estatisticas.profundidade === null ? "—" : estatisticas.profundidade}`
    );
    linhas.push(
      `  tempo de busca:     ${(estatisticas.duracaoMicros / 1000).toFixed(2).replace(".", ",")} ms`
    );
    return linhas.join("\n");
  }

  function sinalizarCopiado() {
    el.btnRelatorio.dataset.copiado = "true";
    el.btnRelatorio.textContent = "Copiado!";
    window.setTimeout(() => {
      el.btnRelatorio.dataset.copiado = "false";
      el.btnRelatorio.textContent = "Copiar relatório";
    }, 1600);
  }

  async function copiarRelatorio() {
    const texto = gerarRelatorio();
    try {
      await navigator.clipboard.writeText(texto);
      sinalizarCopiado();
    } catch (erro) {
      const area = document.createElement("textarea");
      area.value = texto;
      area.style.position = "fixed";
      area.style.opacity = "0";
      document.body.appendChild(area);
      area.select();
      try {
        document.execCommand("copy");
        sinalizarCopiado();
      } catch (falha) {
        mostrarErro("Não foi possível copiar o relatório para a área de transferência.");
      } finally {
        area.remove();
      }
    }
  }

  // --------------------------------------------------------------- eventos
  function configurarEventos() {
    el.form.addEventListener("submit", executarBusca);

    el.btnInicio.addEventListener("click", () => {
      pausar();
      definirIndice(0);
    });
    el.btnAnterior.addEventListener("click", () => {
      pausar();
      definirIndice(estado.indice - 1);
    });
    el.btnProximo.addEventListener("click", () => {
      pausar();
      definirIndice(estado.indice + 1);
    });
    el.btnFim.addEventListener("click", () => {
      pausar();
      definirIndice(tamanhoLinha() - 1);
    });
    el.btnReproduzir.addEventListener("click", alternarReproducao);

    document.querySelectorAll('input[name="modo"]').forEach((radio) => {
      radio.addEventListener("change", () => {
        if (radio.checked) definirModo(radio.value);
      });
    });

    el.btnZoomMais.addEventListener("click", () => alterarZoom(1.15));
    el.btnZoomMenos.addEventListener("click", () => alterarZoom(0.87));
    el.btnAjustar.addEventListener("click", aplicarZoomAutomatico);

    let redimensionamento = null;
    window.addEventListener("resize", () => {
      if (!estado.resultado || estado.zoomManual) return;
      window.clearTimeout(redimensionamento);
      redimensionamento = window.setTimeout(aplicarZoomAutomatico, 140);
    });

    el.btnRelatorio.addEventListener("click", copiarRelatorio);

    document.addEventListener("keydown", (evento) => {
      const alvo = evento.target;
      if (alvo instanceof HTMLElement && alvo.matches("input, select, textarea")) return;

      switch (evento.key) {
        case "ArrowLeft":
          evento.preventDefault();
          pausar();
          definirIndice(estado.indice - 1);
          break;
        case "ArrowRight":
          evento.preventDefault();
          pausar();
          definirIndice(estado.indice + 1);
          break;
        case "Home":
          evento.preventDefault();
          pausar();
          definirIndice(0);
          break;
        case "End":
          evento.preventDefault();
          pausar();
          definirIndice(tamanhoLinha() - 1);
          break;
        case " ":
          if (!(alvo instanceof HTMLElement) || !alvo.matches("button")) {
            evento.preventDefault();
            alternarReproducao();
          }
          break;
        default:
          break;
      }
    });
  }

  // --------------------------------------------------------------- inicial
  function inicializar() {
    configurarEventos();

    const configuracao = configuracaoDoFormulario() ?? {
      capacidadeA: 5,
      capacidadeB: 3,
      meta: 4,
      alvo: "qualquer",
      algoritmo: "bfs",
    };
    montarJarros(configuracao);
    renderizarVazio();
    executarBusca();
  }

  inicializar();
})();
