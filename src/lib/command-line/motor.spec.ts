// Caminho relativo: src/lib/command-line/motor.spec.ts

/**
 * \file src/lib/command-line/motor.spec.ts
 * \brief O motor da linha de comando (spec 08: C1, C1b, C5, C6, C7, C8, C10).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it, vi } from 'vitest';
import { CadKernelOperationError } from '$lib/services/cad-document';
import { documentoFalso, selecaoFalsa } from './documento-falso';
import {
	lerOpcao,
	MotorDeComandos,
	type ComandoEmCurso,
	type FabricaDeComando,
	type Mensagem,
	type Opcao
} from './motor';
import type { ComandoDoCatalogo } from './resolver';

const CATALOGO: ComandoDoCatalogo[] = [
	{ en: 'LINE', ptbr: 'LINHA', apelidos: ['L'] },
	{ en: 'HATCH', ptbr: 'HACHURA', apelidos: ['H'] }
];

/** Comando de teste: pede dois textos; "ruim" é inválido; "kernel" faz o kernel recusar. */
function comandoDeEco(registro: string[]): FabricaDeComando {
	return (contexto) => {
		let pedidos = 0;
		const comando: ComandoEmCurso = {
			prompt: () => `Pedido ${pedidos + 1}:`,
			responder: (texto) => {
				if (texto === 'ruim') return { tipo: 'invalida', mensagem: 'Esperava outra coisa.' };
				if (texto === 'kernel') throw new CadKernelOperationError('motivo simulado');
				registro.push(texto);
				contexto.aposEditar();
				pedidos += 1;
				return pedidos === 2 ? { tipo: 'fim' } : { tipo: 'continua' };
			},
			encerrar: () => registro.push('encerrado')
		};
		return comando;
	};
}

function montar(
	documento: ReturnType<typeof documentoFalso>['documento'] | null = documentoFalso().documento
) {
	const registro: string[] = [];
	const mensagens: Mensagem[] = [];
	const aposEditar = vi.fn();
	const fabrica = vi.fn(comandoDeEco(registro));
	const motor = new MotorDeComandos({
		catalogo: CATALOGO,
		comandos: { LINE: fabrica },
		documento: () => documento,
		aposEditar,
		selecao: selecaoFalsa().selecao,
		mensagem: (m) => mensagens.push(m)
	});
	return { motor, registro, mensagens, aposEditar, fabrica };
}

describe('motor: sem comando em curso', () => {
	it('o prompt é "Comando:" e o nome, em qualquer idioma, inicia o comando (C1)', () => {
		for (const texto of ['L', 'linha', 'LINE', '_LINE']) {
			const { motor, fabrica } = montar();
			expect(motor.prompt).toBe('Comando:');
			motor.confirmar(texto);
			expect(motor.emCurso, texto).toBe('LINE');
			expect(motor.prompt).toBe('Pedido 1:');
			expect(fabrica).toHaveBeenCalledTimes(1);
		}
	});

	it('o texto confirmado aparece no histórico depois do prompt', () => {
		const { motor, mensagens } = montar();
		motor.confirmar('L');
		expect(mensagens[0]).toEqual({ tipo: 'eco', texto: 'Comando: L' });
	});

	it('ainda não disponível e desconhecido viram mensagem, sem iniciar nada (C5)', () => {
		const { motor, mensagens, fabrica } = montar();
		motor.confirmar('hachura');
		motor.confirmar('xyzzy');
		expect(fabrica).not.toHaveBeenCalled();
		expect(motor.emCurso).toBeNull();
		expect(mensagens.filter((m) => m.tipo === 'erro').map((m) => m.texto)).toEqual([
			'hachura: ainda não disponível.',
			'xyzzy: comando desconhecido.'
		]);
	});

	it('sem desenho aberto, o comando não começa e a linha diz por quê', () => {
		const { motor, mensagens, fabrica } = montar(null);
		motor.confirmar('L');
		expect(fabrica).not.toHaveBeenCalled();
		expect(mensagens.at(-1)).toEqual({
			tipo: 'erro',
			texto: 'Abra um desenho antes de usar LINE.'
		});
	});

	it('vazio sem comando anterior não faz nada; vazio depois repete o último iniciado (C6)', () => {
		const { motor, mensagens, fabrica } = montar();
		motor.confirmar('');
		expect(mensagens).toEqual([]);
		motor.confirmar('hachura');
		motor.confirmar('');
		expect(fabrica).not.toHaveBeenCalled();

		motor.confirmar('L');
		motor.cancelar();
		motor.confirmar('');
		expect(fabrica).toHaveBeenCalledTimes(2);
		expect(motor.emCurso).toBe('LINE');
	});

	it('Esc sem comando em curso não emite nada (C7)', () => {
		const { motor, mensagens } = montar();
		motor.cancelar();
		expect(mensagens).toEqual([]);
	});
});

describe('motor: com comando em curso', () => {
	it('o texto vai ao comando, e não inicia outro, mesmo sendo nome de comando (C1b)', () => {
		const { motor, registro, fabrica } = montar();
		motor.confirmar('L');
		motor.confirmar('L');
		expect(fabrica).toHaveBeenCalledTimes(1);
		expect(registro).toEqual(['L']);
		motor.confirmar('fim');
		expect(motor.emCurso).toBeNull();
		expect(motor.prompt).toBe('Comando:');
	});

	it('resposta inválida diz o esperado e repete o prompt, sem encerrar (C1b)', () => {
		const { motor, mensagens } = montar();
		motor.confirmar('L');
		motor.confirmar('ruim');
		expect(mensagens.at(-1)).toEqual({ tipo: 'erro', texto: 'Esperava outra coisa.' });
		expect(motor.emCurso).toBe('LINE');
		expect(motor.prompt).toBe('Pedido 1:');
	});

	it('recusa do kernel vira mensagem com o motivo, e o comando continua (V5b)', () => {
		const { motor, mensagens } = montar();
		motor.confirmar('L');
		motor.confirmar('kernel');
		expect(mensagens.at(-1)).toEqual({
			tipo: 'erro',
			texto: 'O kernel recusou: motivo simulado.'
		});
		expect(motor.emCurso).toBe('LINE');
	});

	it('Esc encerra o comando e avisa; o que ele já confirmou fica (C7)', () => {
		const { motor, registro, mensagens } = montar();
		motor.confirmar('L');
		motor.confirmar('primeiro');
		motor.cancelar();
		expect(registro).toEqual(['primeiro', 'encerrado']);
		expect(motor.emCurso).toBeNull();
		expect(mensagens.at(-1)).toEqual({ tipo: 'info', texto: '*Cancelado*' });
	});

	it('cada edição do comando avisa a página (C10)', () => {
		const { motor, aposEditar } = montar();
		motor.confirmar('L');
		motor.confirmar('a');
		motor.confirmar('b');
		expect(aposEditar).toHaveBeenCalledTimes(2);
	});
});

describe('motor: sugestões (C9)', () => {
	it('sem comando em curso, sugere pelo começo do nome nos dois idiomas', () => {
		const { motor } = montar();
		expect(motor.sugerir('lin').map((c) => c.en)).toEqual(['LINE']);
		expect(motor.sugerir('hach').map((c) => c.en)).toEqual(['HATCH']);
	});

	it('com comando em curso, não sugere comando: o texto é resposta ao prompt', () => {
		const { motor } = montar();
		motor.confirmar('L');
		expect(motor.sugerir('l')).toEqual([]);
	});
});

describe('lerOpcao (C8)', () => {
	const desfazer: Opcao = { pt: 'Desfazer', abreviacaoPt: 'D', en: 'Undo', abreviacaoEn: 'U' };
	const fechar: Opcao = { pt: 'Fechar', abreviacaoPt: 'F', en: 'Close', abreviacaoEn: 'C' };
	// Abreviação em português igual à palavra inglesa de outra opção: vale o português.
	const conflito: Opcao = { pt: 'Undo', abreviacaoPt: 'X', en: 'Other', abreviacaoEn: 'O' };
	const opcoes = [desfazer, fechar];

	it('aceita a palavra em português, a abreviação dela e a palavra em inglês, sem caixa nem acento', () => {
		for (const t of ['Desfazer', 'desfazer', 'D', 'd', 'UNDO', 'undo', '_Undo']) {
			expect(lerOpcao(t, opcoes), t).toBe(desfazer);
		}
	});

	it('a abreviação inglesa só com _', () => {
		expect(lerOpcao('_U', opcoes)).toBe(desfazer);
		expect(lerOpcao('_C', opcoes)).toBe(fechar);
		expect(lerOpcao('U', opcoes)).toBeNull();
		expect(lerOpcao('C', opcoes)).toBeNull();
	});

	it('com _, o português não vale', () => {
		expect(lerOpcao('_D', opcoes)).toBeNull();
		expect(lerOpcao('_Desfazer', opcoes)).toBeNull();
	});

	it('na colisão, vale o português; o inglês exige _', () => {
		expect(lerOpcao('Undo', [desfazer, conflito])).toBe(conflito);
		expect(lerOpcao('_Undo', [desfazer, conflito])).toBe(desfazer);
	});

	it('o que não é opção dá null', () => {
		expect(lerOpcao('10,10', opcoes)).toBeNull();
		expect(lerOpcao('', opcoes)).toBeNull();
	});
});
