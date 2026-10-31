<!-- Caminho relativo: docs/adr/0009-orcamento-de-tamanho-do-aplicativo.md -->

# ADR 0009: Orçamento de tamanho: teto do kernel e catraca do total

- **Status:** Proposed <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-10-31
- **Decisores:** Iago Leal
- **Tags:** kernel, WebAssembly, tamanho, upstream, desempenho, fase KV

## Contexto

A spec 06 (W5) limita o `.wasm` de release do kernel a 465 000 bytes, medidos em
2026-10-29 com 50% de folga, e diz que o limite só sobe por ADR. O limite existe porque
todo usuário baixa o `.wasm` a cada visita, e cálculo novo pesa.

O ADR 0008 tirou do upstream o cálculo da tela e o pôs no kernel. Depois do MT-KV-08,
que prepara a geometria da tela, o `.wasm` tem **463 233 bytes**: sobram 1 767. O texto
do kernel (MT-KV-11) e a fase KP não cabem.

O limite mede só o arquivo do kernel. Ele não enxerga o que o aplicativo inteiro baixa,
que hoje é **12 921 011 bytes** (`build/`, medido em 2026-10-31; 3,7 MB comprimidos):

| Parte                               | Origem   | Tamanho |
| ----------------------------------- | -------- | ------: |
| Leitor de DWG (libredwg, em worker) | upstream | 8,85 MB |
| Visualizador e three.js             | upstream | 2,26 MB |
| Renderizador de MTEXT (worker)      | upstream | 1,06 MB |
| Leitor de DXF (worker)              | upstream | 0,11 MB |
| Kernel (`.wasm`)                    | NeoCAD   | 0,46 MB |

Cada passo que o kernel assume tira megabytes do upstream e põe dezenas de KB no
`.wasm`. Com só o teto do kernel, o projeto vê o custo e não vê o ganho; e nada impede
que o total cresça por outro lado (dependência nova de JavaScript, worker novo).

O spike do MT-KV-09 ([relatório](../spikes/kv-camada-gpu.md)) escolheu WebGL2 em
TypeScript para desenhar: a alternativa `wgpu` levaria o `.wasm` a 2 042 252 bytes sem
ganho de desempenho medido. O crescimento previsto do kernel fica, então, na casa das
dezenas de KB por fase.

## Decisão

Fica acordado um orçamento de tamanho em dois limites, conferidos no build:

**1. O teto do `.wasm` do kernel passa de 465 000 para 600 000 bytes.** São ~130 KB de
folga para o texto do kernel (MT-KV-11, incluindo a fonte de traço) e o começo da fase
KP. Continua só subindo por ADR, e cada ADR que o suba cita a medida que o justifica.

**2. O total que o aplicativo baixa tem uma catraca.** A soma dos arquivos de `build/`
fica registrada no repositório (hoje, 12 921 011 bytes) e o build falha se o total
passar do registro. Quando uma parte do upstream sai, o registro **desce** para a nova
medida no mesmo trabalho; subir o registro exige ADR, como o teto do kernel.

**3. A medida é a do arquivo bruto.** Comprimido é o que trafega, mas depende do
servidor; bruto é o que o build vê e o que se compara entre versões. Os relatórios citam
os dois.

## Consequências

- **Positivo:** o MT-KV-10 e o MT-KV-11 cabem sem renegociar o teto a cada ticket.
- **Positivo:** o ganho de tirar o upstream fica registrado e não se perde: o total só
  desce, salvo decisão explícita.
- **Positivo:** dependência nova de JavaScript passa a ter custo visível no build.
- **Negativo:** o registro do total precisa ser atualizado à mão quando o upstream sai;
  esquecer só deixa folga, não quebra nada.
- **Negativo:** o total depende do build do SvelteKit e do Vite; mudança de versão deles
  pode mexer no número sem mudança nossa, e aí o registro é corrigido com a medida.

## Diretriz de Conformidade de Código

- **Obrigatório** que o build de release falhe com o `.wasm` do kernel acima de 600 000
  bytes.
- **Obrigatório** que o build falhe com o total de `build/` acima do registro.
- **Obrigatório** baixar o registro do total, no mesmo commit, quando uma parte do
  upstream ou uma dependência sair.
- **Proibido** subir o teto do kernel ou o registro do total sem ADR que cite a medida.
- **Proibido** contornar os limites tirando arquivos de `build/` que o aplicativo baixa
  (por exemplo, buscando-os de outro endereço em tempo de execução).
