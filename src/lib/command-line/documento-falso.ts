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

/** Guarda as retas e uma pilha de desfazer; as recusas do kernel são simuladas por teste. */
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

	const chamadas: Array<{ nome: string; args: unknown[] }> = [];

	/** Registra a ligação; recusa como o kernel quando `nao_cabe`; cria a reta de ligação. */
	function ligar(nome: string, args: unknown[], naoCabe: boolean): CadEntityId | null {
		if (naoCabe) throw new CadKernelOperationError('o raio ou a distância não cabe');
		chamadas.push({ nome, args });
		const nova = {
			id: String(proximo++) as CadEntityId,
			camada: '7',
			de: { x: 0, y: 0 },
			ate: { x: 1, y: 1 }
		};
		retas.push(nova);
		pilha.push(() => retas.splice(retas.indexOf(nova), 1));
		return nova.id;
	}

	const documento: DocumentoDosComandos = {
		listLayers: () =>
			camadas.map((c) => ({ ...c, isVisible: true, isOff: false, isFrozen: false })) as never,
		drawLine: (camada, de, ate) => {
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
		// Aparar e estender nos testes: retas horizontais cortadas por arestas verticais.
		trimEntity: (id, arestas, clique, captura) => {
			const r = retas.find((x) => x.id === id);
			if (r == null || Math.abs(clique.y - r.de.y) > captura) {
				throw new CadKernelOperationError('o clique não está sobre o objeto');
			}
			const [x0, x1] = [Math.min(r.de.x, r.ate.x), Math.max(r.de.x, r.ate.x)];
			const cortes = retas
				.filter((x) => arestas.includes(x.id) && x.de.x === x.ate.x && x.de.x > x0 && x.de.x < x1)
				.map((x) => x.de.x)
				.sort((a, b) => a - b);
			if (cortes.length === 0) throw new CadKernelOperationError('nenhuma aresta corta o objeto');
			const antes = cortes.filter((c) => c < clique.x).at(-1);
			const depois = cortes.find((c) => c > clique.x);
			const copia = { ...r };
			const pedacos: Array<[number, number]> = [];
			if (antes != null) pedacos.push([x0, antes]);
			if (depois != null) pedacos.push([depois, x1]);
			const y = r.de.y;
			const [primeiro, segundo] = pedacos;
			const ids: CadEntityId[] = [];
			r.de = { x: primeiro[0], y };
			r.ate = { x: primeiro[1], y };
			ids.push(r.id);
			let nova: RetaFalsa | null = null;
			if (segundo != null) {
				nova = {
					id: String(proximo++) as CadEntityId,
					camada: r.camada,
					de: { x: segundo[0], y },
					ate: { x: segundo[1], y }
				};
				retas.push(nova);
				ids.push(nova.id);
			}
			pilha.push(() => {
				Object.assign(r, copia);
				if (nova != null) retas.splice(retas.indexOf(nova), 1);
			});
			return ids;
		},
		extendEntity: (id, contornos, clique, captura) => {
			const r = retas.find((x) => x.id === id);
			if (r == null || Math.abs(clique.y - r.de.y) > captura) {
				throw new CadKernelOperationError('o clique não está sobre o objeto');
			}
			const ponta = Math.abs(clique.x - r.de.x) < Math.abs(clique.x - r.ate.x) ? 'de' : 'ate';
			const outra = ponta === 'de' ? r.ate : r.de;
			const x = r[ponta].x;
			const para = x < outra.x ? -1 : 1;
			const alvo = retas
				.filter((c) => contornos.includes(c.id) && c.de.x === c.ate.x && (c.de.x - x) * para > 0)
				.map((c) => c.de.x)
				.sort((a, b) => (a - b) * para)[0];
			if (alvo == null) throw new CadKernelOperationError('nenhum contorno alcançado');
			const copia = { ...r };
			r[ponta] = { x: alvo, y: r[ponta].y };
			pilha.push(() => Object.assign(r, copia));
		},
		// Concordar e chanfrar nos testes: registram a chamada e criam uma reta de ligação.
		filletEntities: (a, pa, b, pb, raio) => ligar('concordar', [a, pa, b, pb, raio], raio > 50),
		chamferEntities: (a, pa, b, pb, d1, d2) =>
			ligar('chanfrar', [a, pa, b, pb, d1, d2], d1 > 50 || d2 > 50),
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

	return { documento, retas, pilha, chamadas };
}

/** Seleção falsa, que os testes enchem à mão; `todas` é o que o espaço tem. */
export function selecaoFalsa(ids: string[] = [], todas: string[] = []) {
	const estado = { ids: [...ids] as CadEntityId[], todas: [...todas] as CadEntityId[] };
	return {
		estado,
		selecao: {
			ids: () => [...estado.ids],
			esvaziar: () => {
				estado.ids = [];
			},
			todas: () => [...estado.todas]
		}
	};
}

/** Reflexão de p pela reta que passa por `ponto` com o ângulo dado. */
function espelhar(p: CadPoint, ponto: CadPoint, angulo: number): CadPoint {
	const [c, s] = [Math.cos(2 * angulo), Math.sin(2 * angulo)];
	const [dx, dy] = [p.x - ponto.x, p.y - ponto.y];
	return { x: ponto.x + dx * c + dy * s, y: ponto.y + dx * s - dy * c };
}
