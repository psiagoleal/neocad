// Caminho relativo: e2e/home.e2e.ts

/**
 * \file e2e/home.e2e.ts
 * \brief Teste E2E básico da tela principal do NeoCAD na Fase 2.
 * \author Iago Leal
 * \date 2026-05-12
 */

import { expect, test } from '@playwright/test';

test('exibe a interface principal do NeoCAD no fluxo inicial', async ({ page }) => {
	await page.goto('/');

	await expect(page.getByRole('heading', { name: 'NeoCAD', exact: true })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Abrir desenho CAD' }).first()).toBeVisible();
	await expect(page.getByRole('button', { name: 'Arquivo' })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Janela' })).toBeVisible();
	await expect(
		page.getByRole('heading', {
			name: 'Abra um desenho e entre no workspace principal do NeoCAD'
		})
	).toBeVisible();
	await expect(page.getByRole('button', { name: /Mensagens/ }).first()).toBeVisible();
});
