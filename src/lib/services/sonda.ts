// Caminho relativo: src/lib/services/sonda.ts

/**
 * \file src/lib/services/sonda.ts
 * \brief Edição pelo serviço para os testes de ponta a ponta (spec 07, V2; spec 00, N2).
 * \author Iago Leal
 * \date 2026-11-01
 *
 * Enquanto não há linha de comando própria (spec 08), os testes editam o
 * desenho por aqui. Só é instalada no build feito com `VITE_NEOCAD_SONDA=1`;
 * sem a variável, este módulo nem entra no pacote.
 */

import type { CadDocument } from '$lib/services/cad-document';
import type { CadEntityId, CadLayerId } from '$lib/types/cad';

export interface Sonda {
	desenharLinha(x1: number, y1: number, x2: number, y2: number): CadEntityId;
	mover(ids: CadEntityId[], dx: number, dy: number): void;
	apagar(id: CadEntityId): void;
}

export interface OpcoesDaSonda {
	/** O documento aberto agora, ou `null` se nenhum. */
	documento(): CadDocument | null;
	/** Chamado depois de cada edição, para o menu e a tela relerem o kernel. */
	aposEditar(): void;
}

/** Monta a sonda sobre o documento aberto. */
export function criarSonda(opcoes: OpcoesDaSonda): Sonda {
	/** Roda a edição no documento aberto e avisa a página. */
	function editar<T>(edicao: (documento: CadDocument) => T): T {
		const documento = opcoes.documento();
		if (documento == null) {
			throw new Error('Nenhum desenho aberto para a sonda editar.');
		}
		const resultado = edicao(documento);
		opcoes.aposEditar();
		return resultado;
	}

	return {
		desenharLinha: (x1, y1, x2, y2) =>
			editar((documento) =>
				documento.drawLine(camadaCorrente(documento), { x: x1, y: y1 }, { x: x2, y: y2 })
			),
		mover: (ids, dx, dy) => editar((documento) => documento.moveEntities(ids, { x: dx, y: dy })),
		apagar: (id) => editar((documento) => documento.eraseEntity(id))
	};
}

/**
 * A camada onde a linha nova entra.
 *
 * Ainda não há camada corrente escolhida pelo usuário (spec 08); vale a `0`,
 * que todo DXF tem, e na falta dela a primeira.
 */
function camadaCorrente(documento: CadDocument): CadLayerId {
	const camadas = documento.listLayers();
	const camada = camadas.find((c) => c.name === '0') ?? camadas[0];
	if (camada == null) {
		throw new Error('O desenho não tem camada onde desenhar.');
	}
	return camada.id;
}
