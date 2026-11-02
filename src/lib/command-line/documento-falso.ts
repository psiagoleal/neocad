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

	/** Aplica a função aos pontos das retas dadas, num passo de desfazer. */
	function transformar(ids: string[], f: (p: CadPoint) => CadPoint): void {
		const antes = retas.map((r) => ({ ...r }));
		for (const r of retas.filter((r) => ids.includes(r.id))) {
			r.de = f(r.de);
			r.ate = f(r.ate);
		}
		pilha.push(() => retas.splice(0, retas.length, ...antes));
	}

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
		moveEntities: (ids, delta) => {
			const antes = retas.map((r) => ({ ...r }));
			for (const r of retas.filter((r) => ids.includes(r.id))) {
				r.de = { x: r.de.x + delta.x, y: r.de.y + delta.y };
				r.ate = { x: r.ate.x + delta.x, y: r.ate.y + delta.y };
			}
			pilha.push(() => retas.splice(0, retas.length, ...antes));
		},
		copyEntities: (ids, delta) => {
			const novas = retas
				.filter((r) => ids.includes(r.id))
				.map((r) => ({
					id: String(proximo++) as CadEntityId,
					camada: r.camada,
					de: { x: r.de.x + delta.x, y: r.de.y + delta.y },
					ate: { x: r.ate.x + delta.x, y: r.ate.y + delta.y }
				}));
			retas.push(...novas);
			pilha.push(() => novas.forEach((n) => retas.splice(retas.indexOf(n), 1)));
			return novas.map((n) => n.id);
		},
		rotateEntities: (ids, pivo, angulo) => {
			transformar(ids, (p) => {
				const [c, s] = [Math.cos(angulo), Math.sin(angulo)];
				const [dx, dy] = [p.x - pivo.x, p.y - pivo.y];
				return { x: pivo.x + dx * c - dy * s, y: pivo.y + dx * s + dy * c };
			});
		},
		mirrorEntities: (ids, ponto, angulo) => {
			transformar(ids, (p) => espelhar(p, ponto, angulo));
		},
		mirrorCopyEntities: (ids, ponto, angulo) => {
			const novas = retas
				.filter((r) => ids.includes(r.id))
				.map((r) => ({
					id: String(proximo++) as CadEntityId,
					camada: r.camada,
					de: espelhar(r.de, ponto, angulo),
					ate: espelhar(r.ate, ponto, angulo)
				}));
			retas.push(...novas);
			pilha.push(() => novas.forEach((n) => retas.splice(retas.indexOf(n), 1)));
			return novas.map((n) => n.id);
		},
		scaleEntities: (ids, pivo, fator) => {
			if (!(fator > 0) || !Number.isFinite(fator)) {
				throw new CadKernelOperationError(`fator de escala inválido: ${fator}`);
			}
			transformar(ids, (p) => ({
				x: pivo.x + (p.x - pivo.x) * fator,
				y: pivo.y + (p.y - pivo.y) * fator
			}));
		},
		offsetEntity: (id, distancia, lado) => {
			const r = retas.find((x) => x.id === id);
			if (r == null || distancia > 100) {
				throw new CadKernelOperationError('paralela impossível');
			}
			// Só retas horizontais nos testes: a paralela vai para o lado de y do ponto.
			const dy = lado.y >= r.de.y ? distancia : -distancia;
			const nova = {
				id: String(proximo++) as CadEntityId,
				camada: r.camada,
				de: { x: r.de.x, y: r.de.y + dy },
				ate: { x: r.ate.x, y: r.ate.y + dy }
			};
			retas.push(nova);
			pilha.push(() => retas.splice(retas.indexOf(nova), 1));
			return [nova.id];
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

/** Reflexão de p pela reta que passa por `ponto` com o ângulo dado. */
function espelhar(p: CadPoint, ponto: CadPoint, angulo: number): CadPoint {
	const [c, s] = [Math.cos(2 * angulo), Math.sin(2 * angulo)];
	const [dx, dy] = [p.x - ponto.x, p.y - ponto.y];
	return { x: ponto.x + dx * c + dy * s, y: ponto.y + dx * s - dy * c };
}
