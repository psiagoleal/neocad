// Caminho relativo: src/lib/render/canvas2d.ts

/**
 * \file src/lib/render/canvas2d.ts
 * \brief Caminho de reserva: desenha os blocos do kernel em canvas 2D (spec 00, N4).
 * \author Iago Leal
 * \date 2026-11-01
 *
 * Usado quando o navegador não oferece WebGL2. Lê os mesmos blocos que a GPU e
 * aplica a mesma transformação; só a conta do pixel acontece aqui, na CPU.
 */

import type { Rgb } from './cor';
import type { Desenhista } from './desenhista';
import type { Transformacao } from './vista';

type Bloco = { segmentos: Float32Array; cor: string };

export class DesenhistaCanvas2d implements Desenhista {
	readonly caminho = 'canvas2d' as const;
	private readonly blocos = new Map<string, Bloco>();

	constructor(private readonly ctx: CanvasRenderingContext2D) {}

	definirBloco(chave: string, segmentos: Float32Array, cor: Rgb): void {
		this.blocos.set(chave, { segmentos, cor: `rgb(${cor[0]}, ${cor[1]}, ${cor[2]})` });
	}

	removerBloco(chave: string): void {
		this.blocos.delete(chave);
	}

	desenhar(t: Transformacao, largura: number, altura: number, fundo: Rgb): void {
		const ctx = this.ctx;
		ctx.setTransform(1, 0, 0, 1, 0, 0);
		ctx.fillStyle = `rgb(${fundo[0]}, ${fundo[1]}, ${fundo[2]})`;
		ctx.fillRect(0, 0, largura, altura);
		ctx.lineWidth = 1;

		// Do recorte (-1..1, y para cima) ao pixel (0..largura, y para baixo).
		const ax = (t[0] * largura) / 2;
		const bx = ((t[2] + 1) * largura) / 2;
		const ay = (-t[1] * altura) / 2;
		const by = ((1 - t[3]) * altura) / 2;

		for (const { segmentos, cor } of this.blocos.values()) {
			ctx.strokeStyle = cor;
			ctx.beginPath();
			for (let i = 0; i + 3 < segmentos.length; i += 4) {
				ctx.moveTo(segmentos[i] * ax + bx, segmentos[i + 1] * ay + by);
				ctx.lineTo(segmentos[i + 2] * ax + bx, segmentos[i + 3] * ay + by);
			}
			ctx.stroke();
		}
	}

	destruir(): void {
		this.blocos.clear();
	}
}
