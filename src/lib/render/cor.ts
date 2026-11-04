// Caminho relativo: src/lib/render/cor.ts

/**
 * \file src/lib/render/cor.ts
 * \brief Cor de desenho de uma camada, a partir da cor do kernel.
 * \author Iago Leal
 * \date 2026-11-01
 *
 * A paleta de índice do formato DXF (ACI) é regular de 10 a 249: 24 matizes a
 * cada 15°, cada um com cinco brilhos, em versão pura e em versão clareada.
 * Por isso ela é calculada, e não tabelada.
 */

import type { CadColor } from '$lib/types/cad';

export type Rgb = [number, number, number];

const BASICAS: Record<number, Rgb> = {
	1: [255, 0, 0],
	2: [255, 255, 0],
	3: [0, 255, 0],
	4: [0, 255, 255],
	5: [0, 0, 255],
	6: [255, 0, 255],
	8: [128, 128, 128],
	9: [192, 192, 192]
};

/** Brilho de cada degrau da faixa 10–249. */
const BRILHOS = [255, 165, 127, 76, 38];

/** Cinzas de 250 a 255. */
const CINZAS = [51, 91, 132, 173, 214, 255];

/** Cor pura do matiz (em graus), no brilho dado. */
function matiz(graus: number, brilho: number): Rgb {
	const canal = (n: number) => {
		const k = (n + graus / 60) % 6;
		return brilho * (1 - Math.max(0, Math.min(k, 4 - k, 1)));
	};

	return [canal(5), canal(3), canal(1)].map(Math.round) as Rgb;
}

function indice(index: number, traco: Rgb): Rgb {
	if (index === 7) {
		return traco;
	}
	if (BASICAS[index]) {
		return BASICAS[index];
	}
	if (index >= 10 && index <= 249) {
		const passo = index - 10;
		const brilho = BRILHOS[Math.floor((passo % 10) / 2)];
		const pura = matiz(Math.floor(passo / 10) * 15, brilho);
		// Ímpar: metade do caminho até o branco do mesmo brilho.
		return passo % 2 === 0 ? pura : (pura.map((c) => Math.floor((c + brilho) / 2)) as Rgb);
	}
	if (index >= 250 && index <= 255) {
		const cinza = CINZAS[index - 250];
		return [cinza, cinza, cinza];
	}
	return traco;
}

/** Cor em que a camada é desenhada, conforme o fundo. */
export function corDaCamada(cor: CadColor, fundo: { escuro: boolean }): Rgb {
	const traco: Rgb = fundo.escuro ? [255, 255, 255] : [0, 0, 0];

	switch (cor.kind) {
		case 'rgb':
			return [cor.red, cor.green, cor.blue];
		case 'index':
			return indice(cor.index, traco);
		default:
			return traco;
	}
}
