---
name: laco-de-correcao
description: >-
  Laço automático de construir → provar → verificar → corrigir, com tentativas
  contadas e registradas, veredito binário com lista de defeitos e comando de
  prova (sem nota), hipótese nova a cada volta e escalada formatada quando o
  laço não converge. Aciona ao implementar tarefa com prova executável, quando a
  correção precisa acontecer sem o humano a cada volta, quando o agente corrige
  a mesma coisa duas vezes, ou antes de aceitar trabalho longo de código.
---

# laco-de-correcao — corrigir sozinho, mas contando as voltas

Sem laço, cada defeito custa uma ida e volta com você. Com laço mal desenhado, o agente
gira sem sair do lugar e declara vitória no fim. A diferença entre os dois é registro: quem
não conta as tentativas não percebe que está repetindo.

## O princípio

> **Quem constrói não julga; quem julga não propõe a correção.**
> Toda volta é registrada com a hipótese que a motivou. Hipótese repetida encerra o laço.

## As quatro fases

| Fase | Quem faz | O que precisa sair dela |
|---|---|---|
| **Construir** | construtor | o artefato, e só ele |
| **Provar** | comando | saída executada e lida — não "deveria passar" |
| **Verificar** | verificador em contexto limpo | veredito binário + lista de defeitos, cada um com comando de prova |
| **Corrigir** | construtor | a mudança **e a hipótese** que a justifica |

O verificador recebe **objetivo, critério, artefato e a saída das provas**. Nunca recebe o
raciocínio do construtor — é o que garante que ele julgue o que existe, não o que se
pretendia (`critico-independente`).

## Veredito binário, não nota

O laço devolve `passa` ou `não passa`, com os defeitos listados. **Não use pontuação.**

Nota produzida pelo próprio modelo verificador tem aparência de medida sem ser uma: não há
escala calibrada que separe 79 de 81, e o laço converge para o número em vez de para o
defeito. Pior, uma nota alta esconde a diferença entre "não há defeito" e "o verificador não
viu o defeito" — que é exatamente a distinção que importa.

Formato da resposta do verificador:

```
VEREDITO: passa | não passa
DEFEITOS:
  - <o que está errado> · prova: <comando que o demonstra>
MAIOR_LACUNA: <uma frase — o que mais pesa>
```

Se um defeito não tem comando de prova, ele é **opinião** e entra como observação, não como
motivo de reprovação. A exceção honesta: defeito de legibilidade ou de nome, que se declara
como tal.

## Tentativas contadas — e o que fazer com a contagem

Registre cada volta, em duas linhas no ledger do trabalho (`gates-de-conclusao`) ou no
ticket:

```
tentativa 2 · hipótese: o 409 não dispara porque a checagem roda depois do commit
           · mudança: mover a validação para antes da transação
           · resultado: G1 passou; G2 segue falhando
```

Três regras que a contagem serve para cumprir:

1. **Hipótese nova a cada volta.** Tentativa sem hipótese diferente da anterior é a mesma
   tentativa — encerre por impasse.
2. **Teto declarado antes de começar** (voltas, tempo ou cota). Teto não é o critério de
   parada; é o limite de gasto.
3. **Regressão encerra.** Correção que quebra um gate que passava não é progresso: volte ao
   último estado verde e escale.

⚠️ **Contar não é fixar.** "Sempre três voltas" é `while True` disfarçado de disciplina — três
pode ser cedo demais ou tarde demais. Pare quando ocorrer **qualquer** um destes, que já são
os critérios do `gates-de-conclusao`: aceite nos gates · teto esgotado · platô (o ganho de
uma volta ficou perto de zero) · regressão · hipótese repetida · divergência entre
verificadores · gate humano ou de segurança.

## Escalada — o que entregar quando o laço não converge

Escalar não é desistir; é devolver decisão para quem pode tomá-la. O relatório tem cinco
linhas e nada mais:

- **O que falhou** — o gate, com a saída real do comando.
- **Quantas voltas** e qual hipótese cada uma testou.
- **A menor reprodução** que ainda falha.
- **Duas opções** de caminho, com o custo de cada uma.
- **A decisão pedida** — em uma frase.

Sem isso, a escalada vira "não consegui", e quem recebe precisa refazer o diagnóstico inteiro.

## Quando **não** usar o laço

| Situação | O que fazer no lugar |
|---|---|
| Documento: spec, PRD, ADR, ata, plano | Uma passada de revisão com checklist e **aprovação humana**. Texto não tem comando de prova; laço adversarial sobre texto discute redação e atrasa o que importa. |
| Exploração ou diagnóstico | O escopo ainda não fechou; o laço fixaria a pergunta errada (`atribuicao-de-falha`). |
| Mudança de 1 a 3 linhas | O registro custa mais que o trabalho. |
| Não existe prova executável | Primeiro escreva a prova (`teste-primeiro`, `spec-como-contrato`); sem ela, não há o que verificar. |

## O que você vai pensar para deixar o laço solto

| O que você vai pensar | Por que não vale |
|---|---|
| "Mais uma tentativa e sai" | É a frase que precede a quinta volta idêntica. Se a hipótese é a mesma, não é tentativa nova. |
| "O verificador é chato demais" | Afrouxar o verificador não conserta o artefato; só apaga o sinal. |
| "Dou uma nota e sigo" | Nota é o jeito de encerrar sem resolver. Liste o defeito, com prova. |
| "Eu mesmo verifico o que construí" | É a autoavaliação que a skill existe para não aceitar. |
| "Passou, então está pronto" | Passou **nos gates que existem**. Gate que não pode falhar não prova nada. |

## Ligação com o resto do acervo

`gates-de-conclusao` define aceite, abortos e ledger — o laço vive dentro deles ·
`critico-independente` dá o verificador isolado e o formato do veredito · `teste-primeiro`
produz a prova sem a qual não há laço · `micro-ticket-planner` dá o tamanho de trabalho em
que o laço cabe · `atribuicao-de-falha` quando o mesmo defeito volta em ciclos diferentes.

## Definição de pronto da skill

- [ ] O verificador rodou em contexto limpo, sem o raciocínio do construtor.
- [ ] O veredito é binário; nenhum número fez papel de medida.
- [ ] Cada defeito apontado tem comando de prova, ou está declarado como observação.
- [ ] Cada volta foi registrada com hipótese, mudança e resultado.
- [ ] Nenhuma volta repetiu a hipótese anterior.
- [ ] O teto de gasto foi declarado **antes** da primeira volta.
- [ ] Encerrando sem aceite, a escalada trouxe as cinco linhas — inclusive a menor reprodução.
