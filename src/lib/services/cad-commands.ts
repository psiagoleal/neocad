// Caminho relativo: src/lib/services/cad-commands.ts

/**
 * \file src/lib/services/cad-commands.ts
 * \brief Catálogo de comandos CAD para a UI (ADR 0011).
 * \author Iago Leal
 * \date 2026-06-02
 *
 * Os comandos vêm da tabela da spec 08 e a disponibilidade, do registro da linha
 * de comando própria. O upstream não participa mais (MT-KV-13b).
 */

import { CATALOGO_DA_SPEC } from '$lib/command-line/catalogo';
import { COMANDOS } from '$lib/command-line/comandos';
import { buildCadCommandCatalog } from '$lib/config/cad-command-catalog';
import type { CadCommandCatalogItem } from '$lib/types/cad';

/** O catálogo de `Ajuda > Comandos CAD`, com rótulos de apresentação. */
export function listCadCommandCatalog(): CadCommandCatalogItem[] {
	return buildCadCommandCatalog(CATALOGO_DA_SPEC, Object.keys(COMANDOS));
}
