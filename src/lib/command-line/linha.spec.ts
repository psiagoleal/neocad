// Caminho relativo: src/lib/command-line/linha.spec.ts

/**
 * \file src/lib/command-line/linha.spec.ts
 * \brief O LINE pela linha de comando, sobre um documento falso (spec 10, D1–D6).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import { documentoFalso } from './documento-falso';
import { linha } from './linha';
import { MotorDeComandos, type Mensagem } from './motor';
import type { ComandoDoCatalogo } from './resolver';

const CATALOGO: ComandoDoCatalogo[] = [{ en: 'LINE', ptbr: 'LINHA', apelidos: ['L'] }];

function montar(camadas?: Array<{ id: string; name: string }>) {
	const falso = documentoFalso(camadas);
	const mensagens: Mensagem[] = [];
	const aposEditar = vi.fn();
	const motor = new MotorDeComandos({
		catalogo: CATALOGO,
		comandos: { LINE: linha },
		documento: () => falso.documento,
		aposEditar,
		mensagem: (m) => mensagens.push(m)
	});
	const digitar = (...textos: string[]) => textos.forEach((t) => motor.confirmar(t));
	const segmentos = () => falso.retas.map((r) => [r.de.x, r.de.y, r.ate.x, r.ate.y]);
	const erros = () => mensagens.filter((m) => m.tipo === 'erro').map((m) => m.texto);
	return { motor, digitar, segmentos, erros, falso, aposEditar };
}

describe('LINE (spec 10)', () => {
	it('pede o primeiro ponto e cria uma reta a cada ponto seguinte, na camada 0 (D1)', () => {
		const { motor, digitar, segmentos, falso, aposEditar } = montar([
			{ id: '3', name: 'Paredes' },
			{ id: '7', name: '0' }
		]);
		digitar('L');
		expect(motor.prompt).toBe('Especifique o primeiro ponto:');
		digitar('0,0');
		expect(motor.prompt).toBe('Especifique o próximo ponto ou [Desfazer]:');
		expect(segmentos()).toEqual([]);
		digitar('10,0');
		expect(motor.prompt).toBe('Especifique o próximo ponto ou [Desfazer]:');
		digitar('@0,5');
		expect(motor.prompt).toBe('Especifique o próximo ponto ou [Fechar/Desfazer]:');
		expect(segmentos()).toEqual([
			[0, 0, 10, 0],
			[10, 0, 10, 5]
		]);
		expect(falso.retas.every((r) => r.camada === '7')).toBe(true);
		expect(aposEditar).toHaveBeenCalledTimes(2);
	});

	it('sem camada 0, usa a primeira', () => {
		const { digitar, falso } = montar([{ id: '3', name: 'Paredes' }]);
		digitar('L', '0,0', '1,0');
		expect(falso.retas).toHaveLength(1);
		expect(falso.retas[0].camada).toBe('3');
	});

	it('polar relativo encadeia a partir do último ponto (D2)', () => {
		const { digitar, falso } = montar();
		digitar('LINHA', '1,1', '@2<0', '@2<90');
		expect(falso.retas).toHaveLength(2);
		const ultima = falso.retas[1].ate;
		expect(ultima.x).toBeCloseTo(3, 12);
		expect(ultima.y).toBeCloseTo(3, 12);
	});

	it('entrada que não é ponto nem opção diz o esperado e repete o prompt (D3)', () => {
		const { motor, digitar, erros } = montar();
		digitar('L', 'abc');
		expect(motor.emCurso).toBe('LINE');
		expect(motor.prompt).toBe('Especifique o primeiro ponto:');
		expect(erros()).toEqual(['Esperava um ponto: x,y, @dx,dy, d<a ou @d<a.']);

		digitar('@1,1');
		expect(erros().at(-1)).toBe('Não há ponto anterior para a coordenada relativa.');

		digitar('0,0', 'Z');
		expect(erros().at(-1)).toBe(
			'Esperava um ponto (x,y, @dx,dy, d<a ou @d<a) ou uma opção: Desfazer.'
		);
		expect(motor.prompt).toBe('Especifique o próximo ponto ou [Desfazer]:');
	});

	it('Desfazer tira a última reta e volta o ponto; sem reta, avisa (D4)', () => {
		const { motor, digitar, segmentos, erros, falso } = montar();
		digitar('L', '0,0', 'D');
		expect(erros().at(-1)).toBe('Não há reta para desfazer.');
		expect(motor.emCurso).toBe('LINE');

		digitar('10,0', '10,10', 'desfazer');
		expect(segmentos()).toEqual([[0, 0, 10, 0]]);
		expect(falso.pilha).toHaveLength(1);
		digitar('@0,-10');
		expect(segmentos()).toEqual([
			[0, 0, 10, 0],
			[10, 0, 10, -10]
		]);
		digitar('_U', 'undo');
		expect(segmentos()).toEqual([]);
	});

	it('Desfazer não desfaz edição alheia: se a pilha mudou, apaga a reta do LINE', () => {
		const { digitar, segmentos, falso } = montar();
		digitar('L', '0,0', '10,0');
		// Outra ação entrou na pilha no meio do comando (o menu Editar, por exemplo).
		falso.pilha.push(() => undefined);
		digitar('D');
		expect(segmentos()).toEqual([]);
		expect(falso.pilha).toHaveLength(3);
	});

	it('Fechar, com duas ou mais retas, fecha no primeiro ponto e encerra (D5)', () => {
		const { motor, digitar, segmentos, erros } = montar();
		digitar('L', '0,0', '10,0', 'F');
		expect(erros().at(-1)).toBe(
			'Esperava um ponto (x,y, @dx,dy, d<a ou @d<a) ou uma opção: Desfazer.'
		);
		digitar('10,10', '_C');
		expect(segmentos()).toEqual([
			[0, 0, 10, 0],
			[10, 0, 10, 10],
			[10, 10, 0, 0]
		]);
		expect(motor.emCurso).toBeNull();
	});

	it('texto vazio encerra e as retas ficam (D6)', () => {
		const { motor, digitar, segmentos } = montar();
		digitar('L', '0,0', '1,0', '');
		expect(motor.emCurso).toBeNull();
		expect(segmentos()).toHaveLength(1);
		digitar('L', '');
		expect(motor.emCurso).toBeNull();
	});

	it('Esc encerra e as retas ficam (D6)', () => {
		const { motor, digitar, segmentos } = montar();
		digitar('L', '0,0', '1,0', '2,0');
		motor.cancelar();
		expect(motor.emCurso).toBeNull();
		expect(segmentos()).toHaveLength(2);
	});

	it('o kernel recusa reta de comprimento zero: a linha avisa e o ponto não avança', () => {
		const { motor, digitar, segmentos, erros } = montar();
		digitar('L', '1,1', '1,1');
		expect(erros().at(-1)).toBe('O kernel recusou: reta de comprimento zero.');
		expect(motor.prompt).toBe('Especifique o próximo ponto ou [Desfazer]:');
		digitar('2,1');
		expect(segmentos()).toEqual([[1, 1, 2, 1]]);
	});
});
