---
name: paralelizacao-em-grafo
description: >-
  Decide se um trabalho deve virar grafo paralelo e como executá-lo: fecha o
  contrato antes de abrir fatias, exige fatia vertical que não compartilhe
  arquivo nem contexto, isola cada agente em seu worktree e reúne tudo num
  fan-in de dono único. Explica a troca — relógio barato, token caro (N×
  prefill) — e os casos em que paralelizar não compensa. Aciona ao planejar onda
  com várias tarefas, ao pensar em "rodar em paralelo", ao dividir trabalho
  entre agentes, ou quando um merge de trabalho paralelo virar negociação.
---

# paralelizacao-em-grafo — quando abrir em fatias compensa

Rodar três agentes ao mesmo tempo parece render três vezes mais. Rende, no relógio,
se — e só se — as fatias forem de verdade independentes. Se não forem, o custo
reaparece no fim, somado.

## O princípio

> Paralelizar só compensa quando as fatias **não compartilham contexto**.

E a troca, que precisa ser feita de olhos abertos:

> Fan-out compra **relógio** e paga em **token**. Cada janela relê contrato e base de
> código do zero: três fatias não custam um terço, custam três vezes o contexto de uma.

O relógio do fan-out é o da **fatia mais lenta**, não a soma. A fatura é N× prefill.

## A referência que o paralelismo precisa vencer

Antes de decidir, tenha claro o que o laço sequencial já entrega: uma janela, uma
fila, contrato e base de código lidos **uma vez só**. Relógio caro, token barato. O
fan-out precisa provar que compensa contra isso — não contra nada.

⚠️ **Nunca paralelize uma onda de uma tarefa só.** Não há grafo; é overhead puro.

## Fase 0 — feche o contrato antes de abrir

Antes de qualquer paralelismo: schema, tipos e formato de erro fechados. As fatias
**consomem** o contrato; nenhuma o redefine.

Sem isso, dois agentes criam o mesmo tipo com nomes e campos diferentes, e a
incompatibilidade só aparece no fan-in — quando as duas implementações já foram pagas.
Fase 0 não é overhead: é o que faz o fan-out render sem virar retrabalho de costura.

Contrato que embute decisão de arquitetura vira ADR (`adr-writer`).

## Fatia vertical ou nada

| | Fatia vertical — divide de verdade | Fatia horizontal — divide só no papel |
|---|---|---|
| Recorte | uma feature inteira por agente | uma camada por agente |
| Exemplo | `features/exportacao/{schema,api,ui}` | `models.py` · `router.py` · `types.ts` — todos |
| Arquivos | exclusivos daquela fatia | compartilhados entre as três |
| Fan-in | junção: as peças já encaixam pelo contrato | negociação: merge de mudanças incompatíveis |

O teste é literal: **existe algum arquivo que duas fatias leem ou escrevem?** Se sim,
não são independentes, e o ganho de relógio será devolvido — com juros — no fan-in.

**Anti-padrão:** separar por diretório componentes acoplados. Diretório não cria
independência quando os componentes compartilham tipo, estado ou comportamento; cada
agente enxerga uma parte e o fan-in recebe mudanças incompatíveis.

> Paralelizável não significa decomponível. Quando a fronteira não isola, prefira
> sequencial — ou dê o conjunto acoplado a um único responsável.

## Isolamento: um worktree por fatia

`git worktree` é a forma mais barata de dar a cada agente um diretório próprio: mesmo
repositório, checkouts separados. Sem isso, duas fatias editando o mesmo arquivo geram
conflito — ou, pior, sobrescrita silenciosa.

```bash
git worktree add ../projeto-fatia-auth   -b fatia/auth
git worktree add ../projeto-fatia-export -b fatia/export
# ao fim, depois do fan-in:
git worktree remove ../projeto-fatia-auth
```

Cada fatia roda o ciclo inteiro no seu worktree: constrói, prova e verifica com os
próprios gates (`gates-de-conclusao`).

## Contrato mínimo por fatia

Nenhum worktree é aberto antes de a fatia declarar — e alguém confirmar — quais
arquivos são exclusivamente dela:

```yaml
fatia: autenticacao
worktree: .worktrees/onda-2-autenticacao
arquivos_exclusivos:
  - app/features/auth/**
  - app/tests/auth/**
depende_de: []          # vazio, senão não é paralela
```

Sem `arquivos_exclusivos` declarado **e** conferido contra as demais fatias da onda, a
fatia é horizontal disfarçada de vertical: funda com outra ou rode sequencial.

## Fan-in: um dono, uma janela

O fan-in **não paraleliza**. Alguém — agente ou pessoa — precisa ver todas as saídas
na mesma janela para resolver costura, duplicação e decisão divergente, e assinar um
resultado só.

O teto é físico, não só de custo: três fatias cabem na janela do integrador, doze não.
Mais paralelismo no fan-out apenas empurra o gargalo para cá — confira que as N fatias
cabem **antes** de abri-las.

⚠️ **O integrador não constrói.** Ele junta, confere costura e assina. Integrador que
começa a implementar deixa de ser o ponto neutro onde as divergências aparecem e vira
mais uma fatia — sem ninguém para conferi-la.

⚠️ **Fatia reprovada não entra no fan-in.** Verificação falha devolve a fatia ao seu
worktree; não se costura por cima de uma peça que já se sabe quebrada.

**O fan-in existe para achar o que nenhuma fatia podia ver.** O caso típico é a
integração que passa isolada e quebra junta — CORS ausente entre backend e frontend,
com todos os testes unitários verdes dos dois lados. Se o fan-in não procura costura,
ele não está fazendo o trabalho dele.

## O custo em cota

N× prefill sai da mesma cota de assinatura das janelas de 5h e 7d. Uma onda de quatro
fatias pode consumir a janela inteira em relógio curto — exatamente o cenário que
`limites-de-uso` manda dimensionar **antes** de começar. Fan-out é a forma mais rápida
de gastar cota que existe no framework.

Se o objetivo é economizar, e não acelerar, a rota é outra: `delegacao-openai-compat`
(volume fora da cota) ou `delegacao-a-subagentes` (trabalho fora da janela do pai).

## Quando não paralelizar

- Onda de uma tarefa só.
- Fatias que leem ou escrevem os mesmos arquivos.
- Contrato ainda aberto — feche a fase 0 primeiro.
- Trabalho que depende do histórico da conversa para ser feito.
- Quando a fatura importa mais que o prazo.

## Definição de pronto da skill

- [ ] O contrato (schema, tipos, formato de erro) estava **fechado** antes do fan-out.
- [ ] Nenhum arquivo é tocado por mais de uma fatia — verificado, não presumido.
- [ ] Nenhuma onda de tarefa única foi paralelizada.
- [ ] Cada fatia rodou em worktree próprio, com gates próprios.
- [ ] Cada fatia declarou `arquivos_exclusivos`, conferidos contra as demais.
- [ ] O fan-in teve **um** dono, coube numa janela e procurou costura de integração.
- [ ] O integrador só integrou — não implementou.
- [ ] Nenhuma fatia reprovada foi levada ao fan-in.
- [ ] O custo em cota foi dimensionado antes de abrir as fatias.
- [ ] Os worktrees temporários foram removidos ao fim.
