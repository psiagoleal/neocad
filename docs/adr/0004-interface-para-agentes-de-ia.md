<!-- Caminho relativo: docs/adr/0004-interface-para-agentes-de-ia.md -->

# ADR 0004: Interface headless para agentes de IA, com CLI como núcleo e MCP como fachada

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-08-07
- **Decisores:** Iago Leal
- **Tags:** agentes, cli, mcp, kernel, headless, automação

## Contexto

Surgiu o objetivo de permitir que agentes de IA — Claude Code CLI e outros — leiam
e editem arquivos CAD através do NeoCAD.

Hoje isso é **estruturalmente impossível**, e não por ausência de ferramental: o
modelo de documento do NeoCAD vive dentro de uma WebView. Ler um `DWG` exige
inicializar o `@mlightcad/cad-simple-viewer` em um contexto de navegador, com
workers e canvas. Não existe caminho headless, e nenhum agente vai subir um
navegador para inspecionar um desenho.

O [ADR 0003](./0003-kernel-cad-proprio.md) muda essa condição. As crates
`neocad-model`, `neocad-geometry`, `neocad-topology`, `neocad-transaction` e
`neocad-io` são Rust puro, sem GUI e sem dependência de ambiente de execução de
navegador. Assim que houver modelo (K1) e I/O nativo, existe um núcleo CAD
headless — e a interface para agentes passa a ser a primeira aplicação que esse
núcleo habilita além da interface gráfica.

Há uma decisão a tomar agora, e não depois, por dois motivos:

1. **A GUI deixa de ser o único consumidor do kernel.** Uma API pensada para uma
   única interface tende a vazar suposições dela — estado global, dependência de
   sessão, retornos pensados para renderização. Saber desde K1 que haverá um
   segundo consumidor, não interativo, disciplina o desenho da API.
2. **Falta uma peça no ADR 0003.** O faseamento previa `K2 — escrita DXF` e
   `K6 — leitura DWG`, mas **não previa leitura nativa de DXF**, porque assumia o
   upstream como parser (MT-K1-14 popula o modelo a partir do documento já aberto
   no navegador). Para um núcleo headless essa premissa não vale: sem leitura
   própria de DXF, um agente continua dependendo do navegador.

Há ainda um risco específico desta superfície. Um agente editando um desenho
opera sobre trabalho de engenharia de outra pessoa, sem supervisão contínua e sem
o retorno visual que um operador humano tem. Sobrescrever um arquivo de projeto
por interpretação equivocada de uma instrução é uma perda real e potencialmente
silenciosa.

## Decisão

Fica acordada a construção de uma **interface headless do NeoCAD para automação e
agentes de IA**, organizada em duas camadas sobre o kernel:

1. **`neocad-cli`** — executável headless, **o núcleo funcional desta trilha**.
   Expõe inspeção e edição de arquivos CAD por linha de comando, com saída
   estruturada (`--format json`) além da legível. Utilizável por qualquer agente,
   por scripts, por CI e por humanos.
2. **`neocad-mcp`** — servidor Model Context Protocol, **fachada fina** sobre as
   mesmas operações do CLI. Dá aos agentes que falam MCP, como o Claude Code CLI,
   ferramentas tipadas em vez de parsing de saída de terminal.

Fica acordado que **o CLI é implementado primeiro e o MCP é construído sobre ele**,
nunca o inverso. A lógica de operação reside no kernel e no CLI; o servidor MCP
não contém regra de negócio própria. Isso evita que a capacidade fique presa a um
protocolo específico e mantém a trilha útil para agentes que não falam MCP.

Fica acordado que **`K2` passa a cobrir leitura e escrita de DXF**, e não apenas
escrita. Isto é um **acréscimo de escopo ao ADR 0003**, não uma supersessão: a
leitura nativa de DXF é pré-requisito de qualquer operação headless, e leitura e
escrita do mesmo formato compartilham o conhecimento de estrutura do arquivo.

### Faseamento

- **A1 — Inspeção.** `neocad info`, `neocad layers`, `neocad entities`,
  `neocad convert`. Somente leitura. Depende de **K1** e da leitura DXF de **K2**.
- **A2 — Edição.** Criação, alteração e remoção de entidades e camadas, cada
  operação como transação do command stack de K1. Depende de **K2** completo.
- **A3 — Servidor MCP.** Ferramentas tipadas sobre A1 e A2, com esquema
  declarado. Depende de **A2**.

### Salvaguardas obrigatórias da edição

Estas não são recomendações; são condição para a trilha A2 existir:

- **Nunca escrever sobre o arquivo de entrada por padrão.** A saída vai para
  caminho novo, salvo `--in-place` explícito.
- **`--dry-run` que produz o resumo das mudanças sem gravar nada**, para que a
  intenção do agente possa ser inspecionada antes do efeito.
- **Toda edição passa pelo command stack transacional**, o que a torna reversível
  e auditável dentro da sessão.
- **Saída determinística**, para que dois arquivos gerados a partir do mesmo
  modelo sejam idênticos e a diferença entre versões seja legível.

## Consequências

- **Impacto positivo:** o kernel ganha um segundo consumidor real, o que expõe
  cedo suposições indevidas da API; surge um caminho de automação e conversão em
  lote que hoje não existe; testes de regressão sobre arquivos CAD reais passam a
  ser executáveis em CI sem navegador; o NeoCAD se torna utilizável dentro de
  fluxos de trabalho agênticos, o que é um diferencial que nenhum CAD
  open-source de desktop oferece hoje.
- **Impacto negativo:** é uma segunda superfície pública para manter, versionar e
  documentar, concorrendo com a GUI pelo mesmo mantenedor; a estabilidade de
  contrato do CLI e do esquema MCP passa a ser compromisso com terceiros; a
  edição automatizada de arquivos de engenharia carrega risco de dano real, que
  as salvaguardas mitigam mas não eliminam; o MCP é um protocolo jovem, e sua
  evolução pode exigir retrabalho da fachada.
- **Trade-offs aceitos:** aceita-se ampliar a superfície de manutenção em troca de
  um caso de uso que valida a arquitetura do kernel e amplia o alcance do projeto.
  Aceita-se atrasar a trilha até que K1 e K2 estejam prontos, em vez de improvisar
  uma automação sobre o upstream em navegador, que teria de ser descartada.

## Diretriz de Conformidade de Código

- **Proibido:** iniciar `neocad-cli` ou `neocad-mcp` antes da conclusão de K1 e da
  leitura DXF de K2. Automação construída sobre o upstream em navegador é
  trabalho descartável.
- **Proibido:** implementar em `neocad-mcp` qualquer operação que não exista em
  `neocad-cli` ou no kernel. O servidor MCP é fachada, não camada de lógica.
- **Proibido:** gravar sobre o arquivo de entrada sem `--in-place` explícito, em
  qualquer comando de edição.
- **Proibido:** referenciar Tauri, Svelte ou DOM em `neocad-cli` e `neocad-mcp`,
  pelas mesmas razões que valem para as crates do kernel (ADR 0003).
- **Obrigatório:** todo comando de edição oferece `--dry-run` e executa suas
  mutações através do command stack transacional.
- **Obrigatório:** todo comando oferece saída estruturada em JSON, com esquema
  estável, além da saída legível por humanos.
- **Obrigatório:** a escrita de arquivos é determinística — o mesmo modelo produz
  bytes idênticos.
- **Obrigatório:** mudanças incompatíveis no contrato do CLI ou no esquema das
  ferramentas MCP são registradas no `CHANGELOG.md` como quebra.

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do projeto
> e deve ser reportado para revisão antes de prosseguir.
