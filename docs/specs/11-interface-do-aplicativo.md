<!-- Caminho relativo: docs/specs/11-interface-do-aplicativo.md -->

# 11 — Interface do aplicativo

- **Estado:** Parcial. Abertura no desenho novo e aba `Início` com critérios (I1–I4,
  2026-11-04); os demais itens fecham os seus quando o ticket abrir.
- **Fase:** UI/UX · **ADRs:** 0007, 0008 · **Relacionadas:** 07, 08, 10

## Problema e resultado esperado

Hoje o NeoCAD abre numa tela inicial de visualizador, com um menu de aplicativo
genérico, e só mostra a área de desenho depois que um arquivo é aberto. Quem vem do
AutoCAD espera abrir o programa e já estar num desenho, com a barra de menus, a área
de desenho, as abas de espaço, a linha de comando e a barra de status nos lugares de
sempre. O resultado esperado é essa disposição, simplificada: só o que o NeoCAD faz
aparece, e cada item novo entra no lugar que ocupa no AutoCAD.

## Decisões já tomadas (2026-11-04, com o mantenedor)

- **Abre num desenho novo:** `Desenho1`, vazio, no espaço-modelo, como o AutoCAD.
- **A tela inicial vira a aba `Início`**, ao lado da aba do desenho, como a aba
  "Iniciar" do AutoCAD; os recentes seguem também em `Arquivo > Abrir recente`.
- **Barra de menus clássica primeiro**, com os menus e a ordem do AutoCAD em português:
  `Arquivo`, `Editar`, `Exibir`, `Inserir`, `Formatar`, `Ferramentas`, `Desenhar`,
  `Cotar`, `Modificar`, `Janela`, `Ajuda`. A faixa de opções vem depois, com uma
  alternância entre as duas, como no AutoCAD.
- **Tema escuro por padrão**, na interface e no fundo do desenho; o claro continua em
  `Exibir`.
- **Aparência própria:** a disposição e os nomes seguem o AutoCAD; logotipo, ícones e
  arte não. Ícones são desenhados para o projeto, sob a licença dele.
- **Um desenho por vez** até haver ticket de vários documentos abertos: `Novo` e
  `Abrir` trocam o desenho da aba.

## Disposição

De cima para baixo: barra de menus; barra de ferramentas (Desenhar e Modificar);
abas de arquivo (`Início` e o desenho); área de desenho; abas de espaço (`Modelo` e
os layouts); linha de comando; barra de status (coordenadas do cursor e alternâncias).

## Escopo

A casca do aplicativo: abertura, abas de arquivo, barra de menus, barra de
ferramentas, tema, barra de status e, depois, a faixa de opções.

## Fora de escopo

Vários desenhos abertos ao mesmo tempo; paletas (Propriedades, Camadas), que vêm com
KP1; personalização de menus e de atalhos; a caixa de opções (`OPTIONS`).

## Critérios de aceite (EARS) — abertura e aba `Início`

| id  | critério                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | prova                                                                   |
| --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| I1  | QUANDO o aplicativo abre, ele DEVE mostrar a área de desenho com um documento novo e vazio chamado `Desenho1`, no espaço-modelo, com a linha de comando em `Comando:`, sem passar pela tela inicial; a aba `Início` já existe, inativa.                                                                                                                                                                                                                                                                                                                     | `pnpm test:e2e home`                                                    |
| I2  | A barra de abas de arquivo DEVE mostrar a aba `Início` e a aba do desenho, com a do desenho ativa ao abrir; QUANDO o usuário clica numa delas, ela DEVE ser mostrada, e cada uma DEVE voltar como estava (o desenho, a vista e o que a linha de comando tem em curso). Enquanto houver um desenho só, a aba dele não tem botão de fechar.                                                                                                                                                                                                                   | `pnpm test:e2e home`                                                    |
| I3  | QUANDO o usuário desenha no `Desenho1` pela linha de comando, as entidades DEVEM aparecer na tela e `Arquivo > Salvar` DEVE pedir o destino e gravar um DXF que, reaberto, traz as mesmas entidades; SE a gravação falha, ENTÃO o aplicativo DEVE dizer o motivo na área de mensagens e o desenho DEVE continuar aberto e marcado como alterado.                                                                                                                                                                                                            | `pnpm test:e2e home`                                                    |
| I4  | QUANDO o usuário escolhe `Arquivo > Novo`, um documento vazio `DesenhoN` DEVE substituir o da aba, com N o próximo número dos desenhos novos criados na sessão; SE o atual foi alterado desde que abriu, foi criado ou foi gravado (uma edição, um desfazer ou um refazer; zoom e pan não contam), ENTÃO o aplicativo DEVE pedir confirmação antes, e recusar DEVE manter o desenho atual intacto, sem gastar o número. Abrir um arquivo (menu, recentes ou arrastar) troca o desenho da aba e DEVE pedir a mesma confirmação, antes de escolher o arquivo. | `pnpm test:e2e home`; `pnpm test` (nome do próximo e marca de alterado) |

Abrir um arquivo continua como hoje: o desenho aberto ocupa a aba, com o nome do
arquivo. O nome digitado do comando `NEW` / `QNEW` entra quando for registrado no
catálogo (spec 08), pela página oficial de atalhos de cada idioma.
