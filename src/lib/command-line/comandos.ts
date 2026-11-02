// Caminho relativo: src/lib/command-line/comandos.ts

/**
 * \file src/lib/command-line/comandos.ts
 * \brief Os comandos que a linha de comando própria executa (ADR 0011).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * É o registro único: a linha de comando inicia o que está aqui, e o catálogo de
 * `Ajuda > Comandos CAD` marca como disponível exatamente o que está aqui. Todo
 * comando registrado edita pelo `CadDocument` (spec 07, V5).
 */

import { apagar } from './apagar';
import { aparar, estender } from './aparar';
import { copiar } from './copiar';
import { deslocar } from './deslocar';
import { escalar } from './escalar';
import { espelhar } from './espelhar';
import { girar } from './girar';
import { linha } from './linha';
import { mover } from './mover';
import type { FabricaDeComando } from './motor';

/** Comandos implementados, pelo nome em inglês da tabela da spec 08. */
export const COMANDOS: Readonly<Record<string, FabricaDeComando>> = {
	COPY: copiar,
	ERASE: apagar,
	EXTEND: estender,
	LINE: linha,
	MIRROR: espelhar,
	MOVE: mover,
	OFFSET: deslocar,
	ROTATE: girar,
	SCALE: escalar,
	TRIM: aparar
};
