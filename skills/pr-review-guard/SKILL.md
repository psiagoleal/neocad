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

## Checklist antes de abrir/aprovar o PR

### Correção e robustez

- [ ] Compila/builda sem erros nem _warnings_ novos.
- [ ] Linter e checagem de tipos passam (comandos exatos do `AGENTS.md`).
- [ ] Suíte de testes passa, incluindo testes **novos** para o comportamento alterado.
- [ ] Tratamento de exceções presente — sem `except:`/`catch{}` vazios mascarando erros.

### Regressão

- [ ] Funcionalidades adjacentes testadas continuam passando (não só o bug-alvo).
- [ ] _Diff_ não remove validações, _guards_ ou testes existentes "para fazer passar".

### Segurança (OWASP Top 10 / LLM)

- [ ] Sem segredos no diff (ver skill `secrets-guard`); varredura `gitleaks`/`detect-secrets` limpa.
- [ ] Entradas validadas; sem injeção (SQL/cmd/path); _prepared statements_ em consultas.
- [ ] Mudanças sensíveis revisadas com atenção redobrada: `.github/workflows/`, scripts de
      _bootstrap_, configs de CI/CD, o próprio `AGENTS.md` (vetor de injeção indireta).
- [ ] SAST/SCA executados em CI **antes** da revisão humana — não no lugar dela.

### Proveniência e auditoria

- [ ] _Trailer_ de commit ou metadado de PR registra modelo/versão/prompt do trecho gerado.
- [ ] SBOM (CycloneDX/SPDX) gerado/atualizado quando aplicável ao perfil.

## Saída esperada

Produza um **resumo de revisão** com: itens do checklist marcados, riscos residuais e uma
recomendação explícita (`aprovar` / `aprovar com ressalvas` / `bloquear`). Finalize sempre
com: _"Requer validação humana antes do merge."_

## Definição de pronto da skill

- [ ] Checklist percorrido e marcado.
- [ ] Resumo de revisão emitido com recomendação.
- [ ] Validação humana explicitamente requerida.
