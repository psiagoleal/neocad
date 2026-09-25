---
name: critico-independente
description: >-
  Fecha o laço de verificação do trabalho de um agente com três movimentos:
  critério de avaliação declarado antes da execução, crítica por um segundo
  modelo em contexto limpo (de preferência de outra família), e sinal externo
  puxado do mundo real em vez de autoavaliação. Aciona ao revisar saída de
  agente, quando a entrega "parece certa" mas não há prova, antes de aceitar
  trabalho longo, ou quando o mesmo agente que produziu também julgaria.
---

# critico-independente — quem produziu não é quem julga

Um agente não tem motivação intrínseca: gritar, pedir capricho ou dizer "faça melhor" não
muda o resultado. Ele responde ao que consegue **verificar**. Por isso a alavanca não é a
insistência — é o laço de verificação.

## A regra

> **Autoavaliação não é evidência.** Quem produziu o artefato não decide se ele passa.

## Por que o elogio não funciona e o critério sim

O modelo é bom no que é mensurável e cego no que depende do seu contexto. Sem critério
declarado, ele preenche a lacuna com o que lhe parece razoável e relata sucesso com a mesma
confiança nos dois casos — quando acertou e quando inventou. A diferença entre os dois só
aparece se existir algo fora dele para conferir.

## 1. Critério antes, não depois

Declare o que conta como bom **antes** de o agente começar. Critério escrito depois da entrega
descreve o que foi feito; escrito antes, decide.

> Vago: "deixe o relatório bom."
> Preciso: "três seções, cada uma terminando em uma recomendação acionável."

Peça ao próprio agente que explicite o critério antes de executar — e confira o critério, não
só o resultado:

> "Antes de começar, escreva os critérios de avaliação que você usará para julgar o resultado
> final. Seja preciso."

Em trabalho verificável por comando, o critério é o comando (`spec-como-contrato`,
`gates-de-conclusao`). Em trabalho de texto, é a rubrica.

## 2. Um segundo modelo como crítico

O crítico deve receber **o objetivo, o critério e o artefato executado** — nunca o raciocínio
de quem construiu, que contamina o julgamento. Contexto limpo, sempre.

Melhor ainda se for **outra família de modelo**: um modelo diferente errou em lugares
diferentes, e é justamente essa discordância que carrega informação. Modelo igual em contexto
limpo ainda compartilha os mesmos vieses do construtor.

Rotas, em ordem de custo:

| Rota | Quando |
|---|---|
| `delegacao-openai-compat` com `oa-chat -m <outro modelo>` | Padrão: crítico de outra família, fora da cota da assinatura |
| Subagente em janela separada (`delegacao-a-subagentes`) | Quando o material não pode sair da máquina |
| Outra CLI de agente instalada | Quando você quer também outro harness, não só outro modelo |

Peça veredito **e** a maior lacuna, não uma nota:

> "Objetivo: <...>. Critério: <...>. Artefato em anexo. Responda em duas linhas:
> `VEREDITO: passa|não passa` e `MAIOR_LACUNA: <uma frase>`. Não proponha código."

⚠️ **O crítico também erra.** Ele não é árbitro final: é um segundo sinal barato. Discordância
entre construtor e crítico é motivo para você olhar, não para aceitar automaticamente o
crítico. E repetir a mesma rubrica muitas vezes otimiza para a rubrica, não para a qualidade.

## 3. Puxe sinal externo

O sinal mais forte não vem de nenhum modelo: vem do mundo.

- Fez deploy? Consulte o alvo e confirme que subiu, em vez de acreditar no relato.
- Produziu relatório? Dê os relatórios anteriores como referência de formato.
- Mexeu em API? Chame o endpoint e leia a resposta.
- Corrigiu bug? Rode o teste que reproduzia (`teste-primeiro`).

Cada sinal externo tira uma afirmação do terreno da confiança e coloca no terreno da
observação.

## A ordem é econômica

Barato antes de caro, sempre: comando determinístico → sinal externo → crítico por modelo →
revisão humana. Cada filtro barato que reprova poupa uma rodada do caro. Reserve o julgamento
humano para o que nenhum dos três decidiu — é o mais caro e o único que responde pelo
resultado (`pr-review-guard`).

## O que você vai pensar para pular o crítico

| O que você vai pensar | Por que não vale |
|---|---|
| "O próprio agente disse que está pronto" | É exatamente a afirmação que a skill existe para não aceitar. |
| "Rodou sem erro" | Rodar prova que executou, não que resolveu o problema certo. |
| "É só um texto, não dá para verificar" | Dá: rubrica declarada antes, e comparação com um artefato anterior aceito. |
| "Um segundo modelo custa tempo" | Menos que descobrir depois. E fora da cota, pela rota do gateway. |
| "O crítico aprovou" | O crítico é sinal, não sentença — ainda mais se for da mesma família. |

## Definição de pronto da skill

- [ ] O critério de avaliação foi escrito **antes** da execução.
- [ ] O crítico recebeu objetivo, critério e artefato — **não** o raciocínio do construtor.
- [ ] O crítico rodou em contexto limpo e, quando possível, em outra família de modelo.
- [ ] Pelo menos um sinal externo real foi consultado, quando havia um disponível.
- [ ] Discordância entre construtor e crítico foi levada a decisão humana, não resolvida no voto.
- [ ] Nenhuma conclusão se apoia apenas no relato de quem produziu.
