<!-- Caminho relativo: docs/adr/0011-catalogo-de-comandos-da-linha-propria.md -->

# ADR 0011: Catálogo de comandos derivado da tabela da spec 08 e da linha de comando própria

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-11-02 · **Aceito:** 2026-11-02 (pelo mantenedor, antes da implementação)
- **Decisores:** Iago Leal
- **Tags:** comandos, catálogo, linha de comando, upstream, fase KV
- **Substitui:** o item 1 da Decisão do [ADR 0001](0001-catalogo-dinamico-e-paineis-sobre-data-model.md)

## Contexto

O ADR 0001 fez o catálogo de `Ajuda > Comandos CAD` vir da pilha de comandos do upstream,
lida em tempo de execução, para que a interface nunca prometesse um comando que o viewer
não aceitasse. Desde então:

- o kernel passou a ser quem edita (ADR 0003, ADR 0008), e os comandos do upstream editam
  um modelo que o salvamento ignora (achado C4);
- a linha de comando própria (spec 08, MT-KV-14a e 14b) executa os comandos pelo kernel;
- a spec 07, V5, exige que comando que não edita pelo kernel não apareça como disponível;
- o MT-KV-13b tira do upstream todo caminho de comando.

Sem os comandos do upstream, a fonte do ADR 0001 deixa de existir. O que continua valendo
é o propósito dele: o catálogo não pode prometer o que não funciona.

## Decisão

Fica acordado que:

1. Os comandos que **existem** no catálogo são os da tabela
   [`08-comandos-autocad.tsv`](../specs/08-comandos-autocad.tsv) (mais os que outras
   specs acrescentarem), com o nome em inglês, o nome em português e os apelidos.
2. Os comandos **disponíveis** são exatamente os registrados na linha de comando própria
   (`src/lib/command-line/comandos.ts`), lidos em tempo de execução. O catálogo mostra os
   demais como "ainda não disponível".
3. O upstream não participa do catálogo nem executa comando algum.

Os itens 2 e 3 do ADR 0001 (painéis sobre o adaptador e fronteira de integração no
adaptador e nos serviços) continuam em vigor.

## Consequências

- **Impacto positivo:** o catálogo diz a verdade sobre o que edita o desenho; a mesma
  tabela serve à resolução de comandos (spec 08) e à ajuda, sem cópia; o usuário vê os
  dois idiomas e o apelido.
- **Impacto negativo:** o catálogo lista 218 comandos, a maioria ainda indisponível, até
  as fases seguintes os implementarem.
- **Trade-offs aceitos:** a disponibilidade depende de cada comando novo ser registrado em
  `comandos.ts`; é o mesmo registro que a linha de comando usa para executá-lo, então um
  comando não pode funcionar sem aparecer, nem aparecer sem funcionar.

## Diretriz de Conformidade de Código

- **Obrigatório:** todo comando executável pela linha de comando está em
  `src/lib/command-line/comandos.ts` e edita pelo `CadDocument`.
- **Obrigatório:** o catálogo de comandos é montado a partir da tabela da spec 08 e desse
  registro (`listCadCommandCatalog`).
- **Proibido:** marcar como disponível um comando que não esteja nesse registro, ou manter
  outra lista de disponibilidade.
- **Proibido:** encaminhar texto ao `sendStringToExecute` do upstream ou exibir a barra de
  comandos dele.
