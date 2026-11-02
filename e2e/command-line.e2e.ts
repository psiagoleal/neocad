// Caminho relativo: e2e/command-line.e2e.ts

/**
 * \file e2e/command-line.e2e.ts
 * \brief A linha de comando própria, pelo teclado (spec 08: C6, C7, C9, C12).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * O que se observa: a caixa de texto "Linha de comando", com o prompt de agora em
 * `data-prompt`; o histórico (`role="log"`); as sugestões (`role="listbox"`).
 */

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';

const FIXTURE = path.join(process.cwd(), 'e2e/fixtures/minimal.dxf');
const LIVRE = 'Comando:';
const PRIMEIRO = 'Especifique o primeiro ponto:';

async function abrir(page: Page) {
	await page.goto('/');
	const chooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await chooser).setFiles(FIXTURE);
	const tela = page.locator('canvas[data-tela="kernel"]');
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });
	return { tela, linha: page.getByRole('textbox', { name: 'Linha de comando' }) };
}

async function digitar(page: Page, ...textos: string[]) {
	const linha = page.getByRole('textbox', { name: 'Linha de comando' });
	for (const texto of textos) {
		await linha.fill(texto);
		await linha.press('Enter');
	}
}

test('Enter e Espaço na linha vazia repetem o último comando (C6)', async ({ page }) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);
	await expect(linha).toHaveAttribute('data-prompt', LIVRE);

	await digitar(page, 'L', '0,0', '10,0', '');
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await expect(linha).toHaveAttribute('data-prompt', LIVRE);

	await linha.press('Enter');
	await expect(linha).toHaveAttribute('data-prompt', PRIMEIRO);
	await linha.press('Escape');
	await expect(linha).toHaveAttribute('data-prompt', LIVRE);

	await linha.press(' ');
	await expect(linha).toHaveAttribute('data-prompt', PRIMEIRO);
});

test('Esc encerra o comando e as retas confirmadas ficam; sem comando, limpa o texto (C7)', async ({
	page
}) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);

	await digitar(page, 'linha', '0,0', '@10,0');
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await linha.fill('@0,10');
	await linha.press('Escape');

	await expect(linha).toHaveAttribute('data-prompt', LIVRE);
	await expect(linha).toHaveValue('');
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await expect(page.getByRole('log', { name: 'Histórico de comandos' })).toContainText(
		'*Cancelado*'
	);

	await linha.fill('abc');
	await linha.press('Escape');
	await expect(linha).toHaveValue('');
});

test('o histórico mostra o eco e a resposta "comando desconhecido" (C5)', async ({ page }) => {
	test.setTimeout(60_000);
	await abrir(page);
	await digitar(page, 'xyzzy');
	const historico = page.getByRole('log', { name: 'Histórico de comandos' });
	await expect(historico).toContainText('Comando: xyzzy');
	await expect(historico).toContainText('xyzzy: comando desconhecido.');
});

test('as sugestões aparecem nos dois idiomas e em até 50 ms (C9, C12)', async ({ page }) => {
	test.setTimeout(60_000);
	const { linha } = await abrir(page);
	const sugestoes = page.getByRole('listbox', { name: 'Sugestões' });

	await linha.fill('linh');
	await expect(sugestoes.getByRole('option', { name: /^LINE LINHA/ })).toBeVisible();

	// A medida roda no navegador: do evento de entrada à lista atualizada.
	const ms = await linha.evaluate(
		(campo) =>
			new Promise<number>((resolve) => {
				const entrada = campo as HTMLInputElement;
				const t0 = performance.now();
				entrada.value = 'hach';
				entrada.dispatchEvent(new Event('input', { bubbles: true }));
				const conferir = () => {
					const lista = document.querySelector('[role="listbox"][aria-label="Sugestões"]');
					if (lista?.textContent?.includes('HATCH')) resolve(performance.now() - t0);
					else if (performance.now() - t0 > 1000) resolve(Number.POSITIVE_INFINITY);
					else queueMicrotask(conferir);
				};
				conferir();
			})
	);
	test.info().annotations.push({ type: 'C12-ms', description: String(ms) });
	expect(ms).toBeLessThanOrEqual(50);
});

test('a barra de comandos do upstream não é oferecida (V5)', async ({ page }) => {
	test.setTimeout(60_000);
	await abrir(page);
	await expect(page.getByRole('textbox', { name: 'Linha de comando' })).toBeVisible();
	await expect(page.getByPlaceholder('Type command')).toBeHidden();
});
