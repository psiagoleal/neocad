<!-- Caminho relativo: README.md -->

# NeoCAD

![Status](https://img.shields.io/badge/status-planning-blue)
![Build](https://img.shields.io/badge/build-not_configured-lightgrey)
![Coverage](https://img.shields.io/badge/coverage-not_configured-lightgrey)
![Version](https://img.shields.io/badge/version-0.1.0--planning-informational)
![License](https://img.shields.io/badge/license-MIT-green)
![Targets](https://img.shields.io/badge/targets-Windows%20%7C%20Linux-6f42c1)

NeoCAD é um wrapper desktop open-source para o [`cad-viewer`](https://github.com/mlightcad/cad-viewer), construído com **SvelteKit** e **Tauri 2**, com foco inicial em **Windows** e **Linux**. O objetivo é distribuir uma aplicação simples de instalar para abrir, visualizar e evoluir a edição básica de arquivos CAD diretamente no desktop, sem exigir que o usuário final instale `pnpm` ou configure um ambiente JavaScript.

## Estado atual

O repositório está na **Fase 0 — Planejamento e arquitetura**.

Nesta fase, o foco é:

- definir a visão de produto;
- registrar decisões arquiteturais iniciais;
- preparar documentação e governança básica do projeto;
- alinhar a estratégia de integração com o `cad-viewer`;
- planejar o bootstrap oficial com `SvelteKit` e `Tauri 2`.

> **Importante:** neste momento ainda não existe um binário distribuível nem um scaffold funcional commitado. As instruções de ambiente abaixo servem para preparar a próxima fase de implementação.

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
- **Desktop shell:** Tauri 2
- **Linguagem da interface:** TypeScript
- **Backend nativo:** Rust
- **Gerenciador de pacotes JavaScript:** pnpm
- **Renderização CAD:** `cad-viewer`

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

Consulte também:

- `docs/development.md`
- documentação oficial do Tauri 2 para pré-requisitos por sistema operacional

## Instalação

### Para acompanhar o projeto agora

1. Clone este repositório.
2. Leia `docs/architecture.md` e `docs/development.md`.
3. Prepare o ambiente de desenvolvimento conforme sua plataforma.
4. Aguarde ou contribua com a Fase 1, em que o scaffold oficial será adicionado.

### Fluxo planejado para a Fase 1

```bash
# Exemplo planejado após o bootstrap oficial
pnpm install
pnpm tauri dev
```

## Exemplos de uso planejados

### Executar em modo de desenvolvimento

```bash
pnpm tauri dev
```

### Gerar build desktop

```bash
pnpm tauri build
```

## Estrutura de diretórios planejada

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
├── src/                 # UI SvelteKit
├── static/              # Assets estáticos
└── src-tauri/           # Backend Rust e empacotamento Tauri
```

## Roadmap resumido

- **Fase 0:** planejamento, arquitetura, licença e documentação
- **Fase 1:** scaffold oficial com SvelteKit + Tauri 2
- **Fase 2:** integração do `cad-viewer` e abertura de arquivos locais
- **Fase 3:** edição básica, recentes, drag-and-drop e melhorias desktop
- **Fase 4:** extensibilidade, plugins e investigação de módulo BIM separado

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
