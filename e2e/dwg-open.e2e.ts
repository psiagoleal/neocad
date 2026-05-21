// Caminho relativo: e2e/dwg-open.e2e.ts

/// <reference types="node" />

/**
 * \file e2e/dwg-open.e2e.ts
 * \brief Teste E2E para validar a abertura do desenho DWG de referência do projeto.
 * \author Iago Leal
 * \date 2026-05-12
 */

import fs from 'node:fs';
import path from 'node:path';
import { expect, test } from '@playwright/test';

const referenceDwgPath = path.join(process.cwd(), 'ANT.DS.L2.01.0001.01.02 - Básico.dwg');

test.skip(
	!fs.existsSync(referenceDwgPath),
	'Requer o arquivo DWG de referência disponível na raiz do repositório.'
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
	await expect(
		page.getByRole('heading', { name: 'ANT.DS.L2.01.0001.01.02 - Básico.dwg', exact: true })
	).toBeVisible({
		timeout: 120_000
	});
	await expect(page.getByText('Falha ao carregar', { exact: false })).toHaveCount(0);
});
