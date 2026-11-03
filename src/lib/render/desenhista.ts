// Caminho relativo: src/lib/render/desenhista.ts

/**
 * \file src/lib/render/desenhista.ts
 * \brief O que todo caminho de desenho oferece: blocos de segmentos e um quadro.
 * \author Iago Leal
 * \date 2026-11-01
 */

import type { Rgb } from './cor';
import type { Transformacao } from './vista';

/** Caminho que desenhou: a GPU ou a reserva (spec 00, N4). */
export type Caminho = 'webgl2' | 'canvas2d';

export interface Desenhista {
	readonly caminho: Caminho;
	/** Guarda (ou troca) o bloco de segmentos `x1, y1, x2, y2` da chave. */
	definirBloco(chave: string, segmentos: Float32Array, cor: Rgb): void;
	/**
	 * Guarda (ou troca), na chave, o redesenho de trechos de outros blocos na cor
	 * dada: para cada bloco, os primeiros vértices e depois as contagens. Bloco
	 * que não existe é ignorado. Desenha-se na ordem, como os blocos.
	 */
	definirDestaque(chave: string, faixas: Map<string, Uint32Array>, cor: Rgb): void;
	/** Tira o bloco ou o destaque da chave. */
	removerBloco(chave: string): void;
	/**
	 * Desenha todos os blocos; largura e altura em pixels do dispositivo. Fundo
	 * `null` deixa a tela transparente, para o que estiver por baixo aparecer.
	 */
	desenhar(t: Transformacao, largura: number, altura: number, fundo: Rgb | null): void;
	destruir(): void;
}
