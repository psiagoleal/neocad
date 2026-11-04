// Caminho relativo: src/lib/render/vista.ts

/**
 * \file src/lib/render/vista.ts
 * \brief Vista da tela: centro, escala e a transformação que vai à GPU.
 * \author Iago Leal
 * \date 2026-11-01
 *
 * O ADR 0008 deixa no kernel o cálculo da geometria; aqui fica só a vista, que é
 * estado da interface: onde o usuário está olhando e com que aproximação. Tudo
 * em `f64`; só a transformação final, já pequena, vai à GPU como `f32`.
 *
 * Coordenadas de tela em pixels CSS, com y para baixo; do desenho, y para cima.
 */

import type { CadBounds, CadPoint } from '$lib/types/cad';

/** Onde a tela olha: centro em coordenadas do desenho, escala em pixels por unidade. */
export type Vista = {
	centroX: number;
	centroY: number;
	escala: number;
	largura: number;
	altura: number;
};

/** Escala, deslocamento: espaço de recorte = ponto relativo à origem × (sx, sy) + (tx, ty). */
export type Transformacao = [sx: number, sy: number, tx: number, ty: number];

/** Fração da tela que fica livre em volta do desenho enquadrado. */
const MARGEM = 0.05;

/**
 * Divisões do maior lado do desenho que valem um erro de corda.
 *
 * Com o desenho enquadrado numa tela de 4K, o erro fica perto de um pixel.
 */
const DIVISOES_DA_CORDA = 4096;

/** Enquadra a caixa na tela; sem caixa, olha para a origem em escala 1. */
export function enquadrar(caixa: CadBounds | null, largura: number, altura: number): Vista {
	if (caixa == null) {
		return { centroX: 0, centroY: 0, escala: 1, largura, altura };
	}

	const w = caixa.maxX - caixa.minX;
	const h = caixa.maxY - caixa.minY;
	const util = 1 - 2 * MARGEM;
	const porLargura = w > 0 ? (largura * util) / w : Number.POSITIVE_INFINITY;
	const porAltura = h > 0 ? (altura * util) / h : Number.POSITIVE_INFINITY;
	const escala = Math.min(porLargura, porAltura);

	return {
		centroX: (caixa.minX + caixa.maxX) / 2,
		centroY: (caixa.minY + caixa.maxY) / 2,
		escala: Number.isFinite(escala) && escala > 0 ? escala : 1,
		largura,
		altura
	};
}

/** Arrasta a vista: o desenho acompanha o cursor (dy em pixels de tela, para baixo). */
export function deslocar(v: Vista, dx: number, dy: number): Vista {
	return { ...v, centroX: v.centroX - dx / v.escala, centroY: v.centroY + dy / v.escala };
}

/** Aproxima pelo fator, mantendo fixo o ponto do desenho sob (px, py). */
export function ampliar(v: Vista, fator: number, px: number, py: number): Vista {
	const sob = paraMundo(v, px, py);
	const escala = v.escala * fator;

	return {
		...v,
		escala,
		centroX: sob.x - (px - v.largura / 2) / escala,
		centroY: sob.y + (py - v.altura / 2) / escala
	};
}

/** Nova área de tela, mesmo centro e mesma escala. */
export function redimensionar(v: Vista, largura: number, altura: number): Vista {
	return { ...v, largura, altura };
}

/** Ponto do desenho sob o pixel (px, py). */
export function paraMundo(v: Vista, px: number, py: number): CadPoint {
	return {
		x: v.centroX + (px - v.largura / 2) / v.escala,
		y: v.centroY - (py - v.altura / 2) / v.escala
	};
}

/**
 * Transformação dos segmentos de um espaço, relativos à `origem`, para o espaço
 * de recorte. A diferença entre a origem e o centro é feita em `f64`.
 */
export function transformacao(v: Vista, origem: CadPoint): Transformacao {
	const sx = (2 * v.escala) / v.largura;
	const sy = (2 * v.escala) / v.altura;

	return [sx, sy, (origem.x - v.centroX) * sx, (origem.y - v.centroY) * sy];
}

/** Fator de aproximação de um passo da roda: para cima aproxima. */
export function fatorDaRoda(deltaY: number): number {
	return Math.exp(-deltaY * 0.0015);
}

/** Erro de corda para preparar a tela: uma fração do maior lado do desenho. */
export function erroDeCorda(caixa: CadBounds | null): number {
	const lado = caixa == null ? 0 : Math.max(caixa.maxX - caixa.minX, caixa.maxY - caixa.minY);

	return lado > 0 ? lado / DIVISOES_DA_CORDA : 1e-3;
}
