---
name: delegacao-a-subagentes
description: >-
  Orienta quando e como delegar trabalho a um subagente com modelo local, para
  que dados confidenciais nunca cheguem a um modelo remoto, e como escrever o
  pedido de delegação de forma que a resposta volte utilizável. Aciona ao
  planejar uma tarefa que toca dados sensíveis (PII, credenciais, código
  proprietário), ao configurar roteamento por task-class, ou quando o usuário
  pedir para "delegar", "usar modelo local" ou "não mandar isso para a nuvem".
---

# delegacao-a-subagentes — o que fica local, o que vai para a nuvem

Arquiteturas multi-modelo permitem que um agente forte (remoto, caro) planeje enquanto um
modelo pequeno (local, gratuito) toca o que é sensível. O ganho só existe se a fronteira for
respeitada — e **a fronteira não se defende sozinha**.

## O princípio

> O agente remoto recebe **o que ele precisa para decidir**, nunca **o dado bruto de onde
> aquilo foi extraído**.

Um resumo agregado, uma contagem, um nome de coluna, um trecho de esquema: isso decide. A
linha do CSV com o CPF de alguém não decide nada que a contagem já não decidisse.

## O que a ferramenta garante e o que não garante

Antes de confiar na delegação, entenda o limite real — verificado em teste, não suposto:

- ✅ **Contenção estrutural existe.** Com escopo de leitura por caminho (`readAllow` no
  `agentry`) o agente remoto fica *incapaz* de abrir o arquivo, não apenas instruído a evitá-lo.
- ❌ **Sanitização não é imposta.** O agente remoto pode pedir ao subagente "leia e me devolva
  tudo", e o modelo local pode obedecer. A delegação cria o **ponto** onde a sanitização pode
  acontecer; não a garante.
- ❌ **Filtro literal não protege PII.** Bloquear a palavra `cpf` faz o modelo reescrever os
  dados sem aquele rótulo e mandar os nomes assim mesmo (comportamento observado). Padrão de
  PII exige correspondência por padrão, não por literal.

Conclusão prática: **escreva o pedido de delegação como se ele fosse a única proteção**,
porque em boa parte dos casos ele é.

## Como escrever um pedido de delegação

Quatro elementos. Faltando qualquer um, a resposta tende a voltar inútil ou vazada.

1. **A tarefa concreta**, não o objetivo abstrato.
   *Ruim:* "analise os clientes". *Bom:* "conte quantos clientes há por plano".
2. **A forma exata da resposta.** O modelo local é pequeno: se você não disser o formato, ele
   devolve prosa, ou código que *descreveria* a resposta em vez da resposta.
   *Bom:* "responda só uma linha por plano, no formato `plano=<nome> qtd=<n>`".
3. **A proibição explícita, campo a campo.** Não escreva "não mande nada sensível" — liste.
   *Bom:* "não inclua nome, CPF, e-mail nem endereço na resposta".
4. **O que fazer se não der.** Sem isso, um modelo pequeno inventa.
   *Bom:* "se o arquivo não existir, responda exatamente `ARQUIVO_AUSENTE`".

### Exemplo

```
Leia clientes.csv com fs_read. Responda SOMENTE uma linha por plano, no formato
`plano=<nome> qtd=<n> mrr_medio=<valor>`. NÃO inclua nome, CPF, e-mail ou
qualquer identificador individual. Se o arquivo não existir, responda
exatamente ARQUIVO_AUSENTE.
```

## O que delegar ao modelo local

| Delegue | Por quê |
|---|---|
| Leitura e agregação de arquivo com dado pessoal | É o caso que motiva a arquitetura |
| Extração de esquema/estrutura (nomes de coluna, tipos, contagens) | O remoto precisa da forma, não do conteúdo |
| Busca e contagem de ocorrências | Resposta é um número, não o texto encontrado |
| Redação/mascaramento antes de um envio | O texto bruto não deve sair da máquina |

## O que **não** delegar

| Não delegue | Por quê |
|---|---|
| Raciocínio longo, arquitetura, trade-offs | Modelos pequenos degradam rápido; é o que o remoto faz bem |
| Tarefas de múltiplos passos encadeados | Tendem a entrar em laço ou perder o fio (observado) |
| Escrita de código não trivial | A revisão custa mais que escrever |
| Qualquer coisa cujo resultado você não consiga verificar | Sem verificação, você trocou risco de vazamento por risco de erro silencioso |

## Escolha de modelo local

Capacidade de modelos pequenos muda rápido; qualquer lista de nomes aqui envelhece em
silêncio. Por isso esta skill fixa **critérios**, não recomendações de modelo:

- **Exija suporte nativo a *tool-calling*.** Sem isso o modelo descreve a chamada em texto e
  nada executa. Verifique antes de escolher (no Ollama: `ollama show <modelo>` deve listar
  `tools` entre as capacidades).
- **Prefira o menor modelo que passe no seu teste de aceitação**, não o maior que caiba na
  memória — latência entra em cada delegação.
- **Teste com o pedido real, não com um exemplo.** Modelos do mesmo porte falham de formas
  bem diferentes: um obedece à instrução de sanitizar mas erra o formato; outro acerta o
  formato e ignora a proibição.

<!-- USER:BEGIN id=modelos-locais-observados -->
### Observações de campo (perecível — revise a cada ~3 meses)

*Última atualização: 2026-07-28.* Vale só como ponto de partida; refaça o teste no seu caso.

- `qwen2.5:7b` — compõe pedidos de delegação coerentes; **obedeceu** à instrução de sanitizar
  quando ela era explícita, e **vazou o arquivo inteiro** quando não era.
- `llama3.1:8b` — mais conservador no conteúdo, porém instável no protocolo: repassou
  fragmentos da instrução como pedido, e num caso entrou em laço de chamadas repetidas.
<!-- USER:END -->

## Definição de pronto da skill

- [ ] O pedido de delegação nomeia a tarefa, o formato da resposta, a proibição campo a campo
      e o comportamento em caso de falha.
- [ ] Nenhum identificador individual aparece no que sai da máquina — conferido no resultado,
      não presumido.
- [ ] O que foi delegado é verificável por quem pediu.
- [ ] O modelo local escolhido tem *tool-calling* nativo.
