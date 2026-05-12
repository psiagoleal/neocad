// Caminho relativo: e2e/home.e2e.ts

/**
 * \file e2e/home.e2e.ts
 * \brief Teste E2E básico da tela principal do NeoCAD na Fase 2.
 * \author Iago Leal
 * \date 2026-05-12
 */

import { expect, test } from '@playwright/test';

test('exibe a interface principal do NeoCAD', async ({ page }) => {
	await page.goto('/');

	await expect(page.getByRole('heading', { name: 'NeoCAD' })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Abrir desenho CAD' })).toBeVisible();
	await expect(page.getByText('Integração do viewer')).toBeVisible();
	await expect(page.getByText('Fase 2 — integração inicial do viewer')).toBeVisible();
});
