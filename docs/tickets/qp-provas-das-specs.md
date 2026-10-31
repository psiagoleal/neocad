<!-- Caminho relativo: docs/tickets/qp-provas-das-specs.md -->

# QP — Provas das specs: micro-tickets

Provas marcadas "a criar" nas specs [00](../specs/00-transversal.md),
[02](../specs/02-dxf.md), [03](../specs/03-layouts.md) e [06](../specs/06-fachada-wasm.md),
para comportamento que já existe ou é regra transversal. Cada ticket transforma uma linha
de spec em comando que falha quando a regra é violada.

### MT-QP-01: Ida e volta DXF por variante de geometria (T5)

- **Objetivo:** um teste de ida e volta por variante de `Geometry`, escrito sobre `match`
  exaustivo, de modo que variante nova sem leitura e gravação faça o teste deixar de
  compilar.
- **Arquivos no escopo:** `kernel/neocad-io/tests/round_trip.rs`.
- **Critério de aceite:** `make kernel-check` verde; acrescentar uma variante fictícia em
  rascunho local faz o teste não compilar (verificado e descartado).
- **Fora de escopo:** novas variantes.
- **Depende de:** nenhum.

### MT-QP-02: Guarda de tolerância e de licenças do kernel (T6, T7)

- **Objetivo:** script que falha com literal de tolerância fora de `tolerance.rs` e
  `predicate.rs` em código de produção, e com dependência copyleft ou de navegador fora
  das exceções da spec 00; os dois entram em `make kernel-check` e na CI.
- **Arquivos no escopo:** `scripts/check-kernel-rules.mjs` (novo), `Makefile`,
  `.github/workflows/ci.yml`.
- **Critério de aceite:** o script passa hoje; um literal `1e-9` posto em rascunho num
  arquivo de produção o faz falhar (verificado e descartado); `make kernel-check` verde.
- **Fora de escopo:** mudar a tolerância.
- **Depende de:** nenhum.

### MT-QP-03: Origem das fixtures (T10)

- **Objetivo:** `e2e/fixtures/ORIGEM.md` lista cada fixture com origem e licença, e um
  teste falha se houver fixture sem linha.
- **Arquivos no escopo:** `e2e/fixtures/ORIGEM.md` (novo), `scripts/check-fixtures.mjs`
  (novo), `package.json`.
- **Critério de aceite:** `pnpm test` verde; uma fixture sem linha faz o teste falhar.
- **Fora de escopo:** fixtures novas.
- **Depende de:** nenhum.

### MT-QP-04: E2E do aviso de perda ao salvar (T4)

- **Objetivo:** E2E que abre `with-unsupported.dxf`, pede salvar, confere o diálogo com
  tipos e quantidades e confere que cancelar não grava nada.
- **Arquivos no escopo:** `e2e/save-loss.e2e.ts` (novo).
- **Critério de aceite:** `pnpm test:e2e -- save-loss` verde.
- **Fora de escopo:** mudar o texto do diálogo além do que o critério exigir.
- **Depende de:** nenhum.

### MT-QP-05: Teste direto da ordem das abas (03, L1)

- **Objetivo:** testes de unidade de `LayoutTable::in_tab_order`, incluindo empate de
  `tab_order` desfeito pelo nome.
- **Arquivos no escopo:** `kernel/neocad-model/src/layout.rs`.
- **Critério de aceite:** `cargo test -p neocad-model layout::` executa mais de zero testes,
  todos verdes.
- **Fora de escopo:** mudar a ordem.
- **Depende de:** nenhum.

### MT-QP-06: Gravar DXF AC1032 (02, X7)

- **Objetivo:** a gravação passa a declarar `$ACADVER = AC1032`, com o que a versão exige
  de cabeçalho e objetos, e o arquivo gravado abre sem erro num leitor de referência de
  código aberto escolhido no ticket.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/writer/`.
- **Critério de aceite:** teste novo em `neocad-io`; `make kernel-check` verde; validação
  com o leitor de referência registrada no handoff.
- **Fora de escopo:** gravar outras versões.
- **Depende de:** nenhum.

### MT-QP-07: Corpus DXF de origem pública por versão (02, X8; review G4)

- **Objetivo:** corpus pequeno de arquivos DXF gerados por outros programas, de R12 a
  AC1032, de origem pública e licença compatível, lido em teste.
- **Arquivos no escopo:** `kernel/neocad-io/tests/corpus/` (novo), `e2e/fixtures/ORIGEM.md`.
- **Critério de aceite:** um teste por versão lê o arquivo sem erro; licenças conferidas.
- **Fora de escopo:** DWG.
- **Depende de:** MT-QP-03.

### MT-QP-08: Limite de tamanho do `.wasm` (06, W5)

- **Objetivo:** o build do kernel falha se o `.wasm` de release passar de 465 000 bytes.
- **Arquivos no escopo:** `scripts/build-kernel.mjs`.
- **Critério de aceite:** `pnpm kernel:build` verde hoje; com o limite rebaixado em
  rascunho, falha (verificado e descartado).
- **Fora de escopo:** reduzir o tamanho.
- **Depende de:** nenhum.

### MT-QP-09: Testes de propriedade da geometria (review B4)

- **Objetivo:** testes baseados em propriedade para interseção, paralela e transformação:
  ponto de interseção está sobre as duas curvas, espelhar duas vezes devolve o original,
  paralela fica à distância pedida.
- **Arquivos no escopo:** `kernel/neocad-geometry/Cargo.toml` (`proptest` como
  dependência de desenvolvimento), `kernel/neocad-geometry/tests/propriedades.rs` (novo).
- **Critério de aceite:** `make kernel-check` verde; licença do `proptest` (MIT/Apache)
  conferida.
- **Fora de escopo:** corrigir o que os testes acharem — cada achado vira ticket próprio.
- **Depende de:** nenhum.
