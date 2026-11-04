// Caminho relativo: src/lib/render/selecao.spec.ts

/**
 * \file src/lib/render/selecao.spec.ts
 * \brief Regras do clique e da janela (spec 10, S1, S2, S5), com consulta falsa.
 * \author Iago Leal
 * \date 2026-11-02
 *
 * O conjunto selecionado mora no kernel (MT-KV-14g); aqui a consulta falsa o
 * guarda, e o teste confere o que a seleção manda pôr ou tirar.
 */

import { describe, expect, it, vi } from 'vitest';
import type { CadBounds, CadEntityId } from '$lib/types/cad';
import { Selecao } from './selecao';

const id = (n: number) => String(n) as CadEntityId;

/** Consulta falsa: a reta 1 está em y = 0; a área põe ou tira o que a lista mandar. */
function consulta(daArea: CadEntityId[] = []) {
	const selecionadas: CadEntityId[] = [];
	const aplicar = (ids: CadEntityId[], tirar: boolean) => {
		for (const i of ids) {
			const onde = selecionadas.indexOf(i);
			if (tirar && onde >= 0) selecionadas.splice(onde, 1);
			if (!tirar && onde < 0) selecionadas.push(i);
		}
	};
	return {
		selecionadas,
		clique: vi.fn((p: { x: number; y: number }, raio: number) =>
			Math.abs(p.y) <= raio ? [id(1), id(2)] : []
		),
		aplicar: vi.fn(aplicar),
		area: vi.fn((_area: CadBounds, _cruzada: boolean, tirar: boolean) => aplicar(daArea, tirar))
	};
}

describe('Selecao', () => {
	it('clique no traço põe a mais perto; Shift tira (S1)', () => {
		const s = new Selecao();
		const c = consulta();
		s.clicar({ x: 5, y: 0.1 }, 0.5, false, c);
		expect(c.aplicar).toHaveBeenLastCalledWith([id(1)], false);
		expect(c.selecionadas).toEqual([id(1)]);
		s.clicar({ x: 6, y: 0 }, 0.5, false, c);
		expect(c.selecionadas).toEqual([id(1)]);
		s.clicar({ x: 6, y: 0 }, 0.5, true, c);
		expect(c.aplicar).toHaveBeenLastCalledWith([id(1)], true);
		expect(c.selecionadas).toEqual([]);
		expect(s.inicioDaJanela).toBeNull();
	});

	it('clique no vazio abre a janela; o segundo clique fecha (S2)', () => {
		const s = new Selecao();
		const c = consulta([id(3), id(4)]);
		s.clicar({ x: 0, y: 10 }, 0.5, false, c);
		expect(s.inicioDaJanela).toEqual({ x: 0, y: 10 });
		expect(c.area).not.toHaveBeenCalled();

		s.clicar({ x: 20, y: 30 }, 0.5, false, c);
		expect(c.area).toHaveBeenCalledWith({ minX: 0, minY: 10, maxX: 20, maxY: 30 }, false, false);
		expect(c.selecionadas).toEqual([id(3), id(4)]);
		expect(s.inicioDaJanela).toBeNull();
	});

	it('o segundo clique fecha a janela mesmo sobre um traço', () => {
		const s = new Selecao();
		const c = consulta([id(9)]);
		s.clicar({ x: 0, y: 10 }, 0.5, false, c);
		s.clicar({ x: 20, y: 0 }, 0.5, false, c);
		expect(c.area).toHaveBeenCalledTimes(1);
		expect(c.aplicar).not.toHaveBeenCalled();
		expect(c.selecionadas).toEqual([id(9)]);
	});

	it('da direita para a esquerda é cruzada; com Shift, tira (S2)', () => {
		const s = new Selecao();
		const c = consulta([id(1), id(5)]);
		s.clicar({ x: 5, y: 0.1 }, 0.5, false, c);
		s.clicar({ x: 20, y: 10 }, 0.5, false, c);
		s.clicar({ x: 0, y: -10 }, 0.5, true, c);
		expect(c.area).toHaveBeenLastCalledWith({ minX: 0, minY: -10, maxX: 20, maxY: 10 }, true, true);
		expect(c.selecionadas).toEqual([]);
	});

	it('esvaziarJanela fecha a janela aberta (S5)', () => {
		const s = new Selecao();
		const c = consulta();
		s.clicar({ x: 0, y: 10 }, 0.5, false, c);
		s.esvaziarJanela();
		expect(s.inicioDaJanela).toBeNull();
		s.clicar({ x: 20, y: 30 }, 0.5, false, c);
		expect(c.area).not.toHaveBeenCalled();
	});
});
