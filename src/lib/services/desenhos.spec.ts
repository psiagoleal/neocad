// Caminho relativo: src/lib/services/desenhos.spec.ts

/**
 * \file src/lib/services/desenhos.spec.ts
 * \brief Nome do próximo desenho novo e a marca de alterado (spec 11, I4).
 * \author Iago Leal
 * \date 2026-11-04
 */

import { describe, expect, it } from 'vitest';
import { SessaoDeDesenhos } from './desenhos';

describe('SessaoDeDesenhos', () => {
	it('numera os desenhos novos da sessão a partir de 1', () => {
		const s = new SessaoDeDesenhos();
		expect(s.criar()).toBe('Desenho1');
		expect(s.criar()).toBe('Desenho2');
	});

	it('perguntar não gasta o número: só criar gasta', () => {
		const s = new SessaoDeDesenhos();
		s.criar();
		s.marcarAlterado();
		expect(s.precisaConfirmar()).toBe(true);
		expect(s.precisaConfirmar()).toBe(true);
		expect(s.criar()).toBe('Desenho2');
	});

	it('alterado desde que abriu, foi criado ou foi gravado', () => {
		const s = new SessaoDeDesenhos();
		s.criar();
		expect(s.precisaConfirmar()).toBe(false);
		s.marcarAlterado();
		expect(s.precisaConfirmar()).toBe(true);
		s.marcarGravado();
		expect(s.precisaConfirmar()).toBe(false);
		s.marcarAlterado();
		s.marcarAberto();
		expect(s.precisaConfirmar()).toBe(false);
		s.marcarAlterado();
		s.criar();
		expect(s.precisaConfirmar()).toBe(false);
	});
});
