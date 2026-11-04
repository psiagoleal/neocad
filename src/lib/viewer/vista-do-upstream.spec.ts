// Caminho relativo: src/lib/viewer/vista-do-upstream.spec.ts

/**
 * \file src/lib/viewer/vista-do-upstream.spec.ts
 * \brief Testes do filtro de entidades e do seguir a vista do upstream (MT-KV-13a).
 * \author Iago Leal
 * \date 2026-11-01
 *
 * A vista falsa reproduz a câmera do upstream: ortográfica, meia altura fixa em
 * 400 unidades com zoom 1. Assim a escala em pixels depende da altura da tela, e
 * não dá para acertar o zoom sem medir.
 */

import { describe, expect, it } from 'vitest';
import { medirVista, paraOUpstream, seguirVista, type VistaDoUpstream } from './neocad-viewer';

function vistaFalsa(largura: number, altura: number): VistaDoUpstream & { zoom: number } {
	const v = {
		zoom: 3,
		cx: 10,
		cy: -4,
		get center() {
			return { x: v.cx, y: v.cy };
		},
		flyTo(p: { x: number; y: number }, escala?: number) {
			v.cx = p.x;
			v.cy = p.y;
			if (escala != null) v.zoom = escala;
		},
		screenToWorld(p: { x: number; y: number }) {
			const porUnidade = (altura * v.zoom) / 800;
			return {
				x: v.cx + (p.x - largura / 2) / porUnidade,
				y: v.cy - (p.y - altura / 2) / porUnidade
			};
		}
	};
	return v;
}

describe('paraOUpstream', () => {
	const tipos = new Set(['LINE', 'TEXT']);

	it('deixa ao upstream só o que a tela do kernel não desenha', () => {
		const linha = { dxfTypeName: 'LINE' };
		const solido = { dxfTypeName: 'TRACE' };
		const texto = { dxfTypeName: 'text' };
		const hachura = { dxfTypeName: 'HATCH' };

		expect(paraOUpstream([linha, solido, texto, hachura], tipos)).toEqual([solido, hachura]);
	});

	it('sem tipos do kernel, o upstream desenha tudo', () => {
		const linha = { dxfTypeName: 'LINE' };
		expect(paraOUpstream([linha], new Set())).toEqual([linha]);
	});

	it('entidade sem tipo legível fica com o upstream', () => {
		const estranha = {};
		expect(paraOUpstream([estranha], tipos)).toEqual([estranha]);
	});
});

describe('seguirVista e medirVista', () => {
	it('o upstream passa a olhar o mesmo centro com a mesma escala em pixels', () => {
		const v = vistaFalsa(1200, 700);
		seguirVista(v, { x: 5_000_000.5, y: -250.25, escala: 0.037 });

		const medida = medirVista(v);
		expect(medida.x).toBeCloseTo(5_000_000.5, 6);
		expect(medida.y).toBeCloseTo(-250.25, 6);
		expect(medida.escala / 0.037).toBeCloseTo(1, 9);
	});

	it('a mesma escala vale noutra altura de tela', () => {
		const v = vistaFalsa(800, 1300);
		seguirVista(v, { x: 0, y: 0, escala: 42 });
		expect(medirVista(v).escala).toBeCloseTo(42, 9);
	});

	it('escala que não é finita e positiva não mexe na vista', () => {
		const v = vistaFalsa(800, 600);
		const antes = medirVista(v);
		seguirVista(v, { x: 1, y: 1, escala: 0 });
		seguirVista(v, { x: 1, y: 1, escala: Number.NaN });
		expect(medirVista(v)).toEqual(antes);
	});
});
