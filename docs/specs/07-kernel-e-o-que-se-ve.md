<!-- Caminho relativo: docs/specs/07-kernel-e-o-que-se-ve.md -->

# 07 — O kernel é o que se vê

- **Estado:** Aprovada; mecanismo no [ADR 0008](../adr/0008-tela-desenhada-a-partir-do-kernel.md)
- **Fase:** KV · **ADRs:** 0001, 0003, 0007, 0008 · **Transversais:** 00 (T4, N1–N5)

## Problema e resultado esperado

Hoje há dois modelos vivos. O canvas é desenhado pelo upstream a partir do modelo dele;
os comandos da barra do upstream editam esse modelo; e salvar grava o modelo do kernel,
que não recebe essas edições (review de 2026-10-29, achados A4 e C4). Na direção
contrária, o que o kernel cria não aparece na tela.

Ao fim de KV: **o que o kernel contém é o que aparece, o que o usuário faz passa pelo
kernel, e o que se salva é o que se vê** — para tudo o que o kernel modela. Entidade de
arquivo que o kernel ainda não modela continua exibida pelo upstream, não é editável e
é reportada como perda ao salvar (00, T4) até a fase KP modelá-la.

## Escopo

O caminho kernel → tela para todas as entidades que o kernel modela, nos dois espaços;
o fim de toda edição que altere só o modelo do upstream — a barra de comandos do upstream
deixa de ser oferecida (spec 08); o spike que mede as alternativas
e o ADR que escolhe uma.

## Fora de escopo

Linha de comando própria (spec 08). Renderização própria completa, com seleção fina e
alças (K5), salvo se o ADR de KV decidir antecipá-la. Entidades que o kernel ainda não
modela: continuam exibidas pelo upstream na leitura e reportadas como perda (T4).

## Restrições e decisões já tomadas

- ADR 0001: componentes não falam com kernel nem upstream diretamente.
- ADR 0007: a fase de UI/UX não cria edição pela interface antes de KV.
- ADR 0008: a tela é desenhada a partir do kernel, pela GPU (WebGL2) com reserva em
  canvas 2D; o kernel prepara a geometria para a tela e o TypeScript só a repassa; o
  upstream fica só com a leitura e o desenho do que o kernel não modela.

## Critérios de aceite (EARS)

| id  | critério                                                                                                                                                                                 | prova                                                                                                                                      |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| V1  | QUANDO um desenho é aberto, a tela DEVE exibir as entidades do kernel, com a mesma contagem por tipo e por camada, no espaço-modelo e em cada layout.                                    | `pnpm test:e2e -- kernel-document` (estender)                                                                                              |
| V2  | QUANDO o kernel cria, altera ou remove uma entidade, a tela DEVE refletir a mudança sem reabrir o arquivo.                                                                               | `pnpm test:e2e -- kernel-view` (a criar): criar, mover e apagar uma linha pelo serviço e conferir a contagem e a caixa envolvente exibidas |
| V3  | QUANDO o usuário desfaz ou refaz pelo menu `Editar`, a tela DEVE refletir o estado do kernel.                                                                                            | `pnpm test:e2e -- kernel-view` (a criar)                                                                                                   |
| V4  | QUANDO uma linha é desenhada por comando e o arquivo é salvo e reaberto, a linha DEVE estar no arquivo.                                                                                  | `pnpm test:e2e -- kernel-view` — falha hoje (achado C4) e fica `test.fail()` até KV fechar                                                 |
| V5  | Todo comando que edite o desenho DEVE fazê-lo pelo kernel; comando de edição que não passe pelo kernel NÃO DEVE aparecer no catálogo de comandos.                                        | `pnpm test -- cad-command-catalog`                                                                                                         |
| V5b | SE o kernel recusa uma edição pedida por comando, ENTÃO a tela NÃO DEVE mudar e uma mensagem em português, com o motivo da recusa, DEVE aparecer na área de mensagens.                   | `pnpm test:e2e -- kernel-view` (a criar)                                                                                                   |
| V6  | SE o caminho de exibição recusa uma entidade do kernel, ENTÃO a área de mensagens DEVE informar o tipo e a quantidade das entidades não exibidas; V1 vale para todas as demais.          | teste de unidade do adaptador de exibição com entidade recusada simulada, conferindo a mensagem emitida (a criar)                          |
| V7  | QUANDO o usuário pede uma edição sobre entidade que o kernel não modela, a edição DEVE ser recusada com a mensagem "entidade ainda não editável" e o tipo dela, sem alterar o documento. | `pnpm test:e2e -- kernel-view` com fixture `with-unsupported.dxf` (a criar)                                                                |

## Requisitos não-funcionais

Em EARS, com prova, na [spec 00](00-transversal.md) (N1–N5): abrir 100 mil entidades em
até 5 s, refletir uma edição em até 100 ms, pan e zoom a 30 quadros por segundo pela GPU,
reserva em canvas 2D e funcionamento em AMD, NVIDIA e Intel.

## Questões em aberto

Nenhuma. O corpus dos não-funcionais é o desenho sintético de
`scripts/gerar-dxf-sintetico.mjs` (MT-KV-02).
