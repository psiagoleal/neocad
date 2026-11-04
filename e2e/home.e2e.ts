// Caminho relativo: e2e/home.e2e.ts

/**
 * \file e2e/home.e2e.ts
 * \brief Abertura no Desenho1, abas de arquivo e Arquivo > Novo (spec 11, I1–I4).
 * \author Iago Leal
 * \date 2026-11-04
 */

import { readFile } from 'node:fs/promises';
import { expect, test, type Page } from '@playwright/test';

const LIVRE = 'Comando:';

function abas(page: Page) {
	return page.getByRole('tablist', { name: 'Desenhos abertos' });
}

async function digitar(page: Page, ...textos: string[]) {
	const linha = page.getByRole('textbox', { name: 'Linha de comando' });
	for (const texto of textos) {
		await linha.fill(texto);
		await linha.press('Enter');
	}
}

async function menuArquivo(page: Page, item: string) {
	await page.getByRole('button', { name: 'Arquivo' }).click();
	await page.getByRole('button', { name: item, exact: true }).click();
}

test('abre direto no Desenho1 vazio, com a aba Início inativa (I1)', async ({ page }) => {
	await page.goto('/');

	const tela = page.locator('canvas[data-tela="kernel"]');
	await expect(tela).toBeVisible();
	await expect(tela).toHaveAttribute('data-entidades', '0');
	await expect(page.getByRole('textbox', { name: 'Linha de comando' })).toHaveAttribute(
		'data-prompt',
		LIVRE
	);
	await expect(abas(page).getByRole('tab', { name: 'Desenho1' })).toHaveAttribute(
		'aria-selected',
		'true'
	);
	await expect(abas(page).getByRole('tab', { name: 'Início' })).toHaveAttribute(
		'aria-selected',
		'false'
	);
	await expect(page.getByRole('button', { name: 'Arquivo' })).toBeVisible();
});

test('as abas Início e Desenho1 se alternam e o desenho volta como estava (I2)', async ({
	page
}) => {
	test.setTimeout(60_000);
	await page.goto('/');
	const tela = page.locator('canvas[data-tela="kernel"]');
	await expect(tela).toBeVisible();

	await digitar(page, 'L', '0,0', '10,0');
	await expect(tela).toHaveAttribute('data-entidades', '1');

	await abas(page).getByRole('tab', { name: 'Início' }).click();
	await expect(
		page.getByRole('heading', { name: 'Abra um desenho e entre no workspace principal do NeoCAD' })
	).toBeVisible();
	await expect(tela).toBeHidden();

	await abas(page).getByRole('tab', { name: 'Desenho1' }).click();
	await expect(tela).toBeVisible();
	await expect(tela).toHaveAttribute('data-entidades', '1');
	await expect(page.getByRole('textbox', { name: 'Linha de comando' })).toHaveAttribute(
		'data-prompt',
		'Especifique o próximo ponto ou [Desfazer]:'
	);
	await expect(abas(page).getByRole('button', { name: /fechar/i })).toHaveCount(0);
});

test('o Desenho1 se salva em DXF e reabre com as mesmas entidades (I3)', async ({ page }) => {
	test.setTimeout(60_000);
	await page.goto('/');
	const tela = page.locator('canvas[data-tela="kernel"]');
	await expect(tela).toBeVisible();
	await digitar(page, 'L', '0,0', '10,0', '10,10', '');
	await expect(tela).toHaveAttribute('data-entidades', '2');

	const baixado = page.waitForEvent('download');
	await menuArquivo(page, 'Salvar');
	const download = await baixado;
	expect(download.suggestedFilename()).toBe('Desenho1.dxf');
	const caminho = await download.path();
	const conteudo = await readFile(caminho);
	expect(conteudo.toString('utf8')).toContain('LINE');

	const chooser = page.waitForEvent('filechooser');
	await menuArquivo(page, 'Abrir desenho CAD');
	await (
		await chooser
	).setFiles({
		name: 'Desenho1.dxf',
		mimeType: 'application/dxf',
		buffer: conteudo
	});
	await expect(abas(page).getByRole('tab', { name: 'Desenho1.dxf' })).toBeVisible({
		timeout: 30_000
	});
	await expect(tela).toHaveAttribute('data-entidades', '2');
});

test('Arquivo > Novo pede confirmação se houve edição; recusar mantém (I4)', async ({ page }) => {
	test.setTimeout(60_000);
	await page.goto('/');
	const tela = page.locator('canvas[data-tela="kernel"]');
	await expect(tela).toBeVisible();

	// Sem edição, não pergunta.
	const perguntas: string[] = [];
	let aceitar = false;
	page.on('dialog', (dialogo) => {
		perguntas.push(dialogo.message());
		void (aceitar ? dialogo.accept() : dialogo.dismiss());
	});
	await menuArquivo(page, 'Novo');
	await expect(abas(page).getByRole('tab', { name: 'Desenho2' })).toBeVisible();
	expect(perguntas).toHaveLength(0);

	await digitar(page, 'L', '0,0', '10,0', '');
	await expect(tela).toHaveAttribute('data-entidades', '1');

	await menuArquivo(page, 'Novo');
	await expect.poll(() => perguntas.length).toBe(1);
	await expect(abas(page).getByRole('tab', { name: 'Desenho2' })).toBeVisible();
	await expect(tela).toHaveAttribute('data-entidades', '1');

	aceitar = true;
	await menuArquivo(page, 'Novo');
	await expect(abas(page).getByRole('tab', { name: 'Desenho3' })).toBeVisible();
	await expect(tela).toHaveAttribute('data-entidades', '0');
	expect(perguntas).toHaveLength(2);
});

test('Abrir também pede confirmação se houve edição; recusar não abre nada (I4)', async ({
	page
}) => {
	test.setTimeout(60_000);
	await page.goto('/');
	const tela = page.locator('canvas[data-tela="kernel"]');
	await expect(tela).toBeVisible();
	await digitar(page, 'L', '0,0', '10,0', '');
	await expect(tela).toHaveAttribute('data-entidades', '1');

	const perguntas: string[] = [];
	page.on('dialog', (dialogo) => {
		perguntas.push(dialogo.message());
		void dialogo.dismiss();
	});
	let escolheu = false;
	page.on('filechooser', () => {
		escolheu = true;
	});
	await menuArquivo(page, 'Abrir desenho CAD');
	await expect.poll(() => perguntas.length).toBe(1);
	expect(perguntas[0]).toContain('Desenho1');
	expect(escolheu).toBe(false);
	await expect(abas(page).getByRole('tab', { name: 'Desenho1' })).toBeVisible();
	await expect(tela).toHaveAttribute('data-entidades', '1');
});
