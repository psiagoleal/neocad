// Caminho relativo: e2e/kernel-view.e2e.ts

/**
 * \file e2e/kernel-view.e2e.ts
 * \brief Verifica que o que se desenha é o que se salva (spec 07).
 * \author Iago Leal
 * \date 2026-10-30
 *
 * Hoje o canvas e a barra de comandos são do upstream, e salvar grava o modelo
 * do kernel, que não recebe as edições feitas por essa barra (achado C4). Este
 * arquivo prova o defeito antes de qualquer correção; os demais critérios da
 * spec 07 (V2, V3, V5b, V7) entram aqui quando a fase KV os implementar.
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
