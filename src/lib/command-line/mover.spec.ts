// Caminho relativo: src/lib/command-line/mover.spec.ts

/**
 * \file src/lib/command-line/mover.spec.ts
 * \brief MOVE e COPY pela linha de comando (spec 10, M1–M4).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import { copiar } from './copiar';
import { documentoFalso, selecaoFalsa } from './documento-falso';
import { MotorDeComandos, type Mensagem } from './motor';
import { mover } from './mover';
import type { ComandoDoCatalogo } from './resolver';

const CATALOGO: ComandoDoCatalogo[] = [
	{ en: 'MOVE', ptbr: 'MOVER', apelidos: ['M'] },
	{ en: 'COPY', ptbr: 'COPIAR', apelidos: ['CO'] }
];
const BASE = 'Especifique o ponto base:';
const SEGUNDO = 'Especifique o segundo ponto ou <usar o primeiro ponto como deslocamento>:';
const SEGUNDO_DE_NOVO = 'Especifique o segundo ponto ou [Sair/Desfazer] <Sair>:';

function montar() {
	const falso = documentoFalso();
	const a = falso.documento.drawLine('7' as never, { x: 0, y: 0 }, { x: 1, y: 0 });
	const b = falso.documento.drawLine('7' as never, { x: 0, y: 5 }, { x: 1, y: 5 });
	const passos = falso.pilha.length;
	const sel = selecaoFalsa();
	const mensagens: Mensagem[] = [];
	const aposEditar = vi.fn();
	const motor = new MotorDeComandos({
		catalogo: CATALOGO,
		comandos: { MOVE: mover, COPY: copiar },
		documento: () => falso.documento,
		aposEditar,
		selecao: sel.selecao,
		mensagem: (m) => mensagens.push(m)
	});
	const digitar = (...t: string[]) => t.forEach((x) => motor.confirmar(x));
	const inicio = (id: string) => falso.retas.find((r) => r.id === id)?.de;
	return { motor, falso, a, b, passos, sel, mensagens, aposEditar, digitar, inicio };
}

describe('MOVE (spec 10)', () => {
	it('com seleção, pede base e segundo ponto e move num só passo (M1)', () => {
		const { motor, falso, a, b, passos, sel, digitar, inicio, aposEditar } = montar();
		sel.estado.ids = [a];
		digitar('MOVER');
		expect(motor.prompt).toBe(BASE);
		digitar('10,10');
		expect(motor.prompt).toBe(SEGUNDO);
		digitar('13,14');
		expect(inicio(a)).toEqual({ x: 3, y: 4 });
		expect(inicio(b)).toEqual({ x: 0, y: 5 });
		expect(falso.pilha).toHaveLength(passos + 1);
		expect(sel.estado.ids).toEqual([]);
		expect(aposEditar).toHaveBeenCalledTimes(1);
		expect(motor.emCurso).toBeNull();
	});

	it('sem seleção, pede objetos e segue ao confirmar o vazio (M1)', () => {
		const { motor, a, sel, digitar, inicio } = montar();
		digitar('M');
		expect(motor.prompt).toBe('Selecione objetos:');
		sel.estado.ids = [a];
		digitar('', '0,0', '@2,0');
		expect(inicio(a)).toEqual({ x: 2, y: 0 });
	});

	it('o @ do segundo ponto parte da base; o vazio usa a base como deslocamento (M2)', () => {
		const { a, b, sel, digitar, inicio } = montar();
		sel.estado.ids = [a];
		digitar('MOVE', '5,5', '@1,1');
		expect(inicio(a)).toEqual({ x: 1, y: 1 });
		sel.estado.ids = [b];
		digitar('MOVE', '2,3', '');
		expect(inicio(b)).toEqual({ x: 2, y: 8 });
	});

	it('sem nada selecionado, o vazio encerra sem mudar nada (M4)', () => {
		const { motor, falso, passos, digitar } = montar();
		digitar('M', '');
		expect(motor.emCurso).toBeNull();
		expect(falso.pilha).toHaveLength(passos);
	});

	it('ponto inválido diz o esperado e repete o prompt', () => {
		const { motor, a, sel, digitar, mensagens } = montar();
		sel.estado.ids = [a];
		digitar('M', 'abc');
		expect(mensagens.at(-1)).toEqual({
			tipo: 'erro',
			texto: 'Esperava um ponto: x,y, @dx,dy, d<a ou @d<a.'
		});
		expect(motor.prompt).toBe(BASE);
	});
});

describe('COPY (spec 10, M3)', () => {
	it('cada segundo ponto é uma cópia e um passo; o vazio encerra', () => {
		const { motor, falso, a, passos, sel, digitar } = montar();
		sel.estado.ids = [a];
		digitar('COPIAR', '0,0', '10,0');
		expect(motor.prompt).toBe(SEGUNDO_DE_NOVO);
		digitar('20,0');
		expect(falso.retas.map((r) => r.de.x).sort((x, y) => x - y)).toEqual([0, 0, 10, 20]);
		expect(falso.pilha).toHaveLength(passos + 2);
		digitar('');
		expect(motor.emCurso).toBeNull();
		expect(sel.estado.ids).toEqual([]);
	});

	it('Desfazer tira a última cópia; Sair encerra', () => {
		const { motor, falso, a, sel, digitar } = montar();
		sel.estado.ids = [a];
		digitar('CO', '0,0', '10,0', '20,0', 'D');
		expect(falso.retas).toHaveLength(3);
		digitar('_U');
		expect(falso.retas).toHaveLength(2);
		digitar('_E');
		expect(motor.emCurso).toBeNull();
	});

	it('o vazio na primeira cópia usa a base como deslocamento e encerra', () => {
		const { motor, falso, a, sel, digitar } = montar();
		sel.estado.ids = [a];
		digitar('COPY', '3,4', '');
		expect(falso.retas.map((r) => r.de)).toContainEqual({ x: 3, y: 4 });
		expect(motor.emCurso).toBeNull();
	});
});
