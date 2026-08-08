<!-- Caminho relativo: docs/adr/0002-relicenciamento-para-gpl-3.md -->

# ADR 0002: Relicenciamento do NeoCAD para GPL-3.0

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-08-06
- **Decisores:** Iago Leal
- **Tags:** licenciamento, dependências, distribuição, upstream

## Contexto

Até este ADR, o NeoCAD declarava-se licenciado sob **MIT** (`LICENSE`, badge do
`README.md`, `src-tauri/Cargo.toml`), e o pipeline `scripts/release.sh` já
publicava binários Windows no GitHub sob essa premissa.

Uma auditoria da árvore de dependências de runtime (2026-08-06, sobre o
`pnpm-lock.yaml` corrente) identificou **duas dependências sob GPL-3.0**, ambas
no caminho crítico de leitura de arquivos CAD e ambas embarcadas nos binários
distribuídos:

- **`@mlightcad/libredwg-web@0.7.1`** — build WebAssembly da
  [LibreDWG](https://www.gnu.org/software/libredwg/) (projeto GNU), alcançada por
  `@mlightcad/cad-simple-viewer -> @mlightcad/libredwg-converter`. Responsável
  pelo parsing **DWG**. O binário WASM está embutido em
  `libredwg-parser-worker.js` (~8,5 MB).
- **`@mlightcad/dxf-json@1.2.0`** — alcançada por `@mlightcad/data-model`, que é
  dependência **direta** do NeoCAD. Responsável pelo parsing **DXF**, e entra no
  bundle principal da aplicação, não apenas nos workers.

A MIT é compatível com a GPL-3.0 em apenas uma direção: código MIT pode ser
incorporado a uma obra GPL, não o inverso. Como os binários distribuídos são obra
combinada com bibliotecas GPL-3.0, o entendimento padrão é que **a distribuição
já estava sujeita à GPL-3.0** — enquanto o projeto anunciava MIT. Havia, portanto,
divergência entre a licença declarada e a licença efetiva dos artefatos
publicados.

Nenhuma das duas dependências é removível sem perder a função central do produto:
o formato DWG é proprietário e sem especificação aberta, e as únicas alternativas
à LibreDWG são as bibliotecas da Open Design Alliance (comerciais e fechadas) ou
um parser próprio de custo proibitivo e altíssimo risco de compatibilidade.

A decisão precisa ser tomada **agora** porque o pipeline de release já está
automatizado e cada publicação repete a divergência.

## Decisão

Fica acordado o **relicenciamento integral do NeoCAD para a GNU General Public
License versão 3.0 ou posterior (`GPL-3.0-or-later`)**, substituindo a MIT em
todos os pontos de declaração: `LICENSE`, `README.md`, `src-tauri/Cargo.toml` e
metadados de empacotamento.

A escolha alinha a licença declarada à licença efetiva da distribuição e alinha o
projeto ao ecossistema CAD/CAE open-source do qual ele depende e ao qual pretende
contribuir — LibreDWG, planegcs, OpenCASCADE, Gmsh, CalculiX, OpenFOAM.

Fica igualmente acordado que a conformidade de licenças é **verificada
automaticamente**: `scripts/check-licenses.mjs`, executado na CI, valida a árvore
de dependências de runtime contra `scripts/license-policy.json` e falha quando
surge licença desconhecida ou incompatível com a GPL-3.0.

O `AGENTS.md` §6 já admite GPL entre as licenças preferenciais do perfil pessoal,
de modo que esta decisão não conflita com a governança vigente.

## Consequências

- **Impacto positivo:** a licença declarada passa a corresponder à distribuição
  real, eliminando risco jurídico e de reputação; todo o ecossistema copyleft de
  CAD/CAE fica disponível sem atrito para as próximas fases, incluindo a trilha
  de kernel próprio (ADR 0003) e uma eventual trilha FEM/CFD; o caráter copyleft
  garante que melhorias derivadas retornem à comunidade, o que é coerente com o
  objetivo declarado do projeto.
- **Impacto negativo:** inviabiliza a incorporação do NeoCAD em produtos
  proprietários, reduzindo o universo de adotantes corporativos; contribuidores
  que exigem licença permissiva podem se afastar; a obrigação de fornecer código
  correspondente acompanha toda redistribuição de binários.
- **Trade-offs aceitos:** abre-se mão da adoção permissiva em troca de
  legalidade, coerência com o ecossistema e liberdade técnica para depender de
  bibliotecas copyleft de alta qualidade. Como o projeto já era GPL de fato, o
  custo marginal desta decisão é baixo e o ganho de clareza é alto.

## Diretriz de Conformidade de Código

- **Proibido:** declarar o projeto, seus pacotes ou seus artefatos como MIT ou
  sob qualquer licença permissiva, em qualquer arquivo, metadado de
  empacotamento, badge ou documentação.
- **Proibido:** introduzir dependência de runtime cuja licença seja incompatível
  com a GPL-3.0 — notadamente licenças proprietárias, de fonte disponível (SSPL,
  BUSL) ou Apache-2.0 em combinação que exija GPL-2.0-only. Dependências apenas
  de desenvolvimento não estão sujeitas a esta restrição.
- **Proibido:** alterar `scripts/license-policy.json` para acomodar uma
  dependência nova sem registro da avaliação em ADR ou em
  `THIRD-PARTY-LICENSES.md`.
- **Obrigatório:** manter `THIRD-PARTY-LICENSES.md` atualizado a cada mudança na
  árvore de dependências de runtime, com proveniência e licença de cada item
  distribuído.
- **Obrigatório:** manter o job de política de licenças na CI como gate
  bloqueante de merge.
- **Obrigatório:** acompanhar toda distribuição de binários da oferta de código
  correspondente exigida pela GPL-3.0.

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do projeto
> e deve ser reportado para revisão antes de prosseguir.
