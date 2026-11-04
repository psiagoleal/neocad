// Caminho relativo: src/lib/command-line/transformar.spec.ts

/**
 * \file src/lib/command-line/transformar.spec.ts
 * \brief ROTATE, MIRROR e SCALE pela linha de comando (spec 10, R1–R4).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import { documentoFalso, selecaoFalsa } from './documento-falso';
import { escalar } from './escalar';
import { espelhar } from './espelhar';
import { girar } from './girar';
import { MotorDeComandos, type Mensagem } from './motor';
import { lerNumero } from './pontos';
import type { ComandoDoCatalogo } from './resolver';

const CATALOGO: ComandoDoCatalogo[] = [
	{ en: 'ROTATE', ptbr: 'ROTAC', apelidos: ['RO'] },
	{ en: 'MIRROR', ptbr: 'ESPELHAR', apelidos: ['MI'] },
	{ en: 'SCALE', ptbr: 'ESCALA', apelidos: ['SC'] }
];

function montar() {
	const falso = documentoFalso();
	const a = falso.documento.drawLine('7' as never, { x: 1, y: 0 }, { x: 2, y: 0 });
	const passos = falso.pilha.length;
	const sel = selecaoFalsa([a]);
	const mensagens: Mensagem[] = [];
	const motor = new MotorDeComandos({
		catalogo: CATALOGO,
		comandos: { ROTATE: girar, MIRROR: espelhar, SCALE: escalar },
		documento: () => falso.documento,
		aposEditar: vi.fn(),
		selecao: sel.selecao,
		mensagem: (m) => mensagens.push(m)
	});
	const digitar = (...t: string[]) => t.forEach((x) => motor.confirmar(x));
	const reta = (id = a) => falso.retas.find((r) => r.id === id);
	const erro = () => mensagens.filter((m) => m.tipo === 'erro').at(-1)?.texto;
	return { motor, falso, a, passos, sel, digitar, reta, erro };
}

const perto = (p: { x: number; y: number } | undefined, x: number, y: number) => {
	expect(p?.x).toBeCloseTo(x, 9);
	expect(p?.y).toBeCloseTo(y, 9);
};

describe('lerNumero', () => {
	it('aceita o número do AutoCAD e recusa o resto', () => {
		expect(lerNumero(' -2.5 ')).toBe(-2.5);
		expect(lerNumero('1e2')).toBe(100);
		for (const t of ['', 'abc', '1,5', '1..2', 'NaN', 'Infinity']) expect(lerNumero(t)).toBeNull();
	});
});

describe('ROTATE (R1)', () => {
	it('gira em graus anti-horários em torno da base, num passo', () => {
		const { motor, falso, passos, sel, digitar, reta } = montar();
		digitar('ROTAC');
		expect(motor.prompt).toBe('Especifique o ponto base:');
		digitar('0,0');
		expect(motor.prompt).toBe('Especifique o ângulo de rotação:');
		digitar('90');
		perto(reta()?.de, 0, 1);
		perto(reta()?.ate, 0, 2);
		expect(falso.pilha).toHaveLength(passos + 1);
		expect(sel.estado.ids).toEqual([]);
		expect(motor.emCurso).toBeNull();
	});

	it('o ângulo também vale como ponto: a direção da base até ele', () => {
		const { digitar, reta } = montar();
		digitar('RO', '0,0', '-5,0');
		perto(reta()?.de, -1, 0);
	});
});

describe('MIRROR (R2)', () => {
	it('com Não (o padrão), as originais ficam e entram as cópias espelhadas, num passo', () => {
		const { motor, falso, a, passos, digitar, reta } = montar();
		digitar('ESPELHAR');
		expect(motor.prompt).toBe('Especifique o primeiro ponto da linha de espelhamento:');
		digitar('0,0');
		expect(motor.prompt).toBe('Especifique o segundo ponto da linha de espelhamento:');
		digitar('0,10');
		expect(motor.prompt).toBe('Apagar objetos de origem? [Sim/Não] <Não>:');
		digitar('');
		perto(reta(a)?.de, 1, 0);
		const copia = falso.retas.find((r) => r.id !== a);
		perto(copia?.de, -1, 0);
		expect(falso.pilha).toHaveLength(passos + 1);
		expect(motor.emCurso).toBeNull();
	});

	it('com Sim, as originais são espelhadas', () => {
		const { falso, digitar, reta } = montar();
		digitar('MI', '0,0', '0,10', 'S');
		expect(falso.retas).toHaveLength(1);
		perto(reta()?.de, -1, 0);
	});

	it('aceita Não por extenso e _Y em inglês', () => {
		const um = montar();
		um.digitar('MI', '0,0', '0,10', 'não');
		expect(um.falso.retas).toHaveLength(2);
		const dois = montar();
		dois.digitar('MI', '0,0', '0,10', '_Y');
		expect(dois.falso.retas).toHaveLength(1);
	});
});

describe('SCALE (R3)', () => {
	it('escala em torno da base, num passo', () => {
		const { motor, falso, passos, digitar, reta } = montar();
		digitar('ESCALA', '0,0');
		expect(motor.prompt).toBe('Especifique o fator de escala:');
		digitar('3');
		perto(reta()?.de, 3, 0);
		perto(reta()?.ate, 6, 0);
		expect(falso.pilha).toHaveLength(passos + 1);
	});
});

describe('entradas inválidas e Esc (R4)', () => {
	it('ângulo que não é número nem ponto repete o prompt', () => {
		const { motor, digitar, erro } = montar();
		digitar('RO', '0,0', 'abc');
		expect(erro()).toBe('Esperava um ângulo em graus ou um ponto.');
		expect(motor.prompt).toBe('Especifique o ângulo de rotação:');
	});

	it('fator zero, negativo ou que não é número repete o prompt', () => {
		const { motor, falso, passos, digitar, erro } = montar();
		for (const t of ['0', '-2', 'x']) {
			digitar(t === '0' ? 'SC' : '', ...(t === '0' ? ['0,0', t] : [t]));
		}
		expect(erro()).toBe('Esperava um fator de escala maior que zero.');
		expect(motor.prompt).toBe('Especifique o fator de escala:');
		expect(falso.pilha).toHaveLength(passos);
	});

	it('os dois pontos da linha de espelhamento iguais repetem o prompt', () => {
		const { motor, digitar, erro } = montar();
		digitar('MI', '1,1', '1,1');
		expect(erro()).toBe('O segundo ponto precisa ser diferente do primeiro.');
		expect(motor.prompt).toBe('Especifique o segundo ponto da linha de espelhamento:');
	});

	it('Esc antes da última resposta não muda nada', () => {
		const { motor, falso, passos, digitar } = montar();
		digitar('MI', '0,0', '0,10');
		motor.cancelar();
		digitar('RO', '0,0');
		motor.cancelar();
		expect(falso.pilha).toHaveLength(passos);
	});
});
