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
 * - o que o parser do upstream **não consegue ler** fica registrado como falha
 *   conhecida, para que a correção seja percebida quando acontecer.
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

test('polilinha de estilo antigo abre e é contada como não suportada', async ({ page }) => {
	test.setTimeout(90_000);

	// POLYLINE/VERTEX/SEQEND é como o DXF R12 representa polilinha, e aparece em
	// desenho real. O kernel modela LWPOLYLINE, não esta — o esperado é contar,
	// não engasgar.
	expect(await abrir(page, 'legacy-polyline.dxf')).toBe(true);
	await expect(await mensagemDoKernel(page)).toContainText('não suportada(s)');
});

test('referência de bloco abre e é contada como não suportada', async ({ page }) => {
	test.setTimeout(90_000);

	expect(await abrir(page, 'block-reference.dxf')).toBe(true);
	await expect(await mensagemDoKernel(page)).toContainText('não suportada(s)');
});

test('bloco com entidades dentro ainda não é legível pelo parser do upstream', async ({ page }) => {
	test.setTimeout(90_000);

	// Falha conhecida, e não expectativa: `test.fail()` inverte o resultado, de
	// modo que este teste passa enquanto o defeito existir e **quebra no dia em
	// que ele for corrigido** — que é quando queremos ser avisados, para trocar
	// esta asserção pela definitiva.
	//
	// Bloco com conteúdo é como se define todo símbolo, carimbo e marcador de
	// estrutura, então na prática DXF de origem real não abre. A leitura DXF
	// nativa de K2 substitui este parser e deve corrigir o caso.
	test.fail();

	expect(await abrir(page, 'block-with-entities.dxf')).toBe(true);
});
