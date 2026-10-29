// Caminho relativo: e2e/dwg-open.e2e.ts

/// <reference types="node" />

/**
 * \file e2e/dwg-open.e2e.ts
 * \brief Teste E2E para validar a abertura de um desenho DWG de referência local.
 *
 * O desenho não é versionado: o caminho vem de `NEOCAD_REFERENCE_DWG`, e sem
 * ele o teste é pulado.
 * \author Iago Leal
 * \date 2026-05-12
 */

import fs from 'node:fs';
import path from 'node:path';
import { expect, test } from '@playwright/test';

const referenceDwgPath = process.env.NEOCAD_REFERENCE_DWG ?? '';
const referenceDwgName = path.basename(referenceDwgPath);

test.skip(
	!referenceDwgPath || !fs.existsSync(referenceDwgPath),
	'Requer NEOCAD_REFERENCE_DWG apontando para um DWG local.'
);

test('abre o arquivo DWG de referência do NeoCAD', async ({ page }) => {
	test.setTimeout(120_000);

	await page.goto('/');

	const fileChooserPromise = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();

	const fileChooser = await fileChooserPromise;
	await fileChooser.setFiles(referenceDwgPath);

	await expect(page.getByRole('button', { name: 'Fundo escuro' })).toBeVisible({
		timeout: 120_000
	});
	await expect(page.getByRole('heading', { name: referenceDwgName, exact: true })).toBeVisible({
		timeout: 120_000
	});
	await expect(page.getByText('Falha ao carregar', { exact: false })).toHaveCount(0);
});
