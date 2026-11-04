<!-- Caminho relativo: kernel/neocad-geometry/data/LEIA-ME.md -->

# Dados embutidos na geometria

## `rowmans.jhf` — fonte Hershey Roman Simplex

Fonte de traço usada para desenhar o texto na tela (ADR 0010). Copiada sem alteração de
[`kamalmostafa/hershey-fonts`](https://github.com/kamalmostafa/hershey-fonts), arquivo
`hershey-fonts/rowmans.jhf`, em 2026-11-01. A licença dos dados pede que estas
atribuições acompanhem a fonte:

- The Hershey Fonts were originally created by Dr. A. V. Hershey while working at the
  U. S. National Bureau of Standards.
- The format of the Font data in this distribution was originally created by James
  Hurt, Cognition, Inc., 900 Technology Park Drive, Billerica, MA 01821.

Texto da licença, como distribuído com os dados:

```text
This distribution of the Hershey Fonts may be used by anyone for
any purpose, commercial or otherwise, providing that:
	1. The following acknowledgements must be distributed with
		the font data:
		- The Hershey Fonts were originally created by Dr.
			A. V. Hershey while working at the U. S.
			National Bureau of Standards.
		- The format of the Font data in this distribution
			was originally created by
				James Hurt
				Cognition, Inc.
				900 Technology Park Drive
				Billerica, MA 01821
				(mit-eddie!ci-dandelion!hurt)
	2. The font data in this distribution may be converted into
		any other format *EXCEPT* the format distributed by
		the U.S. NTIS (which organization holds the rights
		to the distribution and use of the font data in that
		particular format).
```

Formato: uma linha por glifo, do caractere 32 ao 127 (este último é o sinal de grau).
Colunas 1–5, número do glifo; 6–8, quantidade de pares; depois, pares de caracteres em
que cada coordenada é o código menos o de `R`. O primeiro par são as margens esquerda e
direita; ` R` levanta a pena. A linha de base fica em 9 e o topo das maiúsculas em −12.
