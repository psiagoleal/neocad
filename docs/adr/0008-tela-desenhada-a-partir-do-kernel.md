<!-- Caminho relativo: docs/adr/0008-tela-desenhada-a-partir-do-kernel.md -->

# ADR 0008: A tela é desenhada a partir do kernel

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-10-30
- **Decisores:** Iago Leal
- **Tags:** kernel, renderização, GPU, upstream, desempenho, fase KV

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

1. Um **desenhista próprio** desenha num canvas próprio, **pela GPU, com WebGL2**, e cai
   sozinho num **caminho de reserva em canvas 2D** quando o WebGL2 não está disponível
   (máquina virtual, driver bloqueado pelo navegador, CI sem GPU). A independência de
   fabricante (AMD, NVIDIA, Intel) vem da camada do navegador: no Windows a WebView2
   traduz para Direct3D 11; no Linux o WebKitGTK passa pelo ANGLE sobre OpenGL. WebGPU
   fica para depois, quando estiver garantido no WebKitGTK.
2. **O cálculo fica no kernel; o TypeScript só repassa.** O kernel prepara o que vai para
   a tela: transforma arcos, círculos e polilinhas em segmentos, posiciona o texto, mantém
   o índice espacial para recortar pela vista e para a seleção, e entrega o resultado em
   blocos contínuos de números (`Float32Array`), prontos para a GPU. O lado TypeScript
   recebe esses blocos pelo serviço `cad-document`, envia-os à GPU e, a cada pan ou zoom,
   envia só a matriz da vista. No spike, o JavaScript gastou ~480 ms listando entidades e
   montando os caminhos com 100 mil; é esse custo que sai dele. A camada que fala com a
   GPU é escolhida no primeiro ticket de implementação, por medida no desenho de 100 mil
   entidades, entre **WebGL2 direto em TypeScript** e **`wgpu` no próprio Rust** (que
   passaria a WebGPU sem reescrita, ao custo de um WASM maior e do texto também em Rust).
   O three.js não é opção: só está no projeto por causa do upstream, que sai em K6, e
   traz uma cena de objetos de que o desenhista não precisa.
3. O kernel passa a **informar o que cada operação mudou**: ids criados, alterados e
   removidos. A fachada também ganha a consulta de entidades pelo id. Sem isso, a tela só
   saberia o que mudou listando tudo (355 ms com 100 mil).
4. O **upstream fica com três papéis** até a fase KP e K6 os absorverem:
   - ler DWG;
   - ler as entidades que o kernel ainda não modela;
   - desenhar essas entidades numa camada abaixo da do kernel, sem edição.

   A barra de comandos e todo comando de edição do upstream deixam de ser oferecidos
   (spec 08).

5. As metas da spec 07 viram critério EARS na spec 00, com prova por medida no desenho
   sintético:
   - abrir 100 mil entidades em até 5 s;
   - refletir a edição de uma entidade em até 100 ms;
   - **pan e zoom a pelo menos 30 quadros por segundo**. Esta meta não está na spec 07:
     entra aqui porque o spike mediu 5 a 6 quadros por segundo e mostrou que é o ponto
     fraco. É o que a GPU resolve: a geometria vai à placa uma vez, e o quadro só aplica a
     matriz da vista;
   - no caminho de reserva em canvas 2D, abrir e editar seguem as mesmas metas; o pan não
     tem meta, só a exigência de funcionar.

## Consequências

**Positivas**

- Um modelo só para o que o kernel modela: salvar grava o que se vê, e V4 da spec 07
  passa a valer.
- A edição fica a cerca de 20 ms da meta já no protótipo. Recortar pela vista e desenhar só a
  região alterada ainda têm folga para ganhar.
- K5 (seleção fina, alças, prévia dinâmica) passa a ter onde se apoiar.
- O pan deixa de depender do número de entidades no lado da CPU.
- O preparo para a tela em Rust fica testável com `cargo test`, sem navegador.

**Negativas**

- Paridade visual com o upstream nas entidades modeladas é trabalho novo. A estimativa
  grosseira do spike é de duas a quatro semanas, com o texto fiel e o pan fluido como
  itens mais caros.
- Durante a fase KP, há dois desenhos sobrepostos (o do kernel e o do upstream para o
  não modelado), que precisam concordar em vista, zoom e ordem. O leitor do upstream não separa as entidades de espaço-papel ao abrir; uma entidade não modelada num layout pode aparecer no lugar errado até o kernel modelá-la.
- A meta de edição depende de a consulta pelo id ser barata, o que o spike não mediu.
- As medidas são de uma máquina só (Ryzen 5 3500X, Chrome 156, NVIDIA RTX 3070). A prova
  de desempenho roda localmente, não na CI, cujas máquinas variam.
- Dois caminhos de desenho para manter (GPU e reserva), que precisam concordar.
- A GPU soma, pela estimativa, uma a duas semanas às duas a quatro da paridade visual:
  texto e espessura de linha são os itens caros também nela.
- AMD e Intel precisam de máquinas além da do mantenedor para a verificação. A NVIDIA com
  WebKitGTK tem histórico de tela em branco no Tauri no Linux e exige teste próprio.

## Diretriz de Conformidade de Código

- **Proibido** editar o modelo do upstream, por comando ou por código, a partir do fim de
  KV.
- **Proibido** exibir uma entidade que o kernel modela a partir do modelo do upstream.
- **Obrigatório** que o desenhista receba as entidades só pelo serviço `cad-document`
  (ADR 0001); ele não importa o kernel nem o upstream.
- **Proibido** calcular geometria no lado TypeScript do desenhista (segmentação de curvas,
  posição de texto, recorte, índice espacial, seleção por ponto ou janela): isso é do
  kernel. O TypeScript só repassa blocos e a matriz da vista.
- **Proibido** usar API gráfica de fabricante (CUDA, extensões de um só fabricante) ou
  dependência que funcione só em uma marca de GPU.
- **Obrigatório** o caminho de reserva em canvas 2D, escolhido sozinho quando o WebGL2
  falta, e coberto pelos mesmos testes de V1 e V2 da spec 07.
- **Obrigatório** que toda operação de edição da fachada informe os ids criados,
  alterados e removidos.
- **Obrigatório** que mudança no desenhista que afete desempenho rode a prova local de
  desempenho no desenho sintético de 100 mil entidades, com o resultado citado no PR.
