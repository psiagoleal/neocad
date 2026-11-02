// Caminho relativo: src/lib/command-line/deslocar.spec.ts

/**
 * \file src/lib/command-line/deslocar.spec.ts
 * \brief OFFSET pela linha de comando, com o objeto escolhido pelo clique (spec 10, O1–O4).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import { deslocar } from './deslocar';
import { documentoFalso, selecaoFalsa } from './documento-falso';
import { MotorDeComandos, type Mensagem } from './motor';
import type { ComandoDoCatalogo } from './resolver';

const CATALOGO: ComandoDoCatalogo[] = [
	{ en: 'OFFSET', ptbr: 'DESLOCAMENTO', apelidos: ['O'] },
	{ en: 'LINE', ptbr: 'LINHA', apelidos: ['L'] }
];
const DISTANCIA = 'Especifique a distância de deslocamento:';
const OBJETO = 'Selecione o objeto a deslocar ou [Sair/Desfazer] <Sair>:';
const LADO = 'Especifique o ponto no lado a deslocar:';

function montar() {
	const falso = documentoFalso();
	const a = falso.documento.drawLine('7' as never, { x: 0, y: 0 }, { x: 10, y: 0 });
	const passos = falso.pilha.length;
	const sel = selecaoFalsa();
	const mensagens: Mensagem[] = [];
	const motor = new MotorDeComandos({
		catalogo: CATALOGO,
		comandos: { OFFSET: deslocar },
		documento: () => falso.documento,
		aposEditar: vi.fn(),
		selecao: sel.selecao,
		mensagem: (m) => mensagens.push(m)
	});
	const digitar = (...t: string[]) => t.forEach((x) => motor.confirmar(x));
	/** O clique na tela: a seleção muda e o motor é avisado. */
	const clicar = (id: string) => {
		sel.estado.ids = [...sel.estado.ids, id as never];
		motor.selecaoMudou();
	};
	const erro = () => mensagens.filter((m) => m.tipo === 'erro').at(-1)?.texto;
	return { motor, falso, a, passos, sel, digitar, clicar, erro };
}

describe('OFFSET (spec 10)', () => {
	it('distância, clique no objeto, lado digitado; cria a paralela e volta a pedir objeto (O1)', () => {
		const { motor, falso, a, passos, sel, digitar, clicar } = montar();
		digitar('DESLOCAMENTO');
		expect(motor.prompt).toBe(DISTANCIA);
		digitar('2.5');
		expect(motor.prompt).toBe(OBJETO);
		clicar(a);
		expect(motor.prompt).toBe(LADO);
		digitar('5,-1');
		expect(falso.retas.map((r) => r.de.y)).toEqual([0, -2.5]);
		expect(falso.pilha).toHaveLength(passos + 1);
		expect(motor.prompt).toBe(OBJETO);
		expect(sel.estado.ids).toEqual([]);
	});

	it('Desfazer tira a última paralela; o vazio encerra (O2)', () => {
		const { motor, falso, a, digitar, clicar, erro } = montar();
		digitar('O', '1');
		digitar('D');
		expect(erro()).toBe('Não há paralela para desfazer.');
		clicar(a);
		digitar('0,1');
		clicar(a);
		digitar('0,1');
		expect(falso.retas).toHaveLength(3);
		digitar('_U');
		expect(falso.retas).toHaveLength(2);
		digitar('');
		expect(motor.emCurso).toBeNull();
	});

	it('Sair encerra', () => {
		const { motor, digitar } = montar();
		digitar('O', '1', 'S');
		expect(motor.emCurso).toBeNull();
	});

	it('a recusa do kernel vira mensagem e volta a pedir objeto, sem mudar nada (O3)', () => {
		const { motor, falso, a, passos, digitar, clicar, erro } = montar();
		digitar('O', '500');
		clicar(a);
		digitar('0,1');
		expect(erro()).toBe('O kernel recusou: paralela impossível.');
		expect(motor.prompt).toBe(OBJETO);
		expect(falso.pilha).toHaveLength(passos);
	});

	it('distância que não é número maior que zero repete o pedido (O4)', () => {
		const { motor, digitar, erro } = montar();
		digitar('O', '0');
		expect(erro()).toBe('Esperava uma distância maior que zero.');
		digitar('abc');
		expect(motor.prompt).toBe(DISTANCIA);
	});

	it('texto no pedido de objeto diz que o objeto se escolhe na tela', () => {
		const { motor, digitar, erro } = montar();
		digitar('O', '1', 'abc');
		expect(erro()).toBe('Clique no objeto a deslocar, na tela.');
		expect(motor.prompt).toBe(OBJETO);
	});

	it('mudar a seleção sem comando que a queira não faz nada', () => {
		const { motor, falso, a, clicar, passos } = montar();
		clicar(a);
		motor.confirmar('L');
		clicar(a);
		expect(motor.emCurso).toBeNull();
		expect(falso.pilha).toHaveLength(passos);
	});
});
