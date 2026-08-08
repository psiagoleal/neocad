<!-- Caminho relativo: docs/adr/0003-kernel-cad-proprio.md -->

# ADR 0003: Kernel CAD próprio e reutilizável, com upstream rebaixado a parser e renderer

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-08-06
- **Decisores:** Iago Leal
- **Tags:** arquitetura, kernel, geometria, brep, rust, wasm, upstream, biblioteca

## Contexto

O objetivo declarado do NeoCAD é ser uma alternativa open-source de CAD com as
funcionalidades principais. A arquitetura vigente até este ADR — wrapper de UI
sobre `@mlightcad/cad-simple-viewer` — impõe um teto funcional que **não pode ser
superado do lado do NeoCAD**:

- **Não há undo/redo.** O ADR 0001 documentou que o command stack do upstream não
  registra `UNDO/REDO`, e também não expõe `SCALE, MIRROR, ARRAY, OFFSET,
TRIM/EXTEND, BLOCK/INSERT`. Undo/redo não é um comando que se adicione por fora:
  é propriedade do modelo de documento. Enquanto o modelo pertencer ao upstream,
  a capacidade é inalcançável.
- **Não há escrita.** O produto lê `DWG`/`DXF` e não persiste nada. Não existe
  `salvar`, `salvar como` nem exportação. `src-tauri/capabilities/default.json`
  concede apenas leitura de arquivos e escrita em `AppConfig`.
- **Não há modelagem 3D.** O upstream é um visualizador 2D com modelo de dados
  orientado a entidades de desenho, sem topologia B-rep nem geometria NURBS.
- **Não há controle de evolução.** `@mlightcad/*` é tratado pelo próprio ADR 0001
  como API interna não-estável. Cada capacidade nova depende de o upstream
  decidir implementá-la.

Além do produto NeoCAD, existe um objetivo explícito de **reaproveitamento**: o
kernel deve poder ser usado em outros projetos futuros, e não apenas dentro deste
aplicativo. Isso o caracteriza como **produto de biblioteca**, não como camada
interna — o que impõe disciplina de API pública, versionamento semântico,
documentação própria e independência de qualquer detalhe do NeoCAD.

Três condições favoráveis já estão dadas:

1. **A fronteira certa existe.** O ADR 0001 concentrou todo acesso ao upstream no
   adaptador `NeoCadViewer` e nos serviços de `src/lib/services/`. A UI já não
   conhece tipos do upstream. Trocar o que está atrás dessa fronteira é uma
   operação contida.
2. **A licença deixou de ser restrição de dependências.** Com o ADR 0002
   (GPL-3.0), o ecossistema copyleft de CAD/CAE fica disponível para o
   aplicativo: LibreDWG, planegcs, OpenCASCADE, Gmsh, CalculiX, OpenFOAM.
3. **O backend já é Rust.** Um kernel em Rust compila para WebAssembly no
   frontend **e** liga-se nativamente ao backend Tauri, com uma única base de
   código — o que não seria possível mantendo o modelo em TypeScript. Rust
   também é o alvo natural para uma biblioteca reutilizável fora deste projeto.

A decisão precisa ser tomada agora porque as Frentes 1 e 3 do roadmap (painéis de
camadas/propriedades e disparo de comandos) serão construídas sobre algum modelo
de documento. Construí-las sobre o modelo do upstream significaria refazê-las
depois.

## Decisão

Fica acordada a construção de um **kernel CAD próprio, completo e reutilizável**,
escrito em **Rust**, organizado como **workspace de crates independentes do
NeoCAD** e distribuído em duas formas a partir de uma única base de código:
compilado para **WebAssembly** para uso no frontend, e ligado nativamente ao
backend **Tauri** para operações de arquivo e processamento pesado.

O kernel é tratado como **biblioteca de propósito geral**, não como camada interna
do aplicativo. Nenhuma crate do kernel conhece Tauri, Svelte, o DOM ou qualquer
tipo do NeoCAD.

O `@mlightcad/*` deixa de ser o núcleo do produto e passa a ser **dependência
substituível de parsing e renderização**, consumida atrás da mesma fronteira
definida pelo ADR 0001, que permanece em vigor. O ADR 0001 **não é revogado**:
sua regra de isolamento passa a proteger tanto o upstream quanto o kernel próprio.

### Organização das crates

```text
kernel/
├── neocad-geometry      # vetores, curvas, superfícies, NURBS, tesselação
├── neocad-topology      # B-rep: vértice, aresta, face, shell, sólido
├── neocad-model         # entidades de desenho, tabelas de símbolos, documento
├── neocad-transaction   # command stack transacional, undo/redo
├── neocad-io            # DXF/DWG/STEP — única camada autorizada a depender de copyleft
└── neocad-wasm          # fachada wasm-bindgen consumida pelo frontend
```

A separação entre `neocad-io` e as demais é **estrutural, não estética**: as
dependências copyleft do projeto (LibreDWG, `dxf-json`) vivem exclusivamente na
camada de I/O, mantendo as crates de geometria, topologia, modelo e transações
livres de copyleft. Isso preserva a possibilidade de licenciar o kernel de forma
independente do aplicativo — condição prática para o reaproveitamento pretendido,
já que uma biblioteca sob GPL-3.0 só pode ser usada por projetos GPL. A licença
definitiva das crates do kernel será objeto de ADR próprio no momento da extração
para repositório independente; até lá, elas herdam a licença do repositório.

### Faseamento

Cada fase entrega valor observável antes de a seguinte começar.

**Núcleo 2D e documento**

- **K1 — Modelo de documento e transações.** Entidades, tabelas de símbolos
  (camadas, blocos, estilos) e command stack transacional com **undo/redo**. O
  modelo do NeoCAD passa a ser a fonte de verdade; o upstream é usado para
  produzir esse modelo a partir do arquivo e para desenhá-lo na tela.
- **K2 — Escrita DXF.** Serialização do modelo próprio para DXF, a partir da
  especificação pública da Autodesk. Converte o produto de visualizador em
  ferramenta.
- **K3 — Geometria 2D e operações de edição.** Primitivas, interseções, offset,
  trim/extend, fillet/chamfer, booleanas 2D, índice espacial e snapping.
- **K4 — Solver de restrições 2D.** Desenho paramétrico. Avaliar reaproveitamento
  do `planegcs` (FreeCAD, LGPL) antes de implementação própria.
- **K5 — Renderização própria.** Substituição progressiva da camada de desenho do
  upstream, quando a de terceiros passar a limitar a interação (seleção fina,
  grips, preview dinâmico).
- **K6 — Leitura DWG.** Permanece sobre a LibreDWG, em `neocad-io`. Escrita DWG
  fica fora de escopo por ausência de especificação aberta.

**Núcleo 3D**

- **K7 — Geometria 3D.** Curvas e superfícies NURBS: avaliação, derivadas,
  interseção curva-superfície, projeção, tesselação adaptativa.
- **K8 — Topologia B-rep.** Vértice, aresta, laço, face, shell e sólido;
  operadores de Euler; validação topológica; consulta de adjacência.
- **K9 — Modelagem sólida.** Extrusão, revolução, varredura, loft; operações
  booleanas 3D; fillet e chamfer 3D; importação e exportação **STEP/IGES**.

O **OpenCASCADE** passa a ser tratado como **referência técnica e base de
comparação** — para validar resultados de operações booleanas e tolerâncias —, e
não como dependência do kernel.

## Consequências

- **Impacto positivo:** o teto funcional passa a ser determinado pelo projeto, e
  não por terceiros; undo/redo, salvamento, edição real e modelagem 3D tornam-se
  alcançáveis; uma única base Rust serve frontend (WASM), backend (nativo) e
  projetos futuros; o desempenho em desenhos grandes passa a ser otimizável onde
  hoje é opaco; o projeto ganha um ativo técnico reutilizável, condição para
  atrair contribuição de longo prazo.
- **Impacto negativo:** é um esforço de **muitos anos**, concentrado em um único
  mantenedor. As fases K7–K9 são, isoladamente, a parte mais difícil: kernels
  sólidos maduros (Parasolid, ACIS, OpenCASCADE) são obra de décadas e de equipes
  dedicadas, e a robustez de operações booleanas sobre geometria NURBS com
  tolerâncias reais é um problema notoriamente hostil. Cada fase é uma
  oportunidade de estagnação. O modelo próprio precisa ser mantido em paridade
  com o upstream durante toda a transição, o que significa manter duas
  representações vivas por um período longo. Funcionalidade hoje herdada de graça
  do upstream passará a ter custo de implementação e de correção de regressões. O
  risco de compatibilidade com arquivos reais do mundo — DXF/DWG produzidos por
  décadas de versões do AutoCAD — é alto e só se revela em campo. A disciplina de
  biblioteca reutilizável (API estável, semver, documentação) impõe custo
  adicional sobre cada fase.
- **Trade-offs aceitos:** aceita-se prazo longo e risco de execução em troca de
  controle integral sobre a evolução do produto e de um ativo reaproveitável.
  Aceita-se manter dependência copyleft de terceiros exatamente onde o custo
  próprio seria proibitivo — parsing DWG — e investir esforço próprio onde ele é
  decisivo. Aceita-se que as fases 2D (K1–K6) precedam as 3D (K7–K9), de modo que
  o produto seja útil muito antes de o kernel estar completo.

## Diretriz de Conformidade de Código

- **Proibido:** construir novas funcionalidades de edição sobre o modelo de
  documento do `@mlightcad/*` a partir da conclusão de K1. O modelo do NeoCAD é a
  fonte de verdade sobre entidades, camadas e estado do documento.
- **Proibido:** mutar o modelo de documento fora do command stack transacional.
  Toda alteração de estado do desenho ocorre por meio de um comando reversível.
- **Proibido:** importar tipos ou símbolos de `@mlightcad/*` fora do adaptador
  `src/lib/viewer/` e dos serviços `src/lib/services/`, conforme o ADR 0001, que
  permanece em vigor.
- **Proibido:** referenciar Tauri, Svelte, DOM, `wasm-bindgen` ou qualquer tipo
  do NeoCAD nas crates `neocad-geometry`, `neocad-topology`, `neocad-model` e
  `neocad-transaction`. A ligação com o ambiente ocorre apenas em `neocad-wasm` e
  no backend Tauri.
- **Proibido:** introduzir dependência copyleft em qualquer crate do kernel que
  não seja `neocad-io`, sob pena de inviabilizar o licenciamento independente do
  kernel.
- **Obrigatório:** o kernel reside no workspace Rust `kernel/`, compilável tanto
  para `wasm32-unknown-unknown` quanto para os alvos nativos do Tauri, com
  `cargo test` próprio por crate.
- **Obrigatório:** toda API pública das crates do kernel é documentada com
  `///` e validada por `cargo doc`, e segue versionamento semântico a partir da
  primeira publicação.
- **Obrigatório:** cada fase K1–K9 é precedida de micro-tickets (skill
  `micro-ticket-planner`) e acompanhada de testes de unidade da crate e de testes
  de regressão sobre arquivos CAD reais em `e2e/fixtures/`.
- **Obrigatório:** enquanto uma fase não estiver concluída, a capacidade
  correspondente continua sendo servida pelo upstream, sem regressão observável
  para o usuário.
- **Obrigatório:** operações booleanas e de tolerância implementadas em K9 são
  validadas contra resultados do OpenCASCADE antes de serem consideradas prontas.
- **Obrigatório:** toda substituição de capacidade do upstream por implementação
  própria é registrada no `CHANGELOG.md` e refletida em `THIRD-PARTY-LICENSES.md`
  quando alterar a árvore de dependências distribuídas.

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do projeto
> e deve ser reportado para revisão antes de prosseguir.
