---
name: micro-ticket-planner
description: >-
  Quebra histórias de usuário e demandas amplas em micro-tickets autocontidos
  que cabem em um único ciclo limpo de contexto do agente, evitando "ansiedade
  de contexto" e estouro de tokens. Aciona ao planejar sprint, refinar backlog,
  receber uma tarefa grande/ambígua, ou quando o usuário pedir para "quebrar",
  "planejar" ou "dividir" um trabalho.
---

# micro-ticket-planner — Planejamento por micro-tickets

Agentes confrontados com escopo amplo demais ou dependências ambíguas tendem a
interromper a execução precocemente ou ignorar restrições de segurança à medida que se
aproximam dos limites da janela de contexto ("ansiedade de contexto"). A mitigação é a
**granularidade fina**: cada tarefa cabe em um ciclo limpo de interação.

## Como quebrar (heurística)

1. **Uma saída verificável por ticket.** Se o ticket produz mais de um artefato testável
   independentemente, divida-o.
2. **Escopo de arquivos fechado.** Liste, no ticket, os arquivos que serão tocados. Se a
   lista é incerta ou cresce sem limite, o escopo ainda está amplo demais.
3. **Sem dependências ambíguas.** Se concluir o ticket exige uma decisão arquitetural
   ainda não tomada, primeiro registre um ADR (ver skill `adr-writer`).
4. **Cabe em um turno.** Estime se o agente consegue ler o contexto necessário, alterar e
   validar dentro de uma sessão sem podar histórico. Se não, divida.

## Formato de um micro-ticket

```markdown
### MT-<n>: <título imperativo curto>
- **Objetivo:** <resultado único e verificável>
- **Arquivos no escopo:** path/a.py, path/b.py
- **Critério de aceite:** <comando de teste/linter que deve passar>
- **Fora de escopo:** <o que explicitamente NÃO fazer aqui>
- **Depende de:** MT-<m> | ADR-<NNNN> | nenhum
```

## Painel de tickets — `docs/TICKETS.md`

Todo ticket gerado entra num checklist único, para quem acompanha ver **de relance** o que foi
feito e o que falta. É um índice, não um segundo lugar para o detalhe:

```markdown
## Em aberto
- [ ] [MT-12](plano.md#mt-12) — Rejeitar reserva sobreposta com 409
- [>] [MT-14](plano.md#mt-14) — Migrar sessões para Redis · falta o caminho de erro
- [!] [MT-15](plano.md#mt-15) — Corrigir vazamento de token no log
- [ ] [MT-16](plano.md#mt-16) — Exportar relatório mensal — bloqueado por credencial do SAP

## Concluídos
- [x] [MT-11](plano.md#mt-11) — Validar tamanho do corpo da requisição

## Abandonados
- [~] [MT-9](plano.md#mt-9) — Cache de sessão · motivo: substituído pelo MT-11
```

### Os marcadores são um conjunto fechado

**Não invente marcador fora desta lista.** O painel serve à varredura visual; símbolo que cada
sessão inventa destrói exatamente isso.

| Marcador | Seção | Significa |
|---|---|---|
| `[ ]` | Em aberto | em aberto, nada a destacar — **o caso comum** |
| `[>]` | Em aberto | em andamento: começou e não terminou |
| `[!]` | Em aberto | **crítico ou urgente** — exige atenção agora |
| `[x]` | Concluídos | entregue |
| `[~]` | Abandonados | abandonado, **com motivo** |

**O estado é a seção; o marcador é a situação dentro dela.** `[x]` e `[~]` são redundantes com
a seção de propósito: ajudam a ler a linha fora de contexto.

Quatro regras que fecham as ambiguidades:

1. **Prioridade vence o marcador.** Só há um caractere e duas dimensões (progresso e
   prioridade). Quando as duas valem, o marcador é `[!]` e o progresso vai para o texto
   (`· em andamento`). O marcador responde "isto exige atenção agora?"; o texto responde "onde
   está".
2. **`[!]` é raro por definição.** Se mais de **três** tickets estão `[!]`, nada está — e a
   resposta é repriorizar, não acrescentar. Quem marca é quem prioriza, por decisão explícita;
   o agente não promove ticket a urgente sozinho.
3. **`[>]` diz o que falta.** "Em andamento" sem o que falta é `[ ]` com enfeite. Se você não
   consegue escrever o que resta em meia linha, o ticket é grande demais — fatie.
4. **Bloqueio é texto, não marcador.** Bloqueio é uma **dependência**, e o que importa é *de
   quê*: `— bloqueado por <o quê>: <frase>`. Não cabe em um caractere. Se o bloqueio também é
   urgente, aí sim `[!]` — pela regra 1.

   Esta regra faz mais do que informar quem lê: **desmascara bloqueio que é só trabalho
   parado**. Medido em campo (2026-09-24, painel de 63 tickets): de **9** rotulados como
   bloqueados, **2 não tinham dependência nenhuma** — eram trabalho em aberto que ninguém
   começou, herdados da lista de "impedimentos" do handoff. Ao ser obrigado a escrever de que
   dependiam, não havia o que escrever.

**Sobre a renderização:** o GitHub transforma em caixa de seleção apenas `[ ]` e `[x]`; `[>]`,
`[!]` e `[~]` aparecem como texto e desalinham a linha. É consciente: como `[!]` e `[>]` são
raros e `[~]` vive numa seção separada, a maior parte do painel continua uma lista de tarefas
normal — e o desalinhamento do que é raro **ajuda** a varredura em vez de atrapalhar.

O verificador `scripts/checar-painel.py` confere formato, marcador fora do conjunto, limite de
`[!]`, link ausente e `[~]` sem motivo:

```bash
python3 skills/micro-ticket-planner/scripts/checar-painel.py docs/TICKETS.md
```

Regras, todas obrigatórias:

- **Uma linha por ticket:** identificador com **link para o detalhe** e uma frase curta. Nada
  de critério de aceite, arquivos ou discussão aqui — isso mora no destino do link.
- **O link aponta para onde o ticket está detalhado:** âncora no plano (`plano.md#mt-12`) ou
  arquivo próprio (`tickets/MT-12.md`). Link quebrado é painel mentindo — confira com
  `skills/pr-review-guard/scripts/checar-links.py docs/TICKETS.md`.
- **Link que resolve não é o mesmo que detalhe que serve.** Apontar para a seção de um
  inventário ou para o handoff descreve o assunto, não o ticket — falta critério de aceite e
  escopo de arquivos. Nenhum verificador pega isso; é julgamento. A regra prática: enquanto o
  ticket está `[ ]`, o link pode apontar para a **origem** (spec, inventário, discussão); ao
  passar para `[>]`, ele precisa de **detalhe próprio**, com objetivo, escopo e critério de
  aceite. Assim ninguém gera trinta arquivos de uma vez para um painel que mal começou.

  **Escrever o critério de aceite é um teste de tamanho do ticket.** Se ele não é
  especificável, são dois. Campo (2026-09-24): um ticket de "grampeamento deslocado" parecia
  um só até o detalhe ser escrito — não havia como aceitar a implementação contra uma
  tolerância que ninguém tinha estabelecido. Virou *estabelecer o método* (entrega texto) e
  *implementar* (bloqueado pelo primeiro).

  O discriminador é o **objetivo**, não o tamanho aparente: metades com objetivos diferentes
  (descobrir × construir) são dois tickets; um objetivo com várias entregas — quatro formatos
  de entrada, por exemplo — é um ticket só, com as quatro enumeradas no detalhe. Note que este
  teste pega o que a regra do `[>]` não pega: lá o "o que falta" cabia em meia linha; o que
  não existia era o **aceite**.
- **Atualize sempre, no mesmo trabalho:** ao **criar** (entra em aberto), ao **começar**
  (marca `[>]` com o que falta), ao **concluir** (marca `[x]` e move), ao **abandonar** (marca
  `[~]` com o motivo em uma frase). Nunca apague um ticket: o abandonado também é história.
- **Sem duplicar o handoff.** `docs/CURRENT-STATE.md` diz o que está acontecendo agora; o
  painel diz o que existe. O handoff aponta para o painel, não o copia.

O modelo está em `templates/TICKETS.template.md`; o instalador já entrega `docs/TICKETS.md` em
todo projeto novo, e nunca o sobrescreve.

## Conexão com os rituais ágeis (DoD)

Um micro-ticket só é "Concluído" quando:
- os scripts de teste/linter definidos no `AGENTS.md` passam;
- o `docs/CURRENT-STATE.md` foi atualizado (ver skill `handoff-updater`);
- o `docs/TICKETS.md` marca o ticket como concluído;
- a revisão humana de PR foi feita (ver skill `pr-review-guard`).

## Definição de pronto da skill

- [ ] Cada micro-ticket tem objetivo único, escopo de arquivos fechado e critério de aceite.
- [ ] Dependências e itens fora de escopo estão explícitos.
- [ ] Nenhum ticket exige decisão arquitetural não registrada em ADR.
- [ ] Todo ticket criado, concluído ou abandonado está refletido em `docs/TICKETS.md`, com
      link válido para o detalhe e uma frase curta.
- [ ] Todo marcador está no conjunto fechado (`[ ]`, `[>]`, `[!]`, `[x]`, `[~]`); nenhum foi
      inventado.
- [ ] No máximo três `[!]`, e cada um por decisão de quem prioriza.
- [ ] Todo `[>]` diz o que falta; todo `[~]` diz o motivo; todo bloqueio diz de que depende.
- [ ] Todo ticket em `[>]` tem detalhe próprio com objetivo, escopo e critério de aceite.
- [ ] `checar-painel.py` passou, com a saída lida.
