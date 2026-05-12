// Caminho relativo: e2e/dwg-open.e2e.ts

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
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).click();

	const fileChooser = await fileChooserPromise;
	await fileChooser.setFiles(referenceDwgPath);

	await expect(page.getByText('Desenho carregado com sucesso:', { exact: false })).toBeVisible({
		timeout: 120_000
	});
	await expect(page.getByText('Falha ao carregar', { exact: false })).toHaveCount(0);
});
