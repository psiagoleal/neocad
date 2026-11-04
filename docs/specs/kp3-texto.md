<!-- Caminho relativo: docs/specs/kp3-texto.md -->

# KP3 — Texto

- **Estado:** Esqueleto — critérios EARS a fechar quando a fase abrir
- **Fase:** KP · **ADRs:** 0007 · **Transversais e relacionadas:** 00, 02

## Problema e resultado esperado

Notas, legendas e carimbos usam texto de várias linhas com formatação. O kernel modela só o texto de linha única.

## Escopo

Texto de linha única completo (alinhamentos); texto de várias linhas (`MTEXT`) com formatação em linha; estilos de texto com fonte e altura; edição; linhas de chamada (`QLEADER`/`MLEADER`) e seus estilos; verificação ortográfica.

## Fora de escopo

Tabelas (`TABLE`): spec própria. Campos (`FIELD`) que se atualizam sozinhos.

## Comandos envolvidos

Grafia oficial de cada idioma, de [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv).

| comando EN       | comando PT-BR      | apelido   |
| ---------------- | ------------------ | --------- |
| `TEXT`           | `TEXTO`            | `DT`      |
| `MTEXT`          | `TEXTOM`           | `MT`, `T` |
| `STYLE`          | `ESTILO`           | `ST`      |
| `DDEDIT`         | `EDITARD`          | `ED`      |
| `TEXTEDIT`       | `EDITARTEXTO`      | `TEDIT`   |
| `TEXTALIGN`      | `ALINHARTEXTO`     | `TA`      |
| `SPELL`          | `ORTOGRAF`         | `SP`      |
| `QLEADER`        | `LINHACHAMR`       | `LE`      |
| `MLEADER`        | `LINCHMULT`        | `MLD`     |
| `MLEADEREDIT`    | `EDITARLINCHMULT`  | `MLE`     |
| `MLEADERSTYLE`   | `ESTILOLINCHMULT`  | `MLS`     |
| `MLEADERALIGN`   | `ALINHARLINCHMULT` | `MLA`     |
| `MLEADERCOLLECT` | `CONJLINCHMULT`    | `MLC`     |

## Restrições já tomadas

- Toda entidade nova é lida e gravada em DXF no mesmo ticket (00, T5).
- Todo comando aceita os dois idiomas e o apelido (spec 08).

## Questões em aberto

- Fontes: SHX do AutoCAD não podem ser redistribuídas; definir a substituição por fontes livres e como avisar o usuário.
