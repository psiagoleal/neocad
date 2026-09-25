---
name: novo-projeto
description: >-
  Instala o framework de regramento de agentes (perfil + biblioteca de skills)
  em um repositório novo ou já existente, escolhendo o perfil pela árvore de
  decisão, preservando conteúdo prévio e deixando o projeto pronto para
  --update não-destrutivo. Aciona ao criar um projeto novo, ao pedir para
  "configurar o agente", "aplicar o perfil", "instalar as regras", "usar o
  framework" em um diretório, ao adotar o framework em um repositório que já
  tem CLAUDE.md/AGENTS.md, ou ao perguntar "por onde começo" num projeto novo
  ou num projeto existente que está entrando agora no processo agêntico.
---

# novo-projeto — Adotar o framework em um repositório

O `scripts/setup-profile.sh` faz a cópia dos arquivos. Esta skill cobre o que o script
**não** faz: escolher o perfil, preservar o que já existia, colocar o conteúdo específico
do projeto **onde ele sobrevive** às atualizações futuras, e o **roteiro de entrada** — a
primeira rodada de trabalho, que é diferente num projeto novo e num projeto que já existe
(seção 7).

## 1. Escolher o perfil

```
É da empresa?                        → empresa
Tem cliente / NDA / dado sensível?   → externo-confidencial
É open-source / público?             → pessoal
Nenhum dos anteriores                → externo-confidencial (padrão conservador)
```

A escolha define postura de confidencialidade, sandbox e licenciamento — **não a adivinhe.**
Pergunte ao usuário, apresentando a árvore. Ver `docs/comparativo-perfis.md`.

## 2. Preparar o alvo antes de instalar

```bash
cd <alvo>
git init                       # se ainda não for repo
git add -A && git commit -m "estado inicial antes do framework"
```

O commit base não é burocracia: o `--update` futuro é revisado por `git diff`, e sem
baseline você não distingue o que o script escreveu do que já estava lá.

⚠️ **Se o alvo já tem `CLAUDE.md` com conteúdo próprio**, resolva **antes** de rodar o
script. O `setup-profile.sh` **pula** arquivos existentes: seu `CLAUDE.md` antigo
permaneceria e passaria a competir com o `AGENTS.md` — que é a fonte única da verdade.

Fluxo correto:

1. Copie o conteúdo atual para um lugar seguro.
2. Remova o `CLAUDE.md` antigo (o perfil instala um ponteiro fino no lugar).
3. Instale o perfil.
4. Redistribua o conteúdo conforme a seção 4.

## 3. Instalar — sempre com `--dry-run` primeiro

```bash
scripts/setup-profile.sh --doctor                    # pré-requisitos, por sistema
scripts/setup-profile.sh <perfil> <alvo> --dry-run   # revise o plano
scripts/setup-profile.sh <perfil> <alvo>             # aplique
```

Com o usuário presente e sem pressa, `scripts/setup-profile.sh` **sem argumentos** abre a
instalação guiada: pergunta perfil e alvo, mostra a prévia e pede confirmação. Em script ou
CI, use a forma direta acima — o modo guiado só liga com terminal interativo.

Opções que costumam importar:

| Situação | Opção |
|---|---|
| Checkout no Windows | `--skills-mode copy` (o modo guiado já detecta e troca sozinho) |
| Só a pasta neutra, sem agente | `--skills-mode none` |
| Subconjunto de skills | `--skills a,b,c` |
| Repo já configurado | `--update` (ver seção 6) |

## 4. Redistribuir o conteúdo do projeto — a parte que importa

Depois de instalar, **quase tudo em `AGENTS.md` é regenerado** no próximo `--update`. Só
sobrevive o que está dentro das ilhas de marcadores:

| Arquivo | Ilhas disponíveis |
|---|---|
| `AGENTS.md` | `id=comandos-exatos`, `id=estrutura-diretorios` |
| `.claudeignore` | `id=projeto-ignore` |
| `.env.example` | conforme o perfil |

Regra prática:

- **Cabe em poucas linhas e é comando ou caminho?** → dentro da ilha correspondente.
- **É conhecimento extenso** (ambiente, formatos de arquivo, domínio, achados de
  investigação)? → arquivo próprio em `docs/`, **referenciado de dentro de uma ilha** para
  que o agente seja obrigado a encontrá-lo.

Escrever conhecimento de projeto fora das ilhas e fora de `docs/` é perdê-lo silenciosamente
no primeiro `--update`.

Ajuste também:

- `.claudeignore` — cubra os diretórios de dados **reais** do projeto. Os padrões do
  template (`data/`, `client_data/`) raramente batem com a estrutura real.
- `.gitignore` — dados volumosos e segredos.
- `docs/CURRENT-STATE.md` — preencha o estado real; é o primeiro arquivo que a próxima
  sessão lê. Registre impedimentos abertos e testes pendentes.

## 5. Verificar antes de encerrar

```bash
cd <alvo>
git status --short                    # o que entrou
ls -la .claude/skills/                # symlinks resolvem?
grep -n "USER:BEGIN" AGENTS.md        # ilhas presentes e preenchidas
```

## 6. Atualizações futuras (`--update`)

```bash
scripts/setup-profile.sh <perfil> <alvo> --update --dry-run
scripts/setup-profile.sh <perfil> <alvo> --update
```

Baldes: **regra/ponteiro** (sobrescrito) · **híbrido** (merge preservando ilhas `USER:*`) ·
**vivo** (`CURRENT-STATE.md`, ADRs, `.env` — nunca tocados). Nada é apagado.

Requer `jq` para o merge do `.claude/settings.json`. Árvore git limpa antes; revise com
`git diff` depois.

**Edição fora das ilhas não se perde mais em silêncio.** O instalador grava em
`.agent-profile/baseline.sha256` a impressão digital do texto do framework em cada arquivo
(versione essa pasta). Se alguém editou esse texto no projeto, o `--update` não sobrescreve:
deixa o original, grava `<arquivo>.new` e lista os casos no fim. Resolva comparando os dois,
movendo o que for do projeto para uma ilha `USER:BEGIN/END`, substituindo pelo `.new` e
rodando de novo. `--force` descarta a edição local. Instalação anterior a esse mecanismo
recebe um aviso "sem linha de base" uma única vez — é o momento de revisar com `git diff`.

## 7. Roteiro de entrada — a primeira rodada depois de instalar

Os dois casos começam igual (seções 1–5) e divergem no que vem antes do primeiro código.
Em ambos, **o humano decide e o agente prepara**: o agente não escolhe perfil, não aprova a
própria spec e não reescreve código existente para caber no processo.

### Projeto novo

| # | Passo | Skill | Quem decide |
|---|---|---|---|
| 1 | Instalar o perfil (seções 1–5) | `novo-projeto` | humano escolhe o perfil |
| 2 | Agente **entrevista** o humano sobre o objetivo; spec **pequena** da primeira feature | `spec-como-contrato` | humano aprova a spec |
| 2b | Questionário de stack pelo tipo de projeto; havendo interface, `docs/DESIGN.md` | `definir-stack`, `contrato-de-design` | humano responde e aprova |
| 3 | ADR das decisões de stack que o agente precisará respeitar | `adr-writer` | humano aprova |
| 4 | Fatiar em micro-tickets e abrir o `docs/TICKETS.md` | `micro-ticket-planner` | humano ordena |
| 5 | Cada ticket: teste falhando → implementação → gate com evidência | `teste-primeiro`, `gates-de-conclusao` | comando decide |
| 6 | Ao surgir o segundo módulo, gerar `docs/architecture.md` | `mapa-de-arquitetura` | humano revisa |
| 7 | Antes do merge: revisão; ao encerrar a sessão: handoff | `pr-review-guard`, `handoff-updater` | humano faz o merge |

### Projeto existente entrando agora no processo

A regra da entrada é **levantar antes de mudar**: nenhum código de produto é alterado até os
passos 2–4 estarem feitos. Os documentos gerados aqui descrevem o que existe, não o que se
gostaria que existisse.

| # | Passo | Skill | Quem decide |
|---|---|---|---|
| 1 | Commit base, redistribuir o `CLAUDE.md` antigo, instalar (seções 1–5) | `novo-projeto` | humano escolhe o perfil |
| 2 | Varrer segredos já versionados e ajustar `.claudeignore`/`.gitignore` | `secrets-guard` | humano decide o que fazer com achados |
| 3 | Rodar build e testes **de verdade** e registrar a stack e os comandos que funcionam nas ilhas — e o que não funciona, comentado | `definir-stack`, `gates-de-conclusao` | comando decide |
| 4 | Gerar `docs/architecture.md` a partir do código | `mapa-de-arquitetura` | humano corrige responsabilidades |
| 5 | `docs/CURRENT-STATE.md` com o estado real: dívidas, testes quebrados, trabalho em curso | `handoff-updater` | humano confirma |
| 6 | ADR **retroativo** só para decisões vigentes que o agente precisa respeitar — não reconstituir a história | `adr-writer` | humano aprova |
| 7 | `docs/TICKETS.md` a partir do trabalho **em curso**; o passado não vira ticket | `micro-ticket-planner` | humano ordena |
| 8 | Daqui em diante, o ciclo do projeto novo (passos 2, 4, 5, 7). Mexer em código sem teste começa por **teste de caracterização** | `spec-como-contrato`, `teste-primeiro` | — |

### Ativar o hook de mensagem de commit

Nos dois casos, uma vez por clone (o `.git/hooks` não é versionado):

```bash
ln -sf ../../skills/pr-review-guard/scripts/checar-mensagem-commit.sh .git/hooks/commit-msg
```

## Princípios

- **Perfil se pergunta, não se deduz** — a escolha tem consequência de confidencialidade.
- **Baseline em git antes de instalar** — sem diff não há revisão.
- **Conteúdo de projeto vive em ilha `USER:*` ou em `docs/`** — nunca solto no `AGENTS.md`.
- **`--dry-run` sempre** — na instalação e no update.
- **Não duplique regra** entre `CLAUDE.md` e `AGENTS.md`: o primeiro é ponteiro, o segundo
  é a fonte da verdade.

## Definição de pronto da skill

- [ ] Perfil escolhido **com o usuário**, pela árvore de decisão.
- [ ] Alvo é repo git com commit base anterior à instalação.
- [ ] Conteúdo prévio de `CLAUDE.md` preservado e redistribuído (nada perdido, nada duplicado).
- [ ] Ilhas `comandos-exatos` e `estrutura-diretorios` preenchidas com a realidade do projeto.
- [ ] Conhecimento extenso em `docs/`, referenciado de dentro de uma ilha.
- [ ] `.claudeignore` e `.gitignore` cobrem os dados e segredos reais do projeto.
- [ ] `docs/CURRENT-STATE.md` reflete o estado real, com impedimentos e pendências.
- [ ] `.claude/skills/` resolve corretamente (symlinks válidos ou cópias presentes).
- [ ] Hook `commit-msg` ativo no clone.
- [ ] Roteiro de entrada da seção 7 seguido — em projeto existente, `docs/architecture.md` e
      `docs/CURRENT-STATE.md` gerados **antes** da primeira mudança de código.
