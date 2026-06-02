<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> Opcional em projetos solo; recomendado em colaborações. Atualizado a cada commit.
> Não inclua segredos. Mantido conforme a skill `handoff-updater`.

## Último turno

- **Data:** 2026-06-02
- **Branch:** `master`
- **Commit:** `65081dc` (último commitado) — alterações deste turno ainda **não commitadas**

## Metas cumpridas / Em andamento / Próximo passo

- [x] Spike técnico do upstream `@mlightcad` concluído e documentado em
  `docs/upstream-capabilities-spike.md`.
- [x] Decisão arquitetural registrada em
  `docs/adr/0001-catalogo-dinamico-e-paineis-sobre-data-model.md` (status `Proposed`).
- [ ] **Em andamento:** scaffold de governança de agentes (`AGENTS.md`, `CLAUDE.md`,
  `.claude/`, `skills/`, `.github/`, `docs/adr/`, etc.) ainda **não rastreado** no git.
- [ ] **Próximo passo sugerido:** aceitar o ADR 0001 e iniciar a Frente 2 (catálogo de
  comandos derivado do `AcEdCommandStack` + menu `Ajuda > Comandos CAD`) ou a Frente 1
  (painéis de camadas/propriedades em modo leitura). API confirmada para ambas.

### Achados-chave do spike (desbloqueiam Frentes 1–3)

- Comandos enumeráveis em runtime via `docManager.commandManager.iterator()` /
  `searchCommandsByPrefix`; ~31 comandos reais (desenho/edição/camadas).
- Camadas em `docManager.curDocument.database.tables.layerTable` (iterável, com setters).
- Seleção em `docManager.curView.selectionSet` (+ eventos); entidade via
  `database.tables.blockTable.getEntityById(id)`; `entity.properties` pronto para UI.

---

## Histórico (mais recente no topo)

| Data | Commit | Resumo | MT |
|------|--------|--------|----|
| 2026-06-02 | _(não commitado)_ | Spike do upstream + ADR 0001 + handoff | — |
| 2026-05-21 | `65081dc` | Planejamento de painéis e comandos CAD (roadmap) | — |
