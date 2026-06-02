// Caminho relativo: src/lib/services/cad-commands.ts

/**
 * \file src/lib/services/cad-commands.ts
 * \brief Serviço de comandos CAD: ponte entre a UI e o adaptador do viewer.
 * \author Iago Leal
 * \date 2026-06-02
 *
 * Mantém a fronteira do ADR 0001: a UI consome estes contratos NeoCAD e nunca
 * importa `@mlightcad/*` diretamente. O serviço delega a enumeração ao
 * adaptador `NeoCadViewer` e funde com os metadados de apresentação.
 */

import { buildCadCommandCatalog } from '$lib/config/cad-command-catalog';
import type { CadCommandCatalogItem } from '$lib/types/cad';
import type { NeoCadViewer } from '$lib/viewer/neocad-viewer';

/**
 * Lista o catálogo de comandos disponíveis no viewer ativo, já com rótulos de
 * apresentação. Retorna lista vazia quando o viewer não está inicializado.
 */
export function listCadCommandCatalog(viewer: NeoCadViewer | null): CadCommandCatalogItem[] {
	if (viewer == null) {
		return [];
	}

	return buildCadCommandCatalog(viewer.listCommandDescriptors());
}

/** Dispara um comando CAD pelo seu nome global, via adaptador do viewer. */
export function executeCadCommand(viewer: NeoCadViewer | null, command: string): void {
	if (viewer == null) {
		throw new Error('Viewer CAD ainda não foi inicializado.');
	}

	viewer.executeCommand(command);
}
