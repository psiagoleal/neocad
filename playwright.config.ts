// Caminho relativo: playwright.config.ts

/**
 * \file playwright.config.ts
 * \brief Configuração dos testes E2E do frontend NeoCAD.
 * \author Iago Leal
 * \date 2026-05-12
 */

import { existsSync } from 'node:fs';
import { defineConfig } from '@playwright/test';

const SYSTEM_CHROME = '/usr/bin/google-chrome';

/**
 * Em Ubuntu 26.04, `playwright install` recusa baixar o navegador — a versão do
 * Playwright deste projeto pede um build que o servidor não entrega para esta
 * release. O Chrome do sistema resolve, e é o que se usa fora da CI.
 *
 * Na CI o runner é uma release suportada e o download funciona, então lá se
 * mantém o navegador do próprio Playwright, que é o que a suíte oficialmente
 * tem como alvo.
 */
const useSystemChrome = !process.env.CI && existsSync(SYSTEM_CHROME);

export default defineConfig({
	// A sonda de edição (spec 07) entra no build dos testes; o de produção não a tem.
	webServer: {
		command: 'pnpm build && pnpm preview',
		port: 4173,
		env: { VITE_NEOCAD_SONDA: '1' }
	},
	testMatch: '**/*.e2e.{ts,js}',
	use: useSystemChrome ? { channel: 'chrome' } : {}
});
