// Caminho relativo: e2e/desempenho.e2e.ts

/**
 * \file e2e/desempenho.e2e.ts
 * \brief Prova local dos não-funcionais N1–N3 (spec 00) no desenho sintético.
 * \author Iago Leal
 * \date 2026-10-31
 *
 * Mede, contra a tela desenhada pelo kernel (ADR 0008), o tempo de abrir o
 * desenho inteiro (N1), o de refletir uma linha nova (N2) e o tempo entre
 * quadros durante pan e zoom (N3). O que se observa na tela está na spec 07,
 * seção "O que os testes observam na tela".
 *
 * Só roda com `NEOCAD_SINTETICO_N`; N2 pede ainda o build com a sonda:
 *
 *     VITE_NEOCAD_SONDA=1 NEOCAD_SINTETICO_N=100000 pnpm test:e2e -- desempenho
 *
 * Até o MT-KV-10 a tela própria não existe e os três testes falham por isso.
 */

import { writeFile } from 'node:fs/promises';
import { expect, test, type Locator, type Page } from '@playwright/test';
import { gerarDxfSintetico } from '../scripts/gerar-dxf-sintetico.mjs';

/** O que a página expõe aos testes; nada disso existe fora deles. */
type JanelaDeTeste = {
	__neocadSonda?: { desenharLinha(x1: number, y1: number, x2: number, y2: number): string };
	__intervalos?: number[];
	__parar?: boolean;
};

/** Espera generosa para a tela aparecer: o limite de N1 é conferido à parte. */
const ESPERA_DA_TELA_MS = 60_000;

/** Gera o desenho, abre-o pelo diálogo e espera a tela mostrá-lo inteiro. */
async function abrirSintetico(page: Page): Promise<{ ms: number; tela: Locator }> {
	const n = Number(process.env.NEOCAD_SINTETICO_N);
	const { texto, contagem } = gerarDxfSintetico({ entidades: n, camadas: 20, semente: 1 });
	const caminho = test.info().outputPath('sintetico.dxf');
	await writeFile(caminho, texto);

	await page.goto('/');
	const chooser = page.waitForEvent('filechooser');
	const inicio = Date.now();
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await chooser).setFiles(caminho);

	const tela = page.locator('canvas[data-tela="kernel"]');
	await expect(tela).toHaveAttribute('data-entidades', String(contagem.modelo), {
		timeout: ESPERA_DA_TELA_MS
	});

	return { ms: Date.now() - inicio, tela };
}

/** Mediana de uma lista não vazia. */
function mediana(valores: number[]): number {
	const ordenados = [...valores].sort((a, b) => a - b);
	const meio = Math.floor(ordenados.length / 2);
	return ordenados.length % 2 ? ordenados[meio] : (ordenados[meio - 1] + ordenados[meio]) / 2;
}

test.describe('desempenho no desenho sintético (spec 00, N1–N3)', () => {
	test.skip(!process.env.NEOCAD_SINTETICO_N, 'defina NEOCAD_SINTETICO_N para medir N1–N3');
	test.beforeEach(() => test.setTimeout(600_000));

	test('N1: abre o desenho inteiro em até 5 s', async ({ page }) => {
		const { ms } = await abrirSintetico(page);

		test.info().annotations.push({ type: 'N1-ms', description: String(ms) });
		expect(ms).toBeLessThanOrEqual(5000);
	});

	test('N2: reflete uma linha nova em até 100 ms', async ({ page }) => {
		await abrirSintetico(page);

		const temSonda = await page.evaluate(
			() => (window as unknown as JanelaDeTeste).__neocadSonda !== undefined
		);
		expect(temSonda, 'build sem VITE_NEOCAD_SONDA=1').toBe(true);

		// A medida inteira roda no navegador: o vaivém com o Playwright custaria
		// mais que o limite que se quer provar.
		const ms = await page.evaluate(
			() =>
				new Promise<number>((resolve) => {
					const janela = window as unknown as JanelaDeTeste;
					const tela = document.querySelector('canvas[data-tela="kernel"]');
					const contar = () => Number(tela?.getAttribute('data-entidades'));
					const antes = contar();

					const t0 = performance.now();
					janela.__neocadSonda?.desenharLinha(0, 0, 10, 10);

					const conferir = () => {
						const decorrido = performance.now() - t0;
						if (contar() === antes + 1) resolve(decorrido);
						else if (decorrido > 2000) resolve(Number.POSITIVE_INFINITY);
						else requestAnimationFrame(conferir);
					};
					conferir();
				})
		);

		test.info().annotations.push({ type: 'N2-ms', description: String(ms) });
		expect(ms).toBeLessThanOrEqual(100);
	});

	test('N3: pan e zoom a 30 quadros por segundo pela GPU', async ({ page }) => {
		const { tela } = await abrirSintetico(page);

		// N3 vale enquanto a GPU desenha; no caminho de reserva não há meta de pan.
		await expect(tela).toHaveAttribute('data-caminho', 'webgl2');
		const quadrosAntes = Number(await tela.getAttribute('data-quadros'));

		await page.evaluate(() => {
			const janela = window as unknown as JanelaDeTeste;
			janela.__intervalos = [];
			janela.__parar = false;
			let anterior = performance.now();
			const medir = (agora: number) => {
				janela.__intervalos?.push(agora - anterior);
				anterior = agora;
				if (!janela.__parar) requestAnimationFrame(medir);
			};
			requestAnimationFrame(medir);
		});

		const caixa = await tela.boundingBox();
		expect(caixa, 'a tela não tem área visível').not.toBeNull();
		const cx = (caixa?.x ?? 0) + (caixa?.width ?? 0) / 2;
		const cy = (caixa?.y ?? 0) + (caixa?.height ?? 0) / 2;

		await page.mouse.move(cx, cy);
		await page.mouse.down({ button: 'middle' });
		for (let passo = 1; passo <= 60; passo++) await page.mouse.move(cx + 5 * passo, cy);
		await page.mouse.up({ button: 'middle' });

		for (let passo = 0; passo < 20; passo++) await page.mouse.wheel(0, passo % 2 ? 120 : -120);

		const intervalos = await page.evaluate(() => {
			const janela = window as unknown as JanelaDeTeste;
			janela.__parar = true;
			return janela.__intervalos ?? [];
		});

		const quadrosDepois = Number(await tela.getAttribute('data-quadros'));
		expect(quadrosDepois, 'pan e zoom não redesenharam a tela').toBeGreaterThan(quadrosAntes);

		// O primeiro intervalo mede desde o registro, não entre dois quadros.
		const m = mediana(intervalos.slice(1));
		test.info().annotations.push({ type: 'N3-mediana-ms', description: String(m) });
		expect(m).toBeLessThanOrEqual(1000 / 30);
	});
});
