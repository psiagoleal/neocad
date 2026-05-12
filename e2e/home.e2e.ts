// Caminho relativo: e2e/home.e2e.ts

/**
 * \file e2e/home.e2e.ts
 * \brief Teste E2E básico da tela inicial do NeoCAD.
 * \author Iago Leal
 * \date 2026-05-12
 */

import { expect, test } from '@playwright/test';

test('exibe a tela inicial do NeoCAD', async ({ page }) => {
	await page.goto('/');

	await expect(page.getByRole('heading', { name: 'NeoCAD' })).toBeVisible();
	await expect(page.getByText('Fase 1 — scaffold base concluído')).toBeVisible();
	await expect(page.getByText('Plataformas alvo')).toBeVisible();
});
