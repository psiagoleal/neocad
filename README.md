<!-- Caminho relativo: README.md -->

# NeoCAD

![Status](https://img.shields.io/badge/status-phase%202-blue)
![Build](https://img.shields.io/badge/build-viewer_integration_in_progress-success)
![Coverage](https://img.shields.io/badge/coverage-not_configured-lightgrey)
![Version](https://img.shields.io/badge/version-0.1.0-informational)
![License](https://img.shields.io/badge/license-MIT-green)
![Targets](https://img.shields.io/badge/targets-Windows%20%7C%20Linux-6f42c1)

NeoCAD é um wrapper desktop open-source para o [`cad-viewer`](https://github.com/mlightcad/cad-viewer), construído com **SvelteKit** e **Tauri 2**, com foco inicial em **Windows** e **Linux**. O objetivo é distribuir uma aplicação simples de instalar para abrir, visualizar e evoluir a edição básica de arquivos CAD diretamente no desktop, sem exigir que o usuário final instale `pnpm` ou configure um ambiente JavaScript.

## Estado atual

O repositório está na **Fase 2 — integração inicial do viewer**.

Nesta etapa, o projeto já possui:

- frontend em **SvelteKit + Svelte 5 + TypeScript**;
- configuração de **SPA mode** com `@sveltejs/adapter-static`, compatível com Tauri;
- shell desktop inicial em **Tauri 2 + Rust**;
- integração inicial com o ecossistema MLightCAD via **`@mlightcad/cad-simple-viewer`**;
- abertura local de arquivos `DWG` e `DXF` por diálogo nativo no Tauri;
- fallback de abertura local no navegador para desenvolvimento web;
- workspace frontend modularizado em componentes Svelte dedicados sob `src/lib/components/workspace`;
- estilos compartilhados centralizados em `src/lib/styles`, carregados globalmente pelo layout da aplicação;
- fluxo desktop com menu superior no estilo desktop, tela inicial compacta, canvas principal e dock recolhível de mensagens;
- ações iniciais de viewport e envio de comandos CAD;
- barra de comandos do viewer upstream preservada dentro do canvas como mecanismo principal de comandos;
- lista de recentes persistida entre sessões no runtime Tauri, com fallback no navegador, e suporte inicial a arrastar-e-soltar;
- workers estáticos para DXF, DWG e MTEXT em `static/workers`;
- `Makefile`, `CMakeLists.txt` e `CMakePresets.json` para smoke checks e geração inicial de entregas Windows x64 (portable `.zip` e instalador NSIS current-user);
- lint com **ESLint**;
- formatação com **Prettier**;
- testes unitários com **Vitest**;
- base de testes E2E com **Playwright**;
- documentação de arquitetura, desenvolvimento e roadmap.

> **Importante:** a integração atual usa o pacote framework-agnostic `@mlightcad/cad-simple-viewer`, que é o núcleo mais adequado do upstream para uma interface Svelte. O pacote `@mlightcad/cad-viewer` continua sendo a referência principal do ecossistema e da evolução funcional.

## Objetivos do MVP

- empacotar o ecossistema `cad-viewer` em uma aplicação desktop amigável;
- suportar abertura de arquivos locais `DWG` e `DXF`;
- oferecer visualização com navegação fluida;
- iniciar o suporte a **edição básica** aproveitando as capacidades já existentes no upstream;
- preparar a base para recursos futuros, como plugins, extensões BIM e melhorias de UX desktop.

## Princípios do projeto

- **MIT real open-source** para maximizar adoção e colaboração;
- **wrapper separado do upstream** para reduzir custo de manutenção;
- **desktop first**, sem perder a possibilidade de reaproveitamento web;
- **arquitetura modular**, com uma camada de adaptação entre NeoCAD e o núcleo do viewer;
- **documentação desde o início**, para facilitar contribuição futura.

## Stack escolhida

- **Frontend:** SvelteKit
- **UI:** Svelte 5
- **Desktop shell:** Tauri 2
- **Linguagem da interface:** TypeScript
- **Backend nativo:** Rust
- **Gerenciador de pacotes JavaScript:** pnpm
- **Núcleo CAD integrado na Fase 2:** `@mlightcad/cad-simple-viewer`
- **Ecossistema upstream de referência:** `cad-viewer`

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
├── CHANGELOG.md
├── docs/
│   ├── architecture.md
│   ├── api.md
│   ├── development.md
│   ├── windows-cross-build.md
│   └── changelog.md
├── e2e/                # Testes end-to-end do frontend
├── src/                # UI SvelteKit
│   └── lib/
│       ├── components/ # Componentes Svelte do workspace desktop
│       ├── config/     # Metadados e configuração do app
│       ├── services/   # Serviços como seleção de arquivos CAD
│       ├── styles/     # Tokens, layout e estilos compartilhados
│       ├── types/      # Tipos compartilhados
│       └── viewer/     # Adaptador do viewer integrado
├── static/             # Assets estáticos
│   └── workers/        # Workers do viewer para DXF, DWG e MTEXT
├── src-tauri/          # Backend Rust, plugins e empacotamento Tauri
├── package.json
└── svelte.config.js
```

## Roadmap resumido

- [x] **Fase 0:** planejamento, arquitetura, licença e documentação
- [x] **Fase 1:** scaffold oficial com SvelteKit + Tauri 2
- [ ] **Fase 2:** integração do viewer, modularização do workspace e abertura de arquivos locais _(em andamento)_
- [ ] **Fase 3:** painéis de propriedades/camadas, catálogo de comandos em `Ajuda` e comandos CAD básicos
- [ ] **Fase 4:** persistência de preferências, atalhos e evolução da shell desktop
- [ ] **Fase 5:** investigação de módulo opcional de simulação FEM/CFD e extensibilidade avançada

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

Este projeto está licenciado sob a **MIT License**. Consulte o arquivo `LICENSE` para detalhes.

---

## Apoie

**Feito com ❤️ por Iago Leal** | [☕ Apoie o criador]

Se este projeto ajudou você, considere apoiar:

- Buy Me a Coffee: https://buymeacoffee.com/psiagoleal

<a href="https://buymeacoffee.com/psiagoleal" target="_blank" rel="noopener">
  <img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" height="41" width="174" />
</a>
