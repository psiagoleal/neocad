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

### MT-UI-05: Catálogo de comandos bilíngue

- **Objetivo:** `Ajuda > Comandos CAD` mostra, para cada comando, inglês, português,
  apelido e disponibilidade, a partir do resolvedor.
- **Arquivos no escopo:** `src/lib/config/cad-command-catalog.ts` e o teste dele,
  `src/lib/components/workspace/HelpCommandsDialog.svelte`.
- **Critério de aceite:** C11 — `pnpm test -- cad-command-catalog` verde; `pnpm check`.
- **Fora de escopo:** despacho.
- **Depende de:** MT-UI-04.

## Depois do ADR 0008

Componente da linha de comando (C1b, C6–C8, C10, C12), retirada da barra do upstream, e
os comandos de desenho básicos da spec 10.
