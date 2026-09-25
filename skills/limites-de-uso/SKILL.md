---
name: limites-de-uso
description: >-
  Dá ao agente visibilidade da cota consumida da conta (janelas de 5h e 7d, que
  não chegam ao contexto do modelo) e define como planejar em torno dela: quando
  consultar, como dimensionar trabalho que caiba na janela, como reservar
  orçamento para o handoff e como retomar o trabalho depois do reset. Aciona ao
  planejar tarefa longa ou paralela, ao receber aviso de cota alta, quando o
  usuário perguntar quanto resta do limite, ou quando um trabalho precisar
  atravessar a virada da janela.
---

# limites-de-uso — trabalhar dentro da cota sem perder o fio

Planos por assinatura medem consumo em **janelas móveis** (tipicamente uma de ~5h e
uma de 7 dias). O percentual dessas janelas costuma existir só na barra de status da
CLI e **não chega ao contexto do modelo** — por isso o agente responde "não tenho
essa visibilidade" e, sem querer, planeja como se a cota fosse infinita.

## O princípio

> Bater no limite não é o dano. O dano é bater no limite **no meio de uma edição**,
> com a árvore inconsistente e nada escrito sobre onde o trabalho parou.

A cota volta sozinha no reset; o contexto da sessão, não. Por isso a estratégia
inteira desta skill se resume a **terminar em pontos onde parar é barato** e a
**gastar antes do fim o pouco que o handoff custa**.

## Como consultar

```bash
python3 skills/limites-de-uso/scripts/usage-limits.py            # texto
python3 skills/limites-de-uso/scripts/usage-limits.py --oneline  # uma linha
python3 skills/limites-de-uso/scripts/usage-limits.py --json     # programático
```

**Consulte quando decidir**, não por hábito — o número não muda a resposta de
"corrija este typo". Vale a chamada quando:

- o trabalho pedido é longo, ou vai abrir subagentes em paralelo;
- chegou um aviso `[limites]` do hook (ver adiante);
- o usuário perguntou sobre cota, custo ou "dá tempo de fazer hoje?";
- uma sessão vai atravessar a virada da janela.

## Como ler o que ele devolve

| Campo | O que fazer com ele |
|---|---|
| `five_hour` | Tático. Restringe **esta** sessão; volta hoje. |
| `seven_day` | Estratégico. Se estiver alto, o reset de 5h **não salva** — repense o ritmo da semana. |
| `status: stale` | Nenhuma sessão atualizou a barra há tempo. Diga que o número pode estar defasado; não afirme como atual. |
| `status: expired` | A janela já virou; o percentual guardado é de outra janela. **Não use o número.** |
| `burn.confidence` | `baixa` = poucas amostras ou intervalo curto. Cite como indício, nunca como previsão. |
| `exhausts_before_reset` | O sinal que decide: no ritmo atual a cota acaba **antes** de reabastecer. |

Um percentual acima de 100% é possível (consumo em excedente) e não é erro de leitura.

## Postura por faixa

Use a **maior** das duas janelas.

| Faixa | Postura |
|---|---|
| < 60% | Normal. Não mencione cota ao usuário sem que ele pergunte. |
| 60–80% | Prefira incrementos commitáveis. Evite abrir vários subagentes de uma vez. |
| 80–90% | Avise o usuário **uma vez**, com número e horário do reset. Negocie escopo antes de começar, não no meio. |
| > 90% | Não inicie tarefa nova que não caiba. Feche o que está aberto, **escreva o handoff** e proponha retomar após o reset. |

## A regra que evita o dano: reserve o handoff

O handoff é a única coisa que precisa existir **antes** do limite, porque depois dele
você não consegue mais pedir nada ao agente. Trate-o como despesa fixa, não como
última tarefa.

1. **Ao cruzar 80%**, pare de abrir frentes novas e feche as abertas.
2. **Escreva o handoff enquanto ainda há folga** — ver skill `handoff-updater`. Um
   handoff escrito com 85% de cota é útil; um planejado para 99% não existe.
3. **Deixe a árvore em estado válido**: compila, testes passam ou o que falha está
   anotado. Ninguém retoma um refactor pela metade sem mapa.
4. **Registre o horário do reset** no handoff — quem retoma precisa saber quando dá.

## Continuidade entre janelas

Ao retomar depois do reset, a cota voltou mas **o contexto não**. A retomada é uma
sessão nova lendo um documento, não uma continuação:

- Comece lendo o handoff, não relendo o código todo — releitura ampla é justamente
  o que consumiu a janela anterior.
- Confirme o commit/hash citado no handoff antes de confiar nele; a árvore pode ter
  andado.
- Se o trabalho não coube na janela, ele estava grande demais para um ciclo de
  contexto: quebre com `micro-ticket-planner` antes de recomeçar, senão a próxima
  janela estoura no mesmo ponto.

## O que consome desproporcionalmente

Quando a cota aperta, estes são os primeiros a cortar — em ordem de retorno:

| Prática | Por que pesa |
|---|---|
| Subagentes em paralelo | Cada um paga o próprio contexto inicial; N agentes ≈ N vezes o custo de partida. |
| Reler arquivos grandes já lidos | O conteúdo já está no contexto; a releitura paga tudo de novo. |
| Busca ampla sem filtro | Varredura de repositório grande entra inteira no contexto. |
| Deixar o contexto encher até compactar | A compactação custa uma passada sobre tudo que foi dito. |
| Esforço/modelo máximo em tarefa mecânica | Renomear símbolo não precisa do modelo mais caro. |

## Armadilhas

- **Não trate número defasado como atual.** Sem sessão ativa o dado envelhece em
  silêncio; cheque `age_seconds`.
- **Não faça alarde.** Abaixo de 80% o usuário não pediu para saber. Um aviso por
  sessão basta; repetir a cada resposta é ruído.
- **Não desista de trabalho por causa da cota sem perguntar.** Reduzir escopo é
  decisão do usuário — apresente o número e a opção, não o fato consumado.
- **Não invente saldo.** Se não há provedor instalado, diga que não há dado; não
  estime cota por custo em dólar, que é outra métrica.

## Instalação do provedor de dados

O script é **agnóstico ao agente**: a leitura fica atrás da lista `PROVIDERS`, e cada
provedor devolve o mesmo dicionário normalizado ou `None`.

- **Claude Code** — o dado vem do payload da statusline, persistido em SQLite por um
  logger do usuário (`~/.claude/usage/usage-logger.py` disparado pelo
  `statusline-command.sh`). O script só lê, em modo somente-leitura.
  Caminho alternativo do banco: env var `AGENT_USAGE_DB`.
- **Outro agente** — acrescente uma função `provider_<nome>(now)` e registre-a em
  `PROVIDERS`. Nada mais no script, na skill ou no hook precisa mudar.
- **Sem provedor** — o script responde `no_data` de forma limpa. Neste caso a skill
  ainda vale pelas estratégias; só não há número.

## Aviso automático (hook opcional)

O agente não consulta a cota espontaneamente — e quando o usuário pensa em perguntar,
já é tarde. Um hook de *prompt* fecha essa lacuna sem custo em sessão tranquila:

```bash
python3 <caminho>/scripts/usage-limits.py --gate --warn-at 80
```

O modo `--gate` **não imprime nada** abaixo do limiar (logo, nada entra no contexto) e
só emite uma linha quando a cota passa do valor. Limiar também por `AGENT_USAGE_WARN_AT`.
Falha sempre em silêncio: um erro aqui não pode quebrar o prompt do usuário.

No Claude Code, em `~/.claude/settings.json`:

```json
{ "hooks": { "UserPromptSubmit": [ { "hooks": [ {
  "type": "command",
  "command": "python3 ~/.claude/usage/usage-limits.py --gate --warn-at 80"
} ] } ] } }
```

<!-- USER:BEGIN id=limiares-locais -->
### Ajuste local dos limiares

*(registre aqui os valores que funcionaram na sua conta e plano)*
<!-- USER:END -->

## Definição de pronto da skill

- [ ] O número citado ao usuário veio do script nesta sessão, não de memória.
- [ ] Idade do dado conferida; se `stale` ou `expired`, isso foi dito explicitamente.
- [ ] Projeção de ritmo com `confidence: baixa` foi apresentada como indício, não previsão.
- [ ] Acima de 80%: handoff escrito e árvore em estado válido **antes** de a cota acabar.
- [ ] Redução de escopo por cota foi proposta ao usuário, não decidida sozinha.
