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

import { linha } from './linha';
import type { FabricaDeComando } from './motor';

/** Comandos implementados, pelo nome em inglês da tabela da spec 08. */
export const COMANDOS: Readonly<Record<string, FabricaDeComando>> = { LINE: linha };
