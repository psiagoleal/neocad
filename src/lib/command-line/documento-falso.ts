// Caminho relativo: src/lib/command-line/documento-falso.ts

/**
 * \file src/lib/command-line/documento-falso.ts
 * \brief Documento em memória para os testes dos comandos (não entra no app).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { CadKernelOperationError } from '$lib/services/cad-document';
import type { CadEntityId, CadPoint } from '$lib/types/cad';
import type { DocumentoDosComandos } from './motor';

export type RetaFalsa = { id: CadEntityId; camada: string; de: CadPoint; ate: CadPoint };

/** Guarda as retas e uma pilha de desfazer; recusa reta de comprimento zero, como o kernel. */
export function documentoFalso(camadas = [{ id: '7', name: '0' }]) {
	let proximo = 1;
	const retas: RetaFalsa[] = [];
	const pilha: Array<() => void> = [];

	const documento: DocumentoDosComandos = {
		listLayers: () =>
			camadas.map((c) => ({ ...c, isVisible: true, isOff: false, isFrozen: false })) as never,
		drawLine: (camada, de, ate) => {
			if (de.x === ate.x && de.y === ate.y) {
				throw new CadKernelOperationError('reta de comprimento zero');
			}
			const reta: RetaFalsa = { id: String(proximo++) as CadEntityId, camada, de, ate };
			retas.push(reta);
			pilha.push(() => retas.splice(retas.indexOf(reta), 1));
			return reta.id;
		},
		eraseEntities: (ids) => {
			if (ids.some((id) => !retas.some((r) => r.id === id))) {
				throw new CadKernelOperationError('entidade inexistente');
			}
			const antes = [...retas];
			for (const id of ids)
				retas.splice(
					retas.findIndex((r) => r.id === id),
					1
				);
			pilha.push(() => retas.splice(0, retas.length, ...antes));
		},
		eraseEntity: (id) => {
			const i = retas.findIndex((r) => r.id === id);
			const [reta] = retas.splice(i, 1);
			pilha.push(() => retas.splice(i, 0, reta));
		},
		undo: () => {
			const desfazer = pilha.pop();
			desfazer?.();
			return desfazer != null;
		},
		getHistory: () => ({
			canUndo: pilha.length > 0,
			canRedo: false,
			undoLabel: null,
			redoLabel: null,
			undoDepth: pilha.length,
			redoDepth: 0
		})
	};

	return { documento, retas, pilha };
}

/** Seleção falsa, que os testes enchem à mão. */
export function selecaoFalsa(ids: string[] = []) {
	const estado = { ids: [...ids] as CadEntityId[] };
	return {
		estado,
		selecao: {
			ids: () => [...estado.ids],
			esvaziar: () => {
				estado.ids = [];
			}
		}
	};
}
