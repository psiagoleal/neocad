<!-- Caminho relativo: docs/specs/10-desenho-basico.md -->

# 10 — Comandos de desenho básicos

- **Estado:** Parcial — `LINE` e a entrada de pontos com critérios (2026-11-02); os demais
  comandos fecham os seus quando entrarem
- **Fase:** UI/UX · **ADRs:** 0007 · **Transversais e relacionadas:** 05, 07, 08

## Problema e resultado esperado

O usuário precisa criar a geometria que o kernel já modela — reta, circunferência, arco, polilinha — pela linha de comando, com os pontos pedidos no prompt, prendimento e pré-visualização, como no AutoCAD.

## Escopo

Criação interativa de reta, circunferência (centro e raio, diâmetro, 2 e 3 pontos), arco (3 pontos e variantes usuais), polilinha com segmentos de arco, retângulo e polígono; entrada de coordenada absoluta, relativa (`@x,y`) e polar (`@d<a`); apagar.

## Fora de escopo

Elipse, spline, ponto, linha infinita (KP2). Rastreamento polar, ortogonal e entrada dinâmica (spec própria de UI/UX).

## Comandos envolvidos

Grafia oficial de cada idioma, de [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv).

| comando EN | comando PT-BR | apelido |
| ---------- | ------------- | ------- |
| `LINE`     | `LINHA`       | `L`     |
| `CIRCLE`   | `CIRCULO`     | `C`     |
| `ARC`      | `ARC`         | `A`     |
| `PLINE`    | `PLINHA`      | `PL`    |
| `RECTANG`  | `RETANGULO`   | `REC`   |
| `POLYGON`  | `POLIGONO`    | `POL`   |
| `ERASE`    | `APAGAR`      | `E`     |

## Restrições já tomadas

- Toda entidade nova é lida e gravada em DXF no mesmo ticket (00, T5).
- Todo comando aceita os dois idiomas e o apelido (spec 08).

- **Coordenadas como no AutoCAD:** ponto é o separador decimal e vírgula separa as
  coordenadas (`1.5,2`), em qualquer idioma. Decidido em 2026-11-02, pela paridade com o
  AutoCAD (spec 08). É o padrão.
- **Segundo modo, com vírgula decimal** (decidido em 2026-11-02 com o mantenedor; a
  implementar no [MT-UI-08](../tickets/ui-interface.md#mt-ui-08-modo-de-vírgula-decimal)):
  uma configuração explícita troca o decimal para a vírgula e o separador para o ponto e
  vírgula (`1,5;2`). O ponto e vírgula vale como separador também no modo padrão
  (`1.5;2`). A configuração rege também a saída: distâncias e coordenadas nas mensagens e
  na barra de status. Um valor único (distância, raio, fator) usa o decimal do modo.
- **Camada corrente:** até a spec de camadas (KP1), a reta nova vai para a camada `0`, ou
  para a primeira, se não houver `0`.

## Critérios de aceite (EARS) — `LINE` e entrada de pontos

| id  | critério                                                                                                                                                                                                                              | prova                                                              |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| D1  | QUANDO o `LINE` começa, a linha DEVE pedir o primeiro ponto; cada ponto confirmado depois dele DEVE criar, pelo kernel, uma reta do ponto anterior até ele, na camada corrente.                                                       | `pnpm test command-line` (`linha.spec.ts`)                         |
| D2  | Um ponto DEVE ser aceito como `x,y` (absoluto), `@dx,dy` (relativo ao último ponto), `d<a` (polar absoluto) ou `@d<a` (polar relativo), com o ângulo em graus no sentido anti-horário a partir de +X; espaços em volta são ignorados. | `pnpm test command-line` (`pontos.spec.ts`)                        |
| D3  | SE a entrada não é ponto nem opção do prompt, ou é relativa sem ponto anterior, ENTÃO a linha DEVE dizer o que era esperado e repetir o prompt, sem encerrar o comando (spec 08, C1b).                                                | `pnpm test command-line`                                           |
| D4  | QUANDO o usuário escolhe `Desfazer` (`D`, `UNDO`, `_U`) durante o `LINE`, a última reta criada por ele DEVE ser desfeita e o ponto anterior volta a ser o último; sem reta criada, a linha DEVE avisar e repetir o prompt.            | `pnpm test command-line`                                           |
| D5  | QUANDO o usuário escolhe `Fechar` (`F`, `CLOSE`, `_C`) com duas ou mais retas criadas, o `LINE` DEVE criar a reta até o primeiro ponto e encerrar; com menos, a opção NÃO DEVE ser oferecida.                                         | `pnpm test command-line`                                           |
| D6  | QUANDO o usuário confirma texto vazio ou pressiona `Esc` durante o `LINE`, o comando DEVE encerrar e as retas já criadas DEVEM permanecer (spec 08, C7).                                                                              | `pnpm test command-line`; `pnpm test:e2e command-line` (MT-KV-14b) |

Os prompts seguem a versão em português do AutoCAD: "Especifique o primeiro ponto:",
"Especifique o próximo ponto ou [Desfazer]:" e, a partir da segunda reta, "Especifique o
próximo ponto ou [Fechar/Desfazer]:".

**Limitação aceita:** cada reta é um passo de desfazer do menu `Editar`; no AutoCAD, um
`U` depois do `LINE` desfaz o comando inteiro. Agrupar passos entre prompts pede mudança
na pilha do kernel e fica para quando houver um segundo comando que precise disso.

## Questões em aberto

- Variantes de `ARC` e `CIRCLE` que entram primeiro.
