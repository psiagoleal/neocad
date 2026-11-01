// Caminho relativo: src/lib/render/vista.spec.ts

import { describe, expect, it } from 'vitest';
import {
	ampliar,
	deslocar,
	enquadrar,
	erroDeCorda,
	fatorDaRoda,
	paraMundo,
	redimensionar,
	transformacao
} from './vista';

/** Aplica a transformação a um ponto relativo à origem e devolve o pixel. */
function pixel(
	t: [number, number, number, number],
	largura: number,
	altura: number,
	x: number,
	y: number
) {
	const cx = x * t[0] + t[2];
	const cy = y * t[1] + t[3];
	return { px: ((cx + 1) / 2) * largura, py: ((1 - cy) / 2) * altura };
}

describe('vista da tela', () => {
	it('enquadrar põe a caixa inteira no centro, com margem', () => {
		const v = enquadrar({ minX: 0, minY: 0, maxX: 200, maxY: 100 }, 800, 400);

		expect(v.centroX).toBe(100);
		expect(v.centroY).toBe(50);
		// A largura limita: 200 unidades em 800 px, menos a margem.
		expect(v.escala).toBeLessThan(4);
		expect(v.escala).toBeGreaterThan(3.5);
	});

	it('enquadrar sem caixa ou com caixa degenerada não divide por zero', () => {
		const vazio = enquadrar(null, 800, 400);
		const ponto = enquadrar({ minX: 5, minY: 5, maxX: 5, maxY: 5 }, 800, 400);

		for (const v of [vazio, ponto]) {
			expect(Number.isFinite(v.escala) && v.escala > 0).toBe(true);
		}
		expect(ponto.centroX).toBe(5);
	});

	it('a transformação leva o centro da vista ao centro da tela, com a origem somada', () => {
		const v = enquadrar(
			{ minX: 5_000_000, minY: 7_500_000, maxX: 5_000_200, maxY: 7_500_100 },
			800,
			400
		);
		const origem = { x: 5_000_100, y: 7_500_050 };
		const t = transformacao(v, origem);

		const centro = pixel(t, 800, 400, 0, 0);
		expect(centro.px).toBeCloseTo(400, 6);
		expect(centro.py).toBeCloseTo(200, 6);
		// O canto inferior esquerdo da caixa fica embaixo e à esquerda do centro.
		const canto = pixel(t, 800, 400, -100, -50);
		expect(canto.px).toBeLessThan(400);
		expect(canto.py).toBeGreaterThan(200);
	});

	it('deslocar acompanha o arrasto: o desenho anda junto com o cursor', () => {
		const v = enquadrar({ minX: 0, minY: 0, maxX: 100, maxY: 100 }, 500, 500);
		const antes = paraMundo(v, 250, 250);
		const depois = deslocar(v, 50, 20);

		// O ponto que estava em (250, 250) agora está em (300, 270).
		const mesmo = paraMundo(depois, 300, 270);
		expect(mesmo.x).toBeCloseTo(antes.x, 9);
		expect(mesmo.y).toBeCloseTo(antes.y, 9);
	});

	it('ampliar mantém fixo o ponto sob o cursor', () => {
		const v = enquadrar({ minX: 0, minY: 0, maxX: 100, maxY: 100 }, 500, 500);
		const sob = paraMundo(v, 120, 340);
		const ampliada = ampliar(v, 2, 120, 340);

		expect(ampliada.escala).toBeCloseTo(v.escala * 2, 9);
		const depois = paraMundo(ampliada, 120, 340);
		expect(depois.x).toBeCloseTo(sob.x, 9);
		expect(depois.y).toBeCloseTo(sob.y, 9);
	});

	it('a roda para baixo afasta e para cima aproxima', () => {
		expect(fatorDaRoda(120)).toBeLessThan(1);
		expect(fatorDaRoda(-120)).toBeGreaterThan(1);
		expect(fatorDaRoda(120) * fatorDaRoda(-120)).toBeCloseTo(1, 12);
	});

	it('redimensionar mantém o centro e a escala', () => {
		const v = enquadrar({ minX: 0, minY: 0, maxX: 100, maxY: 100 }, 500, 500);
		const r = redimensionar(v, 1000, 300);

		expect([r.centroX, r.centroY, r.escala]).toEqual([v.centroX, v.centroY, v.escala]);
		expect([r.largura, r.altura]).toEqual([1000, 300]);
	});

	it('o erro de corda é uma fração do desenho, e positivo mesmo sem desenho', () => {
		expect(erroDeCorda({ minX: 0, minY: 0, maxX: 4096, maxY: 10 })).toBeCloseTo(1, 9);
		expect(erroDeCorda(null)).toBeGreaterThan(0);
		expect(erroDeCorda({ minX: 3, minY: 3, maxX: 3, maxY: 3 })).toBeGreaterThan(0);
	});
});
