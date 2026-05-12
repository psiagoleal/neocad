<!-- Caminho relativo: README.md -->

# NeoCAD

![Status](https://img.shields.io/badge/status-phase%201-blue)
![Build](https://img.shields.io/badge/build-scaffold_ready-success)
![Coverage](https://img.shields.io/badge/coverage-not_configured-lightgrey)
![Version](https://img.shields.io/badge/version-0.1.0-informational)
![License](https://img.shields.io/badge/license-MIT-green)
![Targets](https://img.shields.io/badge/targets-Windows%20%7C%20Linux-6f42c1)

NeoCAD é um wrapper desktop open-source para o [`cad-viewer`](https://github.com/mlightcad/cad-viewer), construído com **SvelteKit** e **Tauri 2**, com foco inicial em **Windows** e **Linux**. O objetivo é distribuir uma aplicação simples de instalar para abrir, visualizar e evoluir a edição básica de arquivos CAD diretamente no desktop, sem exigir que o usuário final instale `pnpm` ou configure um ambiente JavaScript.

## Estado atual

O repositório está na **Fase 1 — scaffold base concluído**.

Nesta fase, o projeto já possui:

- frontend em **SvelteKit + Svelte 5 + TypeScript**;
- configuração de **SPA mode** com `@sveltejs/adapter-static`, compatível com Tauri;
- shell desktop inicial em **Tauri 2 + Rust**;
- lint com **ESLint**;
- formatação com **Prettier**;
- testes unitários com **Vitest**;
- base de testes E2E com **Playwright**;
- documentação de arquitetura, desenvolvimento e roadmap.

> **Importante:** a integração com o `cad-viewer` ainda será implementada na próxima fase. Neste momento, a aplicação entrega o esqueleto técnico validado para evoluir o desktop wrapper.

## Objetivos do MVP

- empacotar o `cad-viewer` em uma aplicação desktop amigável;
- suportar abertura de arquivos locais `DWG` e `DXF`;
- oferecer visualização com navegação fluida;
- iniciar o suporte a **edição básica** aproveitando as capacidades já existentes no upstream;
- preparar a base para recursos futuros, como plugins, extensões BIM e melhorias de UX desktop.

## Princípios do projeto

- **MIT real open-source** para maximizar adoção e colaboração;
- **wrapper separado do upstream** para reduzir custo de manutenção;
- **desktop first**, sem perder a possibilidade de reaproveitamento web;
- **arquitetura modular**, com uma camada de adaptação entre NeoCAD e `cad-viewer`;
- **documentação desde o início**, para facilitar contribuição futura.

## Stack escolhida

- **Frontend:** SvelteKit
- **UI:** Svelte 5
- **Desktop shell:** Tauri 2
- **Linguagem da interface:** TypeScript
- **Backend nativo:** Rust
- **Gerenciador de pacotes JavaScript:** pnpm
- **Renderização CAD planejada:** `cad-viewer`

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
pnpm test
pnpm build
```

### Executar o app desktop

```bash
pnpm tauri dev
```

### Gerar build desktop

```bash
pnpm tauri build
```

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
│   └── changelog.md
├── e2e/                # Testes end-to-end do frontend
├── src/                # UI SvelteKit
├── static/             # Assets estáticos
├── src-tauri/          # Backend Rust e empacotamento Tauri
├── package.json
└── svelte.config.js
```

## Roadmap resumido

- [x] **Fase 0:** planejamento, arquitetura, licença e documentação
- [x] **Fase 1:** scaffold oficial com SvelteKit + Tauri 2
- [ ] **Fase 2:** integração do `cad-viewer` e abertura de arquivos locais
- [ ] **Fase 3:** edição básica, recentes, drag-and-drop e melhorias desktop
- [ ] **Fase 4:** extensibilidade, plugins e investigação de módulo BIM separado

## Como contribuir

Contribuições são bem-vindas, especialmente em:

- integração com `cad-viewer`;
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

## Referências externas

- [`cad-viewer` no GitHub](https://github.com/mlightcad/cad-viewer)
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
