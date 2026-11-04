---
name: licoes-aprendidas
description: >-
  Colher lição de trabalho terminado e transformá-la em mudança de artefato, em
  vez de documento que ninguém lê: o que conta como lição, quando parar para
  colher, a escada de destinos do mais forte (comando, teste, hook) ao mais
  fraco (nota escrita), e a fila curta do que ainda não foi aplicado. Aciona ao
  terminar trabalho não trivial, depois de incidente ou rollback, quando o mesmo
  erro aparece pela segunda vez, ao receber revisão de PR com padrão repetido, e
  quando alguém diz "da próxima vez a gente lembra".
---

# licoes-aprendidas — lição sem destino é anedota

"Da próxima vez a gente lembra" é a frase que precede esquecer. Documento de lições aprendidas
é onde elas vão morrer: escrito no fim do projeto, lido por ninguém, e a mesma falha volta no
projeto seguinte com outro nome.

## A lei

> **Toda lição vira mudança em artefato — ou é descartada, explicitamente.**
> Registrar sem aplicar não conserva a lição; conserva a ilusão de tê-la aprendido.

## O que conta como lição

Três critérios, e precisa dos três:

1. **Surpresa** — o que aconteceu foi diferente do esperado. Sem surpresa, é confirmação.
2. **Custo** — gastou tempo, retrabalho, cota, ou quase quebrou algo.
3. **Generalidade** — vale além deste caso. Um erro de digitação não gera lição; um erro de
   digitação que passou por três revisões, sim.

Não é lição: reclamação sobre ferramenta, detalhe que só vale neste arquivo, e o que já está
escrito numa regra que simplesmente não foi seguida — isso é outro problema
(`atribuicao-de-falha`).

## Quando colher

Ao **terminar** — trabalho não trivial, incidente, rollback, PR com correção repetida — e
quando o mesmo erro aparece **pela segunda vez**. A segunda vez é o sinal mais confiável que
existe: uma vez é acaso, duas é padrão.

Não é ritual de calendário. Reunião mensal de lições produz texto por obrigação, e texto por
obrigação é o que enche o cemitério.

⚠️ **Não colha de trabalho que ainda não terminou.** Enquanto a causa não está confirmada, a
"lição" é palpite — e palpite vira regra difícil de remover depois.

## A escada de destinos

Este é o coração da skill. Para cada lição, desça a escada **até onde der**: quanto mais alto,
menos depende de alguém lembrar.

| Nível | Destino | Por que é mais forte |
|---|---|---|
| 1 | **Comando ou verificador** | Falha sozinho, sem depender de leitura |
| 2 | **Teste** | Prende o comportamento; regride explícito |
| 3 | **Hook ou gate** (`commit-msg`, CI) | Impede antes de acontecer |
| 4 | **Template ou valor padrão** | O caminho certo vira o caminho fácil |
| 5 | **Regra escrita** (perfil, skill) | Probabilística: depende de ser lida e obedecida |
| 6 | **Nota no handoff** | Vale uma sessão; some depois |

> **Parar no nível 5 quando o 1 era possível é o erro mais comum.** Regra em texto pede
> disciplina de quem lê; comando tira a escolha.

Dois exemplos desta própria biblioteca, ambos do mesmo dia:

- Um auxiliar de cálculo devolvia valor **arredondado**, e encadear os passos propagava erro.
  A saída fraca seria escrever "cuidado com o arredondamento" (nível 5). A aplicada foi
  devolver precisão cheia **e** acrescentar autoteste (níveis 1 e 2). Resíduo medido caiu de
  3,15e-5 para 5,18e-15.
- O painel de tickets tinha conjunto **aberto** de marcadores, e uma sessão inventou o seu.
  A regra "não invente marcador" (nível 5) entrou — mas acompanhada de um verificador que
  recusa marcador fora da lista (nível 1).

## O formato: quatro campos, nada mais

```markdown
- **Esperávamos:** o teste de rota cobria o caminho de erro.
  **Aconteceu:** cobria só o caminho feliz; o 409 nunca foi exercitado.
  **Por quê:** o critério de aceite não pedia o caminho infeliz, e ninguém notou na revisão.
  **Muda:** item no checklist do `pr-review-guard` + teste de caso de erro obrigatório na DoD.
```

Sem narrativa, sem culpado, sem "deveríamos ter tido mais cuidado". O campo **Muda** é
obrigatório: lição sem ele não entra.

## A fila, não o cemitério

Lição aplicada **vive no artefato que mudou** — o verificador, o teste, a regra. Não se guarda
cópia.

O que precisa de registro é só a lição **cuja mudança ainda não foi aplicada**: vai para
`docs/TICKETS.md` como ticket comum, com o destino no texto. Assim ela compete por prioridade
com o resto do trabalho, em vez de repousar num documento paralelo.

Se um arquivo de lições existir no projeto, ele tem uma regra de saída: **item aplicado sai da
lista**. Lista que só cresce é cemitério.

## Lição que vem de fora

Revisão de PR, incidente relatado por quem opera, retorno de outra equipe ou de outra sessão de
agente: tudo entra pelo mesmo funil — três critérios, quatro campos, escada de destinos. O que
muda é a verificação: lição de terceiro descreve o **sintoma** com precisão e a **causa** por
suposição. Confirme a causa antes de mexer no artefato (`atribuicao-de-falha`,
`critico-independente`).

## O que você vai pensar

| O que você vai pensar | Por que não vale |
|---|---|
| "Anoto agora e aplico depois" | "Depois" é o nome do cemitério. Aplique, ou vire ticket com destino. |
| "Escrevo uma regra para não acontecer de novo" | Regra é o nível 5. Antes dela, pergunte se cabe comando, teste ou hook. |
| "Foi caso isolado" | Se foi a segunda vez, não foi isolado — e a segunda vez é o sinal. |
| "A lição é óbvia, todo mundo entendeu" | Óbvio para quem estava lá. A próxima sessão começa em branco. |
| "Vou registrar tudo para não perder nada" | Lista que só cresce deixa de ser lida, e aí perdeu tudo. |

## Ligação com o resto do acervo

`atribuicao-de-falha` diagnostica **em que camada** a falha nasceu — use antes de escolher o
destino · `adr-writer` para a lição que muda uma decisão · `pr-review-guard` e
`gates-de-conclusao` costumam ser o destino · `micro-ticket-planner` recebe a lição ainda não
aplicada · `handoff-updater` carrega só a lição desta sessão, nunca o acervo.

## Definição de pronto da skill

- [ ] Cada lição passou nos três critérios: surpresa, custo e generalidade.
- [ ] Cada lição tem os quatro campos, com o **Muda** preenchido.
- [ ] O destino escolhido é o nível mais alto viável da escada — e, se parou no nível 5,
      está dito por que comando, teste ou hook não serviam.
- [ ] A mudança foi **aplicada**, ou virou ticket com o destino escrito.
- [ ] Nenhuma lição foi colhida de trabalho ainda em andamento.
- [ ] Lição vinda de fora teve a **causa confirmada** antes de virar mudança.
- [ ] Nenhum item aplicado ficou duplicado numa lista paralela.
