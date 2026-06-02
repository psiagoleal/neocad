// Caminho relativo: e2e/viewer-render.e2e.ts

/// <reference types="node" />

/**
 * \file e2e/viewer-render.e2e.ts
 * \brief Regressão: o canvas do viewer deve ter altura real após abrir um desenho.
 * \author Iago Leal
 * \date 2026-06-02
 *
 * Guarda contra o colapso de layout em que o `.viewer-surface` caía na trilha
 * `auto` do grid (em vez da trilha `1fr`) quando a barra de progresso estava
 * ausente, deixando o canvas com altura 0 e a tela em branco.
 */

import path from 'node:path';
import { expect, test } from '@playwright/test';

const fixture = path.join(process.cwd(), 'e2e', 'fixtures', 'minimal.dxf');

test('o canvas do viewer tem altura real após abrir um DXF', async ({ page }) => {
	test.setTimeout(120_000);

	await page.goto('/');

	const fileChooserPromise = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	const fileChooser = await fileChooserPromise;
	await fileChooser.setFiles(fixture);

	await expect(page.getByRole('heading', { name: 'minimal.dxf', exact: true })).toBeVisible({
		timeout: 60_000
	});

	const viewerCanvas = page.locator('.viewer-container canvas').first();
	await expect(viewerCanvas).toBeVisible();

	// O bug deixava o canvas com altura ~0; exigimos uma área de desenho real.
	const box = await viewerCanvas.boundingBox();
	expect(box, 'o canvas do viewer deve existir e ter caixa de layout').not.toBeNull();
	expect(box!.height).toBeGreaterThan(200);
	expect(box!.width).toBeGreaterThan(200);
});
