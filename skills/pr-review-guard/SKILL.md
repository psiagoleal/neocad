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
- [ ] A mensagem de commit **não** tem nenhuma outra entrada: nem link ou ID de sessão de
      agente, nem URL de conversa, nem rodapé "Generated with…", nem *trailer* `*-Session` ou
      `Generated-by` — mesmo quando a ferramenta os acrescenta por padrão.
- [ ] `docs/TICKETS.md` reflete os tickets que este PR cria ou conclui.
- [ ] Se o PR cria, remove ou renomeia módulo, ou acrescenta dependência entre módulos,
      `docs/architecture.md` foi atualizado no mesmo PR (`mapa-de-arquitetura`).
- [ ] Nenhum achado marcado como bloqueante sem citação `arquivo:linha` de **fora** do diff.
- [ ] Nenhuma afirmação de ausência feita a partir do diff — conferida com `git grep` no ramo.
- [ ] Gravidade atribuída só depois de conferir o **alcance** (`git tag --contains`).
- [ ] Em PR que toca teste, a **asserção** foi lida, não o nome.
- [ ] Renomeação de campo de API: contagem do nome antigo no repositório par é zero.
- [ ] Onde a dúvida era de comportamento, o ramo foi **executado** (`reproduzir-branch.sh`).
- [ ] Os links de Markdown tocados pelo PR resolvem — arquivo e âncora:
      `python3 skills/pr-review-guard/scripts/checar-links.py docs README.md`
- [ ] SBOM (CycloneDX/SPDX) gerado/atualizado quando aplicável ao perfil.

## Afirmação exige verificação — e a verificação precisa ser barata

Medido em campo: ~12 PRs revisadas em duas semanas, dois repositórios. **Cinco achados
marcados como bloqueantes eram falsos**, e três lições distintas acabaram sendo a mesma
falha em roupas diferentes — **afirmar com confiança sobre o que não foi verificado**.

O antídoto que funcionou não foi "ter mais cuidado". Foi tornar a verificação barata:
enquanto reproduzir custava caro, o revisor deduzia.

### Reproduzir o ramo sem tocar no seu checkout

```bash
skills/pr-review-guard/scripts/reproduzir-branch.sh origin/<ramo> -C <repo> \
  -- .venv/bin/python -m pytest tests -q
```

`git archive` é somente leitura: não faz checkout, não mexe no índice, não exige árvore limpa.
O que ele transformou, no caso real: de "acho que quebra" para "354 passam com a correção, 3
falham sem ela" — e a segunda frase muda a decisão de merge, a primeira não.

**Revisão por leitura tem teto**, e comportamento de biblioteca fica abaixo dele. Num caso, a
leitura só produzia "a confirmar"; a execução mostrou em 30 segundos que o módulo **não
importava** e derrubava o backend inteiro.

### As cinco checagens que substituem suposição

| Afirmação que você quer fazer | O que a sustenta |
|---|---|
| "Isto é **bloqueante**" | Uma citação `arquivo:linha` **de fora do diff**. Sem ela, o achado sai como *a confirmar* — o diff não mostra o hook central, o campo calculado no outro repositório nem o comentário logo acima |
| "**Não existe** tratamento para X" | `git grep -n '<padrão>' <ref>` — na **árvore do ramo**, nunca no diff, e com as variações de nome (`loading`/`isLoading`). Ausência é a afirmação mais frágil de uma revisão |
| "O teste cobre isso" | **Leia a asserção, não o nome.** `assert cabeca` não prova "cabeça única"; nome de teste é documentação, e documentação mente |
| "Isto é **crítico**" | `git tag --contains <commit>` / `git branch -r --contains <commit>` — gravidade sem alcance é opinião. Mecanismo errado em código que nunca saiu de `develop` não é perda em produção |
| "A PR está **aprovada**" | Comparar a data da aprovação com a do último commit. Plataforma não invalida aprovação a cada push |

### Renomeação de campo atravessa repositório

Quando o PR renomeia campo de API, conte os usos do nome **antigo** no repositório par: tem
de dar zero. Dois mascaradores tornam a quebra silenciosa em vez de ruidosa — `extra="ignore"`
no modelo de entrada descarta a chave desconhecida, e `?? 0` no consumo transforma ausência em
zero. O resultado não é erro: é **dado errado gravado**.

> Sempre que uma leitura transforma falha em valor plausível, o defeito deixa de ser
> detectável. Procure `?? 0`, `or 0`, `|| ''` e `get(chave, 0)` no diff — cada um é um
> candidato a esconder exatamente o defeito que o PR introduziu.

## Links que o PR toca

Documentação com link quebrado envelhece mais rápido do que se corrige: o leitor conclui que
a referência sumiu e para de confiar no resto. `scripts/checar-links.py` confere arquivo e
âncora dos links relativos, sem ir à rede:

```bash
python3 skills/pr-review-guard/scripts/checar-links.py docs skills README.md
# repositório que guarda templates: link de template resolve no projeto instalado, não aqui
python3 skills/pr-review-guard/scripts/checar-links.py . --excluir '*/templates/*'
```

A regra de âncora é a do GitHub; num repositório publicado por outro renderizador (mdBook,
por exemplo), o resultado precisa ser lido com essa ressalva.

## Controle estrutural da mensagem de commit

A regra de proveniência é texto, e texto é probabilístico: a ferramenta pode acrescentar link
de sessão ou *trailer* por padrão, e o agente segue a ferramenta. Ative o hook, uma vez por
clone (não é versionado):

```bash
ln -sf ../../skills/pr-review-guard/scripts/checar-mensagem-commit.sh .git/hooks/commit-msg
```

Ele recusa qualquer entrada de agente além do marcador entre chaves — antes do commit existir.

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
