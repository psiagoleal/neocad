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
	await page.getByRole('button', { name: 'Arquivo' }).click();
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).click();
	await (await chooser).setFiles(FIXTURE);
	const tela = page.locator('canvas[data-tela="kernel"]');
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });
	// Só depois da carga do upstream a tela para de mudar de tamanho (a barra de
	// progresso some): antes disso, um clique calculado em pixels pode errar o alvo.
	await expect(page.getByText('Desenho carregado com sucesso')).toBeVisible({ timeout: 30_000 });
	await expect(tela).toHaveAttribute('data-fundo', 'transparente');
	await expect(page.locator('.viewer-status-row')).toHaveCount(0);
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

/** Pixel da página onde fica o ponto (x, y) do desenho, pela vista exposta em `data-vista`. */
async function pixel(page: Page, x: number, y: number) {
	const tela = page.locator('canvas[data-tela="kernel"]');
	const [cx, cy, escala] = ((await tela.getAttribute('data-vista')) ?? '').split(' ').map(Number);
	const caixa = await tela.boundingBox();
	if (caixa == null) throw new Error('a tela não tem área visível');
	return {
		x: caixa.x + caixa.width / 2 + (x - cx) * escala,
		y: caixa.y + caixa.height / 2 - (y - cy) * escala
	};
}

async function clicarNoDesenho(page: Page, x: number, y: number) {
	const p = await pixel(page, x, y);
	await page.mouse.click(p.x, p.y);
}

test('ERASE com seleção apaga num só passo de desfazer (E1)', async ({ page }) => {
	test.setTimeout(60_000);
	const { tela } = await abrir(page);
	// A fixture: LINE de (0,0) a (100,100) e CIRCLE de raio 40 em (50,50).
	await clicarNoDesenho(page, -3, -3);
	await clicarNoDesenho(page, 103, 103);
	await expect(tela).toHaveAttribute('data-selecao', '2');

	await digitar(page, 'apagar');
	await expect(tela).toHaveAttribute('data-entidades', '0');
	await expect(tela).toHaveAttribute('data-selecao', '0');

	await page.getByRole('button', { name: 'Editar' }).click();
	await page.getByRole('button', { name: /^Desfazer/ }).click();
	await expect(tela).toHaveAttribute('data-entidades', '2');
});

test('ERASE sem seleção pede objetos e apaga ao confirmar (E2)', async ({ page }) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);
	await digitar(page, 'E');
	await expect(linha).toHaveAttribute('data-prompt', 'Selecione objetos:');

	await clicarNoDesenho(page, 10, 10);
	await expect(tela).toHaveAttribute('data-selecao', '1');
	await digitar(page, '');
	await expect(tela).toHaveAttribute('data-entidades', '1');
	await expect(linha).toHaveAttribute('data-prompt', LIVRE);
});

test('Delete fora de campo de texto apaga a seleção (E3)', async ({ page }) => {
	test.setTimeout(60_000);
	const { tela } = await abrir(page);
	await clicarNoDesenho(page, 10, 10);
	await expect(tela).toHaveAttribute('data-selecao', '1');

	await page.locator('body').focus();
	await page.keyboard.press('Delete');
	await expect(tela).toHaveAttribute('data-entidades', '1');
	await expect(tela).toHaveAttribute('data-selecao', '0');
});

test('MOVE e COPY sobre a seleção, pelo kernel (M1, M3)', async ({ page }) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);
	// A fixture: LINE de (0,0) a (100,100) e CIRCLE de raio 40 em (50,50).
	await clicarNoDesenho(page, 10, 10);
	await expect(tela).toHaveAttribute('data-selecao', '1');
	await digitar(page, 'mover', '0,0', '@0,50');
	await expect(tela).toHaveAttribute('data-caixa', '0 10 100 150');
	await expect(tela).toHaveAttribute('data-selecao', '0');
	await expect(linha).toHaveAttribute('data-prompt', LIVRE);

	await digitar(page, 'CO');
	await expect(linha).toHaveAttribute('data-prompt', 'Selecione objetos:');
	await clicarNoDesenho(page, 90, 50);
	await expect(tela).toHaveAttribute('data-selecao', '1');
	await digitar(page, '', '50,50', '250,50', '');
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await expect(tela).toHaveAttribute('data-caixa', '0 10 290 150');

	await page.getByRole('button', { name: 'Editar' }).click();
	await page.getByRole('button', { name: /^Desfazer/ }).click();
	await expect(tela).toHaveAttribute('data-entidades', '2');
});

test('ROTATE, MIRROR e SCALE sobre a seleção, pelo kernel (R1–R3)', async ({ page }) => {
	test.setTimeout(60_000);
	const { tela } = await abrir(page);
	// A fixture: LINE de (0,0) a (100,100) e CIRCLE de raio 40 em (50,50).
	await clicarNoDesenho(page, 10, 10);
	await digitar(page, 'RO', '0,0', '90');
	// A reta vai para (0,0)–(-100,100).
	await expect(tela).toHaveAttribute('data-caixa', '-100 0 90 100');

	await clicarNoDesenho(page, 90, 50);
	await digitar(page, 'espelhar', '0,0', '0,10', '');
	// Fica o círculo original e entra a cópia em (-50,50).
	await expect(tela).toHaveAttribute('data-entidades', '3');

	await clicarNoDesenho(page, -10, 10);
	await expect(tela).toHaveAttribute('data-selecao', '1');
	await digitar(page, 'SC', '0,0', '0.5');
	await expect(tela).toHaveAttribute('data-caixa', '-90 0 90 90');
});

test('OFFSET: o objeto é escolhido pelo clique, sem Enter (O1)', async ({ page }) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);
	await digitar(page, 'DESLOCAMENTO', '10');
	await expect(linha).toHaveAttribute(
		'data-prompt',
		'Selecione o objeto a deslocar ou [Sair/Desfazer] <Sair>:'
	);
	// A fixture: LINE de (0,0) a (100,100) e CIRCLE de raio 40 em (50,50).
	await clicarNoDesenho(page, 10, 10);
	await expect(linha).toHaveAttribute('data-prompt', 'Especifique o ponto no lado a deslocar:');
	await digitar(page, '0,50');
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await expect(linha).toHaveAttribute(
		'data-prompt',
		'Selecione o objeto a deslocar ou [Sair/Desfazer] <Sair>:'
	);
	await digitar(page, '');
	await expect(linha).toHaveAttribute('data-prompt', LIVRE);
});

test('TRIM e EXTEND: arestas por todas, objeto pelo clique (T1, T2, T4)', async ({ page }) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);
	// A fixture: LINE de (0,0) a (100,100) e CIRCLE de raio 40 em (50,50).
	await digitar(page, 'APARAR', '');
	await expect(linha).toHaveAttribute('data-prompt', 'Selecione o objeto a aparar ou [Desfazer]:');
	await clicarNoDesenho(page, 5, 5);
	// O trecho da reta até o círculo sai: a caixa passa a começar no círculo.
	await expect(tela).toHaveAttribute('data-caixa', '10 10 100 100');
	await expect(tela).toHaveAttribute('data-selecao', '0');
	await digitar(page, '');

	// Uma reta à direita do círculo, em y = 50, estendida para a esquerda até ele (x = 90).
	await digitar(page, 'L', '150,50', '130,50', '');
	await digitar(page, 'EX', '');
	await expect(linha).toHaveAttribute(
		'data-prompt',
		'Selecione o objeto a estender ou [Desfazer]:'
	);
	await clicarNoDesenho(page, 131, 50);
	await digitar(page, '');
	await expect(linha).toHaveAttribute('data-prompt', LIVRE);
	// O clique de escolha não selecionou nada (T4).
	await expect(tela).toHaveAttribute('data-selecao', '0');
	// O trecho novo existe: clicar nele seleciona a reta.
	await clicarNoDesenho(page, 100, 50);
	await expect(tela).toHaveAttribute('data-selecao', '1');
});

test('FILLET e CHAMFER: dois cliques ligam os objetos pelo kernel (F1–F3)', async ({ page }) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);
	// Duas retas perpendiculares que não se tocam, à direita da fixture.
	await digitar(page, 'L', '150,0', '150,40', '', 'L', '160,50', '200,50', '');
	await expect(tela).toHaveAttribute('data-entidades', '4');

	await digitar(page, 'CONCORD', 'R', '5');
	await expect(linha).toHaveAttribute('data-prompt', 'Selecione o primeiro objeto ou [Raio]:');
	await clicarNoDesenho(page, 150, 20);
	await expect(linha).toHaveAttribute('data-prompt', 'Selecione o segundo objeto:');
	await clicarNoDesenho(page, 180, 50);
	// Entra o arco de raio 5.
	await expect(tela).toHaveAttribute('data-entidades', '5');
	await expect(linha).toHaveAttribute('data-prompt', LIVRE);

	await digitar(page, 'L', '250,0', '250,40', '', 'L', '260,50', '300,50', '');
	await digitar(page, 'CHA', 'D', '5', '5');
	await clicarNoDesenho(page, 250, 20);
	await clicarNoDesenho(page, 280, 50);
	// Entra o segmento do chanfro.
	await expect(tela).toHaveAttribute('data-entidades', '8');
	await expect(tela).toHaveAttribute('data-selecao', '0');
});

test('a recusa do kernel aparece no histórico e a tela não muda (V5b)', async ({ page }) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);
	// Duas retas paralelas: não há concordância possível.
	await digitar(page, 'L', '150,0', '190,0', '', 'L', '150,20', '190,20', '');
	await expect(tela).toHaveAttribute('data-entidades', '4');
	await digitar(page, 'F');
	await clicarNoDesenho(page, 170, 0);
	await clicarNoDesenho(page, 170, 20);
	await expect(page.getByRole('log', { name: 'Histórico de comandos' })).toContainText(
		'O kernel recusou:'
	);
	await expect(tela).toHaveAttribute('data-entidades', '4');
	await expect(linha).toHaveAttribute('data-prompt', 'Selecione o primeiro objeto ou [Raio]:');
});

test('LINE pelos cliques, prendendo nos pontos notáveis (P1, P2)', async ({ page }) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);
	await expect(tela).toHaveAttribute('data-clique', 'selecao');
	await digitar(page, 'L');
	await expect(tela).toHaveAttribute('data-clique', 'ponto');

	// A fixture: LINE de (0,0) a (100,100) e CIRCLE de raio 40 em (50,50).
	const perto = await pixel(page, 99, 101);
	await page.mouse.move(perto.x, perto.y);
	await expect(tela).toHaveAttribute('data-prendimento', 'endpoint 100 100');
	await page.mouse.click(perto.x, perto.y);
	await expect(linha).toHaveAttribute('data-prompt', 'Especifique o próximo ponto ou [Desfazer]:');

	// O quadrante direito do círculo, em (90, 50).
	const quadrante = await pixel(page, 91, 51);
	await page.mouse.move(quadrante.x, quadrante.y);
	await expect(tela).toHaveAttribute('data-prendimento', 'quadrant 90 50');
	await page.mouse.click(quadrante.x, quadrante.y);
	await digitar(page, '');

	const historico = page.getByRole('log', { name: 'Histórico de comandos' });
	await expect(historico).toContainText('Especifique o primeiro ponto: 100,100');
	await expect(historico).toContainText('Especifique o próximo ponto ou [Desfazer]: 90,50');
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await expect(tela).toHaveAttribute('data-selecao', '0');
	await expect(tela).toHaveAttribute('data-clique', 'selecao');
});

test('clique em entidade só do upstream: "entidade ainda não editável" (V7)', async ({ page }) => {
	test.setTimeout(60_000);
	await page.goto('/');
	const chooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Arquivo' }).click();
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).click();
	await (await chooser).setFiles(path.join(process.cwd(), 'e2e/fixtures/with-unsupported.dxf'));
	const tela = page.locator('canvas[data-tela="kernel"]');
	// A LINE é do kernel; a SOLID, de (10,10) a (40,40), só o upstream desenha.
	await expect(tela).toHaveAttribute('data-fundo', 'transparente', { timeout: 30_000 });
	await expect(page.getByText('Desenho carregado com sucesso')).toBeVisible({ timeout: 30_000 });
	const historico = page.getByRole('log', { name: 'Histórico de comandos' });
	const linha = page.getByRole('textbox', { name: 'Linha de comando' });

	// Sem comando: avisa e não abre janela (o segundo clique não seleciona nada).
	await clicarNoDesenho(page, 35, 15);
	await expect(historico).toContainText(/\w+: entidade ainda não editável\./);
	console.log('V7-tipo:', await historico.innerText());
	await clicarNoDesenho(page, 90, 90);
	await expect(tela).toHaveAttribute('data-selecao', '1');

	// Num pedido de objeto, o comando não recebe o clique.
	await page.keyboard.press('Escape');
	await digitar(page, 'TR', '');
	await clicarNoDesenho(page, 35, 15);
	await expect(historico.locator('li').last()).toContainText('entidade ainda não editável.');
	await expect(linha).toHaveAttribute('data-prompt', 'Selecione o objeto a aparar ou [Desfazer]:');
	await expect(tela).toHaveAttribute('data-entidades', '1');
});

test('Exibir > Vírgula decimal troca a entrada e é lembrada ao reabrir (D8, D10)', async ({
	page
}) => {
	test.setTimeout(60_000);
	const { linha, tela } = await abrir(page);
	const historico = page.getByRole('log', { name: 'Histórico de comandos' });

	await page.getByRole('button', { name: 'Exibir' }).click();
	const chave = page.getByRole('menuitemcheckbox', { name: 'Vírgula decimal' });
	await expect(chave).toHaveAttribute('aria-checked', 'false');
	await chave.click();

	await digitar(page, 'L', '1.5,2');
	await expect(historico).toContainText('Esperava um ponto: x;y, @dx;dy, d<a ou @d<a.');
	await digitar(page, '0;0', '1,5;2', '');
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await expect(linha).toHaveAttribute('data-prompt', LIVRE);

	await page.reload();
	await page.getByRole('button', { name: 'Exibir' }).click();
	await expect(page.getByRole('menuitemcheckbox', { name: 'Vírgula decimal' })).toHaveAttribute(
		'aria-checked',
		'true'
	);
});
