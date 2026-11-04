---
name: spec-como-contrato
description: >-
  Desenvolvimento guiado por especificação: a spec é o contrato versionado que
  fixa o resultado esperado, o código é artefato derivado e o teste é a spec em
  forma executável. Cobre o que entra e o que não entra numa spec, critério de
  aceite em notação EARS, uma spec por feature, a separação entre spec (durável)
  e plano (descartável), e como impedir que a spec apodreça. Aciona ao começar
  feature ou projeto novo, quando o pedido chega vago ("faça um CRUD"), antes de
  delegar trabalho a um agente, ou quando o resultado entregue não é o que se
  queria apesar de funcionar.
---

# spec-como-contrato — o agente não adivinha o que você quis dizer

Agentes escrevem código bem e adivinham intenção mal. Sem um contrato escrito, o modelo
preenche o silêncio com o que lhe parece razoável — e entrega algo que **funciona sem nunca
ter sido o que se pediu**.

## A regra

> **Antes de mandar construir, escreva o que conta como pronto.**
> Se duas pessoas leem o pedido e entregam coisas diferentes, ainda falta spec.

## O princípio

> A **spec** fixa o resultado e dura. O **plano** fixa a sequência e é descartável.
> O **teste** é a spec em forma executável. Os três são artefatos distintos.

Juntar spec e plano num arquivo só é o erro mais comum: o plano muda a cada rodada e leva a
spec junto, e em pouco tempo não há mais contrato — há um diário.

## Primeiro o objetivo, depois a tarefa

O pedido que chega é quase sempre uma **tarefa**; o que falta é o **objetivo** — a decisão que
aquilo vai sustentar. "Faça o relatório de fim de mês" é tarefa. O objetivo é a conclusão que
alguém precisa tirar dele, e é a única parte que o agente **nunca** tem como deduzir: ele não
tem sinal sobre o seu contexto.

O modo mais barato de extrair isso de você mesmo é inverter quem pergunta:

> "Antes de escrever qualquer coisa, **me entreviste** para identificar o objetivo deste
> trabalho: que decisão ele sustenta, quem usa o resultado, o que seria um fracasso."

Duas frases que valem colar no mesmo pedido, porque atacam os dois modos de falha seguintes:

> "Prefira specs **pequenas e compartimentadas** a uma spec grande."
> "**Me faça confirmar explicitamente** cada decisão-chave, para nada passar batido."

A primeira evita a spec em cascata — escrever tudo, mandar construir tudo, descobrir no fim
que derivou. A segunda evita que uma suposição do agente entre na spec como se fosse sua.

## Quando vale, e quando não

O custo da spec se paga quando o erro é caro. Calibre pelo risco:

| Situação | Profundidade |
|---|---|
| Domínio crítico, dado sensível, contrato de API, migração | Spec exaustiva, com casos de erro e não-funcionais |
| Feature comum em projeto existente | Spec curta de uma feature: resultado, fronteiras, critérios |
| Correção pontual, ajuste visual, exploração | Não escreva spec — escreva o critério de aceite e vá |

Trabalho leve não justifica o overhead. Dizer isso faz parte da skill: spec obrigatória em
tudo vira burocracia que as pessoas aprendem a contornar.

## O que entra na spec

- **Problema e resultado esperado** — por que existe, o que muda quando estiver pronto.
- **Fronteiras de escopo** — e, explicitamente, **o que está fora**.
- **Critérios de aceite verificáveis** (abaixo, em EARS).
- **Restrições e decisões já tomadas** — stack, padrões, o que não pode ser tocado.
- **Casos de erro e caminhos infelizes** — é onde o agente mais inventa.
- **Requisitos não-funcionais** que mudam o desenho (latência, volume, privacidade).

## O que **não** entra

- **Detalhe de implementação** que não seja restrição real. Spec que dita cada função vira
  código em prosa: perde o valor de contrato e envelhece junto com o código.
- **A sequência de tarefas** — isso é plano.
- **Decisão de arquitetura com alternativas e trade-off** — isso é ADR (`adr-writer`); a spec
  cita a decisão, não a rediscute.

> Sub-especificar deixa o agente com discrição demais. Super-especificar apaga a fronteira
> entre spec e código. Especifique **o que importa**; deixe o resto ao julgamento de quem
> implementa.

## Critério de aceite em EARS

EARS (*Easy Approach to Requirements Syntax*, de engenharia de requisitos aeroespacial) evita
a ambiguidade que faz o agente errar. Cinco formas cobrem quase tudo — sempre com **DEVE**,
nunca "deveria", "pode" ou "idealmente":

| Forma | Padrão | Exemplo |
|---|---|---|
| Universal | `O <sistema> DEVE <resposta>` | A API DEVE rejeitar corpo acima de 1 MB |
| Evento | `QUANDO <gatilho>, o <sistema> DEVE <resposta>` | QUANDO a reserva se sobrepõe a outra, a API DEVE responder 409 |
| Estado | `ENQUANTO <condição>, o <sistema> DEVE <resposta>` | ENQUANTO o usuário não estiver autenticado, a tela DEVE exibir o convite de login |
| Opcional | `ONDE <recurso existe>, o <sistema> DEVE <resposta>` | ONDE houver integração SSO, o login local DEVE ficar oculto |
| Erro | `SE <condição indesejada>, ENTÃO o <sistema> DEVE <resposta>` | SE o provedor não responder em 5 s, ENTÃO o serviço DEVE devolver 503 e registrar o evento |

Quanto mais preciso o critério, menos o agente supõe — e cada suposição é uma chance de
desviar do que você queria. Cada critério precisa ter uma prova associada. **Critério sem prova não vira tarefa** — ou se
escreve o comando que o verifica, ou ele não está pronto para ser implementado.

## Uma spec por feature

Spec monolítica de projeto inteiro estoura a janela de contexto e ninguém mantém. Em projeto
existente, uma spec por feature funciona melhor. Requisitos transversais (segurança, logging,
convenções) vão para **um** documento compartilhado, citado pelas demais em vez de repetido.

## A spec apodrece se você deixar

> Mudou o comportamento, atualize a spec no mesmo trabalho. Não no fim da sprint.

Spec desatualizada é pior que spec inexistente: o agente a lê como verdade e reintroduz o que
você removeu. Se a mudança contraria a spec, ou a spec muda, ou a mudança está errada — decida
qual, explicitamente.

## Antes de implementar, submeta a spec à crítica

Peça a um agente em **contexto limpo** que aponte ambiguidade, contradição e caso de erro
faltando — sem propor código. Quando as objeções que sobram são triviais, a spec está pronta.

A revisão humana continua sendo a que decide: spec é onde erro custa menos para corrigir e
mais para deixar passar.

## Anti-padrões

- **Spec escrita por uma pessoa só.** Requisito é acordo; solo vira preferência.
- **Implementar a spec inteira numa sessão.** Ela cabe no disco, não na janela — fatie em
  micro-tickets (`micro-ticket-planner`).
- **Spec e plano no mesmo arquivo.**
- **Critério em prosa** ("deve ser rápido") em vez de EARS com prova.
- **Spec como enfeite**: escrita, aprovada e nunca mais lida durante a implementação.

## Ligação com o resto do acervo

`micro-ticket-planner` fatia a spec em trabalho de um ciclo · `teste-primeiro` transforma o
critério em teste que nasce falhando · `gates-de-conclusao` exige evidência de execução ·
`adr-writer` guarda a decisão que a spec apenas cita · `pr-review-guard` confere o entregue
contra o contrato.

## Definição de pronto da skill

- [ ] Existe spec versionada, separada do plano de execução.
- [ ] Todo critério de aceite está em EARS e tem **comando de prova** associado.
- [ ] O que está **fora** de escopo foi escrito, não subentendido.
- [ ] Casos de erro e não-funcionais que mudam o desenho estão cobertos.
- [ ] A spec passou por crítica em contexto limpo antes da implementação.
- [ ] Depois da mudança, a spec foi atualizada no mesmo trabalho.
