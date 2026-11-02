// Caminho relativo: src/lib/command-line/pontos-clique.spec.ts

/**
 * \file src/lib/command-line/pontos-clique.spec.ts
 * \brief Quais prompts pedem ponto pelo clique, e o ponto clicado como texto (spec 10, P1, P3).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import { CATALOGO_DA_SPEC } from './catalogo';
import { COMANDOS } from './comandos';
import { documentoFalso, selecaoFalsa } from './documento-falso';
import { MotorDeComandos } from './motor';
import { formatarPonto, lerPonto } from './pontos';

function montar() {
	const falso = documentoFalso();
	const a = falso.documento.drawLine('7' as never, { x: 0, y: 0 }, { x: 10, y: 0 });
	const sel = selecaoFalsa([a], [a]);
	const motor = new MotorDeComandos({
		catalogo: CATALOGO_DA_SPEC,
		comandos: COMANDOS,
		documento: () => falso.documento,
		aposEditar: vi.fn(),
		selecao: sel.selecao,
		mensagem: () => undefined
	});
	/** O modo do clique a cada passo: 'ponto', 'escolha' ou 'nada'. */
	const modo = () => (motor.querPonto ? 'ponto' : motor.querEscolha ? 'escolha' : 'nada');
	const passos = (...textos: string[]) => [
		modo(),
		...textos.map((t) => (motor.confirmar(t), modo()))
	];
	return { motor, sel, passos };
}

describe('quais prompts pedem ponto pelo clique (P1)', () => {
	it.each([
		['LINE: os pontos', ['L', '0,0', '1,0'], ['nada', 'ponto', 'ponto', 'ponto']],
		['MOVE: base e segundo ponto', ['M', '0,0'], ['nada', 'ponto', 'ponto']],
		['COPY: base e segundos pontos', ['CO', '0,0', '1,0'], ['nada', 'ponto', 'ponto', 'ponto']],
		['ROTATE: base e ângulo', ['RO', '0,0'], ['nada', 'ponto', 'ponto']],
		['SCALE: base, não o fator', ['SC', '0,0'], ['nada', 'ponto', 'nada']],
		[
			'MIRROR: os dois pontos, não o Sim/Não',
			['MI', '0,0', '0,1'],
			['nada', 'ponto', 'ponto', 'nada']
		],
		['OFFSET: só o lado', ['O', '1'], ['nada', 'nada', 'escolha']],
		['TRIM: objeto por escolha', ['TR', ''], ['nada', 'nada', 'escolha']]
	])('%s', (_nome, textos, esperado) => {
		const { passos } = montar();
		expect(passos(...textos)).toEqual(esperado);
	});

	it('sem seleção, o pedido de objetos não pede ponto; depois dele, sim', () => {
		const { motor, sel, passos } = montar();
		sel.estado.ids = [];
		expect(passos('M')).toEqual(['nada', 'nada']);
		sel.estado.ids = ['1' as never];
		motor.confirmar('');
		expect(motor.querPonto).toBe(true);
	});

	it('o lado do OFFSET pede ponto depois do clique no objeto', () => {
		const { motor, passos } = montar();
		passos('O', '1');
		motor.escolher({ ids: ['1' as never], ponto: { x: 5, y: 0 }, raio: 0.5 });
		expect(motor.querPonto).toBe(true);
	});
});

describe('formatarPonto (P3)', () => {
	it.each([
		{ x: 0, y: 0 },
		{ x: 1 / 3, y: -2 / 3 },
		{ x: 1e-7, y: 5_000_000.1 },
		{ x: -0.000123456789, y: 123456789.987654 }
	])('lido de volta, é o mesmo ponto: %o', (ponto) => {
		const texto = formatarPonto(ponto);
		const lido = lerPonto(texto, null);
		expect(lido).toEqual({ tipo: 'ponto', ponto });
	});
});
