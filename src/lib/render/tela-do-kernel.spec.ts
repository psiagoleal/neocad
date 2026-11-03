// Caminho relativo: src/lib/render/tela-do-kernel.spec.ts

/**
 * \file src/lib/render/tela-do-kernel.spec.ts
 * \brief Aviso das entidades que a tela não exibe (spec 07, V6; MT-KV-13c).
 * \author Iago Leal
 * \date 2026-11-01
 *
 * Sem navegador: o canvas falso não oferece WebGL2, então a tela cai no canvas
 * 2D falso; o documento falso responde só o que a tela pergunta.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { CadDocument } from '$lib/services/cad-document';
import type { CadUnsupportedCount } from '$lib/types/cad';
import { TelaDoKernel } from './tela-do-kernel';

/** Ouvintes registrados no último canvas falso, para o teste disparar eventos. */
let ouvintes = new Map<string, (e: unknown) => void>();

function canvasFalso(): HTMLCanvasElement {
	ouvintes = new Map();
	const ctx = {
		setTransform: () => undefined,
		fillRect: () => undefined,
		clearRect: () => undefined,
		beginPath: () => undefined,
		moveTo: () => undefined,
		lineTo: () => undefined,
		stroke: () => undefined
	};
	return {
		dataset: {},
		clientWidth: 800,
		clientHeight: 600,
		width: 0,
		height: 0,
		getContext: (tipo: string) => (tipo === '2d' ? ctx : null),
		addEventListener: (tipo: string, f: (e: unknown) => void) => ouvintes.set(tipo, f),
		removeEventListener: () => undefined,
		setPointerCapture: () => undefined,
		releasePointerCapture: () => undefined,
		getBoundingClientRect: () => ({ left: 0, top: 0 })
	} as unknown as HTMLCanvasElement;
}

/** Documento com dois espaços; as não desenhadas de cada um podem mudar. */
function documentoFalso(recusadas: Record<string, CadUnsupportedCount[]>) {
	return {
		recusadas,
		prepareScreen: () => undefined,
		getBounds: () => null,
		getScreenSpaces: () => ['Model', 'Prancha A1'],
		getScreenOrigin: () => ({ x: 0, y: 0 }),
		getScreenLayers: () => ['0'],
		getScreenExtents: () => null,
		getScreenEntityCount: () => 0,
		getScreenBlock: () => new Float32Array(),
		getScreenRefused: (espaco: string) => recusadas[espaco] ?? [],
		pickOnScreen: vi.fn((_espaco: string, p: { x: number; y: number }, raio: number) =>
			Math.abs(p.y) <= raio && p.x >= 0 && p.x <= 100 ? ['1'] : []
		),
		selectOnScreen: vi.fn(() => ['1', '2']),
		snapOnScreen: vi.fn((_espaco: string, p: { x: number; y: number }, raio: number) =>
			Math.hypot(p.x - 20, p.y) <= raio
				? { mode: 'endpoint', point: { x: 20, y: 0 }, entity: '1' }
				: null
		),
		// A seleção mora no kernel: o falso guarda os ids e devolve a contagem.
		selecionadas: [] as string[],
		existentes: new Set(['1', '2']),
		mudar(ids: string[], tirar: boolean) {
			this.selecionadas = tirar
				? this.selecionadas.filter((i) => !ids.includes(i))
				: [...this.selecionadas, ...ids.filter((i) => !this.selecionadas.includes(i))];
			return this.selecionadas.length;
		},
		changeScreenSelection: vi.fn(function (
			this: { mudar(ids: string[], tirar: boolean): number },
			_espaco: string,
			ids: string[],
			tirar: boolean
		) {
			return this.mudar(ids, tirar);
		}),
		selectAreaOnScreen: vi.fn(function (
			this: { mudar(ids: string[], tirar: boolean): number },
			_espaco: string,
			_area: unknown,
			_cruzada: boolean,
			tirar: boolean
		) {
			return this.mudar(['1', '2'], tirar);
		}),
		clearScreenSelection: vi.fn(function (this: { selecionadas: string[] }) {
			this.selecionadas = [];
		}),
		getScreenSelection: vi.fn(function (this: { selecionadas: string[] }) {
			return [...this.selecionadas];
		}),
		// Como o kernel, a contagem já não conta as apagadas.
		getScreenSelectionCount: vi.fn(function (this: {
			selecionadas: string[];
			existentes: Set<string>;
		}) {
			this.selecionadas = this.selecionadas.filter((i) => this.existentes.has(i));
			return this.selecionadas.length;
		}),
		getScreenSelectionRanges: vi.fn(() => new Uint32Array([0, 2])),
		updateScreen: () => [],
		listLayers: () => []
	};
}

describe('tela do kernel: aviso das entidades não exibidas (V6)', () => {
	beforeEach(() => {
		vi.stubGlobal('requestAnimationFrame', () => 1);
		vi.stubGlobal('cancelAnimationFrame', () => undefined);
	});
	afterEach(() => vi.unstubAllGlobals());

	it('ao mostrar um espaço com entidades recusadas, avisa o tipo e a quantidade', () => {
		const doc = documentoFalso({
			'Prancha A1': [
				{ entityType: 'TEXT', count: 1 },
				{ entityType: 'VIEWPORT', count: 2 }
			]
		});
		const tela = new TelaDoKernel(canvasFalso(), { escuro: true });
		const avisos: string[] = [];
		tela.avisarCom((m) => avisos.push(m));

		tela.mostrar(doc as unknown as CadDocument);
		expect(avisos).toEqual([]);

		tela.mostrarEspaco('Prancha A1');
		expect(avisos).toEqual(['A tela não exibe 3 entidade(s) de Prancha A1: 1 TEXT, 2 VIEWPORT.']);
	});

	it('não repete o mesmo aviso, e avisa de novo quando a edição muda o que falta', () => {
		const doc = documentoFalso({ 'Prancha A1': [{ entityType: 'VIEWPORT', count: 1 }] });
		const tela = new TelaDoKernel(canvasFalso(), { escuro: true });
		const avisos: string[] = [];
		tela.avisarCom((m) => avisos.push(m));
		tela.mostrar(doc as unknown as CadDocument);

		tela.mostrarEspaco('Prancha A1');
		tela.mostrarEspaco('Model');
		tela.mostrarEspaco('Prancha A1');
		tela.atualizar();
		expect(avisos).toHaveLength(1);

		doc.recusadas['Prancha A1'] = [{ entityType: 'VIEWPORT', count: 2 }];
		tela.atualizar();
		expect(avisos).toEqual([
			'A tela não exibe 1 entidade(s) de Prancha A1: 1 VIEWPORT.',
			'A tela não exibe 2 entidade(s) de Prancha A1: 2 VIEWPORT.'
		]);
	});

	it('abrir outro desenho avisa de novo, mesmo com a mesma contagem', () => {
		const tela = new TelaDoKernel(canvasFalso(), { escuro: true });
		const avisos: string[] = [];
		tela.avisarCom((m) => avisos.push(m));
		const recusadas = { Model: [{ entityType: 'TEXT', count: 1 }] };

		tela.mostrar(documentoFalso(recusadas) as unknown as CadDocument);
		tela.mostrar(documentoFalso(recusadas) as unknown as CadDocument);
		expect(avisos).toHaveLength(2);
	});
});

describe('tela do kernel: seleção (spec 10, S1–S5)', () => {
	let quadros: Array<() => void> = [];
	const quadro = () => {
		const pendentes = quadros;
		quadros = [];
		pendentes.forEach((f) => f());
	};
	beforeEach(() => {
		quadros = [];
		vi.stubGlobal('requestAnimationFrame', (f: () => void) => {
			quadros.push(f);
			return quadros.length;
		});
		vi.stubGlobal('cancelAnimationFrame', () => undefined);
	});
	afterEach(() => vi.unstubAllGlobals());

	/** Clique esquerdo no pixel; sem caixa do desenho, o pixel (400, 300) é a origem, escala 1. */
	const clicar = (x: number, y: number, shiftKey = false) =>
		ouvintes.get('pointerdown')?.({
			button: 0,
			clientX: x,
			clientY: y,
			shiftKey,
			pointerId: 1,
			preventDefault: () => undefined
		});

	function montar() {
		const canvas = canvasFalso();
		const tela = new TelaDoKernel(canvas, { escuro: true });
		const doc = documentoFalso({});
		tela.mostrar(doc as unknown as CadDocument);
		quadro();
		return { tela, doc, dados: canvas.dataset };
	}

	it('clique no traço seleciona e pede o destaque ao kernel, com raio de 5 pixels (S1)', () => {
		const { doc, dados } = montar();
		expect(dados.selecao).toBe('0');
		clicar(410, 302);
		quadro();
		expect(doc.pickOnScreen).toHaveBeenCalledWith('Model', { x: 10, y: -2 }, 5);
		expect(doc.changeScreenSelection).toHaveBeenLastCalledWith('Model', ['1'], false);
		expect(dados.selecao).toBe('1');
		expect(doc.getScreenSelectionRanges).toHaveBeenLastCalledWith('Model', '0');

		clicar(410, 300, true);
		quadro();
		expect(doc.changeScreenSelection).toHaveBeenLastCalledWith('Model', ['1'], true);
		expect(dados.selecao).toBe('0');
	});

	it('clique no vazio abre a janela e o segundo clique fecha (S2)', () => {
		const { doc, dados } = montar();
		clicar(390, 250);
		clicar(420, 320);
		quadro();
		expect(doc.selectAreaOnScreen).toHaveBeenCalledWith(
			'Model',
			{ minX: -10, minY: -20, maxX: 20, maxY: 50 },
			false,
			false
		);
		expect(dados.selecao).toBe('2');
		// Os ids não vêm à tela para selecionar (S6).
		expect(doc.selectOnScreen).not.toHaveBeenCalled();
		expect(doc.getScreenSelection).not.toHaveBeenCalled();
	});

	it('os ids da seleção vêm do kernel só quando pedidos', () => {
		const { tela, doc } = montar();
		clicar(390, 250);
		clicar(420, 320);
		expect(tela.selecionados).toEqual(['1', '2']);
		expect(doc.getScreenSelection).toHaveBeenCalledWith('Model');
	});

	it('no modo de escolha, o clique vai ao comando e não seleciona (T4)', () => {
		const { tela, doc, dados } = montar();
		const escolha = vi.fn();
		tela.definirEscolha(escolha);
		clicar(410, 302);
		quadro();
		expect(escolha).toHaveBeenCalledWith({ ids: ['1'], ponto: { x: 10, y: -2 }, raio: 5 });
		expect(dados.selecao).toBe('0');
		expect(doc.selectAreaOnScreen).not.toHaveBeenCalled();
		expect(doc.changeScreenSelection).not.toHaveBeenCalled();

		tela.definirEscolha(null);
		clicar(410, 300);
		quadro();
		expect(dados.selecao).toBe('1');
	});

	it('todas devolve as entidades do espaço mostrado, por uma janela em volta da caixa dele', () => {
		const { tela, doc } = montar();
		const caixa = { minX: 0, minY: 0, maxX: 100, maxY: 100 };
		(doc as unknown as { getScreenExtents: () => typeof caixa }).getScreenExtents = () => caixa;
		expect(tela.todas()).toEqual(['1', '2']);
		expect(doc.selectOnScreen).toHaveBeenCalledWith('Model', caixa, true);
	});

	const mover = (x: number, y: number) =>
		ouvintes.get('pointermove')?.({ clientX: x, clientY: y, pointerId: 1 });

	it('no modo de ponto, o cursor prende com marcador e o clique entrega o ponto (P1, P2)', () => {
		const { tela, doc, dados } = montar();
		const apontar = vi.fn();
		expect(dados.clique).toBe('selecao');
		tela.definirPonto(apontar);
		quadro();
		expect(dados.clique).toBe('ponto');

		// Perto de (20, 0): prende, a até 10 pixels (escala 1).
		mover(428, 303);
		quadro();
		expect(doc.snapOnScreen).toHaveBeenLastCalledWith('Model', { x: 28, y: -3 }, 10, [
			'endpoint',
			'midpoint',
			'center',
			'quadrant',
			'intersection'
		]);
		expect(dados.prendimento).toBe('endpoint 20 0');
		clicar(428, 303);
		expect(apontar).toHaveBeenLastCalledWith({ x: 20, y: 0 });

		// Longe de tudo: sem marcador, o ponto do cursor.
		mover(500, 200);
		quadro();
		expect(dados.prendimento).toBe('');
		clicar(500, 200);
		expect(apontar).toHaveBeenLastCalledWith({ x: 100, y: 100 });
		quadro();
		expect(dados.selecao).toBe('0');

		tela.definirPonto(null);
		quadro();
		expect(dados.clique).toBe('selecao');
		expect(dados.prendimento).toBe('');
	});

	it('clique em entidade só do upstream avisa e não abre janela; na escolha, vai com o tipo (V7)', () => {
		const { tela, doc, dados } = montar();
		const aviso = vi.fn();
		const consulta = vi.fn((p: { x: number }) => (p.x < 0 ? 'TRACE' : null));
		// O upstream está por baixo, no espaço-modelo.
		tela.sobrepor(vi.fn());
		tela.consultarNaoModelado(consulta);
		tela.aoNaoEditavel(aviso);

		clicar(380, 300);
		expect(consulta).toHaveBeenCalledWith({ x: -20, y: 0 }, 5);
		expect(aviso).toHaveBeenCalledWith('TRACE');
		clicar(420, 320);
		quadro();
		expect(doc.selectAreaOnScreen).not.toHaveBeenCalled();
		expect(dados.selecao).toBe('0');

		const escolha = vi.fn();
		tela.definirEscolha(escolha);
		clicar(380, 300);
		expect(escolha).toHaveBeenCalledWith({
			ids: [],
			ponto: { x: -20, y: 0 },
			raio: 5,
			naoEditavel: 'TRACE'
		});
	});

	it('sem o upstream por baixo, ninguém é consultado (V7)', () => {
		const { tela } = montar();
		const consulta = vi.fn(() => 'TRACE');
		tela.consultarNaoModelado(consulta);
		clicar(380, 300);
		expect(consulta).not.toHaveBeenCalled();
	});

	it('o modo de escolha e o de ponto se excluem', () => {
		const { tela, dados } = montar();
		tela.definirPonto(vi.fn());
		tela.definirEscolha(vi.fn());
		quadro();
		expect(dados.clique).toBe('escolha');
		tela.definirPonto(vi.fn());
		quadro();
		expect(dados.clique).toBe('ponto');
	});

	it('trocar de espaço e esvaziarSelecao esvaziam a seleção do kernel (S3, S5)', () => {
		const { tela, doc, dados } = montar();
		clicar(410, 300);
		doc.clearScreenSelection.mockClear();
		tela.mostrarEspaco('Prancha A1');
		quadro();
		expect(doc.clearScreenSelection).toHaveBeenCalled();
		expect(dados.selecao).toBe('0');

		tela.mostrarEspaco('Model');
		clicar(410, 300);
		doc.clearScreenSelection.mockClear();
		tela.esvaziarSelecao();
		quadro();
		expect(doc.clearScreenSelection).toHaveBeenCalled();
		expect(dados.selecao).toBe('0');
	});

	it('trocar o fundo refaz o destaque da seleção, que continua', () => {
		const { tela, doc, dados } = montar();
		clicar(410, 302);
		doc.getScreenSelectionRanges.mockClear();
		tela.definirFundo(false);
		quadro();
		expect(dados.selecao).toBe('1');
		expect(doc.getScreenSelectionRanges).toHaveBeenCalledWith('Model', '0');
	});

	it('depois de uma edição, a contagem e o destaque vêm de novo do kernel (S4)', () => {
		const { tela, doc, dados } = montar();
		clicar(390, 250);
		clicar(420, 320);
		doc.existentes.delete('2');
		doc.getScreenSelectionRanges.mockClear();
		tela.atualizar();
		quadro();
		expect(dados.selecao).toBe('1');
		expect(doc.getScreenSelectionRanges).toHaveBeenCalledWith('Model', '0');
	});
});
