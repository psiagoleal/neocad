<!-- Caminho relativo: docs/specs/03-layouts.md -->

# 03 — Layouts de espaço-papel

- **Estado:** Vigente
- **Fase:** KL (concluída) · **ADRs:** 0005 · **Transversais:** 00

## Problema e resultado esperado

O entregável de engenharia é a prancha composta no espaço-papel — carimbo, escalas e
viewports —, e não o desenho solto no espaço-modelo. O NeoCAD abre, exibe, modela e grava
layouts como o AutoCAD, e o usuário troca de layout pelas abas.

## Escopo

Tabela de layouts com ordem de abas; espaço de cada layout; entidade `VIEWPORT` com
conversão papel ↔ modelo; congelamento de camada por viewport; configuração de página
(`PageSetup`) como dado; leitura e gravação DXF de tudo isso; troca de layout exibido.

## Fora de escopo

Plotagem e PDF (KP7). Viewport poligonal ou recortada por entidade, se não estiver no
modelo hoje.

## Critérios de aceite (EARS)

| id  | critério                                                                                                                                       | prova                                                                                                                                                                                                                 |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| L1  | O documento DEVE ter sempre o espaço-modelo, que não pode ser removido nem renomeado, e a ordem das abas DEVE ser determinística.              | `cargo test --manifest-path kernel/Cargo.toml -p neocad-model a_aba_do_espaco_modelo` + `cargo test --manifest-path kernel/Cargo.toml -p neocad-io dxf::build`; **falta** teste direto de `LayoutTable::in_tab_order` |
| L2  | QUANDO um layout é removido, as entidades do espaço dele DEVEM ser removidas na mesma transação.                                               | `cargo test --manifest-path kernel/Cargo.toml -p neocad-model remover_layout_leva_junto`                                                                                                                              |
| L3  | A conversão de coordenada entre papel e modelo por um viewport DEVE ser inversível, inclusive com rotação.                                     | `cargo test --manifest-path kernel/Cargo.toml -p neocad-model viewport::`                                                                                                                                             |
| L4  | A visibilidade de uma camada DEVE considerar o congelamento por viewport.                                                                      | `cargo test --manifest-path kernel/Cargo.toml -p neocad-model viewport::`                                                                                                                                             |
| L5  | QUANDO um DXF com layouts e viewports é lido e gravado, a releitura DEVE trazer os mesmos layouts, viewports e entidades por espaço.           | `cargo test --manifest-path kernel/Cargo.toml -p neocad-io --test round_trip`                                                                                                                                         |
| L6  | QUANDO o usuário troca de layout, o desenho exibido DEVE mudar para o conteúdo daquele layout.                                                 | `pnpm test:e2e` (fixture `two-layouts.dxf`)                                                                                                                                                                           |
| L7  | QUANDO o espaço-modelo está vazio e há conteúdo em layout, a interface DEVE dizer em que layout o desenho está, sem deixar o canvas em branco. | `pnpm test:e2e`                                                                                                                                                                                                       |
