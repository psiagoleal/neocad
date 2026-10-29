<!-- Caminho relativo: docs/tickets/hg-higiene.md -->

# HG — Higiene do repositório: micro-tickets

Correções de processo e documentação apontadas pelo
[review de 2026-10-29](../reviews/2026-10-29-review.md) (achados C1, C2, F2, F5–F11).
Nenhum ticket aqui muda comportamento do produto.

### MT-HG-01: Corrigir as ilhas do AGENTS.md

- **Objetivo:** as ilhas `linguagens-frameworks` e `comandos-exatos` passam a declarar a
  stack e os comandos reais do NeoCAD (Rust, TypeScript/Svelte, `make kernel-check`,
  `pnpm check/lint/test/test:e2e`, `make dist-test`); a ilha `adendos-seguranca` perde o
  fragmento colado; a subseção "8.1" da §10 é renumerada.
- **Arquivos no escopo:** `AGENTS.md` (só dentro dos marcadores `USER:BEGIN/END`).
- **Critério de aceite:** cada comando declarado na ilha foi executado nesta sessão com
  saída zero (skill `definir-stack`); `pnpm lint` verde.
- **Fora de escopo:** texto do framework fora das ilhas, que o `--update` regenera.
- **Depende de:** nenhum. Achados F2, F7.

### MT-HG-02: Enxugar o handoff

- **Objetivo:** `docs/CURRENT-STATE.md` volta a ser o estado corrente — último turno,
  próximo passo, impedimentos — e o histórico vai para `docs/handoff-arquivo.md`.
- **Arquivos no escopo:** `docs/CURRENT-STATE.md`, `docs/handoff-arquivo.md` (novo).
- **Critério de aceite:** `CURRENT-STATE.md` com menos de 150 linhas, citando os commits
  reescritos pelos hashes novos; nenhum conteúdo apagado (o arquivo recebe tudo);
  `checar-links.py` sem link quebrado nos dois arquivos.
- **Fora de escopo:** reescrever o histórico arquivado.
- **Depende de:** nenhum. Achado F5.

### MT-HG-03: Atualizar o README e consolidar o roteiro

- **Objetivo:** o README descreve o NeoCAD de hoje (kernel próprio, fases K1–K3 e KL
  feitas, KV/UI/UX/KP a seguir) e o roteiro aponta para os ADRs 0003 e 0007 em vez das
  "Fases 0–5"; os documentos de roteiro antigos ganham aviso de documento histórico no
  topo.
- **Arquivos no escopo:** `README.md`, `docs/cad-panels-commands-simulation-roadmap.md`,
  `docs/frontend-workspace-refactor.md`, `docs/development.md`.
- **Critério de aceite:** `pnpm lint` verde; `checar-links.py README.md` sem link quebrado;
  nenhuma menção a "Fase 2" como estado atual.
- **Fora de escopo:** apagar os documentos antigos.
- **Depende de:** nenhum. Achado F6.

### MT-HG-04: Corrigir o mapa de arquitetura

- **Objetivo:** o diagrama de `docs/architecture.md` mostra as arestas reais entre as
  crates e o estado de `neocad-topology` (reservada para K8, sem consumidor); a seção de
  licenciamento diz onde as dependências GPL estão de fato (frontend, via upstream).
- **Arquivos no escopo:** `docs/architecture.md`.
- **Critério de aceite:** cada aresta do diagrama confere com `[dependencies]` dos
  `Cargo.toml` do kernel; `pnpm lint` verde.
- **Fora de escopo:** remover `neocad-topology`.
- **Depende de:** nenhum. Achados C1, C2, B5.

### MT-HG-05: Decidir o `.codex/` e a regra do CHANGELOG

- **Objetivo:** `.codex/` entra no `.gitignore` (configuração local de ferramenta, como
  `.claude/settings.local.json`); o preâmbulo do `CHANGELOG.md` fixa uma linha por mudança,
  com o porquê longo no commit ou no ADR.
- **Arquivos no escopo:** `.gitignore`, `CHANGELOG.md` (preâmbulo e entrada desta fase).
- **Critério de aceite:** `git status` não lista `.codex/`; `pnpm lint` verde.
- **Fora de escopo:** reescrever as entradas antigas do CHANGELOG.
- **Depende de:** nenhum. Achados F8, F10.

### MT-HG-06: Pôr o cabeçalho de caminho relativo nos arquivos que faltam

- **Objetivo:** todo arquivo de código versionado (`.rs`, `.ts`, `.svelte`, `.mjs`) tem o
  comentário `Caminho relativo` nas três primeiras linhas (AGENTS.md §4).
- **Arquivos no escopo:** os 15 arquivos listados pelo comando do critério.
- **Critério de aceite:** `git ls-files '*.rs' '*.ts' '*.svelte' '*.mjs' | while read f; do head -3 "$f" | grep -q "Caminho relativo" || echo "$f"; done`
  sem saída; `make kernel-check`, `pnpm check`, `pnpm lint` verdes.
- **Fora de escopo:** arquivos gerados (`src/lib/kernel/pkg`).
- **Depende de:** nenhum. Achado F9.

### MT-HG-07: Varredura de segredos local antes do commit

- **Objetivo:** hook `pre-commit` que roda `gitleaks protect --staged`, complementando a
  varredura que já existe na CI (AGENTS.md §7).
- **Arquivos no escopo:** `scripts/hooks/pre-commit` (novo), instrução em
  `docs/development.md`.
- **Critério de aceite:** um commit de teste com segredo falso é recusado pelo hook e um
  commit limpo passa.
- **Fora de escopo:** trocar a varredura da CI.
- **Depende de:** `gitleaks` instalado na máquina pelo mantenedor (não está hoje). Achado F11.
