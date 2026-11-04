<!-- Caminho relativo: docs/specs/09-seguranca-do-webview.md -->

# 09 — Segurança do webview

- **Estado:** Aprovada
- **Fase:** UI/UX (antes da `v0.2.0`) · **ADRs:** 0003 · **Transversais:** 00

## Problema e resultado esperado

O NeoCAD abre arquivos de terceiros e os interpreta em WebAssembly e em workers dentro do
webview do Tauri, hoje sem política de segurança de conteúdo (`"csp": null`, achado D1 do
review). Um arquivo malicioso que explore o parser teria o webview inteiro, e a ponte com
o sistema, sem barreira. Ao fim: a CSP está ligada, é a mais restritiva com que o app
funciona, e a mesma política vale nos testes.

## Escopo

CSP em `src-tauri/tauri.conf.json`; a mesma política servida pelo servidor de
desenvolvimento e de testes, para que o E2E rode sob ela; revisão das permissões de
`src-tauri/capabilities/default.json`.

## Fora de escopo

Assinatura de binário e atualização automática.

## Restrições e decisões já tomadas

- O app não carrega conteúdo remoto: `connect-src`, `script-src` e `img-src` ficam
  restritos à origem do app (mais `blob:` e `data:` só onde o upstream exigir).
- WebAssembly exige `'wasm-unsafe-eval'`; nada de `'unsafe-eval'` nem de script inline.
- **Lista de exceções**, a única fonte para S1: `'wasm-unsafe-eval'` em `script-src`
  (kernel e LibreDWG em WebAssembly); `blob:` em `worker-src` e `img-src`; `data:` em
  `font-src` e `img-src`; `'unsafe-inline'` em `style-src` só se o upstream o exigir,
  medido no ticket. Exceção nova entra aqui, com motivo, no mesmo commit que a usa.

## Permissões

| permissão                                                        | motivo                                 |
| ---------------------------------------------------------------- | -------------------------------------- |
| `core:default`                                                   | base do Tauri                          |
| `dialog:allow-open`, `dialog:allow-save`, `dialog:allow-confirm` | abrir, salvar e confirmar perda        |
| `fs:default`, `fs:read-files`                                    | ler o arquivo escolhido no diálogo     |
| `fs:allow-write-file`                                            | gravar no caminho escolhido no diálogo |
| `fs:allow-appconfig-write-recursive`                             | preferências e recentes                |

## Critérios de aceite (EARS)

| id  | critério                                                                                                                                                                                                                                                                                                                                                                         | prova                                                                                                                                                                                                                                         |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| S1  | A configuração do Tauri DEVE declarar uma CSP em que `default-src`, `script-src`, `connect-src` e `img-src` contêm apenas `'self'`, mais `blob:`/`data:` e `'wasm-unsafe-eval'` onde a lista de exceções desta spec permitir; nenhuma diretiva DEVE conter o token `'unsafe-eval'`, `*` ou origem remota, e `'unsafe-inline'` só pode aparecer onde a lista de exceções o prevê. | `pnpm test -- security` (a criar), lendo a política diretiva a diretiva, token a token                                                                                                                                                        |
| S2  | O servidor de desenvolvimento e o servidor dos E2E DEVEM servir a CSP do `tauri.conf.json` sem alteração (mesma string).                                                                                                                                                                                                                                                         | `pnpm test:e2e -- security` (a criar): lê o cabeçalho `Content-Security-Policy` da resposta e compara com o `tauri.conf.json`                                                                                                                 |
| S3  | QUANDO os E2E abrem cada fixture, nenhum evento `securitypolicyviolation` DEVE ocorrer; a asserção vale para toda a suíte, exceto o teste de S3b.                                                                                                                                                                                                                                | `pnpm test:e2e`: asserção automática que falha o teste ao capturar o evento (a criar)                                                                                                                                                         |
| S3b | QUANDO uma página tenta executar script inline ou carregar recurso de origem remota, o webview DEVE bloquear.                                                                                                                                                                                                                                                                    | `pnpm test:e2e -- security` (a criar): injeta as duas tentativas e espera o evento de violação                                                                                                                                                |
| S4  | QUANDO o app empacotado abre `e2e/fixtures/minimal.dxf` e o DWG de referência local, a contagem de entidades exibida DEVE ser maior que zero e o console NÃO DEVE registrar violação de CSP.                                                                                                                                                                                     | roteiro manual em `make dist-test`, Linux e Windows, com o console do webview aberto (build de depuração), resultado registrado no handoff — único critério manual desta spec, porque o app empacotado não expõe o console a teste automático |
| S6  | QUANDO um arquivo aberto referencia recurso remoto (imagem ou link), o recurso DEVE ser bloqueado e a área de mensagens DEVE informar quantos recursos não foram carregados.                                                                                                                                                                                                     | `pnpm test:e2e -- security` com fixture sintética que referencia imagem remota (a criar)                                                                                                                                                      |
| S5  | `capabilities/default.json` DEVE conter exatamente as permissões listadas na seção "Permissões" desta spec.                                                                                                                                                                                                                                                                      | `pnpm test -- security` compara as duas listas (a criar)                                                                                                                                                                                      |
