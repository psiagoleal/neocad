<!-- Caminho relativo: docs/adr/0007-fase-de-prancha-de-engenharia.md -->

# ADR 0007: Fase de prancha de engenharia antes do paramétrico e do 3D

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-10-29
- **Decisores:** Iago Leal
- **Tags:** roadmap, kernel, anotação, plotagem, comandos, autocad

## Contexto

O NeoCAD existe para produzir **desenhos de projeto de engenharia**: a prancha que sai
do escritório com carimbo, cotas, textos, hachuras, blocos normalizados, tipos e
espessuras de linha corretos e escala de plotagem definida. O
[ADR 0005](0005-layouts-de-espaco-papel.md) já registrou que o entregável é a folha
composta no espaço-papel, e não o desenho solto no espaço-modelo.

O faseamento do [ADR 0003](0003-kernel-cad-proprio.md) não leva a essa folha. Depois de
K3, ele segue para o solver de restrições (K4), renderização própria (K5), leitura DWG
(K6) e o núcleo 3D B-rep (K7–K9). O review de 2026-10-29
([docs/reviews/2026-10-29-review.md](../reviews/2026-10-29-review.md), achados A1–A9)
mediu a distância:

- o modelo representa seis geometrias (`Line`, `Circle`, `Arc`, `Polyline`, `Text`,
  `Viewport`); cota, MTEXT, hachura, referência de bloco com atributos, elipse e
  spline não existem;
- a entidade não tem tipo de linha, espessura nem escala de tipo de linha próprios, e a
  camada guarda o tipo de linha apenas como nome;
- não há unidades do desenho nem precisão, e não há plotagem nem PDF;
- tudo o que o kernel não modela é **descartado ao salvar**, com aviso ao usuário.

Nenhuma fase do ADR 0003 cobre esses itens. Cumprido à risca, o roteiro entregaria
desenho paramétrico e sólidos antes de uma cota.

O review também mostrou que o modelo do kernel **não chega à tela**. O canvas é
desenhado pelo upstream, que não recebe as edições do kernel; e os comandos do
upstream editam o modelo do upstream, que o salvamento — feito pelo kernel — ignora
(achados A4, C4). Qualquer fase que crie entidades depende de fechar esse caminho
antes.

Por fim, quem usa CAD em projeto **digita comandos**. Quem vem do AutoCAD digita os
nomes e apelidos dele, em inglês ou na versão em português, e a interface atual aceita
só os nomes globais do upstream, sem apelido.

## Decisão

Fica acordada a inserção de duas fases no faseamento do ADR 0003 e a reordenação do
restante:

1. **K3 — Geometria 2D e edição** (em curso): fecha com MT-K3-10 a MT-K3-12, como
   planejado.
2. **KV — O kernel é o que se vê.** Uma única fonte de verdade para tela e arquivo:
   o que o kernel contém é o que aparece, e o que o usuário faz passa pelo kernel. A
   forma — espelhar o kernel no modelo do upstream ou antecipar parte de K5 — é
   decidida em ADR próprio, depois de um spike que meça as duas.
3. **Fase de UI/UX** (já decidida): inclui a **linha de comando própria**, com paridade
   de nomes e apelidos com o AutoCAD nos dois idiomas, e a CSP do Tauri ligada.
4. **KP — Prancha de engenharia.** O kernel passa a modelar, ler, gravar e desenhar o
   que compõe a folha, em blocos que cada um entrega valor observável:
   - **KP1 — Propriedades de desenho:** unidades e precisão, tabela de tipos de linha,
     espessura, escala de tipo de linha, propriedades por entidade (`BYLAYER`,
     `BYBLOCK`, explícita).
   - **KP2 — Geometria complementar:** elipse, spline, ponto, linha infinita e semirreta.
   - **KP3 — Texto:** MTEXT e estilos de texto completos.
   - **KP4 — Blocos:** referência de bloco (`INSERT`), atributos e explosão.
   - **KP5 — Cotas:** linear, alinhada, angular, raio, diâmetro, em série e de linha
     base, com estilos de cota.
   - **KP6 — Hachura:** contorno, padrões e preenchimento; traz de volta as booleanas 2D
     que K3 adiou exatamente por falta desta entidade.
   - **KP7 — Plotagem:** folha em escala, espessuras de pena e saída em PDF.
5. **K6 — Leitura DWG**, **K4 — Restrições 2D** e **K7–K9 — Núcleo 3D** seguem depois
   de KP, nessa ordem. K5 (renderização própria) passa a ser o que o ADR de KV decidir.

A `v0.2.0` continua em rascunho até o fim da fase de UI/UX, como já decidido; KP não a
bloqueia.

## Consequências

**Positivas**

- O produto converge para o entregável do usuário: cada bloco de KP elimina uma classe
  de perda ao salvar e torna o NeoCAD utilizável para mais um tipo de prancha.
- Booleanas 2D voltam com consumidor real (hachura), como K3 previa.
- KV elimina a classe de bug "duas representações vivas" antes que o volume de
  entidades a torne cara.

**Negativas e trade-offs aceitos**

- O desenho paramétrico (K4) e o 3D (K7–K9) recuam, talvez por muito tempo. Quem
  esperava o NeoCAD como modelador sólido vai esperar mais.
- KV pode antecipar parte de K5, que o ADR 0003 adiava até o upstream limitar a
  interação; o custo só fica conhecido depois do spike.
- KP5 (cotas) é grande: associatividade, estilos e sobreposições formam um subsistema.
  O bloco pode precisar de quebra própria.

## Diretriz de Conformidade de Código

- **Proibido** iniciar trabalho de K4 ou K7–K9 antes de KP concluída, salvo novo ADR.
- **Proibido** criar entidade nova no kernel sem leitura **e** gravação DXF no mesmo
  ticket: entidade que entra e não sai continua sendo perda ao salvar.
- **Obrigatório** que cada bloco de KP e a linha de comando tenham spec em `docs/specs/`
  antes dos tickets, com critério de aceite verificável.
- **Obrigatório** que todo comando exposto registre o nome global em inglês, o nome
  localizado em português e os apelidos, conforme as páginas oficiais de atalhos da
  Autodesk de cada idioma; nenhum nome é traduzido de um idioma para o outro.
- **Obrigatório** que a fase de UI/UX não crie edição pela interface antes de KV
  concluída.
