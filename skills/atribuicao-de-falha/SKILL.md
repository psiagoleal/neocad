---
name: atribuicao-de-falha
description: >-
  Diagnostica uma falha de agente antes de corrigi-la: separa se o agente não
  sabia (contexto), não seguiu o fluxo (processo), não deveria poder
  (autoridade) ou passou sem ser visto (evidência) — e escolhe o controle da
  camada certa em vez de acrescentar mais uma advertência no prompt. Aciona
  quando o agente repetir um erro, quando alguém propuser "escrever no AGENTS.md
  para ele não fazer de novo", ao revisar incidente de sessão, ou ao decidir
  entre regra, skill, permissão, hook e teste.
---

# atribuicao-de-falha — corrigir a camada certa

Quando um agente erra, o reflexo é escrever mais uma linha no `AGENTS.md`. Às vezes é
a correção certa. Na maioria das vezes é a mais barata de escrever e a menos provável
de funcionar — porque instrução em linguagem natural compete por atenção com todo o
resto da janela e é, por definição, **probabilística**.

## O princípio

> Não acrescente mais um prompt até saber onde o comportamento nasceu.

O corolário que decide a forma da correção:

> Instrução **orienta**. Permissão, schema, teste e linter **impedem ou detectam**.
> Escolha conforme o custo do erro, não conforme o custo de escrever a correção.

## 1. Observar — sem antecipar a solução

Registre três coisas, e só elas:

- **A entrada** — o pedido exato que iniciou o caminho.
- **A ação** — o que o agente de fato executou (comando, edição, chamada).
- **O efeito** — o que mostrou que havia problema.

"O agente errou" não é reproduzível e não permite saber se a correção funcionou.
`git push origin main sem pedir aprovação` é.

⚠️ Um sintoma que você não consegue repetir não autoriza uma regra nova. Regra criada
sobre impressão vira ruído permanente na janela de todas as sessões seguintes.

## 2. Localizar — as quatro origens

A pergunta é **onde a falha nasceu**, e há quatro respostas possíveis. Escolher a
errada produz mais texto sem remover a causa.

| Sintoma | Origem | Por quê | Controle neste framework |
|---|---|---|---|
| **Não sabia** | Contexto | A informação certa não chegou à decisão | `AGENTS.md` (base curta, sempre lida) para o que vale quase sempre; **skill** para o que é grande e condicional |
| **Não seguiu o fluxo** | Processo | A informação existia, mas o procedimento dependia de improviso | **skill** com passos e critério; subagente com papel próprio (`delegacao-a-subagentes`) |
| **Não deveria poder** | Autoridade | A ação sensível estava disponível demais | `permissions` em `.claude/settings.json`; `readAllow` do `agentry`; `.claudeignore` |
| **Passou sem ser visto** | Evidência | A saída foi ruim e nada acusou | teste, linter e checagem de tipo dos comandos exatos do `AGENTS.md` §3; **hook** `PostToolUse`; `pr-review-guard` |

O teste que separa Contexto de Autoridade: **se o agente soubesse da regra, ele ainda
conseguiria violá-la?** Se sim, a origem é autoridade, e nenhuma quantidade de texto
resolve.

O teste que separa Processo de Evidência: **o defeito estaria lá se o agente tivesse
seguido o procedimento?** Se estaria, falta sensor, não roteiro.

## 3. Controlar — o mais simples que basta para o risco

| Se o erro | Use |
|---|---|
| é barato e raro | orientação: `AGENTS.md` ou skill |
| é caro mas detectável depois | sensor: teste, linter, hook `PostToolUse` |
| não pode acontecer | fronteira: permissão `deny`, remoção da tool, `readAllow` |
| exige julgamento humano | aprovação: permissão `ask`, gate de PR |

Prefira sempre a checagem determinística mais barata que decida a questão. Julgamento
por modelo custa uma janela inteira e não é reprodutível — reserve-o para nuance que
regra nenhuma expressa.

**A ordem importa e é econômica:** cada checagem barata que reprova poupa uma rodada
inteira da checagem cara, que só veria o mesmo defeito mais tarde e por mais dinheiro.

## 4. Provar — replay, não promessa

Repita **exatamente** a entrada que produziu o problema. A prova de sucesso não é o
agente prometer que não fará de novo; é o controle impedir ou acusar.

```
$ git push origin main
→ DENY · tool não executada
```

Se ainda executar, o defeito está na configuração efetiva, não na decisão — investigue
de qual camada veio a regra em vigor antes de escrever outra.

## 5. Versionar — a falha paga uma melhoria durável

Uma correção só acumula valor se sobreviver à conversa. Versione o controle junto do
projeto (`AGENTS.md`, `.claude/settings.json`, skill, teste) e registre, na revisão, o
incidente que o motivou e a prova usada. Decisão de peso vira ADR (`adr-writer`).

## Quando escalar para o humano

- O efeito é difícil de desfazer ou atinge sistema externo.
- Não há prova suficiente para decidir.
- Avaliadores discordam.
- Novas tentativas não mudam o estado observado (ver `gates-de-conclusao`, impasse).

## Armadilhas

- **Corrigir com texto o que é autoridade.** O caso mais comum e o mais caro: o
  `AGENTS.md` engorda a cada incidente e nada para de acontecer.
- **Culpar o modelo.** "Modelo fraco" é diagnóstico só depois de excluir contexto
  ausente, tool mal descrita e sensor inexistente.
- **Regra sem prova.** Se nada reproduz a falha antiga, não há como saber se a
  correção funciona — nem quando ela pode ser removida.
- **Sensor instável.** Um gate que reprova sem motivo real ensina a equipe a
  reexecutar até passar; nesse ponto ele deixou de filtrar.

## Definição de pronto da skill

- [ ] Entrada, ação e efeito registrados de forma **reproduzível**.
- [ ] A origem foi nomeada entre as quatro — não presumida como "contexto".
- [ ] O controle escolhido é o mais simples compatível com o custo do erro.
- [ ] A falha antiga foi **repetida** e o controle provou impedir ou acusar.
- [ ] O controle está versionado, e o incidente que o motivou, registrado.
- [ ] Nada foi acrescentado ao `AGENTS.md` sem que a origem fosse mesmo contexto.
