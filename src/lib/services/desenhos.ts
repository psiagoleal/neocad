// Caminho relativo: src/lib/services/desenhos.ts

/**
 * \file src/lib/services/desenhos.ts
 * \brief Nome do próximo desenho novo e a marca de alterado (spec 11, I4).
 * \author Iago Leal
 * \date 2026-11-04
 */

/**
 * Os desenhos novos da sessão (`Desenho1`, `Desenho2`…) e se o da aba mudou desde
 * que abriu, foi criado ou foi gravado. Zoom e pan não passam por aqui.
 */
export class SessaoDeDesenhos {
	private criados = 0;
	private alterado = false;

	/** Gasta o próximo número e devolve o nome do desenho novo. */
	criar(): string {
		this.criados += 1;
		this.alterado = false;
		return `Desenho${this.criados}`;
	}

	/** Uma edição, um desfazer ou um refazer. */
	marcarAlterado(): void {
		this.alterado = true;
	}

	marcarGravado(): void {
		this.alterado = false;
	}

	marcarAberto(): void {
		this.alterado = false;
	}

	/** Trocar o desenho da aba descartaria alterações. */
	precisaConfirmar(): boolean {
		return this.alterado;
	}
}
