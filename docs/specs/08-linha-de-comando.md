<!-- Caminho relativo: docs/specs/08-linha-de-comando.md -->

# 08 — Linha de comando

- **Estado:** Aprovada
- **Fase:** UI/UX · **ADRs:** 0001, 0007 · **Transversais:** 00 (T8) · **Depende de:** 07

## Problema e resultado esperado

Engenheiros e técnicos **digitam** comandos. Quem vem do AutoCAD digita os nomes e
apelidos dele — em inglês, ou na versão em português do Brasil. Hoje a única entrada
digitável é a barra do upstream, que aceita só os nomes globais em inglês dele, sem
apelido, e edita um modelo que o salvamento ignora.

Ao fim: o NeoCAD tem linha de comando própria, sempre visível, que aceita o nome em
inglês, o nome em português e o apelido de cada comando, **com a grafia oficial da
Autodesk em cada idioma**, e despacha pelo kernel.

## Escopo

- Linha de comando própria, ancorada abaixo do canvas, com histórico de mensagens.
- Resolução de texto digitado em comando: nome em inglês, nome em português, apelido,
  prefixo `_` e prefixo `-`.
- Sugestões enquanto se digita; histórico de comandos com as setas.
- Repetição do último comando com `Enter` ou `Espaço` na linha vazia; cancelamento com
  `Esc`.
- Palavras-chave de opção dos comandos (ex.: `Desfazer`/`Undo`) aceitas nos dois
  idiomas.
- A barra de comandos do upstream deixa de ser oferecida.
- O catálogo `Ajuda > Comandos CAD` passa a mostrar os dois idiomas e o apelido (C11).

## Fora de escopo

Apelidos definidos pelo usuário (equivalente ao `acad.pgp`), scripts (`SCRIPT`), AutoLISP,
entrada dinâmica junto ao cursor (`F12`) e as teclas de alternância `F3`–`F11`: specs
próprias da fase de UI/UX.

## Restrições e decisões já tomadas

- **Fonte dos nomes:** [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv), montado em
  2026-10-29 a partir das páginas oficiais de atalhos da Autodesk —
  <https://www.autodesk.com/shortcuts/autocad> (inglês) e
  <https://www.autodesk.com/br/shortcuts/autocad> (português) —, cruzadas pelo apelido,
  que é igual nas duas. **Nenhum nome é traduzido de um idioma para o outro**; nome que
  não estiver na tabela vem da documentação oficial do idioma correspondente.
- Prefixo `_` força o nome em inglês, como no AutoCAD localizado; prefixo `-` pede a
  variante sem caixa de diálogo, quando ela existe.
- Comandos da tabela que o NeoCAD ainda não implementa continuam **reconhecidos**: a
  resposta é "ainda não disponível", nunca "desconhecido".

## Critérios de aceite (EARS)

| id  | critério                                                                                                                                                                                                                                                                                                                                                                                                           | prova                                                                                                                                                                                                                                                    |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| C1  | ENQUANTO nenhum comando está em curso, QUANDO o usuário digita o nome em inglês, o nome em português ou o apelido de um comando implementado e confirma, a linha de comando DEVE iniciar esse comando.                                                                                                                                                                                                             | `pnpm test -- command-line` com todas as linhas da tabela                                                                                                                                                                                                |
| C1b | ENQUANTO um comando está em curso, o texto confirmado DEVE ser entregue a esse comando como resposta ao prompt, e não iniciar outro; SE a resposta não é válida para o prompt, ENTÃO a linha DEVE dizer o que era esperado e repetir o prompt, sem encerrar o comando.                                                                                                                                             | `pnpm test -- command-line`                                                                                                                                                                                                                              |
| C2  | A resolução DEVE ignorar maiúsculas, minúsculas e acentos (`POLISSÓLIDO` = `polissolido`).                                                                                                                                                                                                                                                                                                                         | idem                                                                                                                                                                                                                                                     |
| C3  | QUANDO o texto começa com `_`, a resolução DEVE aceitar só o nome em inglês; QUANDO começa com `-`, DEVE pedir a variante sem diálogo e, se ela não existir, iniciar a variante normal; os dois prefixos DEVEM ser aceitos juntos, em qualquer ordem (`_-LAYER`, `-_LAYER`).                                                                                                                                       | idem                                                                                                                                                                                                                                                     |
| C4  | Nenhum texto digitável, depois da normalização de C2, DEVE resolver para dois comandos diferentes.                                                                                                                                                                                                                                                                                                                 | `pnpm test -- command-line` (caso de colisão sobre a tabela inteira; hoje: 634 textos, 0 colisões)                                                                                                                                                       |
| C5  | QUANDO o texto corresponde a um comando do catálogo (a tabela mais os comandos acrescentados por spec) que não está implementado, a linha DEVE responder "ainda não disponível" com o nome digitado; QUANDO não corresponde a comando algum do catálogo, DEVE responder "comando desconhecido". Nos dois casos o documento NÃO DEVE mudar.                                                                         | `pnpm test -- command-line`                                                                                                                                                                                                                              |
| C6  | QUANDO a linha está vazia, nenhum comando está em curso e o usuário pressiona `Enter` ou `Espaço`, o último comando **iniciado** por qualquer via (linha, menu ou botão) — concluído ou cancelado — DEVE ser repetido; respostas "desconhecido" e "ainda não disponível" não contam.                                                                                                                               | `pnpm test:e2e -- command-line` (a criar)                                                                                                                                                                                                                |
| C7  | QUANDO o usuário pressiona `Esc` durante um comando, o comando DEVE ser encerrado e o que ele ainda não confirmou DEVE ser descartado; o que já foi confirmado (ex.: segmentos de `LINE` já fechados) DEVE permanecer, como no AutoCAD. QUANDO não há comando em curso, `Esc` DEVE apenas limpar o texto e a seleção.                                                                                              | `pnpm test:e2e -- command-line` (a criar)                                                                                                                                                                                                                |
| C8  | QUANDO um comando pede uma opção, a linha DEVE aceitar a palavra-chave em português, a palavra-chave em inglês — com ou sem prefixo `_` — e a abreviação destacada no prompt em português; SE uma abreviação em português coincidir com a palavra-chave inglesa de outra opção, ENTÃO vale o português, e o inglês exige o `_`. As palavras-chave de cada comando são declaradas na spec que implementa o comando. | `pnpm test -- command-line`                                                                                                                                                                                                                              |
| C9  | ENQUANTO o usuário digita, a linha DEVE sugerir os comandos cujo nome ou apelido começa pelo texto, nos dois idiomas.                                                                                                                                                                                                                                                                                              | `pnpm test -- command-line`                                                                                                                                                                                                                              |
| C10 | Todo comando que edita o desenho, iniciado pela linha de comando, DEVE ser despachado pelo kernel (spec 07, V5).                                                                                                                                                                                                                                                                                                   | `pnpm test -- command-line`: teste parametrizado sobre todo comando de edição implementado, conferindo que o despacho chega ao serviço `cad-document`; mais `pnpm test:e2e -- command-line` (a criar) desenhando uma linha com `L`, salvando e reabrindo |
| C11 | O catálogo `Ajuda > Comandos CAD` DEVE listar, para cada comando, o nome em inglês, o nome em português e o apelido, e indicar se está disponível.                                                                                                                                                                                                                                                                 | `pnpm test -- cad-command-catalog`                                                                                                                                                                                                                       |

## Casos de erro

- Texto vazio com nenhum comando anterior: nada acontece, sem mensagem de erro.
- Comando digitado com outro comando em curso: o AutoCAD trata como entrada do comando
  em curso; o NeoCAD faz o mesmo, e só o `Esc` sai.

## Requisitos não-funcionais

| id  | critério                                                                                          | prova                                                                                       |
| --- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| C12 | QUANDO o usuário digita uma tecla, a lista de sugestões DEVE ser atualizada na tela em até 50 ms. | `pnpm test:e2e -- command-line` medindo do evento de tecla à atualização da lista (a criar) |

## Questões em aberto

- Comandos transparentes (prefixo `'`, como `'ZOOM` dentro de outro comando): decidir na
  spec de navegação da fase de UI/UX.
