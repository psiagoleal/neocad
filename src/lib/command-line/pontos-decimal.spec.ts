// Caminho relativo: src/lib/command-line/pontos-decimal.spec.ts

/**
 * \file src/lib/command-line/pontos-decimal.spec.ts
 * \brief Ponto e vírgula entre coordenadas e o modo de vírgula decimal (spec 10, D7–D9).
 * \author Iago Leal
 * \date 2026-11-04
 */

import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import {
	definirModoDecimal,
	modoDecimal,
	lerPonto,
	lerNumero,
	formatarPonto,
	formatarNumero,
	formatosDePonto
} from './pontos';

describe('pontos-decimal', () => {
	afterEach(() => {
		definirModoDecimal('ponto');
	});

	describe('D7: modo ponto', () => {
		it('usa ; como separador de coordenadas equivalente a ,', () => {
			const r = lerPonto('1.5;2', null);
			expect(r).toEqual({ tipo: 'ponto', ponto: { x: 1.5, y: 2 } });
		});

		it('aceita deslocamento relativo com @ e separador ;', () => {
			const r = lerPonto('@-2;3', { x: 10, y: 10 });
			expect(r).toEqual({ tipo: 'ponto', ponto: { x: 8, y: 13 } });
		});

		it('rejeita mistura de separadores , e ;', () => {
			expect(lerPonto('1,2;3', null)).toEqual({ tipo: 'invalido' });
			expect(lerPonto('1;2,3', null)).toEqual({ tipo: 'invalido' });
		});

		it('rejeita 1,5;2 no modo ponto', () => {
			expect(lerPonto('1,5;2', null)).toEqual({ tipo: 'invalido' });
		});
	});

	describe('D8: modo virgula', () => {
		beforeEach(() => {
			definirModoDecimal('virgula');
		});

		it('aceita 1,5;2 como ponto', () => {
			const r = lerPonto('1,5;2', null);
			expect(r).toEqual({ tipo: 'ponto', ponto: { x: 1.5, y: 2 } });
		});

		it('aceita deslocamento relativo com , e ;', () => {
			const r = lerPonto('@-0,5;3', { x: 1, y: 1 });
			expect(r).toEqual({ tipo: 'ponto', ponto: { x: 0.5, y: 4 } });
		});

		it('aceita ângulo em graus anti-horário a partir de +X', () => {
			const r1 = lerPonto('2,5<90', null);
			expect(r1).toEqual({
				tipo: 'ponto',
				ponto: { x: expect.any(Number), y: expect.any(Number) }
			});
			if (r1.tipo === 'ponto') {
				expect(r1.ponto.x).toBeCloseTo(0, 12);
				expect(r1.ponto.y).toBeCloseTo(2.5, 12);
			}

			const r2 = lerPonto('@1,5<180', { x: 0, y: 0 });
			expect(r2).toEqual({
				tipo: 'ponto',
				ponto: { x: expect.any(Number), y: expect.any(Number) }
			});
			if (r2.tipo === 'ponto') {
				expect(r2.ponto.x).toBeCloseTo(-1.5, 12);
				expect(r2.ponto.y).toBeCloseTo(0, 12);
			}
		});

		it('lerNumero aceita vírgula decimal e rejeita ponto', () => {
			expect(lerNumero('0,5')).toBe(0.5);
			expect(lerNumero('-3')).toBe(-3);
			expect(lerNumero('0.5')).toBeNull();
		});

		it('rejeita formatos inválidos no modo virgula', () => {
			expect(lerPonto('1.5;2', null)).toEqual({ tipo: 'invalido' });
			expect(lerPonto('1,5', null)).toEqual({ tipo: 'invalido' });
			expect(lerPonto('1;2;3', null)).toEqual({ tipo: 'invalido' });
			expect(lerPonto('1,5,2', null)).toEqual({ tipo: 'invalido' });
			expect(lerPonto('', null)).toEqual({ tipo: 'invalido' });
		});

		it('formatosDePonto contém x;y e não x,y no modo virgula', () => {
			const fmt = formatosDePonto();
			expect(fmt).toContain('x;y');
			expect(fmt).not.toContain('x,y');
		});

		it('formatosDePonto contém x,y no modo ponto', () => {
			definirModoDecimal('ponto');
			const fmt = formatosDePonto();
			expect(fmt).toContain('x,y');
		});
	});

	describe('D9: formatação', () => {
		it('formata ponto e número no modo ponto', () => {
			definirModoDecimal('ponto');
			expect(formatarPonto({ x: 1.5, y: -2 })).toBe('1.5,-2');
			expect(formatarNumero(0.25)).toBe('0.25');
		});

		it('formata ponto e número no modo virgula', () => {
			definirModoDecimal('virgula');
			expect(formatarPonto({ x: 1.5, y: -2 })).toBe('1,5;-2');
			expect(formatarNumero(0.25)).toBe('0,25');
			expect(formatarNumero(10)).toBe('10');
		});

		it('ida e volta no modo virgula preserva o ponto exato', () => {
			definirModoDecimal('virgula');
			const p = { x: 0.1 + 0.2, y: -1e-7 };
			const r = lerPonto(formatarPonto(p), null);
			expect(r).toEqual({ tipo: 'ponto', ponto: p });
		});
	});

	it('definirModoDecimal altera o retorno de modoDecimal', () => {
		definirModoDecimal('virgula');
		expect(modoDecimal()).toBe('virgula');
	});
});
