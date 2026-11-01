// Caminho relativo: src/lib/render/cor.spec.ts

import { describe, expect, it } from 'vitest';
import { corDaCamada } from './cor';

const CLARO = { escuro: false };
const ESCURO = { escuro: true };

describe('cor da camada', () => {
	it('as nove cores básicas do índice', () => {
		expect(corDaCamada({ kind: 'index', index: 1 }, ESCURO)).toEqual([255, 0, 0]);
		expect(corDaCamada({ kind: 'index', index: 2 }, ESCURO)).toEqual([255, 255, 0]);
		expect(corDaCamada({ kind: 'index', index: 3 }, ESCURO)).toEqual([0, 255, 0]);
		expect(corDaCamada({ kind: 'index', index: 4 }, ESCURO)).toEqual([0, 255, 255]);
		expect(corDaCamada({ kind: 'index', index: 5 }, ESCURO)).toEqual([0, 0, 255]);
		expect(corDaCamada({ kind: 'index', index: 6 }, ESCURO)).toEqual([255, 0, 255]);
		expect(corDaCamada({ kind: 'index', index: 8 }, ESCURO)).toEqual([128, 128, 128]);
		expect(corDaCamada({ kind: 'index', index: 9 }, ESCURO)).toEqual([192, 192, 192]);
	});

	it('a cor 7 é branca no fundo escuro e preta no claro', () => {
		expect(corDaCamada({ kind: 'index', index: 7 }, ESCURO)).toEqual([255, 255, 255]);
		expect(corDaCamada({ kind: 'index', index: 7 }, CLARO)).toEqual([0, 0, 0]);
	});

	it('a faixa 10–249 segue o padrão de matiz e brilho do índice', () => {
		expect(corDaCamada({ kind: 'index', index: 10 }, ESCURO)).toEqual([255, 0, 0]);
		expect(corDaCamada({ kind: 'index', index: 11 }, ESCURO)).toEqual([255, 127, 127]);
		expect(corDaCamada({ kind: 'index', index: 12 }, ESCURO)).toEqual([165, 0, 0]);
		expect(corDaCamada({ kind: 'index', index: 18 }, ESCURO)).toEqual([38, 0, 0]);
		// Matiz 120°: verde.
		expect(corDaCamada({ kind: 'index', index: 90 }, ESCURO)).toEqual([0, 255, 0]);
	});

	it('os cinzas de 250 a 255', () => {
		expect(corDaCamada({ kind: 'index', index: 250 }, ESCURO)).toEqual([51, 51, 51]);
		expect(corDaCamada({ kind: 'index', index: 255 }, ESCURO)).toEqual([255, 255, 255]);
	});

	it('rgb passa direto; por camada, por bloco e índice inválido usam a cor do traço', () => {
		expect(corDaCamada({ kind: 'rgb', red: 10, green: 20, blue: 30 }, ESCURO)).toEqual([
			10, 20, 30
		]);
		for (const cor of [
			{ kind: 'byLayer' as const },
			{ kind: 'byBlock' as const },
			{ kind: 'index' as const, index: 0 },
			{ kind: 'index' as const, index: 300 }
		]) {
			expect(corDaCamada(cor, ESCURO)).toEqual([255, 255, 255]);
			expect(corDaCamada(cor, CLARO)).toEqual([0, 0, 0]);
		}
	});
});
