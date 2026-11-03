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

type Bloco =
	| { segmentos: Float32Array; cor: string }
	| { faixas: Map<string, Uint32Array>; cor: string };

export class DesenhistaCanvas2d implements Desenhista {
	readonly caminho = 'canvas2d' as const;
	private readonly blocos = new Map<string, Bloco>();

	constructor(private readonly ctx: CanvasRenderingContext2D) {}

	definirBloco(chave: string, segmentos: Float32Array, cor: Rgb): void {
		this.blocos.set(chave, { segmentos, cor: `rgb(${cor[0]}, ${cor[1]}, ${cor[2]})` });
	}

	definirDestaque(chave: string, faixas: Map<string, Uint32Array>, cor: Rgb): void {
		this.blocos.set(chave, { faixas, cor: `rgb(${cor[0]}, ${cor[1]}, ${cor[2]})` });
	}

	removerBloco(chave: string): void {
		this.blocos.delete(chave);
	}

	desenhar(t: Transformacao, largura: number, altura: number, fundo: Rgb | null): void {
		const ctx = this.ctx;
		ctx.setTransform(1, 0, 0, 1, 0, 0);
		if (fundo == null) {
			ctx.clearRect(0, 0, largura, altura);
		} else {
			ctx.fillStyle = `rgb(${fundo[0]}, ${fundo[1]}, ${fundo[2]})`;
			ctx.fillRect(0, 0, largura, altura);
		}
		ctx.lineWidth = 1;

		// Do recorte (-1..1, y para cima) ao pixel (0..largura, y para baixo).
		const ax = (t[0] * largura) / 2;
		const bx = ((t[2] + 1) * largura) / 2;
		const ay = (-t[1] * altura) / 2;
		const by = ((1 - t[3]) * altura) / 2;

		const tracar = (segmentos: Float32Array, de: number, ate: number) => {
			for (let i = de; i + 3 < ate; i += 4) {
				ctx.moveTo(segmentos[i] * ax + bx, segmentos[i + 1] * ay + by);
				ctx.lineTo(segmentos[i + 2] * ax + bx, segmentos[i + 3] * ay + by);
			}
		};

		for (const bloco of this.blocos.values()) {
			ctx.strokeStyle = bloco.cor;
			ctx.beginPath();
			if ('segmentos' in bloco) {
				tracar(bloco.segmentos, 0, bloco.segmentos.length);
			} else {
				// Cada faixa conta vértices; cada vértice tem dois números.
				for (const [chave, faixas] of bloco.faixas) {
					const alvo = this.blocos.get(chave);
					if (alvo == null || !('segmentos' in alvo)) continue;
					const n = faixas.length / 2;
					for (let i = 0; i < n; i++) {
						const de = faixas[i] * 2;
						tracar(alvo.segmentos, de, de + faixas[n + i] * 2);
					}
				}
			}
			ctx.stroke();
		}
	}

	destruir(): void {
		this.blocos.clear();
	}
}
