// Caminho relativo: src/lib/render/selecao.spec.ts

/**
 * \file src/lib/render/selecao.spec.ts
 * \brief Regras da seleção (spec 10, S1, S2, S4, S5), com consulta falsa.
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import type { CadEntityId } from '$lib/types/cad';
import { Selecao, type ConsultaDaSelecao } from './selecao';

const id = (n: number) => String(n) as CadEntityId;

/** Consulta falsa: a reta 1 está em y = 0; a área devolve o que a função mandar. */
function consulta(area: ConsultaDaSelecao['area'] = () => []) {
	return {
		clique: vi.fn((p: { x: number; y: number }, raio: number) =>
			Math.abs(p.y) <= raio ? [id(1), id(2)] : []
		),
		area: vi.fn(area)
	};
}

describe('Selecao', () => {
	it('clique no traço seleciona a mais perto; Shift tira (S1)', () => {
		const s = new Selecao();
		const c = consulta();
		s.clicar({ x: 5, y: 0.1 }, 0.5, false, c);
		expect(s.ids).toEqual([id(1)]);
		s.clicar({ x: 6, y: 0 }, 0.5, false, c);
		expect(s.ids).toEqual([id(1)]);
		s.clicar({ x: 6, y: 0 }, 0.5, true, c);
		expect(s.ids).toEqual([]);
		expect(s.inicioDaJanela).toBeNull();
	});

	it('clique no vazio abre a janela; o segundo clique fecha (S2)', () => {
		const s = new Selecao();
		const c = consulta(() => [id(3), id(4)]);
		s.clicar({ x: 0, y: 10 }, 0.5, false, c);
		expect(s.inicioDaJanela).toEqual({ x: 0, y: 10 });
		expect(c.area).not.toHaveBeenCalled();

		s.clicar({ x: 20, y: 30 }, 0.5, false, c);
		expect(c.area).toHaveBeenCalledWith({ minX: 0, minY: 10, maxX: 20, maxY: 30 }, false);
		expect(s.ids).toEqual([id(3), id(4)]);
		expect(s.inicioDaJanela).toBeNull();
	});

	it('o segundo clique fecha a janela mesmo sobre um traço', () => {
		const s = new Selecao();
		const c = consulta(() => [id(9)]);
		s.clicar({ x: 0, y: 10 }, 0.5, false, c);
		s.clicar({ x: 20, y: 0 }, 0.5, false, c);
		expect(c.area).toHaveBeenCalledTimes(1);
		expect(s.ids).toEqual([id(9)]);
	});

	it('da direita para a esquerda é cruzada; com Shift, tira (S2)', () => {
		const s = new Selecao();
		const c = consulta(() => [id(1), id(5)]);
		s.clicar({ x: 5, y: 0.1 }, 0.5, false, c);
		s.clicar({ x: 20, y: 10 }, 0.5, false, c);
		s.clicar({ x: 0, y: -10 }, 0.5, true, c);
		expect(c.area).toHaveBeenLastCalledWith({ minX: 0, minY: -10, maxX: 20, maxY: 10 }, true);
		expect(s.ids).toEqual([]);
	});

	it('a janela não repete quem já está selecionado', () => {
		const s = new Selecao();
		const c = consulta(() => [id(1), id(3)]);
		s.clicar({ x: 5, y: 0 }, 0.5, false, c);
		s.clicar({ x: 0, y: 10 }, 0.5, false, c);
		s.clicar({ x: 20, y: 30 }, 0.5, false, c);
		expect(s.ids).toEqual([id(1), id(3)]);
	});

	it('esvaziar fecha a janela e limpa (S5)', () => {
		const s = new Selecao();
		const c = consulta();
		s.clicar({ x: 5, y: 0 }, 0.5, false, c);
		s.clicar({ x: 0, y: 10 }, 0.5, false, c);
		s.esvaziar();
		expect(s.ids).toEqual([]);
		expect(s.inicioDaJanela).toBeNull();
	});

	it('podar tira as que deixaram de existir e diz se mudou (S4)', () => {
		const s = new Selecao();
		const c = consulta(() => [id(1), id(2), id(3)]);
		s.clicar({ x: 0, y: 10 }, 0.5, false, c);
		s.clicar({ x: 20, y: 30 }, 0.5, false, c);
		expect(s.podar((x) => x !== id(2))).toBe(true);
		expect(s.ids).toEqual([id(1), id(3)]);
		expect(s.podar(() => true)).toBe(false);
	});
});
