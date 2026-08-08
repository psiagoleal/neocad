<!-- Caminho relativo: docs/tickets/k1-modelo-documento-transacoes.md -->

# K1 — Modelo de documento e transações: micro-tickets

Quebra da fase **K1** do [ADR 0003](../adr/0003-kernel-cad-proprio.md) conforme a
skill `micro-ticket-planner`. Cada ticket tem uma saída verificável, escopo de
arquivos fechado e cabe em um ciclo limpo de contexto.

## Resultado esperado de K1

Ao fim da fase, o NeoCAD tem **modelo de documento próprio em Rust**, exposto ao
frontend por WebAssembly, com **toda mutação passando por transação reversível** e
`Desfazer`/`Refazer` funcionando na interface. O upstream continua abrindo e
desenhando os arquivos — a substituição do parser e do renderer é K5/K6, não esta
fase.

## Restrições que valem para todos os tickets

- Nenhuma crate de `kernel/` referencia Tauri, Svelte, DOM ou tipos do NeoCAD,
  exceto `neocad-wasm` (ADR 0003).
- Nenhuma crate do kernel além de `neocad-io` recebe dependência copyleft
  (ADR 0003).
- Componentes Svelte e rotas não importam tipos do kernel nem do upstream; apenas
  contratos NeoCAD (ADR 0001).
- Toda mutação do documento ocorre por transação — não existe setter público que
  escape do journal.

## Definição de pronto (por ticket)

- os scripts de teste e linter do `AGENTS.md` passam;
- `docs/CURRENT-STATE.md` atualizado (skill `handoff-updater`);
- revisão de PR feita (skill `pr-review-guard`).

---

## Bloco A — Fundação do workspace

### MT-K1-01: Criar o workspace Rust `kernel/` com as crates vazias

- **Objetivo:** workspace Cargo em `kernel/` com as seis crates do ADR 0003
  compilando vazias, independente do workspace de `src-tauri/`.
- **Arquivos no escopo:** `kernel/Cargo.toml`, `kernel/rust-toolchain.toml`,
  `kernel/neocad-geometry/{Cargo.toml,src/lib.rs}`,
  `kernel/neocad-topology/{Cargo.toml,src/lib.rs}`,
  `kernel/neocad-model/{Cargo.toml,src/lib.rs}`,
  `kernel/neocad-transaction/{Cargo.toml,src/lib.rs}`,
  `kernel/neocad-io/{Cargo.toml,src/lib.rs}`,
  `kernel/neocad-wasm/{Cargo.toml,src/lib.rs}`, `.gitignore`.
- **Critério de aceite:** `cargo test --manifest-path kernel/Cargo.toml --all` e
  `cargo clippy --manifest-path kernel/Cargo.toml --all-targets -- -D warnings`
  passam.
- **Fora de escopo:** qualquer tipo de domínio; integração com o frontend; ligação
  com `src-tauri`.
- **Depende de:** ADR-0003.

### MT-K1-02: Adicionar o workspace do kernel à CI

- **Objetivo:** job `kernel` no workflow, com `fmt`, `check`, `clippy` e `test`
  sobre `kernel/`, em Linux e Windows.
- **Arquivos no escopo:** `.github/workflows/ci.yml`, `Makefile`.
- **Critério de aceite:** o job aparece verde na primeira execução do workflow;
  `make kernel-check` reproduz o mesmo conjunto localmente.
- **Fora de escopo:** build WASM (MT-K1-11); cobertura de testes.
- **Depende de:** MT-K1-01.
- **Atenção (constatado em MT-K1-01):** o `kernel/rust-toolchain.toml` só é
  aplicado quando o diretório de trabalho está **dentro** de `kernel/`. Rodar
  `cargo … --manifest-path kernel/Cargo.toml` a partir da raiz ignora o arquivo e,
  com ele, a instalação automática do alvo `wasm32-unknown-unknown`. O job da CI
  deve usar `working-directory: kernel` em vez de `--manifest-path`.

---

## Bloco B — Modelo de documento (`neocad-model`)

### MT-K1-03: Implementar identificadores e arena de entidades

- **Objetivo:** `EntityId` estável e opaco, com arena geracional que detecta uso
  após remoção.
- **Arquivos no escopo:** `kernel/neocad-model/src/{lib.rs,id.rs,arena.rs}`.
- **Critério de aceite:** `cargo test -p neocad-model` cobre inserção, remoção,
  reuso de slot e rejeição de id obsoleto.
- **Fora de escopo:** entidades geométricas; serialização; tabelas de símbolos.
- **Depende de:** MT-K1-01.

### MT-K1-04: Implementar a tabela de camadas

- **Objetivo:** `LayerTable` com registro de camada (nome, cor, visível,
  congelada, bloqueada, tipo de linha, espessura), busca por nome e iteração
  determinística.
- **Arquivos no escopo:** `kernel/neocad-model/src/{lib.rs,layer.rs}`.
- **Critério de aceite:** `cargo test -p neocad-model` cobre criação, nome
  duplicado rejeitado, camada `0` sempre presente e ordem estável de iteração.
- **Fora de escopo:** mutação transacional (MT-K1-09); UI.
- **Depende de:** MT-K1-03.

### MT-K1-05: Implementar as entidades de desenho 2D mínimas

- **Objetivo:** enum de entidade cobrindo `Line`, `Circle`, `Arc`, `Polyline` e
  `Text`, cada uma com camada, cor e caixa envolvente.
- **Arquivos no escopo:** `kernel/neocad-model/src/{lib.rs,entity.rs}`,
  `kernel/neocad-geometry/src/{lib.rs,point.rs,aabb.rs}`.
- **Critério de aceite:** `cargo test -p neocad-model -p neocad-geometry` cobre
  a caixa envolvente de cada tipo, incluindo arco cruzando os eixos.
- **Fora de escopo:** interseções, offset e demais operações de K3; blocos;
  renderização.
- **Depende de:** MT-K1-03.

### MT-K1-06: Implementar as tabelas de blocos e estilos de texto

- **Objetivo:** `BlockTable` com espaço-modelo como bloco raiz, e `TextStyleTable`
  com estilo padrão.
- **Arquivos no escopo:** `kernel/neocad-model/src/{lib.rs,block.rs,text_style.rs}`.
- **Critério de aceite:** `cargo test -p neocad-model` cobre a existência do
  espaço-modelo, inserção de bloco e resolução de referência de estilo.
- **Fora de escopo:** inserção de referência de bloco (`INSERT`) na UI;
  transformação de instância.
- **Depende de:** MT-K1-04, MT-K1-05.

### MT-K1-07: Compor o documento agregando arena e tabelas

- **Objetivo:** tipo `Document` reunindo arena de entidades e tabelas de símbolos,
  com construtor de documento vazio válido.
- **Arquivos no escopo:** `kernel/neocad-model/src/{lib.rs,document.rs}`.
- **Critério de aceite:** `cargo test -p neocad-model` cobre documento vazio com
  camada `0` e espaço-modelo, e consulta de entidades por camada.
- **Fora de escopo:** mutação; leitura de arquivo; WASM.
- **Depende de:** MT-K1-06.

---

## Bloco C — Transações (`neocad-transaction`)

### MT-K1-08: Implementar o journal de mudanças reversíveis

- **Objetivo:** `Change` descrevendo mutação atômica (adicionar, remover,
  substituir entidade; alterar registro de camada) com aplicação e inversão
  exatas.
- **Arquivos no escopo:**
  `kernel/neocad-transaction/src/{lib.rs,change.rs}`, `kernel/neocad-transaction/Cargo.toml`.
- **Critério de aceite:** `cargo test -p neocad-transaction` demonstra que
  aplicar e inverter qualquer `Change` restaura o documento ao estado inicial,
  verificado por igualdade estrutural.
- **Fora de escopo:** agrupamento em comandos; pilha de undo; API pública de
  mutação.
- **Depende de:** MT-K1-07.
- **Escopo revisado na execução (2026-08-07):** o critério de aceite exigia
  inversão exata, e isso é **impossível** apenas com a API que o MT-K1-07
  entregou. Desfazer uma remoção precisa devolver a entidade com o **mesmo
  identificador** — senão seleções, referências entre entidades e as demais
  mudanças da mesma transação passam a apontar para o nada — e na **mesma posição
  da ordem de desenho**, senão muda silenciosamente quem é desenhado por cima de
  quem. Foi necessário acrescentar ao `neocad-model`: `Arena::insert_at`
  (restauração em identificador exato, com religamento da lista de reuso),
  `BlockRecord::insert_entity_at`/`position_of`, e em `Document`
  `entity_placement`, `restore_entity`, `replace_entity`, `set_layer_record` e
  `PartialEq` semântica. Os tickets seguintes que dependem de inversão exata já
  contam com essas primitivas.

### MT-K1-09: Implementar o command stack com undo/redo

- **Objetivo:** `Transaction` agrupando `Change`s em unidade nomeada e
  `CommandStack` com `undo()`, `redo()`, limite configurável e descarte do ramo
  de redo ao surgir nova transação.
- **Arquivos no escopo:**
  `kernel/neocad-transaction/src/{lib.rs,transaction.rs,stack.rs}`.
- **Critério de aceite:** `cargo test -p neocad-transaction` cobre undo/redo
  encadeados, descarte do ramo de redo, limite da pilha e transação vazia que não
  entra na pilha.
- **Fora de escopo:** exposição ao frontend; nomes de comando na UI.
- **Depende de:** MT-K1-08.

### MT-K1-10: Fechar a mutação do documento atrás da transação

- **Objetivo:** tornar privados os caminhos de mutação direta de `Document` e
  expor mutação exclusivamente por `Transaction`, conforme a diretriz do ADR 0003.
- **Arquivos no escopo:** `kernel/neocad-model/src/{document.rs,entity.rs,layer.rs}`,
  `kernel/neocad-transaction/src/transaction.rs`.
- **Critério de aceite:** `cargo test --manifest-path kernel/Cargo.toml --all`
  passa e nenhum teste consegue mutar o documento sem transação; um teste de
  compilação negativa (`trybuild` ou equivalente) registra a restrição.
- **Fora de escopo:** novas operações de edição; performance.
- **Depende de:** MT-K1-09.
- **Escopo revisado na execução (2026-08-07):** o ticket previa apenas mudar
  visibilidade, mas `pub(crate)` **não cruza crates** — e `Document` e
  `Transaction` vivem em crates diferentes. Para que o compilador pudesse exigir
  o registro, o primitivo `Change` teve de migrar de `neocad-transaction` para
  `neocad-model`, junto do documento; `neocad-transaction` passou a reexportá-lo
  e seguiu dono de `Transaction` e `CommandStack`. Foram acrescentados
  `Document::edit()` e `DocumentEditor` (única via pública de mutação),
  `Change::MoveEntity` e `Document::set_entity_placement` (para que mover
  entidade também seja reversível de forma exata), e `CommandStack::edit`, que
  resolve a lacuna registrada no MT-K1-09 — criar entidade nova exige que o
  documento emita o identificador, o que uma `Transaction` pré-montada não
  alcança.
- **Pendência deixada em aberto:** a estrutura das tabelas de símbolos (criar,
  renomear, remover camada/bloco/estilo) segue pública e irreversível, por
  faltarem variantes de `Change` e restauração por identificador exato nas três
  tabelas. Requer ticket próprio.

---

## Bloco D — Ponte com o frontend

### MT-K1-11: Expor a fachada WebAssembly do kernel

- **Objetivo:** `neocad-wasm` com `wasm-bindgen` expondo criar documento, listar
  camadas, listar entidades, executar transação, `undo`, `redo` e consultar o
  estado da pilha.
- **Arquivos no escopo:** `kernel/neocad-wasm/{Cargo.toml,src/lib.rs}`.
- **Critério de aceite:** `wasm-pack build kernel/neocad-wasm --target web` gera o
  pacote sem erro e `cargo clippy` do workspace permanece limpo.
- **Fora de escopo:** integração com o Vite (MT-K1-12); tipos TypeScript à mão.
- **Depende de:** MT-K1-10.

### MT-K1-12: Integrar o build WASM ao pipeline do frontend

- **Objetivo:** script `kernel:build` gerando o pacote em `src/lib/kernel/pkg`,
  encadeado em `dev` e `build` no mesmo padrão de `workers:sync`, com a saída fora
  do versionamento.
- **Arquivos no escopo:** `package.json`, `scripts/build-kernel.mjs`, `.gitignore`,
  `.prettierignore`, `eslint.config.js`, `.github/workflows/ci.yml`, `Makefile`.
- **Critério de aceite:** `pnpm build` gera o pacote e conclui; `pnpm lint` e
  `pnpm check` permanecem verdes; a CI instala a toolchain WASM e reproduz o
  build.
- **Fora de escopo:** uso do kernel pela UI (MT-K1-13).
- **Depende de:** MT-K1-11.

### MT-K1-13: Definir os contratos NeoCAD do documento e o serviço de fronteira

- **Objetivo:** tipos NeoCAD para documento, camada, entidade e estado da pilha em
  `src/lib/types/`, e serviço `cad-document.ts` como única porta de acesso ao
  kernel, conforme o ADR 0001.
- **Arquivos no escopo:** `src/lib/types/cad.ts`, `src/lib/services/cad-document.ts`,
  `src/lib/services/cad-document.spec.ts`.
- **Critério de aceite:** `pnpm test` cobre a conversão dos tipos do kernel para
  contratos NeoCAD; `pnpm check` sem erros; nenhum componente importa
  `$lib/kernel`.
- **Fora de escopo:** ligação com o documento aberto pelo upstream (MT-K1-14); UI.
- **Depende de:** MT-K1-12.

### MT-K1-14: Popular o modelo próprio a partir do documento aberto pelo upstream

- **Objetivo:** ao ativar um documento, converter camadas e entidades do
  `@mlightcad/data-model` para o modelo do kernel, mantendo o upstream como fonte
  de renderização.
- **Arquivos no escopo:** `src/lib/viewer/neocad-viewer.ts`,
  `src/lib/services/cad-document.ts`, `src/lib/types/cad.ts`.
- **Critério de aceite:** `pnpm test` cobre a conversão com um documento sintético;
  ao abrir `e2e/fixtures/minimal.dxf`, a contagem de entidades e de camadas do
  modelo próprio coincide com a do upstream.
- **Fora de escopo:** substituir o parser do upstream (K6); entidades além das de
  MT-K1-05, que devem ser registradas como não suportadas sem quebrar a abertura.
- **Depende de:** MT-K1-13.

---

## Bloco E — Interface e fechamento

### MT-K1-15: Adicionar o menu `Editar` com `Desfazer` e `Refazer`

- **Objetivo:** entrada `Editar` no menu superior, com as duas ações ligadas ao
  command stack, desabilitadas quando não há o que desfazer ou refazer, e rótulo
  exibindo o nome da transação.
- **Arquivos no escopo:** `src/lib/components/workspace/AppTopMenu.svelte`,
  `src/routes/+page.svelte`, `src/lib/styles/components.css`.
- **Critério de aceite:** `pnpm check` e `pnpm lint` verdes; as ações refletem o
  estado real da pilha.
- **Fora de escopo:** atalhos de teclado; toolbar; comandos de edição novos.
- **Depende de:** MT-K1-14.

### MT-K1-16: Cobrir em E2E a chegada do desenho ao kernel

> **Reescrito em 2026-08-08.** A versão original mandava "executar uma alteração
> de camada, desfaz e refaz". Isso não é executável dentro do K1: exercitar
> undo/redo pela interface exige uma ação de edição na UI, e a única prevista é o
> painel de camadas — que pertence à **Frente 1** do roadmap, não ao K1. Nenhum
> ticket de K1 constrói essa superfície, e abrir um arquivo zera o histórico de
> propósito, então o menu `Editar` fica corretamente ocioso. O ticket passa a
> cobrir o que o K1 de fato entrega, e a cobertura de interação com o histórico
> migra para a Frente 1 (ver "Cobertura adiada", abaixo).

- **Objetivo:** teste E2E que abre uma fixture DXF e verifica que o modelo
  próprio recebeu o desenho, que a contagem bate com o conteúdo do arquivo, que
  entidades não modeladas são reportadas sem impedir a abertura, e que o menu
  `Editar` reflete o estado real da pilha.
- **Arquivos no escopo:** `e2e/kernel-document.e2e.ts`, `e2e/fixtures/`.
- **Critério de aceite:** `pnpm test:e2e` passa localmente e na CI, com quatro
  verificações:
  1. **Contagem bate com o arquivo.** Ao abrir `e2e/fixtures/minimal.dxf`, a
     mensagem do kernel reporta `2 entidade(s)` e `1 camada(s)` — exatamente a
     `LINE`, o `CIRCLE` e a camada `0` do arquivo. É a verificação que o
     MT-K1-14 exigia e não pôde fazer por falta de superfície observável.
  2. **Não suportado é reportado, e não interrompe.** Sobre uma fixture nova
     com uma entidade que o kernel ainda não modela — uma `SOLID` ou `ELLIPSE`
     ao lado de uma linha — a abertura conclui, as entidades modeláveis chegam
     ao kernel, e a contagem de não suportadas aparece na mensagem.
  3. **O menu `Editar` existe e diz a verdade.** Logo após abrir, `Desfazer` e
     `Refazer` estão desabilitados — o carregamento zera o histórico de
     propósito, e o menu tem de refletir isso em vez de oferecer ação inócua.
  4. **A abertura não regride.** O desenho continua sendo exibido pelo upstream;
     a chegada do kernel não pode ter quebrado o caminho que já funcionava.
- **Fora de escopo:** interação com `Desfazer`/`Refazer`; cobertura de todos os
  tipos de entidade; comparação da contagem do kernel com a do upstream lida em
  tempo de execução — a fixture é conhecida, e conferir contra ela é mais
  estável do que instrumentar o upstream.
- **Depende de:** MT-K1-15.

#### Cobertura adiada para a Frente 1

Exercitar `Desfazer`/`Refazer` pela interface — alterar uma camada, desfazer,
refazer, conferindo o estado visível a cada passo — continua sendo cobertura
necessária. Ela passa a ser critério de aceite do primeiro ticket da **Frente 1
em modo escrita**, que é quem entrega o painel de camadas capaz de produzir a
transação. O caminho de undo/redo em si já está coberto por 38 testes de unidade
em `neocad-transaction` e pelos testes da fachada em `neocad-wasm`; o que falta é
a ponta de interação, não a lógica.

#### Nota de ambiente

`pnpm test:e2e` usa o **Chrome do sistema** fora da CI: em Ubuntu 26.04 o
`playwright install` recusa baixar o build que esta versão do Playwright pede.
A escolha está em `playwright.config.ts` e é condicional — na CI, onde o runner é
uma release suportada, mantém-se o navegador do próprio Playwright.

### MT-K1-17: Atualizar a documentação de arquitetura e o handoff

- **Objetivo:** `docs/architecture.md` refletindo o kernel como camada própria, com
  o diagrama de camadas atualizado; `CHANGELOG.md` e `docs/CURRENT-STATE.md`
  registrando o encerramento de K1.
- **Arquivos no escopo:** `docs/architecture.md`, `docs/api.md`, `CHANGELOG.md`,
  `docs/CURRENT-STATE.md`.
- **Critério de aceite:** `pnpm lint` verde; nenhuma referência remanescente ao
  modelo do upstream como fonte de verdade.
- **Fora de escopo:** documentação das fases K2 em diante.
- **Depende de:** MT-K1-16.

---

## Ordem de execução

```text
MT-K1-01 → MT-K1-02
        ↘ MT-K1-03 → MT-K1-04 ↘
                   → MT-K1-05 → MT-K1-06 → MT-K1-07 → MT-K1-08 → MT-K1-09 → MT-K1-10
                                                                                    ↓
MT-K1-17 ← MT-K1-16 ← MT-K1-15 ← MT-K1-14 ← MT-K1-13 ← MT-K1-12 ← MT-K1-11 ←────────┘
```

MT-K1-02 é independente dos tickets de domínio e pode ser feito a qualquer momento
após MT-K1-01. Os demais são estritamente sequenciais dentro de cada bloco.

## Riscos conhecidos

- **MT-K1-14 é o ticket de maior incerteza.** A conversão depende da forma real do
  `@mlightcad/data-model`, já inspecionada em
  [`docs/upstream-capabilities-spike.md`](../upstream-capabilities-spike.md), mas
  entidades fora do conjunto mínimo de MT-K1-05 aparecerão em arquivos reais. O
  ticket deve registrá-las como não suportadas, nunca falhar a abertura.
- **Duas representações vivas.** Entre MT-K1-14 e K5/K6, o modelo próprio e o do
  upstream coexistem. Divergência entre eles é a classe de bug mais provável desta
  fase; a verificação de contagem no critério de aceite de MT-K1-14 é a primeira
  defesa.
- **Tamanho do binário WASM.** O pacote entra no bundle distribuído. Convém medir
  em MT-K1-12 e registrar o valor, para que o crescimento seja acompanhado desde o
  início.
