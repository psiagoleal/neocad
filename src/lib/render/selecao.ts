// Caminho relativo: src/lib/render/selecao.ts

/**
 * \file src/lib/render/selecao.ts
 * \brief Seleção por clique, janela e cruzada, como no AutoCAD (spec 10, S1–S5).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Só o estado e as regras; quem decide o que está sob o cursor é o kernel
 * (ADR 0008), pela consulta que a tela passa.
 */

import type { CadBounds, CadEntityId, CadPoint } from '$lib/types/cad';

/** O que a seleção pergunta ao kernel, no espaço mostrado. */
export interface ConsultaDaSelecao {
	/** Entidades a até `raio` do ponto, da mais perto à mais longe. */
	clique(ponto: CadPoint, raio: number): CadEntityId[];
	/** Dentro da área (janela) ou que ela toca (cruzada). */
	area(area: CadBounds, cruzada: boolean): CadEntityId[];
}

export class Selecao {
	private readonly selecionados = new Set<CadEntityId>();
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
			this.aplicar(consulta.area(area, ponto.x < inicio.x), shift);
			return;
		}

		const [maisPerto] = consulta.clique(ponto, raio);
		if (maisPerto == null) {
			this.inicio = ponto;
			return;
		}
		this.aplicar([maisPerto], shift);
	}

	/** Ids selecionados, na ordem em que entraram. */
	get ids(): CadEntityId[] {
		return [...this.selecionados];
	}

	/** Onde a janela aberta começou, ou `null`. */
	get inicioDaJanela(): CadPoint | null {
		return this.inicio;
	}

	esvaziar(): void {
		this.selecionados.clear();
		this.inicio = null;
	}

	/** Tira as que deixaram de existir; diz se mudou. */
	podar(existe: (id: CadEntityId) => boolean): boolean {
		let mudou = false;
		for (const id of this.selecionados) {
			if (!existe(id)) {
				this.selecionados.delete(id);
				mudou = true;
			}
		}
		return mudou;
	}

	private aplicar(ids: CadEntityId[], tirar: boolean): void {
		for (const id of ids) {
			if (tirar) this.selecionados.delete(id);
			else this.selecionados.add(id);
		}
	}
}
