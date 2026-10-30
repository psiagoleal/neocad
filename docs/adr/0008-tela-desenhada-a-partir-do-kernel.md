<!-- Caminho relativo: docs/adr/0008-tela-desenhada-a-partir-do-kernel.md -->

# ADR 0008: A tela é desenhada a partir do kernel

- **Status:** Proposed <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-10-30
- **Decisores:** Iago Leal
- **Tags:** kernel, renderização, upstream, desempenho, fase KV

## Contexto

Hoje convivem dois modelos. O upstream lê o arquivo, desenha e oferece uma barra de
comandos que edita só o modelo dele; salvar grava o modelo do kernel. Uma linha desenhada
pela barra aparece na tela e não chega ao arquivo (MT-KV-01, achado C4). A
[spec 07](../specs/07-kernel-e-o-que-se-ve.md) exige que o que o kernel contém seja o que
aparece, e deixou a escolha do mecanismo para depois de medir as duas opções num desenho
sintético de 100 mil entidades (metas: refletir uma edição em até 100 ms, abrir em até
5 s).

| Medida, 100 mil entidades | Espelhar no upstream ([MT-KV-03](../spikes/kv-espelhamento.md)) | Desenhar do kernel ([MT-KV-04](../spikes/kv-render-kernel.md)) |
| ------------------------- | --------------------------------------------------------------: | -------------------------------------------------------------: |
| Abrir, até a tela         |                                                           8,9 s |                                                         ~2,8 s |
| Refletir uma edição       |                                                          ~2,2 s |                                                     115–122 ms |
| Ressincronizar tudo       |                                             inviável (> 13 min) |                                   0,64 s (refazer os caminhos) |
| Pan, por quadro           |                                                      não medido |                                                     153–196 ms |

O espelhamento custa 2,2 s por edição dentro do upstream, sem relação com o tamanho da
edição, e não tem como ressincronizar. Além disso, mantém os dois modelos vivos, que é o
defeito a eliminar. O desenho a partir do kernel ficou 15 a 22 ms acima da meta de edição, sem nenhuma otimização, e sem medir a ida ao kernel para buscar a entidade editada (o protótipo a montou do lado JavaScript). O pan ficou lento, e o protótipo não tem a fidelidade visual do
upstream.

O [ADR 0003](0003-kernel-cad-proprio.md) previa a renderização própria só em K5, "quando
a de terceiros passar a limitar a interação". Ela já limita: não há caminho barato do
kernel até a tela do upstream.

## Decisão

Fica acordado que **a tela passa a ser desenhada a partir do kernel** para toda entidade
que o kernel modela, nos dois espaços. Isso antecipa a parte de K5 que cabe em KV.

1. Um **desenhista próprio** recebe as entidades do kernel pelo serviço `cad-document` e
   desenha num canvas próprio. A tecnologia (canvas 2D com recorte pela vista, ou WebGL
   sobre o three.js que o upstream já traz) é escolhida no primeiro ticket de
   implementação. O critério é passar nas metas abaixo.
2. O kernel passa a **informar o que cada operação mudou**: ids criados, alterados e
   removidos. A fachada também ganha a consulta de entidades pelo id. Sem isso, a tela só
   saberia o que mudou listando tudo (355 ms com 100 mil).
3. O **upstream fica com três papéis** até a fase KP e K6 os absorverem:
   - ler DWG;
   - ler as entidades que o kernel ainda não modela;
   - desenhar essas entidades numa camada abaixo da do kernel, sem edição.

   A barra de comandos e todo comando de edição do upstream deixam de ser oferecidos
   (spec 08).

4. As metas da spec 07 viram critério EARS na spec 00, com prova por medida no desenho
   sintético:
   - abrir 100 mil entidades em até 5 s;
   - refletir a edição de uma entidade em até 100 ms;
   - **pan e zoom a pelo menos 30 quadros por segundo**. Esta meta não está na spec 07: é proposta aqui porque o spike mediu 5 a 6 quadros por segundo e mostrou que é o ponto fraco.

## Consequências

**Positivas**

- Um modelo só para o que o kernel modela: salvar grava o que se vê, e V4 da spec 07
  passa a valer.
- A edição fica a cerca de 20 ms da meta já no protótipo. Recortar pela vista e desenhar só a
  região alterada ainda têm folga para ganhar.
- K5 (seleção fina, alças, prévia dinâmica) passa a ter onde se apoiar.

**Negativas**

- Paridade visual com o upstream nas entidades modeladas é trabalho novo. A estimativa
  grosseira do spike é de duas a quatro semanas, com o texto fiel e o pan fluido como
  itens mais caros.
- Durante a fase KP, há dois desenhos sobrepostos (o do kernel e o do upstream para o
  não modelado), que precisam concordar em vista, zoom e ordem. O leitor do upstream não separa as entidades de espaço-papel ao abrir; uma entidade não modelada num layout pode aparecer no lugar errado até o kernel modelá-la.
- A meta de edição depende de a consulta pelo id ser barata, o que o spike não mediu.
- As medidas são de uma máquina só (Ryzen 5 3500X, Chrome 156). A prova de desempenho
  roda localmente, não na CI, cujas máquinas variam.

## Diretriz de Conformidade de Código

- **Proibido** editar o modelo do upstream, por comando ou por código, a partir do fim de
  KV.
- **Proibido** exibir uma entidade que o kernel modela a partir do modelo do upstream.
- **Obrigatório** que o desenhista receba as entidades só pelo serviço `cad-document`
  (ADR 0001); ele não importa o kernel nem o upstream.
- **Obrigatório** que toda operação de edição da fachada informe os ids criados,
  alterados e removidos.
- **Obrigatório** que mudança no desenhista que afete desempenho rode a prova local de
  desempenho no desenho sintético de 100 mil entidades, com o resultado citado no PR.
