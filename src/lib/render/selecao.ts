// Caminho relativo: src/lib/render/selecao.ts

/**
 * \file src/lib/render/selecao.ts
 * \brief Seleção por clique, janela e cruzada, como no AutoCAD (spec 10, S1–S5).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Só a janela e as regras do clique; quem decide o que está sob o cursor, e
 * guarda o que está selecionado, é o kernel (ADR 0008; MT-KV-14g).
 */

import type { CadBounds, CadEntityId, CadPoint } from '$lib/types/cad';

/**
 * O que a seleção pede ao kernel, no espaço mostrado. O conjunto selecionado
 * mora lá (MT-KV-14g): aqui só se diz o que pôr ou tirar.
 */
export interface ConsultaDaSelecao {
	/** Entidades a até `raio` do ponto, da mais perto à mais longe. */
	clique(ponto: CadPoint, raio: number): CadEntityId[];
	/** Põe na seleção (ou tira) as dadas. */
	aplicar(ids: CadEntityId[], tirar: boolean): void;
	/** Põe na seleção (ou tira) as de dentro da área (janela) ou que ela toca (cruzada). */
	area(area: CadBounds, cruzada: boolean, tirar: boolean): void;
}

/** O clique e a janela da seleção; quem guarda o selecionado é o kernel. */
export class Selecao {
	private inicio: CadPoint | null = null;

	/**
	 * Um clique no espaço mostrado. Com a janela aberta, fecha-a; senão, pega o
	 * traço sob o cursor ou, no vazio, abre a janela. `Shift` tira em vez de pôr.
	 */
	clicar(ponto: CadPoint, raio: number, shift: boolean, consulta: ConsultaDaSelecao): void {
		const inicio = this.inicio;
		if (inicio != null) {
			this.inicio = null;
			const area = {
				minX: Math.min(inicio.x, ponto.x),
				minY: Math.min(inicio.y, ponto.y),
				maxX: Math.max(inicio.x, ponto.x),
				maxY: Math.max(inicio.y, ponto.y)
			};
			// Da direita para a esquerda é cruzada, como no AutoCAD.
			consulta.area(area, ponto.x < inicio.x, shift);
			return;
		}

		const [maisPerto] = consulta.clique(ponto, raio);
		if (maisPerto == null) {
			this.inicio = ponto;
			return;
		}
		consulta.aplicar([maisPerto], shift);
	}

	/** Onde a janela aberta começou, ou `null`. */
	get inicioDaJanela(): CadPoint | null {
		return this.inicio;
	}

	/** Fecha a janela aberta. */
	esvaziarJanela(): void {
		this.inicio = null;
	}
}
