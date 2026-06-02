<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> Opcional em projetos solo; recomendado em colaborações. Atualizado a cada commit.
> Não inclua segredos. Mantido conforme a skill `handoff-updater`.

## Último turno

- **Data:** 2026-06-02
- **Branch:** `master`
- **Commit:** `d1e4dbd` (governança + spike) — Frente 2 nesta árvore, ainda **não commitada**

## Metas cumpridas / Em andamento / Próximo passo

- [x] Spike técnico do upstream `@mlightcad` documentado em
      `docs/upstream-capabilities-spike.md`; ADR 0001 **Accepted**.
- [x] Governança de agentes versionada no commit `d1e4dbd`.
- [x] **Frente 2 implementada:** catálogo de comandos derivado em runtime do command
      stack, exposto em `Ajuda > Comandos CAD` (diálogo filtrável por categoria).
      Verificado: `pnpm check` (0/0), `pnpm test` (13/13), `pnpm lint` verde.
- [ ] **Próximo passo sugerido:** Frente 1 — painéis de camadas/propriedades em modo
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

| Data       | Commit            | Resumo                                           | MT  |
| ---------- | ----------------- | ------------------------------------------------ | --- |
| 2026-06-02 | _(não commitado)_ | Frente 2: catálogo de comandos em `Ajuda`        | —   |
| 2026-06-02 | `d1e4dbd`         | Governança de agentes + spike do upstream + ADR  | —   |
| 2026-05-21 | `65081dc`         | Planejamento de painéis e comandos CAD (roadmap) | —   |
