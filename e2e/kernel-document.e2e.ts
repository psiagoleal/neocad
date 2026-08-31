// Caminho relativo: e2e/kernel-document.e2e.ts

/**
 * \file e2e/kernel-document.e2e.ts
 * \brief Verifica que o desenho aberto pelo upstream chega ao modelo próprio.
 * \author Iago Leal
 * \date 2026-08-08
 *
 * Enquanto K5 e K6 não chegam, o upstream lê o arquivo e desenha, e o kernel
 * passa a ser a fonte de verdade sobre o que existe no desenho. As duas
 * representações convivem, e é justamente a concordância entre elas que estes
 * testes protegem: uma divergência silenciosa aqui só apareceria muito depois,
 * como painel mostrando o que o canvas não mostra.
 */

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';

const FIXTURES = path.join(process.cwd(), 'e2e/fixtures');

/** Abre uma fixture pelo diálogo de arquivo e espera o upstream concluir. */
async function abrirDesenho(page: Page, fixture: string): Promise<void> {
	await page.goto('/');

	const fileChooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await fileChooser).setFiles(path.join(FIXTURES, fixture));

	await expect(page.getByText('Desenho carregado com sucesso')).toBeVisible({
		timeout: 30_000
	});
}

/**
 * Espera a mensagem que o kernel emite ao receber o desenho, e a devolve.
 *
 * A espera é generosa de propósito: carregar o desenho no kernel envolve buscar
 * e instanciar o WebAssembly, e a primeira execução do dia paga esse custo. É a
 * mesma folga que a suíte já dá às operações do upstream.
 */
async function esperarMensagemDoKernel(page: Page) {
	const mensagem = page.getByText(/^Kernel: /).first();
	await expect(mensagem).toBeVisible({ timeout: 30_000 });

	return mensagem;
}

test('o kernel recebe exatamente as entidades e camadas do arquivo', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirDesenho(page, 'minimal.dxf');

	// A fixture tem uma LINE, um CIRCLE e apenas a camada 0. A contagem do
	// modelo próprio precisa reproduzir o arquivo, e não uma aproximação.
	const mensagem = await esperarMensagemDoKernel(page);

	await expect(mensagem).toHaveText(
		'Kernel: 2 entidade(s) em 1 camada(s). arquivo compreendido por inteiro.'
	);
});

test('entidade não modelada é reportada sem impedir a abertura', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirDesenho(page, 'with-unsupported.dxf');

	const mensagem = await esperarMensagemDoKernel(page);

	// A LINE chega ao kernel; a SOLID é contada como ainda não representada. Um
	// arquivo real não pode deixar de abrir por causa de uma entidade que o
	// kernel ainda não modela.
	await expect(mensagem).toContainText('1 entidade(s)');
	await expect(mensagem).toContainText('não representada(s)');

	// O upstream continua desenhando o arquivo inteiro.
	await expect(page.getByRole('button', { name: 'Fundo escuro' })).toBeVisible();
});

test('o menu Editar reflete o histórico zerado logo após a abertura', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirDesenho(page, 'minimal.dxf');

	await page.getByRole('button', { name: 'Editar' }).click();

	// Carregar um desenho zera o histórico de propósito: desfazer para antes da
	// abertura não faz sentido. O menu tem de dizer isso, e não oferecer uma
	// ação inócua.
	await expect(page.getByRole('button', { name: /^Desfazer/ })).toBeDisabled();
	await expect(page.getByRole('button', { name: /^Refazer/ })).toBeDisabled();
	await expect(page.getByText('Nenhuma ação a desfazer nesta sessão.')).toBeVisible();
});

test('a chegada do kernel não regride a exibição do desenho', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirDesenho(page, 'minimal.dxf');

	// Mesma garantia de `viewer-render.e2e.ts`: o canvas precisa ter altura real.
	// Repetida aqui porque o carregamento no kernel passou a acontecer no mesmo
	// evento que ativa o documento, e uma falha ali não pode levar o canvas junto.
	const surface = page.locator('.viewer-surface');

	await expect(surface).toBeVisible();
	expect((await surface.boundingBox())?.height ?? 0).toBeGreaterThan(0);
});
