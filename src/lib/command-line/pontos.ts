// Caminho relativo: src/lib/command-line/pontos.ts

/**
 * \file src/lib/command-line/pontos.ts
 * \brief Lê um ponto digitado na linha de comando (spec 10, D2).
 * \author Iago Leal
 * \date 2026-11-02
 */

import type { CadPoint } from '$lib/types/cad';

export type LeituraDePonto =
	| { tipo: 'ponto'; ponto: CadPoint }
	| { tipo: 'invalido' }
	| { tipo: 'sem-anterior' };

/** Número como o AutoCAD aceita: ponto decimal, sinal e expoente opcionais. */
const NUMERO = /^[+-]?(\d+\.?\d*|\.\d+)(e[+-]?\d+)?$/i;

function numero(texto: string): number | null {
	const t = texto.trim();
	if (!NUMERO.test(t)) return null;
	const valor = Number(t);
	return Number.isFinite(valor) ? valor : null;
}

/**
 * Lê `x,y`, `@dx,dy`, `d<a` ou `@d<a`. O ângulo é em graus, anti-horário a partir
 * de +X. Vírgula separa coordenadas; o decimal é o ponto, como no AutoCAD.
 */
export function lerPonto(texto: string, anterior: CadPoint | null): LeituraDePonto {
	let resto = texto.trim();
	const relativo = resto.startsWith('@');
	if (relativo) resto = resto.slice(1);

	let dx: number | null = null;
	let dy: number | null = null;
	const polar = resto.split('<');
	const cartesiano = resto.split(',');
	if (polar.length === 2 && cartesiano.length === 1) {
		const distancia = numero(polar[0]);
		const angulo = numero(polar[1]);
		if (distancia != null && angulo != null) {
			const rad = (angulo * Math.PI) / 180;
			dx = distancia * Math.cos(rad);
			dy = distancia * Math.sin(rad);
		}
	} else if (cartesiano.length === 2 && polar.length === 1) {
		dx = numero(cartesiano[0]);
		dy = numero(cartesiano[1]);
	}

	if (dx == null || dy == null) return { tipo: 'invalido' };
	if (!relativo) return { tipo: 'ponto', ponto: { x: dx, y: dy } };
	if (anterior == null) return { tipo: 'sem-anterior' };
	return { tipo: 'ponto', ponto: { x: anterior.x + dx, y: anterior.y + dy } };
}

/** Um número como o AutoCAD aceita (ponto decimal), ou `null`. */
export function lerNumero(texto: string): number | null {
	return numero(texto);
}

/** O ponto como texto que a linha de comando lê de volta sem perder precisão (spec 10, P3). */
export function formatarPonto(ponto: CadPoint): string {
	// `String` dá o menor texto que volta ao mesmo número; o ponto é o decimal.
	return `${String(ponto.x)},${String(ponto.y)}`;
}
