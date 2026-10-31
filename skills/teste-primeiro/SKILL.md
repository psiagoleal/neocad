---
name: teste-primeiro
description: >-
  Disciplina de TDD aplicada a agente: nenhuma linha de implementação antes de
  existir um teste que falha pelo motivo certo, com a saída da falha citada como
  evidência. Cobre o ciclo vermelho-verde-refatora com prova em cada fase, o
  isolamento de contexto que impede o agente de escrever o teste à imagem do
  código, e as desculpas previsíveis para pular a regra. Aciona ao implementar
  comportamento novo, ao corrigir bug, ao receber "escreva os testes depois", ou
  quando a cobertura está alta e a confiança baixa.
---

# teste-primeiro — o teste que nasce depois do código não prova nada

Deixado por conta própria, o agente escreve a implementação e **depois** escreve testes que
confirmam exatamente o que o código já faz. O resultado é cobertura alta com confiança zero:
os testes passam porque foram desenhados a partir da resposta.

## A lei

> **Sem teste falhando, não se escreve implementação.**
> A falha precisa ser **lida** e citada — não presumida.

Sem exceção por pressa, por a mudança ser pequena, por o comportamento ser óbvio ou por já
ter conferido à mão.

## Por que a ordem é o que importa

O valor do teste não está em existir; está em **ter falhado antes**. Um teste que nunca foi
visto falhando não demonstra que detecta a ausência do comportamento — demonstra apenas que
concorda com o código que estava na tela quando foi escrito.

É por isso que a regra é sobre ordem, e não sobre cobertura. Cobertura mede quanto código foi
executado. Nada nela distingue um teste que prova de um teste que descreve.

## O ciclo, com evidência em cada fase

**1. Vermelho — escreva o teste e veja-o falhar.**
Rode. Cite a saída da falha. Só se passa à fase seguinte com essa saída em mãos.

⚠️ **Falha pelo motivo certo.** Erro de importação, de sintaxe ou de arquivo ausente **não é
vermelho** — é teste quebrado. O vermelho legítimo é a asserção falhando: o comportamento não
existe ainda. Confundir os dois é a forma mais comum de fabricar um ciclo que nunca aconteceu.

**2. Verde — o mínimo que faz passar.**
Nada além. Sem generalizar para casos que ninguém pediu, sem abstrair para um único uso. Rode
e cite a saída de sucesso.

**3. Refatora — com a rede armada.**
Agora melhore nomes, remova duplicação, extraia o que se repete. Os testes passam antes e
depois; se não passam, não é refatoração, é mudança de comportamento disfarçada.

## Isolamento de contexto

Quem escreve o teste não deveria estar olhando para o plano de implementação. Na mesma janela,
o raciocínio de como resolver **vaza** para o teste, e ele nasce moldado à solução imaginada.

Delegue a escrita do teste a uma janela separada, passando **só o critério de aceite** — não o
desenho da solução. Quem implementa recebe **só o teste falhando**, não o raciocínio de quem o
escreveu. Ver `delegacao-a-subagentes` para o formato do pedido autocontido.

Esse isolamento é o que separa TDD de verdade de teatro de TDD.

## O que você vai pensar para pular a regra

| O que você vai pensar | Por que não vale |
|---|---|
| "É trivial, o teste é óbvio" | Se é óbvio, escrevê-lo custa um minuto. O que custa caro é descobrir que não era. |
| "Escrevo os testes logo depois" | Depois eles nascem da resposta. É o modo de falha que esta skill existe para impedir. |
| "Já testei à mão e funcionou" | Teste manual não roda de novo no próximo commit. |
| "Falhou com erro de importação, conta como vermelho" | Não conta. Isso é teste quebrado, não comportamento ausente. |
| "O teste está errado, vou ajustá-lo para passar" | Ajustar o teste depois de ver o código é apagar a prova. Se o teste está mesmo errado, corrija-o **antes** de olhar a implementação e volte ao vermelho. |
| "Mocko essa parte e pronto" | Mock no lugar do comportamento sob teste prova que o mock funciona. |
| "A suíte é lenta, rodo no fim" | Rode o teste do caso; a suíte inteira fica para o gate. |

**Pare se pensar qualquer uma delas.** A frase é o sinal de que a lei está prestes a cair.

## Nunca enfraqueça o teste para fechar o verde

Remover asserção, afrouxar tolerância, marcar como ignorado, capturar a exceção que era o
ponto do teste — tudo isso converte uma falha real em verde falso. Se o teste estava certo, o
código está errado. Se o teste estava errado, corrija-o declaradamente, e diga no relato que o
fez. O `pr-review-guard` procura exatamente esse padrão no diff.

## Bug: o teste vem antes da correção

Para defeito, a ordem é a mesma e o ganho é maior: escreva o teste que **reproduz** o bug,
veja-o falhar, corrija. O teste que reproduz vira regressão permanente — sem ele, a correção
não tem prova de que atacou a causa nem impede o retorno. Ver `atribuicao-de-falha` quando o
mesmo defeito reaparece.

## Quando não aplicar

- **Exploração e spike** — enquanto o escopo não fechou, o teste fixaria a pergunta errada.
  Jogue o spike fora e comece com teste.
- **Protótipo descartável** cujo erro custa refazer, nada mais.
- **Mudança sem comportamento observável** (formatação, renomeação mecânica coberta pela
  suíte existente).

Em todos os três, diga que está fora do ciclo — e por quê. O silêncio vira hábito.

## Ligação com o resto do acervo

`spec-como-contrato` dá o critério de aceite que vira o primeiro teste · `micro-ticket-planner`
define o ciclo de trabalho em que o vermelho cabe · `gates-de-conclusao` transforma a suíte em
gate com evidência · `pr-review-guard` checa, no diff, teste novo para o comportamento alterado
e ausência de validação removida.

## Definição de pronto da skill

- [ ] Existe teste novo para cada comportamento alterado.
- [ ] O teste foi **visto falhando** antes da implementação, e a saída da falha foi citada.
- [ ] A falha era de asserção, não de importação, sintaxe ou caminho.
- [ ] A implementação é o mínimo que faz passar — sem recurso não pedido.
- [ ] Nenhuma asserção foi removida, afrouxada ou ignorada para obter verde.
- [ ] Em correção de bug, existe teste que reproduzia o defeito e agora passa.
- [ ] Fora do ciclo (spike, protótipo, mudança sem comportamento) foi declarado, não omitido.
