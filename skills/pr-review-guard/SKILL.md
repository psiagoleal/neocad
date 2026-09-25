---
name: pr-review-guard
description: >-
  Aplica um checklist de revisão para conter o "problema dos 80%": os 20%
  restantes de falhas ocultas de compilação, exceções não tratadas, regressões e
  vulnerabilidades OWASP em código gerado por IA. Aciona antes de abrir/aprovar
  um PR, antes de merge, ao revisar diff gerado por agente, ou quando o usuário
  pedir revisão de mudança.
---

# pr-review-guard — Revisão de PR e contenção do "problema dos 80%"

Agentes concluem rapidamente a maior parte de um requisito, mas deixam uma dívida de ~20%
em falhas ocultas — o que infla o tempo de revisão e introduz regressões. Mais de 75% das
soluções de agentes de mercado introduzem regressões em manutenção de longo prazo, e ~45%
das amostras de código de IA contêm vulnerabilidades do OWASP Top 10. Esta skill **não
substitui** a revisão humana — ela a prepara e a torna obrigatória.

## O corredor de provas — do barato ao caro

O checklist abaixo está em ordem de **custo crescente**, e a ordem não é estética:

> Cada gate barato que reprova economiza uma rodada inteira do gate caro — que só veria
> o mesmo defeito mais tarde, e por muito mais.

| # | Gate | Quem decide | Custo | O que só ele pega |
|---|---|---|---|---|
| 1 | Estático — build, linter, tipos | regra automatizada | milissegundos | tipo que não fecha, import quebrado, regra de estilo |
| 2 | Testes — aceite e regressão | a suíte declarada | segundos | comportamento errado em código que compila e formata bem |
| 3 | Segurança — segredo, dependência, permissão | varredura | segundos | segredo no diff, CVE conhecida, autoridade nova |
| 4 | Revisão — humana ou por agente | julgamento | uma janela inteira | desenho de API, coerência, o que nenhuma regra expressa |

**Não abra o gate 4 antes de os três primeiros passarem.** Gastar julgamento — humano ou
de agente — para descobrir um erro de tipo é o desperdício mais caro do fluxo.

⚠️ **Gate instável é pior que gate ausente.** Um portão que reprova sem motivo real
ensina a equipe a reexecutar até passar; a partir daí ele não filtra mais nada, só
adiciona latência. Gate que falha de forma intermitente é defeito a corrigir, não ruído
a tolerar.

⚠️ **Autor não é juiz do próprio trabalho.** Quando a revisão do gate 4 é feita por
agente, ele recebe o contrato e o artefato executado — nunca a justificativa de quem
implementou — e roda em contexto limpo. Um verificador que leu o raciocínio do autor
tende a validá-lo.

## Checklist antes de abrir/aprovar o PR

### Gate 1 · estático — regra automatizada, milissegundos
- [ ] Compila/builda sem erros nem *warnings* novos.
- [ ] Linter e checagem de tipos passam (comandos exatos do `AGENTS.md`).

### Gate 2 · testes — a suíte declarada, segundos
- [ ] Suíte de testes passa, incluindo testes **novos** para o comportamento alterado.
- [ ] Funcionalidades adjacentes testadas continuam passando (não só o bug-alvo).
- [ ] *Diff* não remove validações, *guards* ou testes existentes "para fazer passar".

> A suíte **é** a barra: teste fraco aprova código fraco. A força desta revisão nunca
> ultrapassa a qualidade do que a avalia.

### Gate 3 · segurança — varredura, segundos (OWASP Top 10 / LLM)
- [ ] Sem segredos no diff (ver skill `secrets-guard`); varredura `gitleaks`/`detect-secrets` limpa.
- [ ] Entradas validadas; sem injeção (SQL/cmd/path); *prepared statements* em consultas.
- [ ] Mudanças sensíveis revisadas com atenção redobrada: `.github/workflows/`, scripts de
      *bootstrap*, configs de CI/CD, o próprio `AGENTS.md` (vetor de injeção indireta).
- [ ] SAST/SCA executados em CI **antes** da revisão humana — não no lugar dela.

> Passar aqui é triagem de risco, **não** autorização: não entrega credencial nem acesso
> a produção.

### Gate 4 · julgamento — humano ou agente em contexto limpo, uma janela inteira
- [ ] Tratamento de exceções presente — sem `except:`/`catch{}` vazios mascarando erros.
- [ ] Desenho de API, nomes e coerência com o que já existe no repositório.
- [ ] A suíte de fato representa a intenção do requisito — e não só o caminho que rodou.
- [ ] Escopo do diff bate com o pedido: nada de abstração para um único caso de uso.

### Proveniência e auditoria — eixo próprio, vale em qualquer gate
- [ ] Quando houve uso de IA, a **mensagem de commit** o registra **entre chaves**
      (`{agente: <nome>; modelo: <modelo/versão>}`) — e **somente ali**.
- [ ] Nenhum outro artefato (descrição/metadado de PR, código, comentários, ADR, handoff)
      menciona uso de IA nem atribui autoria/coautoria/decisão a um agente; sem *trailers*
      `Co-authored-by`/`Assisted-by` de agente.
- [ ] SBOM (CycloneDX/SPDX) gerado/atualizado quando aplicável ao perfil.

## Saída esperada

Produza um **resumo de revisão** com: itens do checklist marcados, riscos residuais e uma
recomendação explícita (`aprovar` / `aprovar com ressalvas` / `bloquear`). Finalize sempre
com: *"Requer validação humana antes do merge."*

## Definição de pronto da skill

- [ ] Checklist percorrido e marcado, **na ordem** — gate caro só depois dos baratos.
- [ ] Todo item marcado tem evidência: comando executado nesta revisão e saída lida
      (ver `gates-de-conclusao`). Checkbox sem evidência conta como não cumprido.
- [ ] Nenhum gate instável foi contornado por reexecução; instabilidade virou defeito.
- [ ] Revisão por agente, quando houve, rodou em contexto limpo, sem a justificativa
      de quem implementou.
- [ ] Resumo de revisão emitido com recomendação.
- [ ] Validação humana explicitamente requerida.
