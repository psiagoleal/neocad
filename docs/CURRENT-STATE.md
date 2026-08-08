<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> Opcional em projetos solo; recomendado em colaborações. Atualizado a cada commit.
> Não inclua segredos. Mantido conforme a skill `handoff-updater`.

## Último turno

- **Data:** 2026-08-08
- **Branch:** `master`
- **Commit:** pendente (trabalho acumulado no working tree)

### Fase K1 concluída

Os 17 micro-tickets de `docs/tickets/k1-modelo-documento-transacoes.md` estão
fechados. O NeoCAD tem **modelo de documento próprio em Rust**, exposto ao
frontend por WebAssembly, com **toda mutação do desenho passando por transação
reversível** e `Desfazer`/`Refazer` na interface.

Números: **238 testes** no kernel (incluindo 4 doctests `compile_fail` que
provam a restrição de mutação), **60** no frontend e **7** E2E. `kernel/` soma
cerca de 7 mil linhas em seis crates.

O upstream `@mlightcad` continua lendo o arquivo e desenhando — sua substituição
é K5 e K6. As duas representações convivem, e a concordância entre elas é
verificada em E2E.

### Decisões estruturais do período

**ADR 0002 — relicenciamento para GPL-3.0-or-later.** Auditoria de dependências
identificou duas dependências de runtime GPL-3.0 no caminho crítico, ambas
embarcadas nos binários publicados: `@mlightcad/libredwg-web@0.7.1`
(LibreDWG/WASM, caminho **DWG**, via `cad-simple-viewer -> libredwg-converter`) e
`@mlightcad/dxf-json@1.2.0` (caminho **DXF**, via `data-model`, dependência
direta). Os binários já eram obra combinada sujeita à GPL-3.0 enquanto o projeto
anunciava MIT. `LICENSE`, `README.md`, `package.json`, `Cargo.toml` e
`src/lib/config/app.ts` foram atualizados.

**ADR 0003 — kernel CAD próprio e reutilizável.** O upstream passa a ser tratado
como parser e renderer substituível. Kernel em Rust, em workspace `kernel/`
independente, compilado para WASM no frontend e ligável nativamente ao backend
Tauri. Faseamento **K1–K9**, cobrindo 2D e **3D B-rep próprio** (NURBS, topologia,
booleanas, STEP/IGES) — o OpenCASCADE é referência de validação, não dependência.
Requisito de reuso em outros projetos: as dependências copyleft ficam confinadas à
crate `neocad-io`, preservando a possibilidade de licenciar o kernel de forma
independente. A fronteira do ADR 0001 permanece em vigor e passa a proteger também
o kernel próprio.

## Último turno anterior

- **Data:** 2026-06-03
- **Commit:** `6502246` (alinhamento à governança de IA + consolidação do CHANGELOG)

## Metas cumpridas / Em andamento / Próximo passo

- [x] Spike técnico do upstream `@mlightcad` documentado em
      `docs/upstream-capabilities-spike.md`; ADR 0001 **Accepted**.
- [x] Governança de agentes versionada no commit `d1e4dbd`.
- [x] **Frente 2 implementada:** catálogo de comandos derivado em runtime do command
      stack, exposto em `Ajuda > Comandos CAD` (diálogo filtrável por categoria).
      Verificado: `pnpm check` (0/0), `pnpm test` (13/13), `pnpm lint` verde.
- [x] **Bug do canvas em branco resolvido:** `.viewer-surface` colapsava para altura 0
      (caía na trilha `auto` do grid quando a barra de progresso estava ausente).
      `.viewer-frame` agora é flex em coluna. Diagnosticado com Playwright (cadeia de
      ancestrais) e protegido por `e2e/viewer-render.e2e.ts` + fixture `minimal.dxf`.
- [x] **Governança alinhada às novas regras de IA:** ADR 0001 com diretriz reescrita como
      restrição do projeto (sem se dirigir a agente); `CHANGELOG.md` consolidado como fonte
      canônica única (política dobrada no preâmbulo; `docs/changelog.md` removido); ênfase
      markdown normalizada ao prettier (`_`); referências pendentes ajustadas em
      `README.md` e `AGENTS.md`. Docs de planejamento verificados — já conformes.
- [x] **CI implantada** (`.github/workflows/ci.yml`): `check`/`lint`/`test`/`build`,
      `cargo fmt`/`check`/`clippy` em Linux e Windows, E2E Playwright, varredura de
      segredos e política de licenças. Fecha a lacuna do DoD (AGENTS.md §8), que
      exigia CI e nunca teve workflow.
- [x] **Workers do upstream desversionados:** ~9,7 MB de JS minificado de terceiros
      saíram do repositório. Agora derivados de `node_modules` por
      `scripts/sync-workers.mjs`; só o manifesto (versão + SHA-256) é versionado, e
      `pnpm workers:check` detecta divergência na CI.
- [x] **Licenciamento resolvido:** ADR 0002 (GPL-3.0-or-later) aplicado em todo o
      repositório; política de licenças verificada na CI.
- [x] **Rumo técnico definido:** ADR 0003 (kernel próprio em Rust/WASM).
- [x] **Roadmap realinhado ao ADR 0003.** As Frentes 1–3 passam a ser sequenciadas
      contra as fases do kernel; os comandos ausentes no upstream deixam de ser
      teto e viram especificação do kernel; a trilha FEM/CFD fica formalmente
      posterior a K8, com STEP (K9) como formato de intercâmbio.
- [x] **K1 quebrada em 17 micro-tickets:**
      `docs/tickets/k1-modelo-documento-transacoes.md`, em cinco blocos (fundação
      do workspace, modelo, transações, ponte com o frontend, interface).
- [x] **MT-K1-01 concluído:** workspace Rust `kernel/` criado com as seis crates
      do ADR 0003 compilando vazias, independente do workspace de `src-tauri/`
      (zero referências a Tauri nos metadados). Grafo de dependências conforme o
      ADR; lints de workspace impondo `missing_docs = deny` e `unsafe_code = deny`
      desde o início. `cargo test --all`, `cargo clippy -- -D warnings` e
      `cargo fmt --check` verdes; `neocad-wasm` já compila para
      `wasm32-unknown-unknown`.
- [x] **MT-K1-02 concluído:** job `kernel` na CI (Linux e Windows) com `fmt`,
      `check`, `clippy` e `test`, usando `working-directory: kernel` e deixando o
      `rust-toolchain.toml` resolver canal, componentes e alvos — sem action de
      toolchain. Alvos `kernel-*` no `Makefile`; `make kernel-check` reproduz a
      mesma sequência localmente e passa. O critério "job verde na primeira
      execução" só é confirmável após o push.
- [x] **MT-K1-03 concluído:** `EntityId` opaco (índice + geração, `NonZeroU32`) e
      `Arena<T>` geracional em `neocad-model`. Identificador obsoleto resolve para
      `None` em vez de alcançar o valor que ocupou o slot — é a propriedade que
      protege seleção, histórico e referências entre entidades. Reuso de slot em
      ordem crescente de índice e iteração por índice, para determinismo. Geração
      esgotada aposenta o slot (`Entry::Retired`) em vez de reemitir identificador
      já entregue. 23 testes + 1 doc-test; `make kernel-check` verde.
- [x] **MT-K1-04 concluído:** `LayerTable` com `LayerId` opaco distinto de
      `EntityId`, registro (nome, cor ACI/RGB, tipo de linha, espessura, off,
      frozen, locked), busca por nome ignorando caixa, camada `0` protegida
      contra remoção e renomeação, e iteração alfabética determinística. Camadas
      são referenciadas por id e não por nome — renomear é O(1) e não varre
      entidades, ao contrário do que o DXF força. 47 testes + 2 doc-tests.
- [x] **ADR 0004 registrado:** interface headless para agentes de IA (CLI como
      núcleo, MCP como fachada), com Frente 5 no roadmap. Acrescenta **leitura**
      nativa de DXF ao escopo de K2 — o ADR 0003 não a previa porque assumia o
      upstream como parser, premissa que não vale sem navegador.
- [x] **MT-K1-05 concluído:** `neocad-geometry` ganhou `Point2` e `Aabb`;
      `neocad-model` ganhou `Entity` (camada + `EntityColor` ByLayer/ByBlock/
      explícita) e `Geometry` com `Line`, `Circle`, `Arc`, `Polyline` e `Text`,
      cada uma com caixa envolvente. **A caixa do arco inclui os ângulos cardeais
      contidos na varredura** — a caixa dos dois extremos é insuficiente e erra
      justamente nos arcos que cruzam os eixos. 88 testes + 4 doc-tests.
- [x] **MT-K1-06 concluído:** `BlockTable` com `*Model_Space` como bloco raiz
      indestrutível e lista ordenada de entidades por bloco (a ordem é a de
      desenho); `TextStyleTable` com `Standard` protegido e resolução de altura
      efetiva — estilo de altura fixa sobrepõe a altura da entidade. Nomes
      iniciados por `*` são recusados na criação, o que torna o espaço-modelo
      inalcançável por colisão. 127 testes no kernel.
- [x] **Validação de nome unificada:** as três tabelas de símbolos passaram a
      compartilhar `crate::symbol_name` (`lib.rs`), em vez de triplicar as regras.
      Exigiu tocar `layer.rs`, fora do escopo declarado do MT-K1-06 — ver nota
      abaixo.
- [x] **MT-K1-07 concluído — bloco B fechado.** `Document` agrega arena e as três
      tabelas e passa a ser o dono das invariantes que **atravessam** estruturas,
      que nenhuma tabela isolada podia verificar: entidade só entra se sua camada
      existir; arena e lista do bloco são mantidas em sincronia; camada com
      entidades não pode ser removida (`LayerInUse`); remover bloco leva junto as
      entidades dele, para não deixar órfã inalcançável. Tabelas expostas só para
      leitura; o que cruza estruturas tem método próprio. 154 testes no kernel.
- [x] **MT-K1-08 concluído:** `Change` em `neocad-transaction`, com as quatro
      mutações atômicas (inserir, remover, substituir entidade; alterar registro
      de camada). `Change::apply` **devolve a mudança que a desfaz**, construída
      a partir do estado observado no momento da aplicação — é isso que torna a
      inversão exata em vez de apenas equivalente. 179 testes no kernel.
- [x] **Resolvida a questão de "igualdade estrutural"** levantada no MT-K1-03:
      `Document` passou a implementar `PartialEq` **semântica**, comparando o
      conteúdo observável e ignorando resíduo de alocação (slot vago deixado por
      remoção, ordem da lista de reuso). Comparar a representação bruta
      reprovaria desfazimentos corretos.
- [x] **MT-K1-09 concluído:** `Transaction` (grupo nomeado e **atômico** de
      mudanças — se uma falha, as anteriores são revertidas) e `CommandStack` com
      `undo`/`redo`, limite configurável, descarte do ramo de refazer e transação
      vazia que não ocupa passo de desfazer. A pilha guarda, para cada ação, a
      transação que a **desfaz**; desfazer é aplicá-la, o que já produz a que
      refaz — `undo` e `redo` são a mesma operação em sentidos opostos, e não
      dois caminhos a manter coerentes. **205 testes** no kernel.
- [x] **MT-K1-10 concluído — bloco C fechado.** A mutação do **desenho** só é
      possível por `Document::edit()`, que devolve um `DocumentEditor` e registra
      a inversa de cada operação. As vias diretas viraram `pub(crate)`, e as três
      que permitiam mutação **não registrada** (`entity_mut`, `layer_mut`,
      `move_entity_to_block`) foram removidas. Quatro doctests `compile_fail`
      provam que a restrição é verificada pelo compilador, não apenas
      documentada. `CommandStack::edit` fechou a lacuna do MT-K1-09: criar
      entidade nova agora é desfazível. **215 testes** no kernel.
- [x] **`Change` migrou para o `neocad-model`.** Era a condição para o
      fechamento: `pub(crate)` não cruza crates, então o primitivo de mudança
      precisa morar junto do documento para que o compilador possa exigir o
      registro. `neocad-transaction` o reexporta e continua dono de `Transaction`
      e `CommandStack` — o que é coerente com o ADR 0003, que atribui a essa
      crate "command stack transacional, undo/redo".
- [x] **MT-K1-11 concluído — bloco D aberto.** `CadSession` em `neocad-wasm`
      expõe criar documento, listar camadas e entidades, caixa envolvente,
      desenhar linha, apagar entidade, alterar camada, `undo`/`redo` e estado do
      histórico. `wasm-pack build --target web` gera o pacote; **`.wasm` de
      107 KB**, valor a acompanhar no MT-K1-12. Identificadores atravessam a
      ponte como texto decimal (`EntityId::to_bits`), e não número, porque um
      `u64` viraria `BigInt` em JS sem nenhum ganho. `kernel/neocad-wasm/pkg`
      está no `.gitignore`.
- [x] **Lógica separada da fronteira JS.** `JsError` só pode ser construído
      dentro do WebAssembly — instanciá-lo no host entra em pânico, e foi o que
      quebrou os primeiros testes. A lógica ficou em um bloco interno que devolve
      `String`, e os métodos `#[wasm_bindgen]` apenas convertem. A fachada
      inteira, inclusive os caminhos de erro, é exercitável por `cargo test` sem
      navegador.
- [x] **MT-K1-12 concluído:** `scripts/build-kernel.mjs` gera o pacote em
      `src/lib/kernel/pkg`, encadeado em `pnpm dev` (perfil dev, mais rápido) e
      `pnpm build` (release), no mesmo padrão do `workers:sync` — saída derivada,
      nunca versionada. O script confere pré-requisitos com mensagem acionável e
      **pula a recompilação quando a fonte não mudou**, comparando mtime; sem
      isso, todo `pnpm build` (inclusive o que o Playwright dispara antes do E2E)
      recompilaria o kernel inteiro. Reporta o tamanho a cada build: **106,2 KB**.
      CI ganhou o passo de toolchain WASM nos jobs `frontend` e `e2e` — o segundo
      porque o `webServer` do Playwright roda `pnpm build`.
- [x] **MT-K1-13 concluído:** contratos NeoCAD do documento em
      `src/lib/types/cad.ts` (`CadLayer`, `CadEntity`, `CadGeometry`, `CadBounds`,
      `CadColor`, `CadHistoryState`, com `CadLayerId`/`CadEntityId` **branded**
      para o compilador recusar a troca entre eles) e `cad-document.ts` como
      única porta de acesso ao kernel. Import dinâmico, para o `.wasm` não entrar
      no bundle inicial. **41 testes** no frontend.
- [x] **Conversores puros, testáveis sem navegador.** O pacote `--target web`
      busca o `.wasm` por `fetch`, que não resolve `file://` no Node — carregar o
      kernel no Vitest não é viável. Os conversores foram escritos como funções
      puras sobre a forma que o kernel produz, então a tradução inteira, com
      caminhos de erro, roda em `pnpm test`.
- [x] **MT-K1-14 concluído:** extração do documento do upstream para o kernel.
      `buildDocumentSnapshot` lê camadas e entidades e produz um retrato; o
      kernel ganhou `CadSession::load`, que reconstrói o documento inteiro em uma
      só travessia da ponte e zera o histórico. 60 testes no frontend, 238 no
      kernel.
- [x] **Detecção por forma, não por `type`.** As declarações do upstream expõem
      `entity.type` como `string` mas **não declaram os valores** que ele assume;
      depender de constantes não declaradas seria adivinhação, e uma renomeação
      silenciosa lá viraria perda de entidades aqui. A conversão reconhece pela
      presença dos acessores característicos — e por isso é exercitável com
      objetos sintéticos, sem navegador. Cuidado necessário: arco tem centro e
      raio como o círculo, e precisa ser testado **antes** dele.
- [x] **Nada interrompe a abertura.** Entidades que o kernel não modela (hachura,
      cota, referência de bloco) entram em `unsupported` e são contadas;
      entidades de camada inexistente são contadas e ignoradas pelo kernel. Um
      arquivo real não deixa de abrir por causa de uma entidade que não cabe
      ainda — e a contagem vira a medida de quanto falta cobrir.
- [x] **MT-K1-15 concluído — o kernel chegou à interface.** Menu `Editar` com
      `Desfazer`/`Refazer` ligados ao command stack, desabilitados conforme o
      estado real da pilha e com o rótulo nomeando a ação a desfazer. A rota cria o
      `CadDocument`, carrega o desenho no kernel quando o
      upstream ativa um documento, e relê o histórico do kernel a cada ação — sem
      manter cópia derivada, que seria a forma mais fácil de o menu passar a
      mentir.
- [x] **Critério pendente do MT-K1-14 fechado.** Verificado em navegador real
      abrindo `e2e/fixtures/minimal.dxf`: a mensagem
      `Kernel: 2 entidade(s) em 1 camada(s)` bate exatamente com o conteúdo do
      arquivo (LINE + CIRCLE na camada `0`). É a primeira vez que o kernel
      WebAssembly roda no navegador.
- [x] **E2E destravado nesta máquina.** Correção de diagnóstico anterior: o
      Playwright **funciona** aqui — há navegador válido em
      `~/.cache/ms-playwright` (build 1228). O que falha é o `playwright install`
      deste projeto: a versão 1.60.0 pede o build 1223 e o download é recusado no
      Ubuntu 26.04. Outros projetos rodam porque o build de que precisam já está
      em cache. O `playwright.config.ts` passou a usar o Chrome do sistema fora
      da CI, condicionalmente; na CI mantém-se o navegador do próprio Playwright.
- [x] **MT-K1-16 reescrito e concluído.** A versão original exigia UI que nenhum
      ticket do K1 constrói — alterar camada pela interface é a Frente 1. O
      ticket passou a cobrir o que o K1 entrega, em
      `e2e/kernel-document.e2e.ts`: a contagem do kernel bate exatamente com
      `minimal.dxf`; a nova fixture `with-unsupported.dxf` (uma `LINE` mais uma
      `SOLID`) confirma que entidade não modelada é reportada **sem impedir a
      abertura**; o menu `Editar` mostra as duas ações desabilitadas após a
      carga; e o canvas continua com altura real, protegendo o caminho que já
      funcionava. Suíte E2E passou de 3 para **7 testes**.
- [x] **Cobertura de interação com o histórico migrada** para o primeiro ticket
      da **Frente 1 em modo escrita**, que é quem entrega o painel capaz de
      produzir a transação. Registrado no próprio MT-K1-16.
- [x] **MT-K1-17 concluído — documentação alinhada.** `docs/architecture.md`
      reescrito: o kernel passa a ser descrito como camada própria e fonte de
      verdade, com diagrama de camadas incluindo as seis crates, fluxo de
      abertura mostrando a carga no kernel depois da ativação pelo upstream, e
      riscos atualizados (escala do kernel, duas representações vivas,
      compatibilidade com arquivos reais). `docs/api.md` reescrito com os
      contratos do documento, a fronteira única e as invariantes do kernel. O
      aviso provisório que apontava para o ADR 0003 saiu.
- [ ] **Fronteira do ADR 0001 ainda não é executável.** Verifiquei por `grep` que
      nenhum componente ou rota importa `$lib/kernel`, mas isso passa
      trivialmente hoje e apodrece. Uma regra `no-restricted-imports` no
      `eslint.config.js`, cobrindo `$lib/kernel` **e** `@mlightcad/*` em
      `src/lib/components/**` e `src/routes/**`, tornaria a diretriz do ADR 0001
      verificável. Vale um ticket próprio — ficou fora daqui por tocar a
      configuração de lint.
- [ ] **Pendência aberta pelo MT-K1-10:** a **estrutura** das tabelas de símbolos
      (criar, renomear e remover camada, bloco e estilo) segue pública e **não é
      reversível** — `Change` não tem variantes para isso. Criá-las exige
      restauração por identificador exato nas três tabelas, como a `Arena` já faz
      para entidades. Merece ticket próprio; ficou fora daqui porque o MT-K1-10
      declara "novas operações de edição" como fora de escopo.
- [ ] **Dívida reconhecida:** `LayerTable`, `BlockTable` e `TextStyleTable` têm
      hoje a mesma máquina de índice por nome (`by_normalized_name`, `create`,
      `rename`, `remove`, `id_of`, `iter`), diferindo apenas no tipo de registro e
      na mensagem de erro. É o conceito de **tabela de símbolos** que os formatos
      CAD já nomeiam. Vale um ticket de extração de `SymbolTable<T>` antes de
      surgir a quarta tabela (tipos de linha, em K2).
- [ ] **Aproximações registradas em código, a resolver depois:** a caixa de
      `Text` estima a largura por contagem de caracteres (superestimando de
      propósito), porque métricas de fonte só existem em K5; `Polyline` não
      modela abaulamento (`bulge`) de segmento, que entra em K3.
- [ ] **A decidir em MT-K1-08:** a `Arena` não implementa `PartialEq` de
      propósito. O critério de aceite de MT-K1-08 fala em "igualdade estrutural"
      para verificar que inverter um `Change` restaura o documento — mas desfazer
      uma inserção devolve o slot à lista de reuso e incrementa a geração, então
      o estado interno **não** volta a ser byte-a-byte o original. A definição de
      igualdade precisa ser semântica (conteúdo vivo), e cabe à crate de
      transações estabelecê-la.
- [x] **Fluxo de builds de teste:** `make dist-test` gera Linux e Windows numa só
      execução (`scripts/build-test.sh`). Validado em 2026-08-07 sobre Ubuntu
      26.04 LTS: o binário Linux inicia e permanece em execução sem erro no
      stderr, com WebKitGTK 2.52 — a incompatibilidade temida com a versão nova
      do WebKit não se confirmou. `.deb` de 5,6 MB também gerado.
- [ ] **Achado a documentar:** o cross-build Windows funciona também pela
      toolchain **MinGW** (`--target x86_64-pc-windows-gnu`), sem `cargo-xwin`,
      sem `llvm-rc` e sem privilégios de administrador — basta
      `x86_64-w64-mingw32-gcc` e o alvo no rustup. Gera `.exe` autossuficiente
      (só depende de DLLs do sistema mais o `WebView2Loader.dll` que o próprio
      build copia). Útil para builds locais de teste; o release oficial continua
      em MSVC. `docs/windows-cross-build.md` só descreve o caminho MSVC.
- [ ] **Em paralelo, sem bloqueio:** Frente 1 em **modo leitura** (painéis de
      camadas e propriedades). Ações de escrita ficam para depois de K1, porque
      edição sem undo é regressão de usabilidade.
- [ ] **Pendente do review:** CSP nula em `tauri.conf.json`, `baseUrl` remoto
      hardcoded no adaptador, cópia dupla de buffer em `cad-file.ts`, estado
      concentrado em `+page.svelte`, ausência de `CONTRIBUTING.md`/`SECURITY.md`.
- [ ] **Frente 1 (adiada até K1):** painéis de camadas/propriedades em modo
      leitura (`cad-layers.ts` via `layerTable.newIterator()`; `cad-selection.ts` via
      `selectionSet.events` + `getEntityById`; `WorkspaceSidebar`). API já confirmada
      no spike. Em seguida, Frente 3 (disparo de comandos básicos pela UI).

### Mapa da Frente 2 (para continuidade)

- Tipos: `CadCommandDescriptor`, `CadCommandCatalogItem` em `src/lib/types/cad.ts`.
- Adaptador: `NeoCadViewer.listCommandDescriptors()` (única fonte de runtime).
- Apresentação pura: `src/lib/config/cad-command-catalog.ts` (+ `.spec.ts`).
- Serviço/fronteira: `src/lib/services/cad-commands.ts`.
- UI: `HelpCommandsDialog.svelte`, fiado em `AppTopMenu.svelte` e `+page.svelte`.

---

## Histórico (mais recente no topo)

| Data       | Commit    | Resumo                                           | MT  |
| ---------- | --------- | ------------------------------------------------ | --- |
| 2026-06-03 | `6502246` | Governança alinhada às regras de IA + CHANGELOG  | —   |
| 2026-06-02 | `2e8853f` | Corrige canvas em branco (altura 0) + regressão  | —   |
| 2026-06-02 | `254cb2f` | Frente 2: catálogo de comandos em `Ajuda`        | —   |
| 2026-06-02 | `b986b4f` | Prettier na documentação e governança            | —   |
| 2026-06-02 | `d1e4dbd` | Governança de agentes + spike do upstream + ADR  | —   |
| 2026-05-21 | `65081dc` | Planejamento de painéis e comandos CAD (roadmap) | —   |
