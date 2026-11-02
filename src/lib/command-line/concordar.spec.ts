// Caminho relativo: src/lib/command-line/concordar.spec.ts

/**
 * \file src/lib/command-line/concordar.spec.ts
 * \brief FILLET e CHAMFER pela linha de comando (spec 10, F1–F4).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import { criarChanfrar, criarConcordar } from './concordar';
import { documentoFalso, selecaoFalsa } from './documento-falso';
import { MotorDeComandos, type Mensagem } from './motor';
import type { ComandoDoCatalogo } from './resolver';

const CATALOGO: ComandoDoCatalogo[] = [
	{ en: 'FILLET', ptbr: 'CONCORD', apelidos: ['F'] },
	{ en: 'CHAMFER', ptbr: 'CHANFRO', apelidos: ['CHA'] }
];
const PRIMEIRO = 'Selecione o primeiro objeto ou [Raio]:';
const SEGUNDO = 'Selecione o segundo objeto:';

function montar() {
	const falso = documentoFalso();
	const passos = falso.pilha.length;
	const mensagens: Mensagem[] = [];
	const motor = new MotorDeComandos({
		catalogo: CATALOGO,
		comandos: { FILLET: criarConcordar(), CHAMFER: criarChanfrar() },
		documento: () => falso.documento,
		aposEditar: vi.fn(),
		selecao: selecaoFalsa().selecao,
		mensagem: (m) => mensagens.push(m)
	});
	const digitar = (...t: string[]) => t.forEach((x) => motor.confirmar(x));
	const clicar = (id: string, x: number, y: number) =>
		motor.escolher({ ids: id === '' ? [] : [id as never], ponto: { x, y }, raio: 0.5 });
	const erro = () => mensagens.filter((m) => m.tipo === 'erro').at(-1)?.texto;
	return { motor, falso, passos, digitar, clicar, erro };
}

describe('FILLET (F1, F2)', () => {
	it('dois cliques concordam com o raio atual (0 de início), num passo, e encerra', () => {
		const { motor, falso, passos, digitar, clicar } = montar();
		digitar('CONCORD');
		expect(motor.prompt).toBe(PRIMEIRO);
		expect(motor.querEscolha).toBe(true);
		clicar('a', 1, 2);
		expect(motor.prompt).toBe(SEGUNDO);
		clicar('b', 3, 4);
		expect(falso.chamadas).toEqual([
			{ nome: 'concordar', args: ['a', { x: 1, y: 2 }, 'b', { x: 3, y: 4 }, 0] }
		]);
		expect(falso.pilha).toHaveLength(passos + 1);
		expect(motor.emCurso).toBeNull();
	});

	it('Raio muda o raio da sessão; o vazio mantém', () => {
		const { motor, falso, digitar, clicar } = montar();
		digitar('F', 'R');
		expect(motor.prompt).toBe('Especifique o raio de concordância <0>:');
		expect(motor.querEscolha).toBe(false);
		digitar('2.5');
		expect(motor.prompt).toBe(PRIMEIRO);
		clicar('a', 0, 0);
		clicar('b', 0, 0);
		digitar('F', '_R');
		expect(motor.prompt).toBe('Especifique o raio de concordância <2.5>:');
		digitar('');
		clicar('a', 0, 0);
		clicar('b', 0, 0);
		expect(falso.chamadas.map((c) => c.args[4])).toEqual([2.5, 2.5]);
	});

	it('raio negativo ou que não é número repete o pedido', () => {
		const { motor, digitar, erro } = montar();
		digitar('F', 'R', '-1');
		expect(erro()).toBe('Esperava um raio maior ou igual a zero.');
		expect(motor.prompt).toBe('Especifique o raio de concordância <0>:');
	});
});

describe('CHAMFER (F3)', () => {
	it('Distância pede as duas; o vazio na segunda mantém a atual; dois cliques chanfram', () => {
		const { motor, falso, digitar, clicar } = montar();
		digitar('CHANFRO');
		expect(motor.prompt).toBe('Selecione a primeira linha ou [Distância]:');
		digitar('D');
		expect(motor.prompt).toBe('Especifique a primeira distância de chanfro <0>:');
		digitar('3');
		expect(motor.prompt).toBe('Especifique a segunda distância de chanfro <0>:');
		digitar('4');
		expect(motor.prompt).toBe('Selecione a primeira linha ou [Distância]:');
		clicar('a', 1, 0);
		expect(motor.prompt).toBe('Selecione a segunda linha:');
		clicar('b', 0, 1);
		expect(falso.chamadas[0]).toEqual({
			nome: 'chanfrar',
			args: ['a', { x: 1, y: 0 }, 'b', { x: 0, y: 1 }, 3, 4]
		});
		digitar('CHA', '_D', '', '');
		expect(motor.prompt).toBe('Selecione a primeira linha ou [Distância]:');
		clicar('a', 0, 0);
		clicar('b', 0, 0);
		expect(falso.chamadas[1].args.slice(4)).toEqual([3, 4]);
	});
});

describe('recusas (F4)', () => {
	it('a recusa do kernel vira mensagem e volta a pedir o primeiro objeto', () => {
		const { motor, falso, passos, digitar, clicar, erro } = montar();
		digitar('F', 'R', '99');
		clicar('a', 0, 0);
		clicar('b', 0, 0);
		expect(erro()).toBe('O kernel recusou: o raio ou a distância não cabe.');
		expect(motor.prompt).toBe(PRIMEIRO);
		expect(falso.pilha).toHaveLength(passos);
	});

	it('clique no vazio diz que não pegou nada', () => {
		const { motor, digitar, clicar, erro } = montar();
		digitar('F');
		clicar('', 5, 5);
		expect(erro()).toBe('Nenhum objeto sob o clique.');
		expect(motor.prompt).toBe(PRIMEIRO);
	});

	it('texto no pedido de objeto diz que o objeto se escolhe na tela', () => {
		const { digitar, erro } = montar();
		digitar('F', 'abc');
		expect(erro()).toBe('Clique no objeto, na tela, ou escolha uma opção.');
	});
});
