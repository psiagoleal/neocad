// Caminho relativo: src/lib/command-line/aparar.spec.ts

/**
 * \file src/lib/command-line/aparar.spec.ts
 * \brief TRIM e EXTEND pela linha de comando (spec 10, T1–T4).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import { aparar, estender } from './aparar';
import { documentoFalso, selecaoFalsa } from './documento-falso';
import { MotorDeComandos, type Mensagem } from './motor';
import type { ComandoDoCatalogo } from './resolver';

const CATALOGO: ComandoDoCatalogo[] = [
	{ en: 'TRIM', ptbr: 'APARAR', apelidos: ['TR'] },
	{ en: 'EXTEND', ptbr: 'ESTENDER', apelidos: ['EX'] }
];

/** Reta horizontal de 0 a 10 em y = 0 e duas verticais, em x = 3 e x = 7, e uma em x = 12. */
function montar() {
	const falso = documentoFalso();
	const d = falso.documento;
	const h = d.drawLine('7' as never, { x: 0, y: 0 }, { x: 10, y: 0 });
	const v3 = d.drawLine('7' as never, { x: 3, y: -5 }, { x: 3, y: 5 });
	const v7 = d.drawLine('7' as never, { x: 7, y: -5 }, { x: 7, y: 5 });
	const v12 = d.drawLine('7' as never, { x: 12, y: -5 }, { x: 12, y: 5 });
	const passos = falso.pilha.length;
	const sel = selecaoFalsa([], [h, v3, v7, v12]);
	const mensagens: Mensagem[] = [];
	const motor = new MotorDeComandos({
		catalogo: CATALOGO,
		comandos: { TRIM: aparar, EXTEND: estender },
		documento: () => falso.documento,
		aposEditar: vi.fn(),
		selecao: sel.selecao,
		mensagem: (m) => mensagens.push(m)
	});
	const digitar = (...t: string[]) => t.forEach((x) => motor.confirmar(x));
	const clicar = (id: string, x: number, y = 0) =>
		motor.escolher({ ids: [id as never], ponto: { x, y }, raio: 0.5 });
	const horizontal = () =>
		falso.retas
			.filter((r) => r.de.y === 0 && r.ate.y === 0)
			.map((r) => [r.de.x, r.ate.x])
			.sort((a, b) => a[0] - b[0]);
	const erro = () => mensagens.filter((m) => m.tipo === 'erro').at(-1)?.texto;
	return { motor, falso, h, v3, v7, v12, passos, sel, digitar, clicar, horizontal, erro };
}

describe('TRIM (T1, T3, T4)', () => {
	it('arestas pela seleção; cada clique apara o trecho, num passo; o vazio encerra', () => {
		const { motor, falso, h, v3, v7, passos, sel, digitar, clicar, horizontal } = montar();
		digitar('APARAR');
		expect(motor.prompt).toBe('Selecione arestas de corte ou <selecionar todos>:');
		expect(motor.querEscolha).toBe(false);
		sel.estado.ids = [v3, v7];
		digitar('');
		expect(motor.prompt).toBe('Selecione o objeto a aparar ou [Desfazer]:');
		expect(motor.querEscolha).toBe(true);

		clicar(h, 5);
		expect(horizontal()).toEqual([
			[0, 3],
			[7, 10]
		]);
		expect(falso.pilha).toHaveLength(passos + 1);
		expect(motor.prompt).toBe('Selecione o objeto a aparar ou [Desfazer]:');
		digitar('');
		expect(motor.emCurso).toBeNull();
		expect(sel.estado.ids).toEqual([]);
	});

	it('sem seleção, o vazio toma todas as entidades do espaço como arestas', () => {
		const { h, digitar, clicar, horizontal } = montar();
		digitar('TR', '');
		clicar(h, 1);
		expect(horizontal()).toEqual([[3, 10]]);
	});

	it('Desfazer desfaz o último corte; sem corte, avisa', () => {
		const { h, digitar, clicar, horizontal, erro } = montar();
		digitar('TR', '', 'D');
		expect(erro()).toBe('Não há o que desfazer.');
		clicar(h, 5);
		digitar('_U');
		expect(horizontal()).toEqual([[0, 10]]);
	});

	it('a recusa do kernel vira mensagem e o pedido se repete', () => {
		const { motor, h, passos, falso, digitar, clicar, erro } = montar();
		digitar('TR', '');
		clicar(h, 5, 3);
		expect(erro()).toBe('O kernel recusou: o clique não está sobre o objeto.');
		expect(motor.prompt).toBe('Selecione o objeto a aparar ou [Desfazer]:');
		expect(falso.pilha).toHaveLength(passos);
	});

	it('clique no vazio, no pedido de objeto, diz que não pegou nada', () => {
		const { motor, digitar, erro } = montar();
		digitar('TR', '');
		motor.escolher({ ids: [], ponto: { x: 50, y: 50 }, raio: 0.5 });
		expect(erro()).toBe('Nenhum objeto sob o clique.');
	});
});

describe('EXTEND (T2)', () => {
	it('estende a ponta mais perto do clique até o contorno', () => {
		const { motor, h, v12, digitar, clicar, sel, falso } = montar();
		digitar('ESTENDER');
		expect(motor.prompt).toBe('Selecione arestas de contorno ou <selecionar todos>:');
		sel.estado.ids = [v12];
		digitar('');
		expect(motor.prompt).toBe('Selecione o objeto a estender ou [Desfazer]:');
		clicar(h, 9);
		expect(falso.retas.find((r) => r.id === h)?.ate).toEqual({ x: 12, y: 0 });
		digitar('');
		expect(motor.emCurso).toBeNull();
	});
});
