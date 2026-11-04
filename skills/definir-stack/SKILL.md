---
name: definir-stack
description: >-
  Questionário de stack conduzido pelo agente, com os blocos escolhidos pelo
  tipo de projeto (biblioteca/CLI, serviço/API, app com interface, pipeline de
  dados ou núcleo de cálculo). Transforma as respostas nos lugares que duram —
  ilhas do AGENTS.md, ADR de stack e, quando há interface, o DESIGN.md — e
  exige que todo comando declarado tenha sido executado. Aciona ao começar
  projeto novo, ao adotar o framework num projeto cuja stack não está escrita,
  antes de escolher biblioteca ou framework, e quando o agente começa a
  adivinhar comandos de build ou teste.
---

# definir-stack — a stack se decide uma vez e se escreve onde dura

Stack não escrita não é ausência de decisão: é decisão delegada ao agente, que a toma de
novo a cada sessão, em silêncio, pelo que viu em outros projetos. O sintoma é sempre o
mesmo — o comando de teste que ele "lembra" não é o do projeto.

## A regra

> **Nenhuma linha de código antes de a stack estar escrita, e nenhum comando declarado sem
> ter sido executado.**

## Antes de perguntar, leia o repositório

Pergunta cuja resposta já está no disco queima paciência e ensina o usuário a responder no
automático. Comece detectando:

| Sinal | Responde |
|---|---|
| `pyproject.toml` / `uv.lock` / `requirements.txt` | linguagem, gerenciador, libs |
| `Cargo.toml` (e `[workspace]`) | Rust, crates, layout |
| `CMakeLists.txt` / `meson.build` | C++, build, dependências |
| `package.json` + `svelte.config.js` / `vite.config.*` | front, framework, bundler |
| `compose.yaml` / `Dockerfile` | serviços, banco, runtime |
| `.github/workflows/` | comandos que já rodam em CI — os mais confiáveis |
| `docs/adr/` | decisões de stack já tomadas: **não reabra sem motivo** |

Apresente o que encontrou e peça **confirmação**, não repetição: "detectei Python com uv e
pytest; confirma?".

## O questionário depende do tipo de projeto

Pergunte primeiro **uma** coisa — o tipo — e deixe que ele decida o resto. Perguntar os cinco
blocos a todo projeto é o modo mais rápido de tornar o questionário um ritual.

| Tipo | Blocos que valem | Blocos que **não** se pergunta |
|---|---|---|
| Biblioteca ou CLI | A, B, F | interface, banco, operação |
| Serviço ou API | A, B, C, D, F | interface (salvo se servir UI) |
| Aplicação com interface | A, B, C, D, E, F | — |
| Pipeline de dados / ML | A, B, C, D, F | interface |
| Núcleo de cálculo (C++/Rust) | A, B, F | banco, operação, interface |

**Bloco A — Identidade.** O que o projeto faz em duas frases; quem usa; o que seria fracasso.
Se o objetivo não estiver claro aqui, pare: isso é spec, não stack (`spec-como-contrato`).

**Bloco B — Núcleo.** Linguagem principal e versão mínima; gerenciador de dependência e
lockfile; formatador, linter e checagem de tipo; framework de teste. E o **layout**: qual é a
raiz única do código do sistema.

| Forma do sistema | Raiz recomendada |
|---|---|
| Mais de um componente (serviço + interface, ou serviço + contrato) | `app/`, com `app/backend/`, `app/frontend/`, `app/contract/` |
| Um componente, stack com convenção forte | a raiz idiomática: `src/` (Python), `src/` + `Cargo.toml` (Rust), `src-tauri/` + `src/` (Tauri) |
| Monorepo de várias aplicações | uma raiz por aplicação, todas irmãs, nomeadas pelo produto |

O que a resposta precisa produzir é **uma fronteira declarada**, não um nome bonito:
documentação, insumos e scripts do processo ficam fora da raiz do código, e a ilha
`estrutura-diretorios` do `AGENTS.md` diz qual é qual.

**Bloco C — Dados e integração.** Banco e versão; camada de acesso (SQL direto com
*prepared statements*, ORM, driver); migrações; filas e processamento assíncrono; APIs de
terceiros.

**Bloco D — Operação.** Onde roda (contêiner local, VPS, nuvem); CI; empacotamento e
publicação; observabilidade mínima (log estruturado já conta).

**Bloco E — Interface.** Framework e roteamento; estilização; onde mora o estado; gráficos e
tabelas; acessibilidade alvo. Em seguida, o contrato visual: **não** o discuta aqui — vá para
`contrato-de-design`, que produz o `docs/DESIGN.md`.

**Bloco F — Convenções.** Idioma do código, dos comentários e dos commits; convenção de
mensagem de commit; licença.

### Como conduzir

- **Um bloco por vez**, aguardando resposta. Bloco inteiro de uma vez vira formulário e
  recebe resposta apressada.
- **Ofereça o padrão do perfil** como resposta pronta: a seção 2 do `AGENTS.md` já diz as
  linguagens e frameworks preferenciais. A pergunta vira "mantemos o padrão?" — e só o desvio
  custa conversa.
- **Nunca invente para preencher.** Campo sem resposta fica pendente por escrito, não
  resolvido por suposição.
- **Divergiu do padrão do perfil?** Isso é decisão, e decisão vira ADR (`adr-writer`) com a
  alternativa descartada e o motivo.

## Onde a resposta é escrita

Não crie um `STACK.md`: o framework já tem os lugares certos, e duplicar é garantir
divergência.

| Resposta | Destino |
|---|---|
| Linguagens, frameworks, banco, runtime | ilha `USER:BEGIN id=linguagens-frameworks` do `AGENTS.md` |
| Comandos de instalar, formatar, checar tipo, testar, rodar | ilha `USER:BEGIN id=comandos-exatos` |
| Layout de diretórios | ilha `USER:BEGIN id=estrutura-diretorios` |
| Escolha com alternativa descartada | ADR em `docs/adr/` (`adr-writer`) |
| Contrato visual (cor, tipografia, espaço, componentes) | `docs/DESIGN.md` (`contrato-de-design`) |
| Dependências reais entre módulos, depois do primeiro código | `docs/architecture.md` (`mapa-de-arquitetura`) |

## Todo comando declarado foi executado

Comando na ilha `comandos-exatos` é promessa ao agente das próximas sessões. Rode cada um
**antes** de escrevê-lo e cole o que ele respondeu. O que não roda entra assim:

```bash
# cargo test        # ainda não configurado — ver MT-7
```

Comentado e com o ticket, nunca como se funcionasse. Comando inventado é pior que ausente: o
agente o executa, falha, e gasta a sessão consertando o que nunca existiu
(`gates-de-conclusao`).

## O que você vai pensar para pular o questionário

| O que você vai pensar | Por que não vale |
|---|---|
| "A stack é óbvia, é o de sempre" | Então o questionário são três confirmações e um minuto. |
| "Decido conforme for precisando" | Quem decide, nesse caso, é o agente — na pressa e sem registro. |
| "Já respondi isso na conversa" | Conversa não sobrevive à sessão. Ilha e ADR sobrevivem. |
| "Depois eu escrevo os comandos" | O primeiro comando errado custa mais que escrever todos. |
| "Perguntar tudo é chato" | Por isso o bloco depende do tipo, e o padrão do perfil responde a maioria. |

## Ligação com o resto do acervo

`novo-projeto` chama esta skill no passo 2 do roteiro de entrada · `spec-como-contrato` trata
do objetivo, que vem **antes** da stack · `adr-writer` guarda a escolha que teve alternativa ·
`contrato-de-design` produz o `DESIGN.md` quando há interface · `mapa-de-arquitetura` registra
o que a stack virou depois de construída · `gates-de-conclusao` exige a execução dos comandos.

## Definição de pronto da skill

- [ ] O tipo de projeto foi perguntado, e só os blocos dele foram usados.
- [ ] O que o repositório já respondia foi **detectado**, não perguntado.
- [ ] As respostas estão nas ilhas `USER:*` do `AGENTS.md`, não na conversa.
- [ ] Todo comando na ilha `comandos-exatos` foi executado; o que não roda está comentado com
      o ticket correspondente.
- [ ] Cada desvio do padrão do perfil virou ADR com a alternativa descartada.
- [ ] Campo sem resposta ficou registrado como pendência, não preenchido por suposição.
- [ ] Havendo interface, o `docs/DESIGN.md` foi criado antes do primeiro componente.
