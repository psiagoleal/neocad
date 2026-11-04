// Caminho relativo: src/lib/command-line/pontos.ts

/**
 * \file src/lib/command-line/pontos.ts
 * \brief Lê e escreve pontos e números da linha de comando (spec 10, D2, D7–D9).
 * \author Iago Leal
 * \date 2026-11-02
 */

import type { CadPoint } from '$lib/types/cad';

export type LeituraDePonto =
	| { tipo: 'ponto'; ponto: CadPoint }
	| { tipo: 'invalido' }
	| { tipo: 'sem-anterior' };

/**
 * `ponto`: decimal com ponto, como no AutoCAD (`1.5,2`); `virgula`: decimal com
 * vírgula e coordenadas separadas por ponto e vírgula (`1,5;2`). Spec 10, D7–D10.
 */
export type ModoDecimal = 'ponto' | 'virgula';

let modo: ModoDecimal = 'ponto';

/** Troca o modo; vale para a próxima entrada e a próxima saída. */
export function definirModoDecimal(novo: ModoDecimal): void {
	modo = novo;
}

export function modoDecimal(): ModoDecimal {
	return modo;
}

/** Número com o decimal do modo, sinal e expoente opcionais. */
const NUMERO = {
	ponto: /^[+-]?(\d+\.?\d*|\.\d+)(e[+-]?\d+)?$/i,
	virgula: /^[+-]?(\d+,?\d*|,\d+)(e[+-]?\d+)?$/i
};

function numero(texto: string): number | null {
	const t = texto.trim();
	if (!NUMERO[modo].test(t)) return null;
	const valor = Number(modo === 'virgula' ? t.replace(',', '.') : t);
	return Number.isFinite(valor) ? valor : null;
}

/** As coordenadas de `x,y`: a vírgula (só no modo padrão) ou o ponto e vírgula. */
function coordenadas(texto: string): string[] {
	const porPontoEVirgula = texto.split(';');
	if (modo === 'virgula') return porPontoEVirgula;
	const porVirgula = texto.split(',');
	// Misturar os dois separadores não é ponto (D7).
	if (porPontoEVirgula.length > 1 && porVirgula.length > 1) return [];
	return porPontoEVirgula.length > 1 ? porPontoEVirgula : porVirgula;
}

/**
 * Lê `x,y`, `@dx,dy`, `d<a` ou `@d<a` (no modo de vírgula decimal, `x;y` e
 * `@dx;dy`). O ângulo é em graus, anti-horário a partir de +X.
 */
export function lerPonto(texto: string, anterior: CadPoint | null): LeituraDePonto {
	let resto = texto.trim();
	const relativo = resto.startsWith('@');
	if (relativo) resto = resto.slice(1);

	let dx: number | null = null;
	let dy: number | null = null;
	const polar = resto.split('<');
	const cartesiano = coordenadas(resto);
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

/** Um número com o decimal do modo, ou `null`. */
export function lerNumero(texto: string): number | null {
	return numero(texto);
}

/** O número como a linha de comando o lê de volta, no decimal do modo. */
export function formatarNumero(n: number): string {
	// `String` dá o menor texto que volta ao mesmo número.
	const texto = String(n);
	return modo === 'virgula' ? texto.replace('.', ',') : texto;
}

/** O ponto como texto que a linha de comando lê de volta sem perder precisão (spec 10, P3). */
export function formatarPonto(ponto: CadPoint): string {
	const separador = modo === 'virgula' ? ';' : ',';
	return `${formatarNumero(ponto.x)}${separador}${formatarNumero(ponto.y)}`;
}

/** Os formatos de ponto do modo, para a mensagem de inválido. */
export function formatosDePonto(): string {
	return modo === 'virgula' ? 'x;y, @dx;dy, d<a ou @d<a' : 'x,y, @dx,dy, d<a ou @d<a';
}
