// Caminho relativo: src/lib/command-line/concordar.ts

/**
 * \file src/lib/command-line/concordar.ts
 * \brief FILLET / CONCORD / F e CHAMFER / CHANFRO / CHA (spec 10, F1–F4).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Os dois ligam dois objetos escolhidos pelo clique. O raio e as distâncias valem
 * para a sessão, como as variáveis do AutoCAD (FILLETRAD, CHAMFERA/B): cada fábrica
 * guarda os seus, e o registro usa uma instância de cada.
 */

import type { CadEntityId, CadPoint } from '$lib/types/cad';
import {
	lerOpcao,
	type ContextoDeComando,
	type DocumentoDosComandos,
	type FabricaDeComando,
	type Opcao
} from './motor';
import { lerNumero } from './pontos';

type Escolhido = { id: CadEntityId; ponto: CadPoint };

type Forma = {
	primeiro: string;
	segundo: string;
	opcao: Opcao;
	/** Os prompts dos valores da opção, na ordem. */
	perguntas: string[];
	/** Mensagem quando o valor não serve. */
	invalido: string;
	ligar(documento: DocumentoDosComandos, a: Escolhido, b: Escolhido, valores: number[]): void;
};

function criarLigacao(forma: Forma, iniciais: number[]): FabricaDeComando {
	const valores = [...iniciais];

	return ({ documento, aposEditar }: ContextoDeComando) => {
		let primeiro: Escolhido | null = null;
		/** Índice da pergunta da opção em curso, ou `null` fora dela. */
		let pergunta: number | null = null;

		return {
			prompt() {
				if (pergunta != null) return `${forma.perguntas[pergunta]} <${valores[pergunta]}>:`;
				return primeiro == null ? `${forma.primeiro} ou [${forma.opcao.pt}]:` : forma.segundo;
			},

			querEscolha: () => pergunta == null,

			aoEscolher({ ids, ponto }) {
				const [maisPerto] = ids;
				if (maisPerto == null) {
					return { tipo: 'invalida', mensagem: 'Nenhum objeto sob o clique.' };
				}
				if (primeiro == null) {
					primeiro = { id: maisPerto, ponto };
					return { tipo: 'continua' };
				}
				// O primeiro sai antes da chamada: se o kernel recusar, volta a pedi-lo (F4).
				const a = primeiro;
				primeiro = null;
				forma.ligar(documento, a, { id: maisPerto, ponto }, valores);
				aposEditar();
				return { tipo: 'fim' };
			},

			responder(texto) {
				if (pergunta != null) {
					if (texto.trim() !== '') {
						const valor = lerNumero(texto);
						if (valor == null || valor < 0) return { tipo: 'invalida', mensagem: forma.invalido };
						valores[pergunta] = valor;
					}
					pergunta = pergunta + 1 < forma.perguntas.length ? pergunta + 1 : null;
					return { tipo: 'continua' };
				}
				if (texto.trim() === '') return { tipo: 'fim' };
				if (primeiro == null && lerOpcao(texto, [forma.opcao]) === forma.opcao) {
					pergunta = 0;
					return { tipo: 'continua' };
				}
				return { tipo: 'invalida', mensagem: 'Clique no objeto, na tela, ou escolha uma opção.' };
			},

			encerrar: () => undefined
		};
	};
}

export function criarConcordar(): FabricaDeComando {
	return criarLigacao(
		{
			primeiro: 'Selecione o primeiro objeto',
			segundo: 'Selecione o segundo objeto:',
			opcao: { pt: 'Raio', abreviacaoPt: 'R', en: 'Radius', abreviacaoEn: 'R' },
			perguntas: ['Especifique o raio de concordância'],
			invalido: 'Esperava um raio maior ou igual a zero.',
			ligar: (documento, a, b, [raio]) =>
				void documento.filletEntities(a.id, a.ponto, b.id, b.ponto, raio)
		},
		[0]
	);
}

export function criarChanfrar(): FabricaDeComando {
	return criarLigacao(
		{
			primeiro: 'Selecione a primeira linha',
			segundo: 'Selecione a segunda linha:',
			opcao: { pt: 'Distância', abreviacaoPt: 'D', en: 'Distance', abreviacaoEn: 'D' },
			perguntas: [
				'Especifique a primeira distância de chanfro',
				'Especifique a segunda distância de chanfro'
			],
			invalido: 'Esperava uma distância maior ou igual a zero.',
			ligar: (documento, a, b, [d1, d2]) =>
				void documento.chamferEntities(a.id, a.ponto, b.id, b.ponto, d1, d2)
		},
		[0, 0]
	);
}
