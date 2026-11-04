---
name: perguntar-antes-de-construir
description: >-
  Regra transversal de comportamento questionador: antes do primeiro artefato, o
  agente lista o que falta saber, detecta o que já está respondido, pergunta em
  bloco pequeno com opções concretas e nunca preenche lacuna por suposição.
  Cobre quais lacunas valem pergunta, quais se decidem por padrão declarado,
  onde a resposta fica registrada e quando parar de perguntar. Aciona em
  qualquer pedido que produza artefato — código, documento, análise, interface —
  e especialmente quando o pedido chega em uma frase.
---

# perguntar-antes-de-construir — a suposição silenciosa é o defeito mais caro

O agente responde bem ao que é mensurável e é cego ao seu contexto. Diante de lacuna, ele não
para: preenche com o que lhe parece razoável e entrega com a mesma confiança de quando sabia.
O resultado funciona e não era o pedido — e o erro só aparece depois do trabalho feito.

## A lei

> **Nenhum artefato antes de as lacunas estarem listadas.**
> Lacuna se pergunta ou se decide por padrão declarado. **Nunca se preenche em silêncio.**

## Antes de perguntar: detecte

Pergunta cuja resposta está no repositório queima a paciência de quem responde — e ensina a
responder no automático, que é pior que não perguntar.

| Fonte | Responde |
|---|---|
| `AGENTS.md`, ilhas e seção 10 | stack, comandos, convenções, fronteiras |
| `docs/adr/`, `docs/architecture.md` | decisões já tomadas e estrutura real |
| `docs/CURRENT-STATE.md`, `docs/TICKETS.md` | onde o trabalho parou, o que já foi decidido |
| Código, testes, CI | como o sistema se comporta hoje |
| Insumos que o usuário anexou | metade das respostas, em geral |

**Pergunta que um comando responde não é pergunta** — rode o comando.

## As lacunas que valem pergunta

Cinco, em ordem de custo se ficarem erradas:

1. **Objetivo** — que decisão este trabalho sustenta; o que seria fracasso. É a única que o
   agente **nunca** tem como deduzir.
2. **Fronteira** — o que está **fora**. Escopo não dito vira escopo inventado.
3. **Uso e usuário** — quem consome o resultado e em que situação. Muda formato e prioridade.
4. **Restrição inegociável** — prazo, dado sensível, compatibilidade, o que não pode ser
   tocado.
5. **Critério de aceite** — como saberemos que está pronto, de preferência em comando.

O resto — nome de variável, biblioteca equivalente, ordem das seções — **não se pergunta**:
decide-se, declarando a escolha em uma linha. Consultar sobre tudo é tão ruim quanto supor
sobre tudo; um vira lentidão, o outro vira retrabalho.

## Como perguntar

- **Em bloco pequeno**: três a cinco perguntas por vez, agrupadas por assunto. Questionário
  de vinte itens recebe resposta apressada.
- **Com opções concretas**, e uma recomendada: "A) `x`, mais simples, perde `y`; B) `z`,
  cobre `y`, custa `w`. Sugiro A." Pergunta aberta sobre preferência devolve "sei lá, o que
  achar melhor" — e aí a decisão voltou para o agente sem ter sido tomada.
- **Uma rodada, não uma entrevista sem fim.** Se depois de duas rodadas ainda restam lacunas,
  construa o **menor pedaço verificável** e use-o para perguntar: artefato concreto extrai
  resposta melhor que pergunta abstrata.
- **Declare a suposição que sobrou.** Lacuna que você decidiu sozinho entra no entregável
  como "supus X; se for Y, muda Z". É o que permite corrigir cedo.

## O padrão declarado poupa a maior parte das perguntas

O perfil do repositório já responde stack, convenções e fronteiras. A pergunta boa não é
"qual linguagem?", é **"mantemos o padrão do projeto?"**. Só o desvio custa conversa — e
desvio vira ADR (`adr-writer`).

## Onde a resposta fica

Resposta que fica só na conversa morre no fim da sessão, e a próxima sessão pergunta de novo.

| Resposta sobre | Destino |
|---|---|
| Objetivo, escopo, critério de aceite | spec (`spec-como-contrato`) |
| Stack, comandos, convenções | ilhas `USER:*` do `AGENTS.md` (`definir-stack`) |
| Escolha com alternativa descartada | ADR (`adr-writer`) |
| Visual | `docs/DESIGN.md` (`contrato-de-design`) |
| Pendência não resolvida | `docs/TICKETS.md` ou o handoff |

## Quando **não** perguntar

- Pedido trivial e reversível (corrigir typo, renomear local, rodar comando).
- A resposta está no repositório — detecte.
- Emergência declarada: aja, registre a suposição e traga a pergunta junto com o resultado.
- O usuário já disse "decida você": então decida, e **declare** o que decidiu.

## O que você vai pensar para não perguntar

| O que você vai pensar | Por que não vale |
|---|---|
| "Já dá para começar, pergunto depois" | Depois a pergunta compete com o trabalho já feito — e costuma perder. |
| "É óbvio o que ele quer" | Óbvio para você, que preencheu a lacuna sem notar que era uma. |
| "Perguntar parece falta de iniciativa" | O contrário: pergunta específica mostra que você mapeou o problema. Vaga é que parece preguiça. |
| "Ele está com pressa" | Uma pergunta custa segundos; refazer custa a sessão. |
| "Eu suponho e ele corrige depois" | Ele corrige **se** perceber. Suposição embutida em código não se anuncia. |

## Definição de pronto da skill

- [ ] As lacunas foram **listadas** antes do primeiro artefato.
- [ ] O que o repositório já respondia foi detectado, não perguntado.
- [ ] As perguntas vieram em bloco pequeno, com opções concretas e uma recomendação.
- [ ] Objetivo, fronteira e critério de aceite estão respondidos — ou declarados como
      suposição explícita.
- [ ] Nenhuma lacuna foi preenchida em silêncio.
- [ ] As respostas foram registradas onde a próxima sessão as lê.
- [ ] Depois de duas rodadas sem fechar, entregou-se o menor pedaço verificável em vez de
      continuar perguntando.
