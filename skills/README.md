<!-- Caminho relativo: skills/README.md -->

# Biblioteca de Skills de Governança para Agentes de IA

Skills (Habilidades de Agente) são pacotes modulares, autocontidos e versionáveis,
descritos por um arquivo `SKILL.md` com _frontmatter_ YAML (`name`, `description`). Sua
propriedade central é a **divulgação progressiva** (_progressive disclosure_): o agente
carrega apenas o cabeçalho descritivo na inicialização e só expande o corpo completo
quando a tarefa se enquadra nos gatilhos da `description`. Isso reduz o consumo de tokens
em sessões onde a capacidade não é necessária e permite manter uma biblioteca extensa sem
inflar o contexto de cada interação.

A biblioteca tem **dois níveis**, e a distinção é o que mantém o catálogo previsível:

- **Governança/fluxo** (raiz de `skills/`) — independentes de setor, reutilizáveis nos três
  perfis. É o conjunto **instalado por padrão** em qualquer projeto.
- **Domínio** (agrupadas em categoria, ex.: `dominio/`) — presas a uma tecnologia ou
  assunto. **Nunca** entram por padrão; exigem `--skills` explícito com o caminho da
  categoria. Um projeto de linhas de transmissão não deve carregar uma skill de
  prototipagem de UI só porque ela existe na biblioteca.

## Catálogo — governança

| Skill                                                       | Para quê                                                                                             | Aciona quando                                                                   |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| [`secrets-guard`](secrets-guard/SKILL.md)                   | Não-exposição de segredos antes de comandos/saída                                                    | Há credenciais, `.env`, cofres, `aws/gcloud/kubectl config`                     |
| [`adr-writer`](adr-writer/SKILL.md)                         | Criar/consultar ADRs como restrição cognitiva                                                        | Decisão de stack/biblioteca/solver; "registrar decisão"                         |
| [`micro-ticket-planner`](micro-ticket-planner/SKILL.md)     | Quebrar trabalho em tickets de um ciclo de contexto                                                  | Planejar sprint; tarefa ampla/ambígua                                           |
| [`handoff-updater`](handoff-updater/SKILL.md)               | Manter `docs/CURRENT-STATE.md`                                                                       | Após commit/ticket; "onde paramos"                                              |
| [`pr-review-guard`](pr-review-guard/SKILL.md)               | Checklist do "problema dos 80%" + OWASP                                                              | Antes de abrir/aprovar PR ou merge                                              |
| [`delegacao-a-subagentes`](delegacao-a-subagentes/SKILL.md) | O que fica no modelo local vs. o que vai para a nuvem, e como pedir                                  | Tarefa toca dado sensível; roteamento por task-class; "delegar"/"modelo local"  |
| [`novo-projeto`](novo-projeto/SKILL.md)                     | Adotar o framework num repo: escolher perfil, preservar conteúdo prévio, deixar pronto p/ `--update` | Projeto novo; "configurar o agente" / "aplicar o perfil" / "instalar as regras" |

## Catálogo — domínio (sob demanda)

| Skill                                               | Para quê                                                                                                  | Aciona quando                                                                |
| --------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| [`dominio/mockup-lab`](dominio/mockup-lab/SKILL.md) | Prototipar UI em HTML/CSS, comparar variantes por render headless, exportar a aprovada como SVG p/ Penpot | Prototipar/mockar tela; comparar variantes de layout; levar design ao Penpot |
| [`dominio/pc-builder`](dominio/pc-builder/SKILL.md) | Preço/disponibilidade em KaBuM, Pichau e Terabyte + compatibilidade de peças                              | Montar/atualizar PC; comparar preço de componente; acompanhar queda de preço |

Estas trazem _scripts_ anexos e dependências próprias (Playwright, `curl_cffi`), instaladas
sob demanda — por isso `node_modules/`, `.venv/` e `__pycache__/` nunca são copiados para o
alvo. Dado do usuário (listas de compra, mockups) mora no diretório de trabalho do projeto,
**nunca dentro da skill**.

## Catálogo — deste projeto

<!-- USER:BEGIN id=catalogo-local -->

_(nenhuma — acrescente aqui as skills criadas especificamente para este repositório)_

<!-- USER:END -->

> Skills próprias do projeto vivem na pasta neutra ao lado das demais e **não são tocadas**
> pelo `setup-profile.sh`: ele só escreve as skills que instala. Registre-as no bloco acima
> para que o catálogo continue completo depois de um `--update`.

## Modelo de instalação (independente de agente)

As skills são descritas pelo padrão `SKILL.md`, **portável entre plataformas**. Para não
amarrar a biblioteca a um único agente, adotamos dois níveis:

1. **Fonte da verdade neutra** — uma pasta `skills/` no repositório alvo, versionável e
   revisável por pares. É **independente de agente** e o único lugar onde o conteúdo vive.
2. **Adaptadores por agente** — como cada ferramenta descobre skills em local próprio
   (o Claude Code, por exemplo, lê de `.claude/skills/`), geramos ali apenas **ponteiros
   (symlinks)** para a pasta neutra. Trocar de agente, ou usar vários, é só gerar outro
   adaptador — sem duplicar conteúdo.

### Forma recomendada: o script

O [`scripts/setup-profile.sh`](../scripts/setup-profile.sh) faz os dois níveis de uma vez:

```bash
# copia o perfil + biblioteca neutra (skills/) + adaptador .claude/skills/ (symlinks)
scripts/setup-profile.sh empresa ~/dev/meu-projeto

# só a pasta neutra, sem adaptar a nenhum agente:
scripts/setup-profile.sh empresa ~/dev/meu-projeto --skills-mode none

# cópias em vez de symlinks (recomendado p/ checkouts no Windows):
scripts/setup-profile.sh empresa ~/dev/meu-projeto --skills-mode copy

# selecionar skills específicas:
scripts/setup-profile.sh pessoal ~/dev/oss --skills secrets-guard,pr-review-guard

# incluir uma skill de domínio (exige o caminho da categoria):
scripts/setup-profile.sh pessoal ~/dev/meu-app --skills secrets-guard,dominio/mockup-lab
```

### Forma manual

```bash
# 1) fonte neutra (independente de agente)
cp -r skills/ /caminho/do/alvo/skills/

# 2) adaptador do agente como ponteiro à fonte neutra (Claude Code)
mkdir -p /caminho/do/alvo/.claude/skills
ln -s ../../skills/secrets-guard /caminho/do/alvo/.claude/skills/secrets-guard
# repita para as demais

# (alternativa: instalação pessoal, todas as sessões) ~/.claude/skills/
```

> Os perfis em `profiles/*/` referenciam estas skills na seção "Skills disponíveis" do seu
> `AGENTS.md` apontando para a pasta neutra; o adaptador `.claude/skills/` é só a ponte de
> descoberta para o Claude Code.

## Governança da biblioteca

Trate este repositório como uma biblioteca interna de software:

- **Revisão por pares** de toda alteração em `SKILL.md` via PR.
- **Fixtures sintéticos** e exemplos de invocação para cada skill.
- **Testes de invocação automatizados** quando a skill tiver _scripts_ anexos.
- _Definition of Done_ específico por skill (ver seção final de cada `SKILL.md`).

## Skills vs. Servidores MCP

São camadas **complementares e ortogonais**:

- **Skill** = camada _declarativa_ — orienta **como** o agente trabalha com uma
  capacidade (convenções, arquivos a consultar, armadilhas a evitar).
- **Servidor MCP** = camada de _execução_ — expõe _tools_, _resources_ e _prompts_ a
  múltiplos agentes via protocolo padronizado, abrindo conexões a sistemas reais (bancos,
  APIs internas, simuladores).

Uma skill pode invocar servidores MCP para executar leituras, disparar simulações ou
consultar APIs, mantendo separados o **conhecimento operacional** (Skill) e a
**infraestrutura de execução** (MCP). Servidores MCP corporativos devem ser tratados como
serviços de produção (gestão de identidade, auditoria, _rate limiting_, isolamento de rede).
