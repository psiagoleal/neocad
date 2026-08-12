---
name: novo-projeto
description: >-
  Instala o framework de regramento de agentes (perfil + biblioteca de skills)
  em um repositório novo ou já existente, escolhendo o perfil pela árvore de
  decisão, preservando conteúdo prévio e deixando o projeto pronto para
  --update não-destrutivo. Aciona ao criar um projeto novo, ao pedir para
  "configurar o agente", "aplicar o perfil", "instalar as regras", "usar o
  framework" em um diretório, ou ao adotar o framework em um repositório que já
  tem CLAUDE.md/AGENTS.md.
---

# novo-projeto — Adotar o framework em um repositório

O `scripts/setup-profile.sh` faz a cópia dos arquivos. Esta skill cobre o que o script
**não** faz: escolher o perfil, preservar o que já existia, e colocar o conteúdo específico
do projeto **onde ele sobrevive** às atualizações futuras.

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
scripts/setup-profile.sh <perfil> <alvo> --dry-run   # revise o plano
scripts/setup-profile.sh <perfil> <alvo>             # aplique
```

Opções que costumam importar:

| Situação | Opção |
|---|---|
| Checkout no Windows | `--skills-mode copy` (symlink não é portável) |
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
