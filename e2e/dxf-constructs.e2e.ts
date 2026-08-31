// Caminho relativo: e2e/dxf-constructs.e2e.ts

/**
 * \file e2e/dxf-constructs.e2e.ts
 * \brief Cobre construtos de DXF real que o caminho de leitura precisa aguentar.
 * \author Iago Leal
 * \date 2026-08-10
 *
 * As fixturas aqui nasceram de uma bissecção sobre um desenho real que não
 * abria: partindo de `minimal.dxf`, que abre, cada construto foi somado
 * isoladamente até identificar qual quebrava a leitura. São sintéticas — nada
 * do arquivo de origem entra neste repositório (`AGENTS.md` §0.1).
 *
 * O valor delas é fixar dois comportamentos distintos:
 *
 * - o que o kernel **ainda não modela** deve ser contado e reportado, sem
 *   impedir a abertura;
 * - o que o parser do upstream **não consegue ler** deve continuar sendo um
 *   documento aqui, porque desde o MT-K2-12 quem lê o DXF é o kernel.
 */

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';

const FIXTURES = path.join(process.cwd(), 'e2e/fixtures');

/** Abre uma fixture e devolve se o upstream concluiu a leitura. */
async function abrir(page: Page, fixture: string): Promise<boolean> {
	await page.goto('/');

	const fileChooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await fileChooser).setFiles(path.join(FIXTURES, fixture));

	const sucesso = page.getByText('Desenho carregado com sucesso');
	const falha = page.getByText('Não foi possível abrir');

	await expect(sucesso.or(falha).first()).toBeVisible({ timeout: 45_000 });

	return (await sucesso.count()) > 0;
}

/** Mensagem que o kernel emite ao receber o desenho. */
async function mensagemDoKernel(page: Page) {
	const mensagem = page.getByText(/^Kernel: /).first();
	await expect(mensagem).toBeVisible({ timeout: 30_000 });

	return mensagem;
}

test('polilinha de estilo antigo é lida como polilinha, e não contada como perda', async ({
	page
}) => {
	test.setTimeout(90_000);

	// POLYLINE/VERTEX/SEQEND é como o DXF R12 representa polilinha, e aparece em
	// desenho real. Enquanto a extração vinha do upstream, ela era **contada
	// como não suportada**; a leitura nativa de K2 a monta como polilinha de
	// verdade. As três entidades da fixture chegam inteiras.
	expect(await abrir(page, 'legacy-polyline.dxf')).toBe(true);

	const mensagem = await mensagemDoKernel(page);
	await expect(mensagem).toContainText('3 entidade(s)');
	await expect(mensagem).toContainText('compreendido por inteiro');
});

test('referência de bloco abre e é contada como não representada', async ({ page }) => {
	test.setTimeout(90_000);

	// `INSERT` exige transformação de instância, que é fase K3. Até lá o esperado
	// é contar e relatar, não engasgar.
	expect(await abrir(page, 'block-reference.dxf')).toBe(true);
	await expect(await mensagemDoKernel(page)).toContainText('não representada(s)');
});

test('bloco com entidades dentro é lido pelo kernel, ainda que o upstream falhe', async ({
	page
}) => {
	test.setTimeout(90_000);

	// Este teste nasceu com `test.fail()`, registrando um defeito: o parser DXF
	// do upstream não abre arquivo cuja seção `BLOCKS` contenha bloco com
	// entidades — cerca de 11% de um acervo real, justamente a fatia dos
	// desenhos acabados, com carimbo e simbologia. Bloco com conteúdo é como se
	// define todo símbolo e marcador de estrutura.
	//
	// A leitura DXF nativa de K2 corrigiu a **compreensão** do arquivo. O que
	// segue sem funcionar é o traçado na tela, porque quem desenha ainda é o
	// upstream até K5. A asserção mede exatamente isso, e não mais: o desenho
	// existe, é contado, e é salvável.
	await page.goto('/');

	const fileChooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await fileChooser).setFiles(path.join(FIXTURES, 'block-with-entities.dxf'));

	// Duas entidades no espaço-modelo e um bloco — o que a fixture contém.
	const mensagem = await mensagemDoKernel(page);
	await expect(mensagem).toContainText('2 entidade(s)');
	await expect(mensagem).toContainText('1 bloco(s)');

	// E a aplicação fica com um documento aberto, e não em estado de falha: o
	// menu `Arquivo` oferece `Salvar` habilitado.
	await page.getByRole('button', { name: 'Arquivo' }).click();
	await expect(page.getByRole('button', { name: 'Salvar', exact: true })).toBeEnabled();
});
