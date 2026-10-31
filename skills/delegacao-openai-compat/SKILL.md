---
name: delegacao-openai-compat
description: >-
  Delega trabalho de volume a um modelo servido por qualquer endpoint
  OpenAI-compatible (gateway LiteLLM, vLLM, Ollama em /v1, LM Studio, OpenRouter)
  para que o custo saia da cota da assinatura e a saída não ocupe o contexto da
  sessão. Traz o script `oa-chat` (chamada direta, sem chave em argv) e a rota
  via `agentry` para modo agente. Cobre a checagem de egresso obrigatória, a
  resolução da chave sem expô-la, modelos de raciocínio que esgotam tokens, o
  pedido autocontido e a verificação do resultado. Aciona quando o usuário pedir
  para "delegar ao gateway", "usar o LiteLLM", "usar modelo OpenAI-compatible",
  "rodar num modelo mais barato", "economizar limite" ou "economizar cota"; ao
  planejar varredura, revisão ou extração volumosa; ou ao configurar
  `providers.litellm` e task-class de delegação no `agentry`.
---

# delegacao-openai-compat — gastar token de endpoint em vez de cota de assinatura

## A regra

> **Nada sai para um endpoint sem egresso conhecido. E nada que volta de lá é usado sem
> verificação.**

Delegar economiza cota; não transfere responsabilidade. O endpoint é um modelo que você não
escolheu, com qualidade que você não mediu, atrás de uma rota que talvez termine na nuvem
pública.

## Por que "OpenAI-compatible" e não "LiteLLM"

A API de *chat completions* da OpenAI virou o protocolo comum: LiteLLM, vLLM, SGLang, Ollama
(em `/v1`), LM Studio, OpenRouter e a própria OpenAI falam o mesmo formato. O que esta skill
ensina vale para todos. Nenhum endereço, chave ou nome de modelo mora neste repositório —
tudo vem do ambiente.

"Compatível", porém, cobre o **núcleo** do protocolo. Fora dele, cada servidor diverge:

| Recurso | Comportamento real |
|---|---|
| `messages`, `temperature`, `max_tokens` | Universal |
| `response_format: json_object` | Comum, não universal — confira a saída mesmo assim |
| Campo de raciocínio (`reasoning_content`) | Não é OpenAI; aparece em vLLM/SGLang com modelos de raciocínio |
| Desligar raciocínio | Específico do servidor: `chat_template_kwargs.enable_thinking=false` (vLLM/SGLang); `reasoning_effort` pode ser **ignorado** em silêncio |
| Tool-calling | Varia muito por modelo e servidor — não presuma |
| Campo desconhecido no corpo | Uns ignoram, outros respondem `400` |

## Antes de tudo: a checagem de egresso

Um endpoint pode estar na sua rede e **ainda assim encaminhar tudo para a nuvem pública**.
O destino efetivo dos dados não é o host do proxy.

1. **Para onde este endpoint roteia?** Modelos hospedados internamente, endpoint corporativo,
   nuvem pública?
2. **Os backends têm opt-out de retenção e treino?** Sem isso, nada sob NDA sai (§0 e §7 do
   `AGENTS.md`).
3. **Quem opera o endpoint declarou a classe?** No `agentry`, isso é o `egressClass`; fora
   dele, é uma resposta explícita de quem opera.

Qualquer "não sei" ⇒ **não delegue material do cliente**. Delegue só o que já seria público,
ou use a rota local da skill `delegacao-a-subagentes`.

## Configuração — só pelo ambiente

| Variável | Papel | Fallback |
|---|---|---|
| `OPENAI_COMPAT_BASE_URL` | Endpoint, com ou sem `/v1` | `AGENTRY_LITELLM_BASE_URL` |
| `OPENAI_COMPAT_API_KEY` | Chave; vazia = endpoint sem autenticação | `AGENTRY_LITELLM_API_KEY` |
| `OPENAI_COMPAT_KEY_CMD` | Comando que imprime a chave (`op read …`, `pass …`) | — |
| `OPENAI_COMPAT_KEY_FILE` | Arquivo cuja 1ª linha é a chave | — |
| `OPENAI_COMPAT_MODEL` | Modelo padrão; sem ele, `-m` é obrigatório | — |

⚠️ **Não use `OPENAI_BASE_URL`/`OPENAI_API_KEY` para isto.** São lidas automaticamente pelos
SDKs oficiais e por várias CLIs. Exportá-las globalmente apontando para um gateway interno
redireciona em silêncio **outras** ferramentas da máquina — inclusive com a chave errada.

A chave segue a `secrets-guard` e a exceção de segredo da ADR 0011: fora do repositório,
nunca em argumento de comando, nunca ecoada. Para conferir presença sem expor:

```bash
python3 -c "import os; v=os.environ.get('OPENAI_COMPAT_API_KEY') or os.environ.get('AGENTRY_LITELLM_API_KEY'); print('chave:', 'presente' if v else 'ausente')"
```

Não confira com `grep "$CHAVE"` nem `echo`: o próprio comando de checagem põe a chave em
argv, visível em `ps`.

## Rota 1 — `oa-chat`: chamada direta, modo texto

O conteúdo vai embutido no pedido. Determinístico, não depende de o modelo saber usar
ferramentas. É a rota padrão para revisar diff, resumir documento, classificar lista,
extrair campos.

```bash
OA=skills/delegacao-openai-compat/scripts/oa-chat
$OA --models                                    # o que o endpoint serve
$OA -m <modelo> --no-think -f pedido.md > resposta.md 2> uso.txt
git diff --staged | $OA -m <modelo> --no-think -s "Voce e revisor." - > revisao.md 2> uso.txt
$OA -m <modelo> --no-think --json -f extrair.md > saida.json 2> uso.txt
```

O script garante o que é fácil esquecer: chave e corpo **fora de argv**; URL normalizada;
resposta não-JSON e erro HTTP viram mensagem legível; uso de tokens em `stderr`
(`[uso] modelo=… entrada=… saida=… fim=…`).

| Código | Significado |
|---|---|
| `0` | Resposta completa |
| `1` | Erro do endpoint ou de conexão |
| `2` | Uso ou configuração |
| `3` | Resposta vazia ou truncada (`finish_reason=length`) |

### Modelos de raciocínio: o gasto invisível

Modelos de raciocínio consomem `max_tokens` **pensando antes de responder**. Com orçamento
curto, a resposta vem **vazia** com `finish_reason=length` — e um script ingênuo imprime
`null` como se fosse resultado. O `oa-chat` sai com código `3` nesse caso.

Medido num GLM servido por vLLM atrás de LiteLLM, pedindo um parágrafo de 150 palavras:

| Configuração | Tokens de saída | Resultado |
|---|---|---|
| Padrão, `max_tokens=3000` | 3000 | **Vazio** — tudo gasto raciocinando |
| `reasoning_effort=low` | 3000 | **Vazio** — parâmetro ignorado em silêncio |
| `--no-think` | ~230 | Parágrafo completo |

Para trabalho mecânico — extração, classificação, formatação — **use `--no-think`**. A
economia é de uma ordem de grandeza. Reserve o raciocínio para quando a tarefa exige
inferência e a verificação barata não basta.

## Rota 2 — `agentry`: modo agente

Para varredura ampla, onde o modelo precisa ler arquivos sozinho. Depende de tool-calling
decente **e** das `permissions` do perfil — sob `externo-confidencial`, `readAllow` é lista
fechada, então **diga o caminho exato**.

O `agentry` fala com o endpoint pelo adapter OpenAI-compatible, configurado no bloco
`providers.litellm` do `.agentry/agentry.settings.json` — o nome do bloco é do schema do
`agentry`, e serve a qualquer endpoint compatível. O template anexo traz o bloco e a
task-class `delegada`:

```bash
jq -s '.[0] * .[1]' .agentry/agentry.settings.json \
   skills/delegacao-openai-compat/templates/agentry-litellm.snippet.json > /tmp/settings.novo
diff -u .agentry/agentry.settings.json /tmp/settings.novo   # revise antes de mover

bash skills/delegacao-openai-compat/scripts/com-chave-litellm.sh --check
bash skills/delegacao-openai-compat/scripts/com-chave-litellm.sh \
  agentry --task-class delegada 'Leia docs/x.md com fs_read. Responda SOMENTE ...'
```

Três armadilhas silenciosas do `agentry`: sem `baseUrl` **e** `model`, o provider não é
registrado; sem `egressClass`, o candidato nunca resolve sob perfil restritivo e a chamada
cai no Ollama sem erro; variável `AGENTRY_LITELLM_API_KEY` **definida e vazia** vence o
`credentials.json` e o gateway responde `401`.

## O pedido: autocontido ou inútil

O endpoint não vê esta conversa. Os quatro elementos da `delegacao-a-subagentes` valem —
tarefa concreta, formato exato da resposta, proibição campo a campo, sentinela de falha —
com três agravantes:

1. **Contexto zero.** Todo caminho, nome e critério vai no pedido.
2. **Você paga por token de entrada.** O arquivo certo, não a pasta; o diff, não o histórico.
3. **O modelo é desconhecido.** Peça o formato mais simples que resolva, e prefira lotes
   pequenos: dez itens por chamada erram menos que cem.

## O padrão que não enche o contexto

1. **`stdout` para arquivo.** Nunca deixe a resposta inteira voltar para a conversa.
2. **Leia um recorte.** `jq`, `grep`, contagem — o arquivo inteiro só se preciso.
3. **`stderr` para `uso.txt`.** O custo fica registrado sem sujar a resposta.

## O que delegar — e o que não

| Delegue | Não delegue |
|---|---|
| Extração estruturada a partir de entrada volumosa | Decisão de arquitetura, trade-off, escolha de dependência |
| Primeira passada de revisão de diff | O que custa mais verificar do que fazer |
| Resumo de log, transcrição, documento longo | Escrita direta em arquivo do projeto sem revisão |
| Normalização e tradução de formato em lote | Material sob NDA para endpoint sem egresso declarado |
| Rascunho de proposta que você vai revisar | Tarefa encadeada de muitos passos |

## O que você vai pensar para pular a verificação

| O que você vai pensar | Por que não vale |
|---|---|
| "O formato veio certo, o conteúdo deve estar" | Formato certo é pré-condição, não evidência. Amostre. |
| "São 135 itens, conferir três não prova nada" | Prova o suficiente para **descartar** o lote. Um erro na amostra derruba tudo. |
| "É o modelo grande do gateway, é confiável" | Você não mediu. E `--no-think` muda o comportamento dele. |
| "Veio `SEM_ACHADOS`, então não há achados" | Pode ser `401`, candidato indisponível ou pedido mal entendido. Sentinela inesperada é falha de rota. |
| "Refazer aqui custaria a cota que eu queria poupar" | Usar resultado errado custa mais que a cota. |

## Verificação obrigatória

- **Formato primeiro.** Resposta fora do formato pedido ⇒ o modelo não entendeu; o conteúdo
  provavelmente também está errado.
- **Amostre na fonte.** Dois ou três itens conferidos contra o original; um errado derruba o
  lote.
- **Código `3` não é resultado.** Resposta vazia ou truncada volta com mais `--max-tokens` ou
  `--no-think` — nunca entra como dado.
- **Registre o custo** (`uso.txt`) quando estiver comparando rotas.

## Definição de pronto da skill

- [ ] O egresso do endpoint era conhecido **antes** do envio, e compatível com o material.
- [ ] Endereço, chave e modelo vieram do ambiente — nada escrito no repositório.
- [ ] A chave nunca apareceu em argv, em saída nem em comando de checagem.
- [ ] O pedido era autocontido: caminhos, formato, proibições, sentinela.
- [ ] A saída foi para arquivo; só um recorte entrou no contexto.
- [ ] Nenhuma resposta com código `3` foi usada como resultado.
- [ ] O resultado foi amostrado na fonte antes de ser usado.
