// Caminho relativo: e2e/kernel-view.e2e.ts

/**
 * \file e2e/kernel-view.e2e.ts
 * \brief Verifica que o que se desenha é o que se salva (spec 07).
 * \author Iago Leal
 * \date 2026-10-30
 *
 * A barra de comandos ainda é do upstream, e salvar grava o modelo do kernel,
 * que não recebe as edições feitas por essa barra (achado C4); o primeiro teste
 * prova o defeito antes da correção. Os seguintes conferem a tela desenhada pelo
 * kernel (V1 sem texto; spec 00, N4). Os demais critérios da spec 07 (V2, V3,
 * V5b, V7) entram aqui quando a fase KV os implementar.
 */

import path from 'node:path';
import { readFile } from 'node:fs/promises';
import { expect, test, type Page } from '@playwright/test';

const FIXTURE = path.join(process.cwd(), 'e2e/fixtures/minimal.dxf');

/** Conta as entidades LINE no texto de um DXF. */
function contarLinhas(dxf: string): number {
	return (dxf.match(/^\s*0\r?\n\s*LINE\r?$/gm) ?? []).length;
}

/** Abre um desenho pelo diálogo de arquivo e espera o upstream e o kernel. */
async function abrirDesenho(page: Page, caminho: string): Promise<void> {
	await page.goto('/');

	const chooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await chooser).setFiles(caminho);

	await expect(page.getByText('Desenho carregado com sucesso')).toBeVisible({ timeout: 30_000 });
	await expect(page.getByText(/^Kernel: /).first()).toBeVisible({ timeout: 30_000 });
}

/**
 * Desenha uma linha pela barra de comandos do upstream.
 *
 * O nome do comando vai na barra; os pontos vão nas duas caixas flutuantes
 * junto ao cursor (X e Y). Cada passo espera o prompt seguinte, para que uma
 * falha aqui seja de preparação, e não se confunda com o defeito provado.
 */
async function desenharLinha(
	page: Page,
	de: { x: number; y: number },
	ate: { x: number; y: number }
): Promise<void> {
	const barra = page.locator('.ml-cli-text');
	const prompt = page.locator('.ml-cli-prompt');
	const caixas = page.locator('input:visible:not(.ml-cli-text)');

	await barra.fill('LINE');
	await barra.press('Enter');
	await expect(prompt).toContainText('first point');

	await caixas.nth(0).fill(String(de.x));
	await caixas.nth(1).fill(String(de.y));
	await caixas.nth(1).press('Enter');
	await expect(prompt).toContainText('next point');

	await caixas.nth(0).fill(String(ate.x));
	await caixas.nth(1).fill(String(ate.y));
	await caixas.nth(1).press('Enter');
	await expect(prompt).toContainText('Close');

	await page.keyboard.press('Escape');
}

test('a linha desenhada por comando está no arquivo salvo e reaberto (V4)', async ({ page }) => {
	// Defeito conhecido (achado C4): a barra do upstream edita só o modelo do
	// upstream, e salvar grava o do kernel. O `test.fail()` sai quando a fase
	// KV fechar; se este teste passar antes disso, a CI acusa.
	test.fail();
	test.setTimeout(120_000);

	await abrirDesenho(page, FIXTURE);

	const original = await readFile(FIXTURE, 'utf-8');
	expect(contarLinhas(original)).toBe(1);

	await desenharLinha(page, { x: 0, y: 0 }, { x: 100, y: 50 });

	const download = page.waitForEvent('download');
	await page.getByRole('button', { name: 'Arquivo' }).click();
	await page.getByRole('button', { name: 'Salvar', exact: true }).click();
	const destino = test.info().outputPath('salvo.dxf');
	await (await download).saveAs(destino);

	// A da fixture e a desenhada.
	expect(contarLinhas(await readFile(destino, 'utf-8'))).toBe(2);

	await abrirDesenho(page, destino);
	await expect(page.getByText(/^Kernel: /).first()).toContainText('3 entidade(s)');
});

/** A tela própria, como a spec 07 a descreve para os testes. */
function telaDoKernel(page: Page) {
	return page.locator('canvas[data-tela="kernel"]');
}

/** Abre pelo diálogo de arquivo, sem esperar o upstream. */
async function abrirSoPeloKernel(page: Page, caminho: string): Promise<void> {
	await page.goto('/');
	const chooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await chooser).setFiles(caminho);
}

test('a tela do kernel mostra as entidades do modelo (V1, sem texto)', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirSoPeloKernel(page, FIXTURE);

	const tela = telaDoKernel(page);
	// A fixture tem uma LINE e um CIRCLE no espaço-modelo.
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });
	await expect(tela).toHaveAttribute('data-espaco', 'Model');
	// Sem GPU, a CI cai na reserva; os dois caminhos valem aqui.
	await expect(tela).toHaveAttribute('data-caminho', /^(webgl2|canvas2d)$/);
	await expect(tela).toBeVisible();
});

test('cada layout tem a sua aba e a sua contagem (V1, sem texto)', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirSoPeloKernel(page, path.join(process.cwd(), 'e2e/fixtures/two-layouts.dxf'));

	const tela = telaDoKernel(page);
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });

	const abas = page.getByRole('tablist', { name: 'Espaços do desenho' });
	await abas.getByRole('tab', { name: 'Prancha A1' }).click();
	await expect(tela).toHaveAttribute('data-espaco', 'Prancha A1');
	await expect(tela).toHaveAttribute('data-entidades', '2');

	await abas.getByRole('tab', { name: 'Prancha A2' }).click();
	await expect(tela).toHaveAttribute('data-espaco', 'Prancha A2');
	await expect(tela).toHaveAttribute('data-entidades', '1');

	await abas.getByRole('tab', { name: 'Model' }).click();
	await expect(tela).toHaveAttribute('data-entidades', '2');
});

test('sem WebGL2, a tela cai sozinha no canvas 2D (N4)', async ({ page }) => {
	test.setTimeout(60_000);
	// Tira o WebGL2 só da tela própria. Tirá-lo da página inteira derruba também
	// o visualizador do upstream, que hoje habilita a abertura de arquivos; isso
	// deixa de valer quando o upstream sair da tela (MT-KV-13).
	await page.addInitScript(() => {
		const original = HTMLCanvasElement.prototype.getContext;
		HTMLCanvasElement.prototype.getContext = function (
			this: HTMLCanvasElement,
			tipo: string,
			...resto: unknown[]
		) {
			if (tipo === 'webgl2' && this.dataset.tela === 'kernel') return null;
			return (original as (...a: unknown[]) => unknown).call(this, tipo, ...resto);
		} as typeof HTMLCanvasElement.prototype.getContext;
	});
	await abrirSoPeloKernel(page, FIXTURE);

	const tela = telaDoKernel(page);
	await expect(tela).toHaveAttribute('data-caminho', 'canvas2d', { timeout: 30_000 });
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });

	// Pan e zoom também redesenham pela reserva.
	const antes = Number(await tela.getAttribute('data-quadros'));
	const caixa = await tela.boundingBox();
	expect(caixa).not.toBeNull();
	await page.mouse.move((caixa?.x ?? 0) + 100, (caixa?.y ?? 0) + 100);
	await page.mouse.wheel(0, -120);
	await expect
		.poll(async () => Number(await tela.getAttribute('data-quadros')))
		.toBeGreaterThan(antes);
});
