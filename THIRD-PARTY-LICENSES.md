<!-- Caminho relativo: THIRD-PARTY-LICENSES.md -->

# Licenças de terceiros

Este documento registra as licenças das dependências de runtime que o NeoCAD
**distribui** dentro de seus binários (`.zip` portátil, instalador NSIS, bundle
Linux). Dependências de desenvolvimento (linters, build tools, test runners) não
são distribuídas e não constam aqui.

O inventário é verificado automaticamente por `pnpm licenses:check`, contra a
política em [`scripts/license-policy.json`](./scripts/license-policy.json). A
verificação roda na CI e falha quando surge uma licença nova, desconhecida ou
copyleft ainda não avaliada.

Última verificação: **2026-08-06**, sobre o `pnpm-lock.yaml` corrente
(23 pacotes de runtime).

## Resumo

| Licença           | Pacotes | Situação                        |
| ----------------- | ------: | ------------------------------- |
| MIT               |      15 | Permissiva, compatível          |
| Apache-2.0        |       1 | Permissiva, compatível          |
| MIT OR Apache-2.0 |       3 | Permissiva, compatível          |
| ISC               |       1 | Permissiva, compatível          |
| 0BSD              |       1 | Permissiva, compatível          |
| **GPL-3.0**       |   **2** | **Copyleft — ver seção abaixo** |

## Dependências copyleft (GPL-3.0)

Duas dependências de runtime estão sob **GPL-3.0**, e ambas estão no caminho
crítico do produto — não são opcionais nem removíveis sem perder a função
central de ler arquivos CAD:

### 1. `@mlightcad/libredwg-web@0.7.1` — caminho DWG

```text
neocad
└── @mlightcad/cad-simple-viewer@1.5.0        (MIT)
    └── @mlightcad/libredwg-converter@3.5.33  (arquivo LICENSE: MIT)
        └── @mlightcad/libredwg-web@0.7.1     (GPL-3.0)
```

É um build WebAssembly da [LibreDWG](https://www.gnu.org/software/libredwg/) do
projeto GNU, licenciada sob GPL-3.0. O binário WASM está embutido em
`static/workers/libredwg-parser-worker.js` (~8,5 MB), que é copiado para o bundle
distribuído.

> Nota: `@mlightcad/libredwg-converter@3.5.33` publica um arquivo `LICENSE` MIT e
> não declara o campo `license` no `package.json`. Versões mais recentes do mesmo
> pacote (ex.: `3.12.3`) declaram GPL-3.0. Independentemente disso, a dependência
> `libredwg-web` que ele carrega é GPL-3.0.

### 2. `@mlightcad/dxf-json@1.2.0` — caminho DXF

```text
neocad
└── @mlightcad/data-model@1.7.33   (MIT)   ← dependência direta do NeoCAD
    └── @mlightcad/dxf-json@1.2.0  (GPL-3.0)
```

Fork do `dxf-json` sob GPL-3.0, usado no parsing de DXF. Entra no bundle
principal da aplicação via Vite, não apenas nos workers.

## Consequência para a licença do NeoCAD

Estas dependências foram a razão determinante do
[ADR 0002](./docs/adr/0002-relicenciamento-para-gpl-3.md), que relicenciou o
NeoCAD de MIT para **GPL-3.0-or-later**.

A MIT é compatível com a GPL-3.0 em apenas uma direção: código MIT pode ser
incorporado a uma obra GPL, não o inverso. Como os binários distribuídos combinam
o código do NeoCAD com bibliotecas GPL-3.0, a obra combinada já estava de fato
sujeita à GPL-3.0 enquanto o projeto anunciava MIT. O relicenciamento eliminou
essa divergência.

Obrigações que acompanham cada distribuição de binários:

- licenciar a distribuição sob GPL-3.0;
- oferecer o código-fonte completo correspondente aos destinatários;
- preservar avisos de licença e copyright.

> Este documento é um levantamento técnico de proveniência, não aconselhamento
> jurídico.

## Alternativas técnicas consideradas e descartadas

Preservar uma distribuição permissiva exigiria eliminar as duas dependências
copyleft. A avaliação registrada no ADR 0002 concluiu que isso é inviável:

- **DXF:** substituível. Existem parsers DXF sob licença permissiva, e o caminho
  DXF seria o mais fácil de trocar.
- **DWG:** inviável. O formato é proprietário e sem especificação aberta. As
  opções reais são a LibreDWG (GPL-3.0), as bibliotecas da Open Design Alliance
  (comerciais e fechadas) ou um parser próprio — este último com custo alto e
  altíssimo risco de compatibilidade. Abrir mão de DWG eliminaria a função
  central do produto.

O [ADR 0003](./docs/adr/0003-kernel-cad-proprio.md) mantém a leitura DWG sobre a
LibreDWG por essa mesma razão, mesmo na trilha de kernel próprio.

## Inventário completo

Gerado por `pnpm licenses:list`.

```text
@fxts/core@1.26.0                        Apache-2.0
@lukeed/csprng@1.1.0                     MIT
@mlightcad/cad-simple-viewer@1.5.0       MIT
@mlightcad/common@1.4.33                 MIT
@mlightcad/data-model@1.7.33             MIT
@mlightcad/dxf-json@1.2.0                GPL-3.0
@mlightcad/geometry-engine@3.2.33        MIT
@mlightcad/graphic-interface@3.3.33      MIT
@mlightcad/libredwg-converter@3.5.33     MIT
@mlightcad/libredwg-web@0.7.1            GPL-3.0
@tauri-apps/api@2.11.0                   Apache-2.0 OR MIT
@tauri-apps/plugin-dialog@2.7.1          MIT OR Apache-2.0
@tauri-apps/plugin-fs@2.5.1              MIT OR Apache-2.0
iconv-lite@0.7.2                         MIT
lodash-es@4.17.21                        MIT
loglevel@1.9.2                           MIT
mitt@3.0.1                               MIT
quickselect@3.0.0                        ISC
rbush@4.0.1                              MIT
safer-buffer@2.1.2                       MIT
three@0.172.0                            MIT
tslib@2.8.1                              0BSD
uid@2.0.2                                MIT
```

## Workers do upstream

Os três workers em `static/workers/` **não são mais versionados** no
repositório. São derivados de `node_modules` em tempo de build por
[`scripts/sync-workers.mjs`](./scripts/sync-workers.mjs); apenas
`static/workers/workers.manifest.json` (versão de origem + SHA-256 de cada
arquivo) é versionado, e `pnpm workers:check` falha na CI se o conteúdo do
upstream mudar sem revisão.

Antes dessa mudança, ~9,7 MB de código minificado de terceiros — incluindo o
build GPL-3.0 da LibreDWG — estavam commitados sem registro de origem, versão ou
licença.
