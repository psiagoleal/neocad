<!-- Caminho relativo: docs/specs/10-desenho-basico.md -->

# 10 — Comandos de desenho básicos

- **Estado:** Parcial — `LINE`, entrada de pontos, seleção e `ERASE` com critérios
  (2026-11-02); os demais comandos fecham os seus quando entrarem
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

## Critérios de aceite (EARS) — seleção na tela

| id  | critério                                                                                                                                                                                                                                                                                                                 | prova                                                                                                                                                 |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| S1  | QUANDO o usuário clica com o botão esquerdo a até 5 pixels do traço de uma entidade do espaço mostrado, ela DEVE entrar na seleção e ficar destacada; com `Shift`, DEVE sair dela.                                                                                                                                       | `cargo test` (`tela.rs`); `pnpm test:e2e kernel-view`                                                                                                 |
| S2  | QUANDO o usuário clica onde não há traço, a tela DEVE abrir uma janela nesse ponto, e o segundo clique a fecha: da esquerda para a direita, entram as entidades inteiramente dentro dela (janela); da direita para a esquerda, as que ela toca ou contém (cruzada). Com `Shift` no segundo clique, elas saem da seleção. | idem                                                                                                                                                  |
| S3  | Só DEVEM ser selecionáveis as entidades desenhadas pela tela, em camada visível, no espaço mostrado; trocar de espaço DEVE esvaziar a seleção.                                                                                                                                                                           | idem                                                                                                                                                  |
| S4  | QUANDO uma edição apaga uma entidade selecionada, ela DEVE sair da seleção; QUANDO a move, o destaque DEVE acompanhá-la.                                                                                                                                                                                                 | `pnpm test:e2e kernel-view`                                                                                                                           |
| S5  | QUANDO o usuário pressiona `Esc` sem comando em curso, a janela aberta DEVE ser cancelada e a seleção esvaziada (spec 08, C7).                                                                                                                                                                                           | `pnpm test:e2e command-line`                                                                                                                          |
| S6  | Com 100 mil entidades, um clique ou o fechamento de uma janela DEVE atualizar a seleção na tela em até 100 ms.                                                                                                                                                                                                           | `pnpm test:e2e desempenho` (com `NEOCAD_SINTETICO_N`); a janela em volta do desenho inteiro falha hoje (~290 ms) e fica `test.fail()` até o MT-KV-14g |

A tela expõe a quantidade selecionada em `data-selecao` (spec 07, "O que os testes
observam na tela"). Camada bloqueada ainda não existe na tela (fase KP1); seleção de
entidade que só o upstream desenha é o V7 (MT-KV-14e).

## Critérios de aceite (EARS) — `ERASE`

| id  | critério                                                                                                                                                               | prova                                                  |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| E1  | QUANDO o `ERASE` (`APAGAR`, `E`) começa com seleção não vazia, as entidades selecionadas DEVEM ser apagadas pelo kernel num só passo de desfazer, e o comando encerra. | `pnpm test command-line`; `pnpm test:e2e command-line` |
| E2  | QUANDO o `ERASE` começa sem seleção, a linha DEVE pedir "Selecione objetos:"; cliques e janelas selecionam, e o texto vazio apaga o que estiver selecionado e encerra. | idem                                                   |
| E3  | QUANDO há seleção, nenhum comando em curso e o foco não está num campo de texto, `Delete` DEVE apagar a seleção como o `ERASE`.                                        | `pnpm test:e2e command-line`                           |

## Critérios de aceite (EARS) — `MOVE` e `COPY`

| id  | critério                                                                                                                                                                                                                                                                                                   | prova                                                  |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| M1  | QUANDO o `MOVE` (`MOVER`, `M`) começa com seleção, ou o usuário seleciona e confirma o vazio em "Selecione objetos:", a linha DEVE pedir o ponto base e o segundo ponto; as entidades selecionadas DEVEM ser movidas pelo kernel da diferença entre eles, num só passo de desfazer, e a seleção esvaziada. | `pnpm test command-line`; `pnpm test:e2e command-line` |
| M2  | No segundo ponto, a coordenada relativa (`@`) DEVE partir do ponto base; o vazio DEVE usar o ponto base como deslocamento.                                                                                                                                                                                 | `pnpm test command-line`                               |
| M3  | O `COPY` (`COPIAR`, `CO`) DEVE fazer o mesmo criando cópias, e continuar pedindo segundos pontos, uma cópia e um passo de desfazer por ponto; `Desfazer` (`D`, `_U`) desfaz a última cópia; o vazio ou `Sair` (`S`, `_E`) encerra.                                                                         | idem                                                   |
| M4  | SE o usuário confirma o vazio em "Selecione objetos:" sem nada selecionado, ENTÃO o comando DEVE encerrar sem mudar o documento.                                                                                                                                                                           | `pnpm test command-line`                               |

Os prompts seguem a versão em português do AutoCAD: "Especifique o ponto base:",
"Especifique o segundo ponto ou <usar o primeiro ponto como deslocamento>:" e, no `COPY`
depois da primeira cópia, "Especifique o segundo ponto ou [Sair/Desfazer] <Sair>:". A
opção `Deslocamento` do ponto base e a `Matriz` do `COPY` ficam para depois.

## Critérios de aceite (EARS) — `ROTATE`, `MIRROR` e `SCALE`

| id  | critério                                                                                                                                                                                                                                                                                                                                    | prova                                                  |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| R1  | QUANDO o `ROTATE` (`ROTAC`, `RO`) recebe a seleção (como em M1), o ponto base e o ângulo, as entidades DEVEM girar pelo kernel em torno da base, num só passo de desfazer. O ângulo DEVE ser aceito em graus, no sentido anti-horário, ou como um ponto, valendo a direção da base até ele.                                                 | `pnpm test command-line`; `pnpm test:e2e command-line` |
| R2  | QUANDO o `MIRROR` (`ESPELHAR`, `MI`) recebe a seleção e os dois pontos da linha de espelhamento, a linha DEVE perguntar "Apagar objetos de origem? [Sim/Não] <Não>:"; com `Não` ou o vazio, DEVEM ficar as originais e entrar as cópias espelhadas; com `Sim`, as originais DEVEM ser espelhadas. Nos dois casos, num só passo de desfazer. | idem                                                   |
| R3  | QUANDO o `SCALE` (`ESCALA`, `SC`) recebe a seleção, o ponto base e o fator, as entidades DEVEM ser escaladas pelo kernel em torno da base, num só passo de desfazer.                                                                                                                                                                        | idem                                                   |
| R4  | SE o ângulo, o fator ou a linha de espelhamento é inválido (texto que não é número nem ponto, fator zero ou negativo, os dois pontos iguais), ENTÃO a linha DEVE dizer o que se esperava e repetir o prompt; `Esc` antes da última resposta NÃO DEVE mudar o documento.                                                                     | `pnpm test command-line`                               |

Os prompts seguem a versão em português do AutoCAD: "Especifique o ponto base:",
"Especifique o ângulo de rotação:", "Especifique o primeiro ponto da linha de
espelhamento:", "Especifique o segundo ponto da linha de espelhamento:" e "Especifique o
fator de escala:". As opções `Copiar` e `Referência` de `ROTATE` e `SCALE` ficam para
depois.

## Critérios de aceite (EARS) — `OFFSET`

| id  | critério                                                                                                                                                                                                                                                                                                                                                                                               | prova                                                  |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------ |
| O1  | QUANDO o `OFFSET` (`DESLOCAMENTO`, `O`) começa, a linha DEVE pedir a distância; depois, "Selecione o objeto a deslocar ou [Sair/Desfazer] <Sair>:", e o clique numa entidade da tela DEVE escolhê-la sem precisar de `Enter`; depois, "Especifique o ponto no lado a deslocar:", e a paralela DEVE ser criada pelo kernel do lado do ponto, num passo de desfazer; a linha DEVE voltar a pedir objeto. | `pnpm test command-line`; `pnpm test:e2e command-line` |
| O2  | QUANDO o usuário escolhe `Desfazer` (`D`, `_U`) no pedido de objeto, a última paralela DEVE ser desfeita; o vazio ou `Sair` (`S`, `_E`) DEVE encerrar.                                                                                                                                                                                                                                                 | `pnpm test command-line`                               |
| O3  | SE o kernel recusa a paralela (por exemplo, arco que encolheria além do centro), ENTÃO a linha DEVE mostrar o motivo e voltar a pedir objeto, sem mudar o documento.                                                                                                                                                                                                                                   | idem                                                   |
| O4  | SE a distância não é um número maior que zero, ENTÃO a linha DEVE dizer o que se esperava e repetir o pedido.                                                                                                                                                                                                                                                                                          | idem                                                   |

As opções `Através`, `Apagar`, `Camada` e `Múltiplo` ficam para depois; o ponto do lado é
digitado até a entrada de pontos pelo clique (MT-KV-14h).

## Critérios de aceite (EARS) — `TRIM` e `EXTEND`

| id  | critério                                                                                                                                                                                                                                                                                                                                                                          | prova                                                  |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| T1  | QUANDO o `TRIM` (`APARAR`, `TR`) começa, a linha DEVE pedir "Selecione arestas de corte ou <selecionar todos>:"; o vazio fixa como arestas a seleção ou, sem seleção, todas as entidades do espaço mostrado. Depois, "Selecione o objeto a aparar ou [Desfazer]:": cada clique DEVE aparar pelo kernel o trecho clicado, num passo de desfazer, e o pedido se repete até o vazio. | `pnpm test command-line`; `pnpm test:e2e command-line` |
| T2  | O `EXTEND` (`ESTENDER`, `EX`) DEVE fazer o mesmo com "Selecione arestas de contorno ou <selecionar todos>:" e "Selecione o objeto a estender ou [Desfazer]:", estendendo pelo kernel a ponta mais perto do clique até a aresta.                                                                                                                                                   | idem                                                   |
| T3  | QUANDO o usuário escolhe `Desfazer` (`D`, `_U`), o último corte ou extensão DEVE ser desfeito; SE o kernel recusa (clique longe do objeto, nada a cortar ou a alcançar), ENTÃO a linha DEVE mostrar o motivo e repetir o pedido.                                                                                                                                                  | `pnpm test command-line`                               |
| T4  | ENQUANTO um prompt pede um objeto pelo clique (`TRIM`, `EXTEND`, `OFFSET`), o clique na tela DEVE ir ao comando, com o ponto clicado, e NÃO DEVE mudar a seleção.                                                                                                                                                                                                                 | `pnpm test command-line`; `pnpm test render`           |

O modo rápido sem arestas, a cerca e as opções de projeção ficam para depois.

## Questões em aberto

- Variantes de `ARC` e `CIRCLE` que entram primeiro.
