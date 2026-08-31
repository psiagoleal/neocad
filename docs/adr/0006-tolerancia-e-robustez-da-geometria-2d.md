<!-- Caminho relativo: docs/adr/0006-tolerancia-e-robustez-da-geometria-2d.md -->

# ADR 0006: Modelo de tolerância e robustez numérica da geometria 2D

- **Status:** Proposed <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-08-31
- **Decisores:** Iago Leal
- **Tags:** kernel, geometria, tolerância, robustez, k3

## Contexto

Até aqui o kernel **guardou e transportou** geometria; nunca precisou calcular
com ela. Ler um DXF, montar o documento e regravá-lo não exige decidir se dois
pontos são o mesmo ponto. A fase K3 é a primeira que calcula: interseção, aparo,
extensão, paralela, concordância e snap são todas, no fundo, a mesma pergunta —
**quando duas coisas se tocam?**

O código de produção hoje não responde a essa pergunta em lugar nenhum. Os
únicos epsilons que existem estão em módulos de teste, e já discordam entre si:
`1e-12` em `neocad-geometry/src/point.rs`, `1e-9` em `neocad-model/src/entity.rs`
e outro `1e-9` escrito à mão em `neocad-model/src/viewport.rs`. Três valores
plausíveis, nenhum justificado, escolhidos separadamente por conveniência do
teste que estava sendo escrito. É assim que a divergência começa, e ela precisa
ser interrompida antes de o primeiro algoritmo de K3 escolher o quarto valor.

### Por que a tolerância absoluta pura não basta

O caminho óbvio — uma constante única em unidades de desenho — quebra numa
situação que este projeto tem por certo encontrar: desenho em coordenadas
geográficas projetadas, com origem a milhões de unidades de distância.

O `f64` tem 52 bits de mantissa, e o espaçamento entre valores representáveis
cresce com a magnitude. Perto de `1.0` esse espaçamento é `2⁻⁵²`, cerca de
`2,2e-16`. Perto de `7 000 000` — uma coordenada UTM ordinária — ele é `2⁻³⁰`,
cerca de `9,3e-10`.

A consequência é dura: uma tolerância absoluta de `1e-9` vale **milhões de
passos representáveis** perto da origem e **pouco mais de um passo** em
coordenada UTM. No segundo caso, "coincidente dentro da tolerância" degenera em
igualdade exata, e qualquer arredondamento acumulado em duas ou três operações
encadeadas passa a responder "não se tocam" para pontos que o desenhista
construiu tocando. O sintoma no produto é conhecido de quem usa CAD: aparo que
não apara e concordância que não fecha, só em desenhos georreferenciados.

### Por que a tolerância relativa pura também não basta

Escalar a tolerância pela magnitude resolve o caso acima e cria outro: perto da
origem a tolerância encolhe até abaixo do que qualquer desenho pretende
distinguir, e o desenhista perde a capacidade de dizer "estes dois pontos são o
mesmo". Pior, a tolerância passaria a depender de **onde** o desenho está, e
transladar um desenho mudaria o resultado das operações — o que viola a
expectativa mais básica de uma ferramenta de projeto.

### Tolerância não é substituto de sinal

Há uma segunda classe de decisão, que epsilon nenhum resolve: **de que lado**.
Saber se um ponto está à esquerda ou à direita de uma reta, se uma polilinha
gira num sentido ou no outro, se uma paralela cruzou a si mesma. Aqui o que
importa é o **sinal** de um determinante, e comparar esse sinal contra um
epsilon produz respostas que se contradizem entre si: três pontos podem sair
"A à esquerda de BC", "B à esquerda de CA" e "C à esquerda de AB" ao mesmo
tempo, o que é geometricamente impossível. Algoritmos que confiam nessa
consistência entram em laço infinito ou emitem geometria inválida. É uma falha
de robustez clássica, documentada desde os anos 1990, e a resposta conhecida não
é ajustar o epsilon: é calcular o sinal de forma exata.

Decidir isso agora custa uma seção de ADR. Decidir depois custa reescrever as
operações de K3.

## Decisão

Fica acordado o seguinte modelo de tolerância e robustez para toda a geometria
do kernel, a partir da fase K3:

**1. A tolerância é absoluta com piso adaptativo, e mora num único lugar.**
`neocad-geometry` passa a expor um tipo `Tolerance` como única fonte da verdade.
A tolerância efetiva de uma comparação é

> `max(BASE, 1024 · magnitude · f64::EPSILON)`

em que `BASE` vale `1e-9` unidades de desenho e `magnitude` é a maior coordenada
absoluta envolvida na comparação. O termo constante governa perto da origem, que
é onde a intenção do desenhista é o critério; o termo adaptativo governa longe
dela, que é onde a aritmética é o critério. O fator `1024` reserva folga para
algumas centenas de operações encadeadas antes de o arredondamento acumulado
alcançar a tolerância.

**2. A tolerância angular é própria, e não derivada da linear.** Um ângulo só
vira distância quando há um raio: `1e-9` radiano é nada num raio de 1 e é meio
centímetro num raio de 5 000 000. Onde a decisão for sobre posição — tangência,
concordância, colinearidade — compara-se a **distância** resultante, nunca o
ângulo. A tolerância angular fica reservada às comparações que são genuinamente
de direção.

**3. Decisões de sinal que definem topologia são exatas.** O predicado de
orientação de três pontos é calculado com filtro de erro: avalia-se em `f64` com
um limite de erro conhecido e, quando o resultado cai dentro desse limite,
recorre-se a aritmética compensada, que dá o sinal correto sem depender de tipo
numérico exótico nem de dependência externa. Nenhuma decisão de lado, sentido ou
dentro/fora é tomada comparando um determinante contra epsilon.

**4. A tolerância é do kernel, não do usuário, nesta fase.** Um único valor,
constante, sem estado global mutável e sem exposição na interface. Quando houver
motivo para variar por documento, o `Tolerance` já é o ponto de passagem por
onde isso entra, e a mudança será localizada.

## Consequências

- **Impacto positivo:** as operações de K3 se comportam igual perto e longe da
  origem, que é a diferença entre uma ferramenta que serve para topografia e uma
  que não serve. O sinal exato elimina uma classe inteira de defeitos —
  travamento e geometria inválida — que só aparece em dados reais e é
  notoriamente difícil de reproduzir. Um único ponto de mudança substitui quatro
  epsilons dispersos.
- **Impacto negativo:** toda comparação geométrica passa a carregar a magnitude
  envolvida, o que torna as assinaturas mais verbosas do que um `==` ingênuo. O
  predicado filtrado é mais lento que o determinante direto quando cai no ramo
  exato, embora esse ramo seja raro por construção. E há custo de conversão: os
  epsilons já escritos nos testes precisam passar a usar o tipo comum.
- **Trade-offs aceitos:** o fator `1024` e a base `1e-9` são escolhas de
  engenharia, não verdades — são grandes o bastante para absorver o
  arredondamento de operações encadeadas e pequenos o bastante para ficarem
  abaixo de qualquer distância que um desenho pretenda distinguir. Ficam
  declarados como constantes nomeadas e justificadas, de modo que revisá-los seja
  uma decisão visível e não uma caçada a literais. Aceita-se também **não**
  tornar a tolerância configurável agora, ao preço de uma migração futura, em
  troca de não espalhar um parâmetro por toda a API antes de existir caso de uso.

## Diretriz de Conformidade de Código

- **Proibido** comparar grandezas geométricas com `==`, `!=` ou contra literal
  numérico escrito no local. Toda comparação passa pelo `Tolerance` de
  `neocad-geometry`.
- **Proibido** decidir lado, sentido de percurso ou dentro/fora comparando um
  determinante contra epsilon. Essas decisões usam o predicado de orientação
  exato.
- **Proibido** introduzir constante de tolerância nova em qualquer crate.
  Existe uma, em `neocad-geometry`, e as demais crates a consomem.
- **Proibido** derivar tolerância angular dividindo tolerância linear por um
  raio arbitrário, ou vice-versa.
- **Obrigatório** que toda operação geométrica nova de K3 em diante tenha teste
  em duas magnitudes: perto da origem e em coordenada da ordem de `1e6`. Um teste
  que só exercita a primeira não demonstra o comportamento que este ADR existe
  para garantir.
- **Obrigatório** que os epsilons hoje presentes nos módulos de teste de
  `neocad-geometry` e `neocad-model` sejam substituídos pelo tipo comum quando o
  arquivo correspondente for tocado.

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do
> projeto e deve ser reportado para revisão antes de prosseguir.
