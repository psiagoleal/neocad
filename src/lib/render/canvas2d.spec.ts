// Caminho relativo: src/lib/render/canvas2d.spec.ts

import { describe, expect, it } from 'vitest';
import { DesenhistaCanvas2d } from './canvas2d';

/** Contexto 2D falso que registra os traços. */
function contextoFalso() {
	const chamadas: string[] = [];
	const ctx = {
		strokeStyle: '',
		fillStyle: '',
		lineWidth: 0,
		setTransform: () => undefined,
		fillRect: () => chamadas.push('fundo'),
		clearRect: () => chamadas.push('limpa'),
		beginPath: () => chamadas.push('início'),
		moveTo: (x: number, y: number) => chamadas.push(`m ${x} ${y}`),
		lineTo: (x: number, y: number) => chamadas.push(`l ${x} ${y}`),
		stroke: () => chamadas.push(`traço ${ctx.strokeStyle}`)
	};
	return { ctx: ctx as unknown as CanvasRenderingContext2D, chamadas };
}

describe('desenhista de reserva em canvas 2D', () => {
	it('desenha cada segmento no pixel que a transformação indica, com a cor do bloco', () => {
		const { ctx, chamadas } = contextoFalso();
		const d = new DesenhistaCanvas2d(ctx);
		expect(d.caminho).toBe('canvas2d');

		// Um segmento do centro (0,0) ao canto superior direito (1,1) do espaço de recorte.
		d.definirBloco('Model|0', new Float32Array([0, 0, 1, 1]), [255, 0, 0]);
		d.desenhar([1, 1, 0, 0], 200, 100, [0, 0, 0]);

		expect(chamadas).toContain('fundo');
		expect(chamadas).toContain('m 100 50');
		expect(chamadas).toContain('l 200 0');
		expect(chamadas).toContain('traço rgb(255, 0, 0)');
	});

	it('sem fundo, limpa a tela em vez de pintá-la, e ainda desenha os traços', () => {
		const { ctx, chamadas } = contextoFalso();
		const d = new DesenhistaCanvas2d(ctx);
		d.definirBloco('Model|0', new Float32Array([0, 0, 1, 1]), [255, 0, 0]);

		d.desenhar([1, 1, 0, 0], 200, 100, null);

		expect(chamadas).toContain('limpa');
		expect(chamadas).not.toContain('fundo');
		expect(chamadas).toContain('l 200 0');
	});

	it('bloco removido deixa de ser desenhado', () => {
		const { ctx, chamadas } = contextoFalso();
		const d = new DesenhistaCanvas2d(ctx);
		d.definirBloco('Model|0', new Float32Array([0, 0, 1, 1]), [255, 0, 0]);
		d.removerBloco('Model|0');

		d.desenhar([1, 1, 0, 0], 200, 100, [0, 0, 0]);

		expect(chamadas.some((c) => c.startsWith('m '))).toBe(false);
	});
});
