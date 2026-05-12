// Caminho relativo: playwright.config.ts

/**
 * \file playwright.config.ts
 * \brief Configuração dos testes E2E do frontend NeoCAD.
 * \author Iago Leal
 * \date 2026-05-12
 */

import { defineConfig } from '@playwright/test';

export default defineConfig({
	webServer: { command: 'pnpm build && pnpm preview', port: 4173 },
	testMatch: '**/*.e2e.{ts,js}'
});
