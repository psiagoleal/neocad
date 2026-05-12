<!-- Caminho relativo: docs/development.md -->

# Guia de desenvolvimento

## Objetivo deste documento

Este guia descreve como preparar o ambiente e como executar a próxima etapa do projeto após a Fase 0 de planejamento.

## Fase atual

A Fase 0 registra decisões de produto, arquitetura e governança. O scaffold do aplicativo será iniciado na Fase 1.

## Stack validada

- **SvelteKit** como frontend principal
- **Tauri 2** como shell desktop
- **TypeScript** para a aplicação web
- **Rust** para integração nativa
- **pnpm** como gerenciador de pacotes JavaScript

## Referências oficiais consideradas

### SvelteKit

A documentação oficial do SvelteKit indica o uso do CLI `sv` para criação de novos projetos:

- `npx sv create my-app`

### Tauri 2

A documentação oficial do Tauri 2 apresenta dois caminhos relevantes para NeoCAD:

1. criar um projeto novo com `create-tauri-app`;
2. adicionar Tauri manualmente a um frontend já existente.

Para NeoCAD, a abordagem preferida é:

- criar primeiro a base SvelteKit pelo CLI oficial;
- depois inicializar o Tauri manualmente no mesmo repositório.

Isso preserva maior controle sobre a estrutura do app e facilita a integração com `cad-viewer`.

## Pré-requisitos

### Node.js e pnpm

- Instalar `Node.js` LTS.
- Executar `corepack enable` para habilitar `pnpm`.
- Confirmar versões com `node -v` e `pnpm -v`.

### Rust

- Instalar `rustup`.
- Usar toolchain estável.
- No Windows, preferir `stable-msvc`.

### Dependências de sistema do Tauri 2

#### Windows

Segundo a documentação oficial do Tauri 2:

- instalar **Microsoft C++ Build Tools** com o workload “Desktop development with C++”;
- garantir a presença do **Microsoft Edge WebView2 Runtime**;
- instalar Rust com toolchain MSVC.

#### Linux

Para Ubuntu/Debian, a documentação oficial do Tauri 2 recomenda dependências como:

- `libwebkit2gtk-4.1-dev`
- `build-essential`
- `curl`
- `wget`
- `file`
- `libxdo-dev`
- `libssl-dev`
- `libayatana-appindicator3-dev`
- `librsvg2-dev`

## Estratégia de bootstrap da Fase 1

### Etapa 1 — criar o frontend com SvelteKit

Usar o CLI oficial do SvelteKit para gerar a base inicial do projeto.

Decisões esperadas no bootstrap:

- TypeScript habilitado;
- configuração alinhada com SvelteKit atual;
- estrutura preparada para uso com Tauri e build estático.

### Etapa 2 — configurar saída compatível com Tauri

Após o scaffold inicial, o projeto deverá:

- adotar `adapter-static` quando necessário para o fluxo de build desktop;
- manter URL de desenvolvimento local para integração com `tauri dev`;
- revisar estratégia de assets estáticos e roteamento para ambiente desktop.

### Etapa 3 — inicializar o Tauri 2 manualmente

Usar a CLI oficial do Tauri para adicionar o backend nativo ao projeto SvelteKit existente.

Parâmetros esperados no `tauri init`:

- diretório dos assets web compatível com o build do SvelteKit;
- URL do servidor de desenvolvimento local;
- comando de desenvolvimento do frontend;
- comando de build do frontend.

### Etapa 4 — validar execução local

Objetivo mínimo:

- `pnpm tauri dev` abrindo uma janela desktop funcional.

## Convenções iniciais

### Organização de código

- componentes reutilizáveis em `src/lib/components`;
- serviços em `src/lib/services`;
- adaptador do `cad-viewer` em `src/lib/viewer`;
- stores em `src/lib/stores`;
- comandos nativos organizados por domínio em `src-tauri/src/commands`.

### Formatação e qualidade

Ferramentas previstas:

- `prettier` para frontend;
- `eslint` para TypeScript/Svelte;
- `rustfmt` para Rust;
- `cargo clippy` para lint nativo.

### Testes

Estratégia inicial prevista:

- testes unitários para serviços e stores;
- testes de componentes críticos;
- validação futura de fluxos desktop com E2E.

## Fluxo de contribuição sugerido

1. abrir issue ou discussão para mudanças significativas;
2. documentar decisões de arquitetura quando houver impacto estrutural;
3. manter `README.md` e `CHANGELOG.md` atualizados;
4. incluir testes quando novas funcionalidades forem introduzidas.

## Riscos técnicos acompanhados

- integração entre roteamento do SvelteKit e empacotamento desktop;
- compatibilidade do `cad-viewer` com o ciclo de vida da aplicação Tauri;
- diferenças de dependências entre Windows e Linux;
- possível necessidade de patches ou contribuições upstream.

## Próxima entrega esperada

A próxima etapa prática do projeto é criar o scaffold oficial e validar a abertura da janela desktop com SvelteKit + Tauri 2.
