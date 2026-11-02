// Caminho relativo: src/lib/command-line/apagar.spec.ts

/**
 * \file src/lib/command-line/apagar.spec.ts
 * \brief O ERASE pela linha de comando, sobre documento e seleção falsos (spec 10, E1, E2).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import { apagar } from './apagar';
import { documentoFalso, selecaoFalsa } from './documento-falso';
import { MotorDeComandos, type Mensagem } from './motor';
import type { ComandoDoCatalogo } from './resolver';

const CATALOGO: ComandoDoCatalogo[] = [{ en: 'ERASE', ptbr: 'APAGAR', apelidos: ['E'] }];

function montar() {
	const falso = documentoFalso();
	const ids = ['0,0', '0,1', '0,2'].map((p, i) =>
		falso.documento.drawLine('7' as never, { x: 0, y: i }, { x: 1, y: i })
	);
	const passos = falso.pilha.length;
	const sel = selecaoFalsa();
	const mensagens: Mensagem[] = [];
	const aposEditar = vi.fn();
	const motor = new MotorDeComandos({
		catalogo: CATALOGO,
		comandos: { ERASE: apagar },
		documento: () => falso.documento,
		aposEditar,
		selecao: sel.selecao,
		mensagem: (m) => mensagens.push(m)
	});
	return { motor, falso, ids, passos, sel, mensagens, aposEditar };
}

describe('ERASE (spec 10)', () => {
	it('com seleção, apaga tudo num só passo, esvazia a seleção e encerra sem prompt (E1)', () => {
		const { motor, falso, ids, passos, sel, aposEditar } = montar();
		sel.estado.ids = [ids[0], ids[2]];

		motor.confirmar('APAGAR');

		expect(falso.retas.map((r) => r.id)).toEqual([ids[1]]);
		expect(falso.pilha).toHaveLength(passos + 1);
		expect(sel.estado.ids).toEqual([]);
		expect(aposEditar).toHaveBeenCalledTimes(1);
		expect(motor.emCurso).toBeNull();
	});

	it('sem seleção, pede objetos; o vazio apaga o que foi selecionado e encerra (E2)', () => {
		const { motor, falso, ids, sel } = montar();
		motor.confirmar('E');
		expect(motor.emCurso).toBe('ERASE');
		expect(motor.prompt).toBe('Selecione objetos:');

		// O usuário seleciona na tela enquanto o prompt espera.
		sel.estado.ids = [ids[1]];
		motor.confirmar('');
		expect(falso.retas.map((r) => r.id)).toEqual([ids[0], ids[2]]);
		expect(sel.estado.ids).toEqual([]);
		expect(motor.emCurso).toBeNull();
	});

	it('sem nada selecionado, o vazio só encerra', () => {
		const { motor, falso, passos } = montar();
		motor.confirmar('ERASE');
		motor.confirmar('');
		expect(falso.pilha).toHaveLength(passos);
		expect(motor.emCurso).toBeNull();
	});

	it('texto no prompt de seleção diz o que se espera e repete', () => {
		const { motor, mensagens } = montar();
		motor.confirmar('E');
		motor.confirmar('TODOS');
		expect(mensagens.at(-1)).toEqual({
			tipo: 'erro',
			texto: 'Selecione objetos na tela; Enter apaga o que estiver selecionado.'
		});
		expect(motor.prompt).toBe('Selecione objetos:');
	});

	it('o kernel recusa: a linha avisa e nada é apagado', () => {
		const { motor, falso, sel, mensagens } = montar();
		sel.estado.ids = ['999' as never];
		motor.confirmar('E');
		expect(mensagens.at(-1)).toEqual({
			tipo: 'erro',
			texto: 'O kernel recusou: entidade inexistente.'
		});
		expect(falso.retas).toHaveLength(3);
	});
});
