<!-- Caminho relativo: docs/adr/0010-texto-da-tela-em-fonte-de-traco.md -->

# ADR 0010: Texto da tela em fonte de traço Hershey, embutida no kernel

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-11-01 · **Aceito:** 2026-11-01 (escolha da fonte e do download pelo
  mantenedor, antes da implementação)
- **Decisores:** Iago Leal
- **Tags:** kernel, texto, fonte, licença, fase KV

## Contexto

O ADR 0008 põe no kernel a posição e a forma dos glifos; o TypeScript só repassa blocos
à GPU. Para isso, o kernel precisa do desenho das letras, e o repositório não tinha
nenhum: o upstream baixa as fontes dele da rede em tempo de execução, o que não serve a
um aplicativo que abre desenhos sem rede.

O AutoCAD desenha o estilo `Standard` com a `txt.shx`, uma fonte de traço: cada letra é
um punhado de segmentos, não um contorno preenchido. A `romans.shx` descende da fonte
Hershey Roman Simplex, criada no U. S. National Bureau of Standards e distribuída sob
licença permissiva.

O MT-KV-09 previa o texto como uma segunda primitiva (triângulos) no desenhista. Uma
fonte de traço dispensa isso: o texto vira segmentos no mesmo bloco da camada.

## Decisão

1. A tela desenha o texto com a fonte **Hershey Roman Simplex** (`rowmans.jhf`), copiada
   sem alteração para `kernel/neocad-geometry/data/` e lida com `include_str!`. A
   atribuição exigida pela licença fica em `kernel/neocad-geometry/data/LEIA-ME.md`.
2. A função `text_strokes` (`neocad-geometry`) transforma conteúdo, posição, altura e
   rotação em segmentos. A altura pedida é a das maiúsculas.
3. A fonte cobre ASCII e o sinal de grau. O kernel compõe as letras acentuadas do
   português e do espanhol (agudo, grave, circunflexo, til, trema, cedilha) e os
   símbolos `±` e `⌀`. Entende os códigos `%%d`, `%%p`, `%%c`, `%%%` e `%%nnn`. O que a
   fonte não tem sai como `?`.
4. A tela põe os segmentos do texto no bloco da camada, como os de uma reta. O
   desenhista não ganha primitiva nova.

## Consequências

**Positivas**

- O texto aparece sem rede e sem dependência nova, e o `.wasm` cresceu 14 872 bytes
  (479 595, teto 600 000).
- Os dois caminhos de desenho (WebGL2 e canvas 2D) desenham o texto sem mudar uma linha.
- A aparência lembra a do AutoCAD com a `txt.shx` e a `romans.shx`.

**Negativas**

- Todo texto sai na mesma fonte: estilos de texto, fontes TrueType e `.shx` próprias do
  desenho, fator de largura, inclinação e alinhamento ficam para a fase KP.
- Sublinhado e sobrelinhado (`%%u`, `%%o`) são ignorados.
- Os acentos foram desenhados à mão para esta composição e não fazem parte da fonte
  original.
- Fora do latim, todo caractere vira `?`.

## Diretriz de Conformidade de Código

- **Obrigatório:** distribuir `kernel/neocad-geometry/data/LEIA-ME.md` junto com
  `rowmans.jhf`; qualquer cópia ou conversão da fonte leva a mesma atribuição.
- **Proibido:** converter os dados Hershey para o formato distribuído pelo U.S. NTIS
  (condição 2 da licença).
- **Proibido:** baixar fonte em tempo de execução para desenhar a tela; fonte nova entra
  no repositório, com licença compatível e ADR.
- **Proibido:** calcular a forma ou a posição dos glifos no TypeScript (ADR 0008).
