// Caminho relativo: src/lib/command-line/camada.ts

/**
 * \file src/lib/command-line/camada.ts
 * \brief A camada onde entra a geometria nova (spec 10, "Camada corrente").
 * \author Iago Leal
 * \date 2026-11-02
 */

import type { CadDocument } from '$lib/services/cad-document';
import type { CadLayerId } from '$lib/types/cad';

/**
 * Ainda não há camada corrente escolhida pelo usuário (fase KP1): vale a `0`, que
 * todo DXF tem, e na falta dela a primeira.
 */
export function camadaCorrente(documento: Pick<CadDocument, 'listLayers'>): CadLayerId {
	const camadas = documento.listLayers();
	const camada = camadas.find((c) => c.name === '0') ?? camadas[0];
	if (camada == null) {
		throw new Error('O desenho não tem camada onde desenhar.');
	}
	return camada.id;
}
