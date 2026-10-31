---
name: mapa-de-arquitetura
description: >-
  Gera e mantém docs/architecture.md com a arquitetura que efetivamente existe
  no código — componentes, responsabilidades, dependências e diagrama Mermaid —
  derivada das dependências reais entre módulos, sem encaixá-la num padrão
  arquitetural. Aciona ao adotar o framework num projeto existente, quando
  docs/architecture.md não existe ou está desatualizado, quando uma mudança
  altera dependências entre módulos, ou antes de planejar refatoração ou feature
  que atravessa vários módulos.
---

# mapa-de-arquitetura — a arquitetura que existe, não a que deveria existir

O agente que não conhece a estrutura do projeto a deduz dos nomes das pastas — e erra com
confiança. O que ele precisa é de um mapa curto, verificável e **derivado do código**, lido
antes de mexer em mais de um módulo.

## A regra

> **Todo projeto tem `docs/architecture.md` descrevendo a arquitetura efetiva.**
> Mudou a dependência entre módulos, o mapa muda **no mesmo trabalho**.

## O pedido que gera o mapa

> "Analise as dependências entre os principais módulos e identifique a arquitetura que
> efetivamente existe no código. Não tente encaixá-la artificialmente em um padrão
> arquitetural. Gere `docs/architecture.md` contendo componentes, responsabilidades,
> dependências e um diagrama Mermaid."

A frase do meio é a que importa. Deixado livre, o modelo reconhece um padrão parecido
("isto é MVC", "isto é hexagonal") e passa a descrever o padrão em vez do código: inventa
camadas que não existem e esconde as dependências que o padrão proibiria — justamente as
que mais interessam.

## Procedimento

**1. Extraia as dependências por comando, não por leitura de nomes.** O nome da pasta diz a
intenção; o `import` diz o que acontece. Use a ferramenta da linguagem quando houver, e
`grep` quando não:

| Linguagem | Caminho barato |
|---|---|
| Python | `grep -rhoE '^(from|import) [a-z_.]+' <pacote>` · `pydeps` |
| Rust | `grep -rhoE 'use (crate|super)::[a-z_]+' src` · `cargo modules` · `Cargo.toml` do workspace |
| JS/TS | `grep -rhoE "from '\.[^']+'" src` · `madge --json` |
| C/C++ | `grep -rhoE '#include "[^"]+"'` · `CMakeLists.txt` (`target_link_libraries`) |
| Go / Java | `go list -deps` · imports por pacote |

Agregue no nível de **módulo** (pacote, crate, diretório de primeiro ou segundo nível), não de
arquivo. Mapa de arquivos é inventário; ninguém o lê.

**2. Escolha os componentes pelo grafo, não pelo organograma.** Um componente é um conjunto
com fronteira observável: muitos imports internos, poucos para fora. Diretório que ninguém
importa pode ser ponto de entrada, script ou código morto — diga qual.

**3. Escreva a responsabilidade em uma frase, pelo que o código faz.** Leia os pontos de entrada
e as funções públicas; não copie o README, que descreve a intenção de quando foi escrito.

**4. Registre o que não encaixa.** Ciclos, dependência "de baixo para cima", módulo que todos
importam, código sem dono. Essa seção é a mais útil do documento: é onde o próximo agente
vai tropeçar. **Registre, não corrija** — refatorar é outra decisão (`adr-writer`,
`micro-ticket-planner`).

**5. Desenhe o Mermaid a partir da tabela de dependências.** Toda aresta do diagrama existe na
tabela; nenhuma aresta da tabela some do diagrama sem nota. Até ~15 nós; acima disso, um
diagrama de visão geral e um por componente grande.

**6. Verifique antes de entregar.** Sorteie três arestas e confirme cada uma com o comando do
passo 1; procure um import entre módulos que o mapa não mostre. Aresta sem evidência sai.
Para projeto grande, passe o mapa e a saída do comando a um crítico em contexto limpo
(`critico-independente`).

## Formato

Use `templates/architecture.template.md`. Seções fixas: visão geral (três a cinco linhas),
componentes (tabela: componente · caminho · responsabilidade), dependências (tabela:
de → para · natureza · evidência), diagrama, desvios e pontos de atenção, e como o mapa foi
gerado (comando e data). Sem nome de padrão no título, a menos que o código o siga de fato —
e aí com a evidência.

O documento é para leitura rápida: **uma a duas telas**. Detalhe de componente grande vai em
documento próprio, linkado da tabela.

## Quando atualizar

- Mudança que cria, remove ou renomeia módulo, ou que acrescenta dependência entre módulos:
  atualize no mesmo commit ou PR. O `pr-review-guard` confere.
- Ao adotar o framework num projeto existente: é o primeiro documento a gerar, antes de
  qualquer mudança de código (`novo-projeto`).
- Projeto novo: gere quando existir o segundo módulo. Antes disso não há o que mapear.

## O que você vai pensar para pular o mapa

| O que você vai pensar | Por que não vale |
|---|---|
| "A estrutura de pastas já diz tudo" | Diz a intenção. O mapa existe para mostrar onde o código diverge dela. |
| "Isso é claramente arquitetura em camadas" | Talvez. Mostre as arestas; se alguma sobe de camada, não é — e essa aresta é a informação. |
| "Atualizo o mapa no fim da sprint" | Mapa desatualizado é lido como verdade e induz o erro que deveria evitar. |
| "O projeto é pequeno demais" | Então o mapa tem cinco linhas e custa dois minutos. |

## Ligação com o resto do acervo

`novo-projeto` gera o mapa na adoção · `adr-writer` registra a decisão de mudar a arquitetura;
o mapa registra o que existe · `spec-como-contrato` e `micro-ticket-planner` consultam o mapa
para fatiar trabalho que atravessa módulos · `pr-review-guard` cobra a atualização.

## Definição de pronto da skill

- [ ] `docs/architecture.md` existe e segue o template.
- [ ] As dependências vieram de comando (registrado no documento), não da leitura de nomes.
- [ ] Toda aresta do diagrama está na tabela de dependências, com evidência.
- [ ] Nenhum padrão arquitetural foi atribuído sem evidência no código.
- [ ] Ciclos, desvios e módulos sem dono estão registrados — não corrigidos em silêncio.
- [ ] Três arestas sorteadas foram conferidas por comando antes da entrega.
- [ ] O documento cabe em uma a duas telas.
