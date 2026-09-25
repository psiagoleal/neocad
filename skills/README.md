<!-- Caminho relativo: skills/README.md -->

# Biblioteca de Skills de Governança para Agentes de IA

Skills (Habilidades de Agente) são pacotes modulares, autocontidos e versionáveis,
descritos por um arquivo `SKILL.md` com *frontmatter* YAML (`name`, `description`). Sua
propriedade central é a **divulgação progressiva** (*progressive disclosure*): o agente
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

| Skill | Para quê | Aciona quando |
|-------|----------|---------------|
| [`secrets-guard`](secrets-guard/SKILL.md) | Não-exposição de segredos antes de comandos/saída | Há credenciais, `.env`, cofres, `aws/gcloud/kubectl config` |
| [`adr-writer`](adr-writer/SKILL.md) | Criar/consultar ADRs como restrição cognitiva | Decisão de stack/biblioteca/solver; "registrar decisão" |
| [`micro-ticket-planner`](micro-ticket-planner/SKILL.md) | Quebrar trabalho em tickets de um ciclo de contexto | Planejar sprint; tarefa ampla/ambígua |
| [`handoff-updater`](handoff-updater/SKILL.md) | Manter `docs/CURRENT-STATE.md` | Após commit/ticket; "onde paramos" |
| [`pr-review-guard`](pr-review-guard/SKILL.md) | Checklist do "problema dos 80%" + OWASP | Antes de abrir/aprovar PR ou merge |
| [`meeting-minutes`](meeting-minutes/SKILL.md) | ATA e registro de reunião a partir de áudio ou notas, respeitando o perímetro do dado (ASR local quando confidencial) | Produzir ATA; transcrever ou resumir reunião |
| [`delegacao-a-subagentes`](delegacao-a-subagentes/SKILL.md) | O que fica no modelo local vs. o que vai para a nuvem, e como pedir | Tarefa toca dado sensível; roteamento por task-class; "delegar"/"modelo local" |
| [`delegacao-openai-compat`](delegacao-openai-compat/SKILL.md) | Delegar volume a qualquer endpoint OpenAI-compatible (LiteLLM, vLLM, Ollama `/v1`): `oa-chat` para chamada direta sem chave em argv, `agentry` para modo agente — custo fora da cota, saída fora do contexto, com checagem de egresso | Varredura/revisão volumosa; "economizar cota"; "usar o LiteLLM"/"modelo OpenAI-compatible"; configurar `providers.litellm` |
| [`novo-projeto`](novo-projeto/SKILL.md) | Adotar o framework num repo: escolher perfil, preservar conteúdo prévio, deixar pronto p/ `--update` | Projeto novo; "configurar o agente" / "aplicar o perfil" / "instalar as regras"; "por onde começo" (roteiro de entrada, projeto novo ou existente) |
| [`limites-de-uso`](limites-de-uso/SKILL.md) | Ver a cota consumida da conta (5h/7d) e planejar em torno dela: reservar orçamento p/ o handoff, atravessar a virada da janela | Tarefa longa ou paralela; aviso `[limites]`; "quanto resta do limite?" |
| [`caveman`](caveman/SKILL.md) | Modo de comunicação comprimida: corta forma (cortesia, preâmbulo, recapitulação) e preserva substância (caminho, número, erro, incerteza, ressalva) | "modo caveman"/"seja breve"/"economiza token"; sessão longa e repetitiva; cota apertada |
| [`critico-independente`](critico-independente/SKILL.md) | Fecha o laço de verificação: critério declarado antes, crítica por segundo modelo em contexto limpo, sinal externo do mundo real | Revisar saída de agente; entrega "parece certa" sem prova; antes de aceitar trabalho longo |
| [`spec-como-contrato`](spec-como-contrato/SKILL.md) | Spec como contrato versionado: o que entra e o que não, critério de aceite em EARS, uma spec por feature, spec separada do plano | Feature ou projeto novo; pedido vago; antes de delegar; entregue funciona mas não era o pedido |
| [`teste-primeiro`](teste-primeiro/SKILL.md) | TDD com agente: nenhuma implementação antes de teste falhando pelo motivo certo, com a saída citada; isolamento de contexto entre quem testa e quem implementa | Comportamento novo; correção de bug; "escrevo os testes depois"; cobertura alta e confiança baixa |
| [`licoes-aprendidas`](licoes-aprendidas/SKILL.md) | Colher lição e transformá-la em mudança de artefato pela escada de destinos (comando > teste > hook > template > regra), com fila em vez de cemitério | Fim de trabalho não trivial; incidente ou rollback; mesmo erro pela segunda vez; "da próxima vez a gente lembra" |
| [`atribuicao-de-falha`](atribuicao-de-falha/SKILL.md) | Diagnosticar onde a falha do agente nasceu — contexto, processo, autoridade ou evidência — e corrigir a camada certa | Agente repete erro; "escrever no AGENTS.md p/ ele não fazer de novo"; escolher entre regra, permissão, hook e teste |
| [`laco-de-correcao`](laco-de-correcao/SKILL.md) | Laço construir→provar→verificar→corrigir com tentativas contadas, veredito binário com prova por defeito (sem nota) e escalada de cinco linhas | Tarefa com prova executável; correção sem humano a cada volta; agente corrigindo a mesma coisa duas vezes |
| [`gates-de-conclusao`](gates-de-conclusao/SKILL.md) | Gates escritos antes do trabalho (`CHECK:`/`EXPECT:`, só vale com exit 0) e as quatro razões de aborto além da saída de aceite | Tarefa longa/repetitiva; decidir se está concluído; agente insiste sem progredir; checkbox sem evidência |
| [`paralelizacao-em-grafo`](paralelizacao-em-grafo/SKILL.md) | Decidir se o trabalho vira grafo paralelo: contrato na fase 0, fatia vertical, worktree por fatia, fan-in de dono único | Onda com várias tarefas; "rodar em paralelo"; dividir trabalho entre agentes; merge paralelo virou negociação |
| [`perguntar-antes-de-construir`](perguntar-antes-de-construir/SKILL.md) | Comportamento questionador: lacuna listada antes do artefato, detecção do que o repo já responde, pergunta em bloco com opções, suposição declarada | Todo pedido que produz artefato, sobretudo o que chega em uma frase |
| [`definir-stack`](definir-stack/SKILL.md) | Questionário de stack por tipo de projeto; respostas vão para as ilhas do `AGENTS.md`, ADR e `DESIGN.md`, com todo comando executado antes de declarado | Projeto novo; stack não escrita; antes de escolher framework; agente adivinhando comando de build |
| [`mapa-de-arquitetura`](mapa-de-arquitetura/SKILL.md) | Gera e mantém `docs/architecture.md` com a arquitetura efetiva — componentes, responsabilidades, dependências com evidência e Mermaid — sem encaixe forçado em padrão | Adoção em projeto existente; `docs/architecture.md` ausente ou velho; mudança altera dependências entre módulos |

## Bibliotecas extras

O instalador aceita bibliotecas **adicionais** com a mesma estrutura desta (`skills/`,
`agents/`) — por exemplo, um repositório privado com material que não pode ser publicado ou
que é de uso pessoal. Aponte-as com `--fonte <dir>` ou `fontes_extras` no `config.toml`
(ADR 0013). Elas podem trazer skills de governança (instaladas por padrão), categorias
opt-in (`--skills 'categoria/*'`) e subagents (`--agents`). Nomes precisam ser únicos entre
todas as fontes: colisão é erro, porque os adaptadores são planos.

## Catálogo — domínio (sob demanda)

| Skill | Para quê | Aciona quando |
|-------|----------|---------------|
| [`dominio/mockup-lab`](dominio/mockup-lab/SKILL.md) | Prototipar UI em HTML/CSS, comparar variantes por render headless, exportar a aprovada como SVG p/ Penpot | Prototipar/mockar tela; comparar variantes de layout; levar design ao Penpot |
| [`dominio/transcrever-video`](dominio/transcrever-video/SKILL.md) | Transcrição pelo caminho barato: a legenda que a plataforma já tem, sem ffmpeg nem modelo de fala; ASR local só quando não há legenda | "transcreva este vídeo"; vídeo como fonte de pesquisa; página do YouTube bloqueada |
| [`dominio/exercicio-resolvido`](dominio/exercicio-resolvido/SKILL.md) | Exercício de engenharia em três artefatos: notebook sympy com fórmula → substituição → resultado, guia por questão com a derivação e o que ela ensina, e a fronteira do que fica com o aluno | Lista, prelab, prova antiga; "notebook com as contas"; "guia passo a passo" |
| [`dominio/docling-local`](dominio/docling-local/SKILL.md) | PDF em Markdown pelo docling local, preservando equações em LaTeX, com procedência declarada | Extrair/converter PDF; citar norma ou artigo; alimentar base de referências |

Estas trazem *scripts* anexos e dependências próprias (Playwright, `curl_cffi`, `pikepdf`, um serviço docling local e um venv Piper para TTS), instaladas
sob demanda — por isso `node_modules/`, `.venv/` e `__pycache__/` nunca são copiados para o
alvo. Dado do usuário (listas de compra, mockups) mora no diretório de trabalho do projeto,
**nunca dentro da skill**.

## Catálogo — stack (sob demanda)

Tudo que depende de uma escolha de tecnologia. Opt-in: `--skills 'stack/*'` ou skill a skill.
Cada uma começa por um **gate de detecção** — se o projeto não for daquela stack, ela manda
parar em vez de escrever código que não compila.

| Skill | Para quê | Aciona quando |
|-------|----------|---------------|
| [`stack/contrato-de-design`](stack/contrato-de-design/SKILL.md) | `docs/DESIGN.md` como contrato visual, sobre tokens em três camadas (primitiva → semântica → componente), independente de framework | Começar a interface; cores/espaços divergindo entre telas; "design system", "tokens", "tema escuro" |
| [`stack/craft-de-interface`](stack/craft-de-interface/SKILL.md) | Craft visual em quatro operações (descobrir, estruturar, refinar, auditar), com playbook lido só quando a operação é pedida e piso de qualidade só antes de editar | "deixe mais bonito", "revise a UI", "está confuso"; antes de entregar interface |
| [`stack/criar-app-tauri`](stack/criar-app-tauri/SKILL.md) | Lado nativo de app desktop Tauri 2: comando fino com validação na borda, estado, trabalho fora da thread principal e permissão por capability com menor privilégio | Criar ou alterar comando, plugin, permissão ou janela em projeto com `src-tauri` |
| [`stack/criar-servico-fastapi`](stack/criar-servico-fastapi/SKILL.md) | Serviço HTTP em FastAPI + Pydantic v2: rota fina, domínio sem framework, erro traduzido na borda, consulta parametrizada e lista paginada | Criar ou alterar endpoint, roteador ou modelo de entrada/saída em projeto FastAPI |
| [`stack/criar-ui-sveltekit`](stack/criar-ui-sveltekit/SKILL.md) | Interface em SvelteKit 2 / Svelte 5 (runes), TS estrito, dado no `load`, formulário com action, estilo por token | Criar página, rota, componente ou formulário em projeto SvelteKit |

## Catálogo — entrega (sob demanda)

Gate que roda em servidor e o caminho até produção. Opt-in: `--skills 'entrega/*'`.

| Skill | Para quê | Aciona quando |
|-------|----------|---------------|
| [`entrega/pipeline-como-gate`](entrega/pipeline-como-gate/SKILL.md) | CI que pode reprovar: os mesmos comandos do `AGENTS.md`, ordem barato→caro, matriz só onde a plataforma muda, cache que invalida sozinho, segredo fora do alcance de fork | Criar ou alterar workflow; adotar CI; CI e máquina local discordam; passo que vive vermelho |
| [`entrega/deploy-reproduzivel`](entrega/deploy-reproduzivel/SKILL.md) | Rollback escrito antes do deploy e sucesso confirmado por sinal externo: imagem com versão fixada e sem segredo, config por ambiente, migração como passo próprio, registro do que subiu | Criar Dockerfile ou compose; preparar ou executar deploy; migração que vai para produção; "como se volta atrás?" |

## Catálogo — conhecimento (sob demanda)

Acervo grande de documentos que precisa ser consultável. Opt-in: `--skills 'conhecimento/*'`.

| Skill | Para quê | Aciona quando |
|-------|----------|---------------|
| [`conhecimento/indexar-acervo`](conhecimento/indexar-acervo/SKILL.md) | Inventário com proveniência, extração com método registrado e índice FTS5 local, retomável; o acervo não sai do lugar | Acervo grande de documentos; "indexar", "base de conhecimento"; agente varrendo com `find`/`grep` |
| [`conhecimento/consultar-acervo`](conhecimento/consultar-acervo/SKILL.md) | Consultar antes de responder, ler o original, citar caminho e proveniência, separar "não achei" de "não existe" | Existe base indexada; pergunta sobre histórico, norma ou projeto anterior |

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
2. **Adaptadores por agente** — cada harness descobre skills em local próprio, e
   geramos ali apenas **ponteiros (symlinks)** para a pasta neutra. Na prática há só
   dois destinos (ADR 0012):

   | Harness | Diretório |
   |---|---|
   | `claude` | `.claude/skills/` |
   | `codex`, `gemini`, `opencode`, `agentry`, `zcode` | `.agents/skills/` |

   Usar vários é uma lista: `--agent claude,codex` ou `--agent all`. Sem duplicar
   conteúdo — a fonte continua sendo uma só.

### Forma recomendada: o script

O [`scripts/setup-profile.sh`](../scripts/setup-profile.sh) faz os dois níveis de uma vez:

```bash
# copia o perfil + biblioteca neutra (skills/) + adaptador .claude/skills/ (symlinks)
scripts/setup-profile.sh empresa ~/dev/meu-projeto

# só a pasta neutra, sem adaptar a nenhum agente:
scripts/setup-profile.sh empresa ~/dev/meu-projeto --skills-mode none

# cópias em vez de symlinks (recomendado p/ checkouts no Windows):
scripts/setup-profile.sh empresa ~/dev/meu-projeto --skills-mode copy

# adaptadores para vários harnesses (ou todos):
scripts/setup-profile.sh empresa ~/dev/meu-projeto --agent claude,codex
scripts/setup-profile.sh empresa ~/dev/meu-projeto --agent all

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

# Não instale a biblioteca em ~/.claude/skills: o acervo é repo-local por
# decisão de projeto (ver docs/adr/0011) — cada repositório carrega o seu.
```

> Os perfis em `profiles/*/` referenciam estas skills na seção "Skills disponíveis" do seu
> `AGENTS.md` apontando para a pasta neutra; o adaptador `.claude/skills/` é só a ponte de
> descoberta para o Claude Code.

## Governança da biblioteca

Trate este repositório como uma biblioteca interna de software:

- **Revisão por pares** de toda alteração em `SKILL.md` via PR.
- **Fixtures sintéticos** e exemplos de invocação para cada skill.
- **Testes de invocação automatizados** quando a skill tiver *scripts* anexos.
- *Definition of Done* específico por skill (ver seção final de cada `SKILL.md`).

## Skills vs. Servidores MCP

São camadas **complementares e ortogonais**:

- **Skill** = camada *declarativa* — orienta **como** o agente trabalha com uma
  capacidade (convenções, arquivos a consultar, armadilhas a evitar).
- **Servidor MCP** = camada de *execução* — expõe *tools*, *resources* e *prompts* a
  múltiplos agentes via protocolo padronizado, abrindo conexões a sistemas reais (bancos,
  APIs internas, simuladores).

Uma skill pode invocar servidores MCP para executar leituras, disparar simulações ou
consultar APIs, mantendo separados o **conhecimento operacional** (Skill) e a
**infraestrutura de execução** (MCP). Servidores MCP corporativos devem ser tratados como
serviços de produção (gestão de identidade, auditoria, *rate limiting*, isolamento de rede).
