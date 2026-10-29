<!-- Caminho relativo: README.md -->

# NeoCAD

[![CI](https://github.com/psiagoleal/neocad/actions/workflows/ci.yml/badge.svg)](https://github.com/psiagoleal/neocad/actions/workflows/ci.yml)
![Status](https://img.shields.io/badge/status-fase%20K3-blue)
![Coverage](https://img.shields.io/badge/coverage-not_configured-lightgrey)
[![Version](https://img.shields.io/github/v/release/psiagoleal/neocad?include_prereleases&sort=semver)](https://github.com/psiagoleal/neocad/releases)
![License](https://img.shields.io/badge/license-GPL--3.0--or--later-green)
![Targets](https://img.shields.io/badge/targets-Windows%20%7C%20Linux-6f42c1)

NeoCAD é um CAD **open-source** para **desenho de projetos de engenharia**, com kernel
próprio em Rust (compilado para WebAssembly), interface em **SvelteKit** e empacotamento
desktop com **Tauri 2** para **Windows** e **Linux**. O objetivo é que um engenheiro ou
técnico vindo do AutoCAD produza a prancha de entrega — carimbo, cotas, textos, hachuras,
blocos, tipos de linha e plotagem em escala — usando **os mesmos comandos**, digitados em
inglês ou em português.

## Estado atual

O NeoCAD **abre, exibe e grava** desenhos, mas ainda não é um editor para uso em projeto.

- **Kernel próprio** (`kernel/`): modelo de documento com toda mutação por transação
  reversível e `Desfazer`/`Refazer`; leitura e gravação DXF nativas; layouts de
  espaço-papel com viewports; geometria 2D com interseção, índice espacial, prendimento do
  cursor, mover, copiar, girar, espelhar, escalar e paralela.
- **Interface:** abre `DWG` e `DXF`, troca de layout, lista recentes, mostra o que o
  kernel não modela e avisa o que se perde ao salvar.
- **Ainda a cargo do upstream** [`@mlightcad`](https://github.com/mlightcad/cad-viewer): a
  leitura de DWG e o desenho na tela. Fazer a tela mostrar o modelo do kernel é a próxima
  fase (KV).

O que está sendo feito agora: [`docs/CURRENT-STATE.md`](docs/CURRENT-STATE.md). O que
existe e o que falta: [`docs/TICKETS.md`](docs/TICKETS.md).

## Princípios do projeto

- **o entregável é a prancha de engenharia**, e cada fase é julgada por quanto aproxima
  o NeoCAD dela ([ADR 0007](docs/adr/0007-fase-de-prancha-de-engenharia.md));
- **paridade de comandos com o AutoCAD**, em inglês e em português, com a grafia oficial
  de cada idioma ([spec 08](docs/specs/08-linha-de-comando.md));
- **kernel próprio e reutilizável**, com o upstream tratado como parser e renderer de
  transição ([ADR 0003](docs/adr/0003-kernel-cad-proprio.md));
- **toda edição é reversível**, e o que não se consegue gravar é avisado antes;
- **copyleft real (GPL-3.0)**, para que melhorias derivadas retornem à comunidade;
- **contrato antes do código**: cada funcionalidade tem spec com critério verificável em
  [`docs/specs/`](docs/specs/README.md).

## Stack escolhida

- **Kernel CAD:** Rust, compilado para WebAssembly com `wasm-pack`
- **Interface:** SvelteKit + Svelte 5 + TypeScript (modo SPA)
- **Desktop:** Tauri 2 (Linux e Windows)
- **Gerenciadores:** pnpm e cargo
- **Upstream de transição:** `@mlightcad/cad-simple-viewer` (leitura DWG e desenho na tela)

## Pré-requisitos de desenvolvimento

### Comuns

- `git`
- `Node.js` LTS atual
- `corepack enable` para habilitar `pnpm`
- `Rust` estável via `rustup`

### Windows

- Microsoft C++ Build Tools
- Microsoft Edge WebView2 Runtime
- `rustup` com toolchain `stable-msvc`

### Linux

As dependências exatas variam conforme a distribuição. Para Ubuntu/Debian, a documentação oficial do Tauri 2 recomenda bibliotecas como `libwebkit2gtk-4.1-dev`, `build-essential`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` e `libxdo-dev`.

## Instalação

1. Clone este repositório.
2. Instale as dependências JavaScript.
3. Garanta que os pré-requisitos do Tauri 2 estejam instalados no seu sistema.

```bash
pnpm install
```

## Execução em desenvolvimento

### Validar frontend

```bash
pnpm check
pnpm lint
pnpm test
pnpm build
```

### Validar backend desktop

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

### Executar o app desktop

```bash
pnpm tauri dev
```

### Gerar build desktop

```bash
pnpm tauri build
```

### Atalhos com Makefile

```bash
make help
make tauri-dev
make tauri-debug-nobundle
make cmake-smoke
make cmake-windows-x64-portable
make cmake-windows-x64-nsis
```

## Fluxo atual do app

A interface atual já permite:

- começar pelo fluxo de integração inicial e navegar para o canvas principal quando um desenho é aberto;
- usar um menu superior no estilo desktop com `Arquivo`, `Exibir`, `Janela` e `Ajuda`;
- mostrar, ocultar e acompanhar o dock de mensagens da integração;
- abrir um desenho CAD local;
- reabrir itens recentes no runtime Tauri quando o caminho estiver disponível;
- persistir a lista de desenhos recentes entre sessões do app desktop;
- arrastar e soltar arquivos CAD na área do viewer;
- ajustar a vista ao desenho carregado;
- alternar o fundo do canvas;
- usar a barra de comandos disponibilizada pelo viewer dentro do canvas;
- acompanhar mensagens e progresso de abertura.

## Testes

### Testes unitários

```bash
pnpm test
```

### Testes E2E

Antes da primeira execução dos testes E2E, instale os browsers do Playwright:

```bash
pnpm exec playwright install
pnpm test:e2e
```

## Estrutura de diretórios

```text
.
├── README.md
├── LICENSE
├── THIRD-PARTY-LICENSES.md   # Proveniência e licenças das dependências distribuídas
├── CHANGELOG.md
├── docs/
│   ├── architecture.md
│   ├── api.md
│   ├── development.md
│   └── windows-cross-build.md
├── e2e/                # Testes end-to-end do frontend
├── src/                # UI SvelteKit
│   └── lib/
│       ├── components/ # Componentes Svelte do workspace desktop
│       ├── config/     # Metadados e configuração do app
│       ├── services/   # Serviços como seleção de arquivos CAD
│       ├── styles/     # Tokens, layout e estilos compartilhados
│       ├── types/      # Tipos compartilhados
│       └── viewer/     # Adaptador do viewer integrado
├── scripts/            # Automação: release, sync de workers, política de licenças
├── static/             # Assets estáticos
│   └── workers/        # Workers do upstream (derivados de node_modules no build)
├── src-tauri/          # Backend Rust, plugins e empacotamento Tauri
├── package.json
└── svelte.config.js
```

## Roadmap resumido

Fases definidas nos ADRs [0003](docs/adr/0003-kernel-cad-proprio.md) e
[0007](docs/adr/0007-fase-de-prancha-de-engenharia.md):

- [x] **K1** — modelo de documento e transações reversíveis
- [x] **K2** — DXF nativo: leitura e gravação
- [x] **KL** — layouts de espaço-papel
- [ ] **K3** — geometria 2D e edição _(em curso: faltam aparar/estender, concordância/chanfro e a fachada WebAssembly)_
- [ ] **KV** — o kernel é o que se vê: uma única fonte de verdade para tela e arquivo
- [ ] **UI/UX** — interface de CAD com linha de comando bilíngue e paridade AutoCAD
- [ ] **KP** — prancha de engenharia: propriedades, geometria complementar, texto, blocos, cotas, hachura e plotagem/PDF
- [ ] **K6** — leitura DWG própria · **K4** — restrições 2D · **K7–K9** — núcleo 3D

## Como contribuir

Contribuições são bem-vindas, especialmente em:

- integração com o ecossistema `cad-viewer`;
- UX desktop com Tauri 2;
- testes automatizados;
- empacotamento para Windows e Linux;
- documentação técnica;
- estudos para recursos BIM em módulo separado.

Antes de abrir uma contribuição grande, prefira registrar uma discussão ou issue com:

- problema a resolver;
- proposta de abordagem;
- impacto esperado no MVP;
- possíveis riscos para manutenção.

Para o planejamento atual de painéis, comandos CAD e trilha macro de simulação numérica, consulte `docs/cad-panels-commands-simulation-roadmap.md`.

## Build Windows via CMake

O repositório agora inclui um fluxo inicial de entregas Windows x64 com **CMake** como orquestrador.

Saídas previstas:

- `windows-x64-portable`: gera um `.zip` extraível a partir de `tauri build --no-bundle`; se existir um Fixed WebView2 Runtime extraído em `.webview2/fixed-runtime-x64`, o pacote inclui esse runtime e um launcher `NeoCAD-portable.cmd`.
- o cross-build Windows a partir de Linux/WSL requer `cargo-xwin` e também o binário host `llvm-rc` disponível para o Tauri compilar recursos Windows.
- `windows-x64-nsis`: gera um instalador **NSIS** simples em modo `currentUser`, evitando admin por padrão e usando `embedBootstrapper` para instalar o WebView2 Runtime quando necessário.
- `windows-x64-nsis-fixed-runtime`: variação opcional para ambientes offline ou controlados, embutindo um Fixed WebView2 Runtime já extraído localmente.
- o runtime fixo foi movido para `.webview2/fixed-runtime-x64`, fora de `build/`, porque o `pnpm build` limpa a pasta de saída do frontend durante o processo do Tauri.

> Importante: esse fluxo é útil para validação e builds iniciais a partir de Linux/WSL, mas o release oficial de Windows continua mais seguro em runner Windows nativo, especialmente para assinatura.

Consulte:

- `docs/windows-cross-build.md`
- `src-tauri/tauri.windows.conf.json`
- `src-tauri/tauri.windows.fixed-runtime.conf.json`

## Referências externas

- [`cad-viewer` no GitHub](https://github.com/mlightcad/cad-viewer)
- [`@mlightcad/cad-viewer` no npm](https://www.npmjs.com/package/@mlightcad/cad-viewer)
- [`@mlightcad/cad-simple-viewer` no npm](https://www.npmjs.com/package/@mlightcad/cad-simple-viewer)
- [Documentação do SvelteKit](https://kit.svelte.dev/docs)
- [Documentação do Tauri 2](https://v2.tauri.app/)

## Licença

O NeoCAD está licenciado sob a **GNU General Public License v3.0 ou posterior** (`GPL-3.0-or-later`). Consulte o arquivo [`LICENSE`](./LICENSE) para o texto integral.

A escolha está registrada no [ADR 0002](./docs/adr/0002-relicenciamento-para-gpl-3.md). O projeto depende, no caminho crítico de leitura de arquivos CAD, de bibliotecas sob GPL-3.0 — a **LibreDWG** (formato DWG) e o **`dxf-json`** (formato DXF) —, e a licença do projeto acompanha essa realidade. O levantamento completo de proveniência das dependências distribuídas está em [`THIRD-PARTY-LICENSES.md`](./THIRD-PARTY-LICENSES.md).

Ao redistribuir binários do NeoCAD, a GPL-3.0 exige a oferta do código-fonte correspondente aos destinatários.

A verificação é automatizada:

```bash
pnpm licenses:check   # falha se entrar uma licença nova ou copyleft não avaliada
pnpm licenses:list    # inventário completo das dependências de runtime
```

---

## Apoie

**Feito com ❤️ por Iago Leal** | [☕ Apoie o criador]

Se este projeto ajudou você, considere apoiar:

- Buy Me a Coffee: https://buymeacoffee.com/psiagoleal

<a href="https://buymeacoffee.com/psiagoleal" target="_blank" rel="noopener">
  <img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" height="41" width="174" />
</a>
