<!-- Caminho relativo: docs/tickets/ui-interface.md -->

# UI — Fase de interface: micro-tickets (primeira leva)

Fase de UI/UX do [ADR 0007](../adr/0007-fase-de-prancha-de-engenharia.md), conduzida
como trabalho de design com a interface do AutoCAD como referência. Esta primeira leva
cobre só o que **não depende** do mecanismo de KV:
[spec 08](../specs/08-linha-de-comando.md) na parte de resolução e
[spec 09](../specs/09-seguranca-do-webview.md). O componente da linha de comando que
edita o desenho, a retirada da barra do upstream e a [spec 10](../specs/10-desenho-basico.md)
entram depois do ADR 0008.

## Bloco A — Segurança do webview (spec 09)

### MT-UI-01: Ligar a CSP e servir a mesma política no Vite

- **Objetivo:** CSP declarada em `tauri.conf.json` conforme a lista de exceções da spec
  09, servida igual pelo servidor de desenvolvimento e dos E2E.
- **Arquivos no escopo:** `src-tauri/tauri.conf.json`, `vite.config.ts`,
  `src/lib/config/security.spec.ts` (novo), `docs/specs/09-seguranca-do-webview.md` (lista
  de exceções medida).
- **Critério de aceite:** S1, S2 e S5 — `pnpm test -- security` verde; `pnpm test:e2e`
  verde sob a CSP.
- **Fora de escopo:** S3, S3b, S6 (próximo ticket) e S4.
- **Depende de:** nenhum.

### MT-UI-02: E2E que falha em violação de CSP

- **Objetivo:** toda a suíte E2E falha ao capturar `securitypolicyviolation`; um teste
  próprio prova que script inline e recurso remoto são bloqueados; recurso remoto em
  arquivo aberto é contado na área de mensagens.
- **Arquivos no escopo:** `e2e/fixtures.ts` ou equivalente de setup (novo),
  `e2e/security.e2e.ts` (novo), fixture sintética com imagem remota, serviço de mensagens.
- **Critério de aceite:** S3, S3b, S6 — `pnpm test:e2e` verde.
- **Fora de escopo:** S4.
- **Depende de:** MT-UI-01.

### MT-UI-03: Conferir a CSP no app empacotado

- **Objetivo:** roteiro S4 executado em Linux e Windows com `make dist-test`.
- **Arquivos no escopo:** `docs/CURRENT-STATE.md` (registro).
- **Critério de aceite:** S4 registrado no handoff para as duas plataformas.
- **Fora de escopo:** correções — cada violação vira ticket.
- **Depende de:** MT-UI-02.

## Bloco B — Resolução de comandos (spec 08)

### MT-UI-04: Resolvedor de comandos bilíngue

- **Objetivo:** módulo puro que carrega `08-comandos-autocad.tsv` e resolve texto
  digitado em comando: nome inglês, nome português, apelido, prefixos `_` e `-`,
  normalização de caixa e acento, distinção entre "ainda não disponível" e "comando
  desconhecido", sugestões por prefixo.
- **Arquivos no escopo:** `src/lib/commands/resolver.ts` (novo),
  `src/lib/commands/resolver.spec.ts` (novo), carga da tabela no build.
- **Critério de aceite:** C1 (resolução), C2, C3, C4, C5 e C9 — `pnpm test -- command-line`
  verde, com teste parametrizado sobre as 225 linhas e o teste de colisão.
- **Fora de escopo:** componente visual, despacho para o kernel, palavras-chave de opção.
- **Depende de:** nenhum.
- **Resultado (2026-11-02):** `src/lib/command-line/resolver.ts` (`lerTabela`,
  `normalizar`, `ResolvedorDeComandos.resolver` e `.sugerir`, `mensagemDaResolucao`) e
  `catalogo.ts`, que embute a tabela no bundle com `?raw`, sem copiá-la. A pasta é
  `command-line/`, e não `commands/`, para que `pnpm test command-line` ache os testes.
  Decisões: com `_`, só o nome em inglês vale, sem apelido (`_L` é desconhecido, como diz
  o C3); prefixo repetido ou sozinho é desconhecido; a variante sem diálogo vem como
  `semDialogo`, e cabe ao comando dizer se ela existe. A colisão (C4) é verificada ao
  montar o resolvedor, que recusa o catálogo. Provas: `resolver.spec.ts` (todas as 225
  linhas em C1; 634 textos e nenhuma colisão em C4) e `catalogo.spec.ts`, vistos falhar
  antes, com 244 e 1 asserções. O catálogo ainda não é importado pelo app: entra com o
  MT-UI-05 e o MT-KV-14. Achado: no pnpm 11, `pnpm test -- X` passa o `--` ao Vitest, que
  então roda a suíte inteira; o filtro é `pnpm test X`.

### MT-UI-05: Catálogo de comandos bilíngue

- **Objetivo:** `Ajuda > Comandos CAD` mostra, para cada comando, inglês, português,
  apelido e disponibilidade, a partir do resolvedor.
- **Arquivos no escopo:** `src/lib/config/cad-command-catalog.ts` e o teste dele,
  `src/lib/components/workspace/HelpCommandsDialog.svelte`.
- **Critério de aceite:** C11 — `pnpm test -- cad-command-catalog` verde; `pnpm check`.
- **Fora de escopo:** despacho.
- **Depende de:** MT-UI-04.
- **Resultado (2026-11-02):** feito com o MT-KV-13b, sob o ADR 0011.
  `buildCadCommandCatalog(tabela, implementados)` lista os 218 comandos da tabela com
  inglês, português, apelidos e disponibilidade; os disponíveis vêm primeiro em cada
  categoria. O diálogo `Ajuda > Comandos CAD` mostra os três nomes e "disponível" ou
  "ainda não disponível", filtra por qualquer deles e ficou mais largo, com quebra de
  linha nas etiquetas. Provas: `cad-command-catalog.spec.ts` (C11), vistos falhar antes.

## Depois do ADR 0008

Componente da linha de comando (C1b, C6–C8, C10, C12), retirada da barra do upstream, e
os comandos de desenho básicos da spec 10.

## Planejados para depois de a interface amadurecer

Ideias do mantenedor (2026-11-01), sem data: entram quando a interface estiver
confortável, prática e bonita o bastante para ser mostrada.

### MT-UI-06: Vitrine do NeoCAD no repositório e no GitHub Pages

- **Objetivo:** quem chega ao repositório ou ao site entende em segundos o que o NeoCAD
  faz e o vê em uso: README com capturas, e uma página no GitHub Pages com capturas e
  vídeos curtos (abrir, desenhar, editar, salvar).
- **Arquivos no escopo:** `README.md`, `docs/site/` (novo), um workflow de publicação do
  Pages.
- **Critério de aceite:** a página publica pelo workflow; as capturas e os vídeos saem
  de roteiros Playwright versionados, para serem refeitos a cada versão sem trabalho
  manual; nenhuma imagem mostra desenho que não seja sintético ou público.
- **Fora de escopo:** domínio próprio, analytics, qualquer coleta de dados.
- **Depende de:** a fase de UI/UX ter a linha de comando própria e a tela sem o upstream
  (MT-KV-13, MT-KV-14).

### MT-UI-08: Modo de vírgula decimal

- **Objetivo:** configuração explícita que troca a entrada e a saída de números para
  vírgula decimal e ponto e vírgula entre coordenadas (`1,5;2`); o padrão continua o do
  AutoCAD, e o ponto e vírgula também separa coordenadas nele.
- **Arquivos no escopo:** `src/lib/command-line/pontos.ts` e o teste dele; o lugar da
  configuração e da formatação de saída, a definir ao abrir.
- **Critério de aceite:** critérios EARS escritos na spec 10 antes de implementar,
  partindo da decisão já registrada lá; `pnpm test command-line`.
- **Fora de escopo:** formato dos números gravados no DXF, que não muda.
- **Depende de:** MT-KV-14b.
- **Resultado (2026-11-04):** critérios D7–D10 na spec 10. O modo mora em
  `pontos.ts` (`definirModoDecimal`): cada modo aceita só o seu decimal; o ponto e
  vírgula separa coordenadas nos dois, e misturar `,` e `;` é inválido. A saída segue o
  modo (`formatarNumero`, `formatarPonto`, os formatos na mensagem de inválido do
  `LINE` e dos prompts de ponto, o valor atual entre `< >` do `FILLET` e do `CHAMFER`).
  A chave fica em `Exibir > Vírgula decimal` (`menuitemcheckbox`), gravada em
  `localStorage` por `services/preferencias.ts`; sem gravação ou com erro de leitura,
  vale o padrão. Muda de propósito: `1;2` passou a ser ponto também no modo padrão (D7).
  Provas vistas falhar antes por asserção: `pontos-decimal.spec.ts` e
  `preferencias.spec.ts` (escritos pelo GLM-5.2 a partir só dos critérios), os casos
  novos em `linha.spec.ts` e `concordar.spec.ts`, e o e2e "Exibir > Vírgula decimal…".
  A barra de status ainda não mostra coordenadas; quando mostrar, segue o modo.

### MT-UI-07: Fontes próprias e pacotes de insumos para baixar

- **Objetivo:** fontes de traço próprias, derivadas da Hershey (ADR 0010), parecidas com
  as mais usadas no AutoCAD (`txt`, `simplex`, `romans`, `isocp`). O instalador traz só as
  mais usadas; o resto (fontes, tipos de linha, hachuras, blocos) fica em pacotes que o
  usuário baixa da página do projeto e instala no NeoCAD.
- **Arquivos no escopo:** a definir no ADR que revisa o ADR 0010.
- **Critério de aceite:** ADR aceito antes de implementar, cobrindo a licença das
  derivadas, o formato do pacote, a verificação de integridade (hash publicado) e o
  funcionamento sem rede. O ADR 0010 hoje proíbe baixar fonte durante o desenho; baixar
  pacotes por ação do usuário é outra coisa e precisa estar escrito.
- **Fora de escopo:** fontes TrueType de terceiros sem licença de redistribuição.
- **Depende de:** fase KP (estilos de texto, tipos de linha, hachura).

## Bloco C — A casca do aplicativo, como no AutoCAD ([spec 11](../specs/11-interface-do-aplicativo.md))

Ordem pensada para que cada passo já se use: abrir no desenho, depois a barra de menus,
o tema, a barra de status, a barra de ferramentas e, por último, a faixa de opções.

### MT-UI-09: Abrir no Desenho1, com a aba Início

- **Objetivo:** o aplicativo abre direto na área de desenho com `Desenho1` vazio; a tela
  inicial vira a aba `Início`; `Arquivo > Novo` dá `DesenhoN`.
- **Arquivos no escopo:** `src/routes/+page.svelte`, `src/lib/components/workspace/`
  (abas de arquivo, `AppTopMenu.svelte`, `HomeScreen.svelte`, `types.ts`), um módulo
  pequeno para o nome do próximo desenho e o teste dele, `e2e/home.e2e.ts`, e os e2e que
  hoje assumem a tela inicial ao abrir.
- **Critério de aceite:** spec 11, I1–I4.
- **Fora de escopo:** vários desenhos abertos; o comando digitado `NEW`; a barra de menus
  completa (MT-UI-10).
- **Depende de:** nenhum.
- **Resultado (2026-11-04):** o app abre na área de desenho com `Desenho1` vazio
  (criado quando a tela do kernel fica pronta); abas `Início` e do desenho
  (`FileTabs.svelte`, `role=tablist`), sem fechar; `Arquivo > Novo` dá `DesenhoN`
  (`services/desenhos.ts`). A marca de alterado acende em edição, desfazer e refazer e
  apaga ao abrir, criar e gravar; com ela, `Novo` e `Abrir` (menu, recentes, arrastar)
  pedem confirmação antes, e recusar não muda nada (critério estendido ao `Abrir` na
  spec, para não perder o desenho em silêncio). Defeitos achados pelos testes e
  corrigidos: a tela do kernel aparecia por cima da aba `Início` (`visibility: visible`
  vencia o `hidden` herdado; agora `inherit`); um `Desenho1` atrasado podia tomar o
  lugar de um arquivo aberto antes dele (contagem de trocas); e o Salvar logo depois da
  leitura nativa saía com o nome do desenho anterior (a leitura do kernel já nomeia a
  aba). Os e2e passam a abrir arquivos por `Arquivo > Abrir`, e o de altura do canvas
  espera a barra de progresso sumir. Caso de borda sem tratamento: `Novo` enquanto um
  arquivo ainda abre. A área de desenho ficou com 270 px de 720 na janela de teste: a
  casca atual come espaço demais, assunto do MT-UI-10 em diante.

### MT-UI-10: Barra de menus clássica

- **Objetivo:** os onze menus do AutoCAD em português, na ordem dele; cada item que o
  NeoCAD faz chama o comando como se fosse digitado (com o eco na linha de comando), e o
  que ainda não existe fica fora do menu.
- **Arquivos no escopo:** `AppTopMenu.svelte` (ou um componente novo de menus), a tabela
  de itens com os nomes oficiais, e2e.
- **Critério de aceite:** critérios EARS na spec 11 antes de implementar.
- **Fora de escopo:** faixa de opções; atalhos de teclado novos.
- **Depende de:** MT-UI-09.

### MT-UI-11: Tema escuro por padrão

- **Objetivo:** interface cinza-escura e fundo de desenho escuro ao abrir, com o claro em
  `Exibir`, lembrado entre sessões como o modo decimal.
- **Arquivos no escopo:** estilos globais, `services/preferencias.ts`, `+page.svelte`.
- **Critério de aceite:** critérios EARS na spec 11; contraste conferido (WCAG AA no texto).
- **Fora de escopo:** temas personalizados.
- **Depende de:** MT-UI-09.

### MT-UI-12: Barra de status com coordenadas

- **Objetivo:** coordenadas do cursor no espaço mostrado, no decimal do modo (spec 10,
  D9), e as alternâncias que o NeoCAD já tem (prendimento).
- **Arquivos no escopo:** componente novo de barra de status, `tela-do-kernel.ts`
  (aviso do cursor), e2e.
- **Critério de aceite:** critérios EARS na spec 11.
- **Fora de escopo:** grade, ortogonal e polar, que ainda não existem.
- **Depende de:** MT-UI-09.

### MT-UI-13: Barra de ferramentas Desenhar e Modificar

- **Objetivo:** botões com ícones próprios para os comandos que existem, chamando-os como
  a linha de comando; dica com o nome do comando.
- **Arquivos no escopo:** componente novo, ícones SVG em `static/` ou `src/lib/`, e2e.
- **Critério de aceite:** critérios EARS na spec 11; ícones com licença do projeto.
- **Fora de escopo:** barras flutuantes e personalizáveis.
- **Depende de:** MT-UI-10.

### MT-UI-14: Faixa de opções e alternância com os menus

- **Objetivo:** abas `Início`, `Inserir`, `Anotar`, `Exibir` com os painéis do AutoCAD
  (Desenhar, Modificar, Camadas…), só com o que existe; alternar entre faixa e menus.
- **Arquivos no escopo:** a definir ao abrir.
- **Critério de aceite:** critérios EARS na spec 11 antes de implementar.
- **Fora de escopo:** painéis que dependem de KP.
- **Depende de:** MT-UI-13.
