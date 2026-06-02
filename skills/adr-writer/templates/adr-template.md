<!-- Caminho relativo: docs/adr/NNNN-titulo-em-kebab-case.md -->

# ADR NNNN: <título da decisão em uma linha>

- **Status:** Proposed <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** AAAA-MM-DD
- **Decisores:** <nomes ou papéis>
- **Tags:** <ex.: solver, dados, segurança>

## Contexto

Descreva as forças em jogo: requisitos técnicos, restrições de orçamento, hardware,
regulação aplicável, prazos. Explique **por que uma decisão precisa ser tomada agora**.

## Decisão

Em voz ativa, declare o que foi decidido. Exemplo:

> Fica acordada a utilização obrigatória da biblioteca de código aberto **PyPSA** para a
> formulação dos modelos de rede; a resolução usará o solver livre **HiGHS** via Linopy
> em todas as execuções locais e de CI/CD.

## Consequências

- **Impacto positivo:** <ex.: zero custo de licenciamento; reprodutibilidade em qualquer hardware>
- **Impacto negativo:** <ex.: limite de convergência acima de 10.000 barras nodais>
- **Trade-offs aceitos:** <...>

## Diretriz de Conformidade de Código

Liste o que o agente está **expressamente proibido** de fazer, e o que **deve** fazer:

- Proibido: <ex.: introduzir scripts baseados em Pyomo, MATLAB ou solvers proprietários (Gurobi/CPLEX)>.
- Obrigatório: <ex.: toda otimização passa por HiGHS/Linopy>.

> Qualquer tentativa de desvio desta regra viola as diretrizes de conformidade
> arquitetural do projeto e deve ser reportada ao operador humano antes de prosseguir.
