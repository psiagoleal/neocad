// Caminho relativo: src/lib/command-line/pontos.spec.ts

/**
 * \file src/lib/command-line/pontos.spec.ts
 * \brief Entrada de pontos como no AutoCAD (spec 10, D2 e D3).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it } from 'vitest';
import { lerPonto } from './pontos';

const ponto = (texto: string, anterior: { x: number; y: number } | null = null) => {
	const r = lerPonto(texto, anterior);
	return r.tipo === 'ponto' ? r.ponto : r.tipo;
};

describe('lerPonto (D2)', () => {
	it('absoluto x,y, com ponto decimal, sinal e espaços em volta', () => {
		expect(ponto('10,20')).toEqual({ x: 10, y: 20 });
		expect(ponto('  -1.5 , .25 ')).toEqual({ x: -1.5, y: 0.25 });
		expect(ponto('+3,1e2')).toEqual({ x: 3, y: 100 });
	});

	it('relativo @dx,dy soma ao ponto anterior', () => {
		expect(ponto('@5,-2', { x: 10, y: 10 })).toEqual({ x: 15, y: 8 });
	});

	it('polar d<a, absoluto e relativo, em graus anti-horários a partir de +X', () => {
		const p = ponto('10<90') as { x: number; y: number };
		expect(p.x).toBeCloseTo(0, 12);
		expect(p.y).toBeCloseTo(10, 12);
		const q = ponto('@2<180', { x: 5, y: 5 }) as { x: number; y: number };
		expect(q.x).toBeCloseTo(3, 12);
		expect(q.y).toBeCloseTo(5, 12);
		const r = ponto('@1<-45', { x: 0, y: 0 }) as { x: number; y: number };
		expect(r.x).toBeCloseTo(Math.SQRT1_2, 12);
		expect(r.y).toBeCloseTo(-Math.SQRT1_2, 12);
	});
});

describe('lerPonto: o que não é ponto (D3)', () => {
	it.each([
		'',
		'abc',
		'10',
		'10,',
		',5',
		'1,2,3',
		'1,5,2',
		'10<',
		'<30',
		'1..2,3',
		'@',
		'D',
		'NaN,1',
		'Infinity,1'
	])('"%s" é inválido', (texto) => {
		expect(ponto(texto, { x: 0, y: 0 })).toBe('invalido');
	});

	it('vírgula decimal não vale: "1,5" é inválido, pois falta a segunda coordenada', () => {
		expect(ponto('1,5,2,5')).toBe('invalido');
	});

	it('relativo sem ponto anterior', () => {
		expect(ponto('@1,1', null)).toBe('sem-anterior');
		expect(ponto('@1<0', null)).toBe('sem-anterior');
	});
});
