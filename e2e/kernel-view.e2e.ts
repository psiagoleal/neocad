// Caminho relativo: e2e/kernel-view.e2e.ts

/**
 * \file e2e/kernel-view.e2e.ts
 * \brief Verifica que o que se desenha é o que se salva (spec 07).
 * \author Iago Leal
 * \date 2026-10-30
 *
 * O primeiro teste desenha pela linha de comando própria e confere que o arquivo
 * salvo traz a linha (V4, MT-KV-14b). Os seguintes conferem a tela desenhada pelo
 * kernel (V1, V2, V3; spec 00, N4) e o upstream por baixo dela (MT-KV-13a). Os
 * demais critérios da spec 07 (V5b, V7) entram aqui quando a fase KV os implementar.
 */

import path from 'node:path';
import { readFile } from 'node:fs/promises';
import { expect, test, type Page } from '@playwright/test';

const FIXTURE = path.join(process.cwd(), 'e2e/fixtures/minimal.dxf');

/** Conta as entidades LINE no texto de um DXF. */
function contarLinhas(dxf: string): number {
	return (dxf.match(/^\s*0\r?\n\s*LINE\r?$/gm) ?? []).length;
}

/** Abre um desenho pelo diálogo de arquivo e espera o upstream e o kernel. */
async function abrirDesenho(page: Page, caminho: string): Promise<void> {
	await page.goto('/');

	const chooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await chooser).setFiles(caminho);

	await expect(page.getByText('Desenho carregado com sucesso')).toBeVisible({ timeout: 30_000 });
	await expect(page.getByText(/^Kernel: /).first()).toBeVisible({ timeout: 30_000 });
}

/** Digita na linha de comando própria e confirma cada texto com Enter. */
async function digitar(page: Page, ...textos: string[]): Promise<void> {
	const linha = page.getByRole('textbox', { name: 'Linha de comando' });
	for (const texto of textos) {
		await linha.fill(texto);
		await linha.press('Enter');
	}
}

test('a linha desenhada por comando está no arquivo salvo e reaberto (V4)', async ({ page }) => {
	test.setTimeout(120_000);

	await abrirDesenho(page, FIXTURE);

	const original = await readFile(FIXTURE, 'utf-8');
	expect(contarLinhas(original)).toBe(1);

	await digitar(page, 'L', '0,0', '100,50', '');
	await expect(telaDoKernel(page)).toHaveAttribute('data-entidades', '3');

	const download = page.waitForEvent('download');
	await page.getByRole('button', { name: 'Arquivo' }).click();
	await page.getByRole('button', { name: 'Salvar', exact: true }).click();
	const destino = test.info().outputPath('salvo.dxf');
	await (await download).saveAs(destino);

	// A da fixture e a desenhada.
	expect(contarLinhas(await readFile(destino, 'utf-8'))).toBe(2);

	await abrirDesenho(page, destino);
	await expect(page.getByText(/^Kernel: /).first()).toContainText('3 entidade(s)');
});

/** A tela própria, como a spec 07 a descreve para os testes. */
function telaDoKernel(page: Page) {
	return page.locator('canvas[data-tela="kernel"]');
}

/** Abre pelo diálogo de arquivo, sem esperar o upstream. */
async function abrirSoPeloKernel(page: Page, caminho: string): Promise<void> {
	await page.goto('/');
	const chooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await chooser).setFiles(caminho);
}

test('a tela do kernel mostra as entidades do modelo (V1)', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirSoPeloKernel(page, FIXTURE);

	const tela = telaDoKernel(page);
	// A fixture tem uma LINE e um CIRCLE no espaço-modelo.
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });
	await expect(tela).toHaveAttribute('data-espaco', 'Model');
	// Sem GPU, a CI cai na reserva; os dois caminhos valem aqui.
	await expect(tela).toHaveAttribute('data-caminho', /^(webgl2|canvas2d)$/);
	await expect(tela).toBeVisible();
});

test('o texto do kernel também é desenhado (V1)', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirSoPeloKernel(page, path.join(process.cwd(), 'e2e/fixtures/with-text.dxf'));

	// Uma LINE e dois TEXT, um deles noutra camada e girado.
	await expect(telaDoKernel(page)).toHaveAttribute('data-entidades', '3', { timeout: 30_000 });
});

test('cada layout tem a sua aba e a sua contagem (V1)', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirSoPeloKernel(page, path.join(process.cwd(), 'e2e/fixtures/two-layouts.dxf'));

	const tela = telaDoKernel(page);
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });

	const abas = page.getByRole('tablist', { name: 'Espaços do desenho' });
	await abas.getByRole('tab', { name: 'Prancha A1' }).click();
	await expect(tela).toHaveAttribute('data-espaco', 'Prancha A1');
	await expect(tela).toHaveAttribute('data-entidades', '2');

	await abas.getByRole('tab', { name: 'Prancha A2' }).click();
	await expect(tela).toHaveAttribute('data-espaco', 'Prancha A2');
	await expect(tela).toHaveAttribute('data-entidades', '1');

	await abas.getByRole('tab', { name: 'Model' }).click();
	await expect(tela).toHaveAttribute('data-entidades', '2');
});

test('sem WebGL2, a tela cai sozinha no canvas 2D (N4)', async ({ page }) => {
	test.setTimeout(60_000);
	// Tira o WebGL2 só da tela própria. Tirá-lo da página inteira derruba também
	// o visualizador do upstream, que hoje habilita a abertura de arquivos; isso
	// deixa de valer quando o upstream sair da tela (MT-KV-13b).
	await page.addInitScript(() => {
		const original = HTMLCanvasElement.prototype.getContext;
		HTMLCanvasElement.prototype.getContext = function (
			this: HTMLCanvasElement,
			tipo: string,
			...resto: unknown[]
		) {
			if (tipo === 'webgl2' && this.dataset.tela === 'kernel') return null;
			return (original as (...a: unknown[]) => unknown).call(this, tipo, ...resto);
		} as typeof HTMLCanvasElement.prototype.getContext;
	});
	await abrirSoPeloKernel(page, FIXTURE);

	const tela = telaDoKernel(page);
	await expect(tela).toHaveAttribute('data-caminho', 'canvas2d', { timeout: 30_000 });
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });

	// Pan e zoom também redesenham pela reserva.
	const antes = Number(await tela.getAttribute('data-quadros'));
	const caixa = await tela.boundingBox();
	expect(caixa).not.toBeNull();
	await page.mouse.move((caixa?.x ?? 0) + 100, (caixa?.y ?? 0) + 100);
	await page.mouse.wheel(0, -120);
	await expect
		.poll(async () => Number(await tela.getAttribute('data-quadros')))
		.toBeGreaterThan(antes);
});

/** O que a página expõe aos testes no build com `VITE_NEOCAD_SONDA=1` (spec 07). */
type JanelaComSonda = {
	__neocadSonda?: {
		desenharLinha(x1: number, y1: number, x2: number, y2: number): string;
		mover(ids: string[], dx: number, dy: number): void;
		apagar(id: string): void;
	};
};

/** Abre a fixture e confere que a sonda existe, para a falha não se confundir com outra. */
async function abrirComSonda(page: Page) {
	await abrirSoPeloKernel(page, FIXTURE);
	const tela = telaDoKernel(page);
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });

	const temSonda = await page.evaluate(
		() => (window as unknown as JanelaComSonda).__neocadSonda !== undefined
	);
	expect(temSonda, 'build sem VITE_NEOCAD_SONDA=1').toBe(true);
	return tela;
}

/** Desfaz ou refaz pelo menu Editar, como o usuário. */
async function peloMenuEditar(page: Page, acao: 'Desfazer' | 'Refazer'): Promise<void> {
	await page.getByRole('button', { name: 'Editar' }).click();
	await page.getByRole('button', { name: new RegExp(`^${acao}`) }).click();
}

test('criar, mover e apagar uma linha reflete na tela sem reabrir (V2)', async ({ page }) => {
	test.setTimeout(60_000);
	const tela = await abrirComSonda(page);
	// A fixture: LINE de (0,0) a (100,100) e CIRCLE de raio 40 em (50,50).
	await expect(tela).toHaveAttribute('data-caixa', '0 0 100 100');

	const id = await page.evaluate(() =>
		(window as unknown as JanelaComSonda).__neocadSonda!.desenharLinha(200, 0, 300, 50)
	);
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await expect(tela).toHaveAttribute('data-caixa', '0 0 300 100');

	await page.evaluate(
		(linha) => (window as unknown as JanelaComSonda).__neocadSonda!.mover([linha], 0, 100),
		id
	);
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await expect(tela).toHaveAttribute('data-caixa', '0 0 300 150');

	await page.evaluate(
		(linha) => (window as unknown as JanelaComSonda).__neocadSonda!.apagar(linha),
		id
	);
	await expect(tela).toHaveAttribute('data-entidades', '2');
	await expect(tela).toHaveAttribute('data-caixa', '0 0 100 100');
});

test('desfazer e refazer pelo menu Editar refletem na tela (V3)', async ({ page }) => {
	test.setTimeout(60_000);
	const tela = await abrirComSonda(page);

	await page.evaluate(() => {
		const sonda = (window as unknown as JanelaComSonda).__neocadSonda!;
		const id = sonda.desenharLinha(200, 0, 300, 50);
		sonda.mover([id], 0, 100);
	});
	await expect(tela).toHaveAttribute('data-caixa', '0 0 300 150');

	await peloMenuEditar(page, 'Desfazer');
	await expect(tela).toHaveAttribute('data-caixa', '0 0 300 100');
	await expect(tela).toHaveAttribute('data-entidades', '3');

	await peloMenuEditar(page, 'Desfazer');
	await expect(tela).toHaveAttribute('data-caixa', '0 0 100 100');
	await expect(tela).toHaveAttribute('data-entidades', '2');

	await peloMenuEditar(page, 'Refazer');
	await expect(tela).toHaveAttribute('data-entidades', '3');
	await expect(tela).toHaveAttribute('data-caixa', '0 0 300 100');
});

/** Centro e escala (pixels por unidade) de uma vista, como os atributos `data-vista` a mostram. */
async function lerVista(alvo: ReturnType<Page['locator']>) {
	const texto = (await alvo.getAttribute('data-vista')) ?? '';
	const [x, y, escala] = texto.split(' ').map(Number);
	return { x, y, escala, texto };
}

/** Quanto as duas vistas diferem, em pixels de tela, no centro e na escala. */
async function diferencaDeVistas(page: Page): Promise<number> {
	const kernel = await lerVista(telaDoKernel(page));
	const upstream = await lerVista(page.locator('[data-tela="upstream"]'));
	if (![kernel, upstream].every((v) => [v.x, v.y, v.escala].every(Number.isFinite))) {
		return Number.POSITIVE_INFINITY;
	}
	const centro = Math.hypot(kernel.x - upstream.x, kernel.y - upstream.y) * kernel.escala;
	// Diferença de escala vista na borda de uma tela de 1000 px.
	const escala = (Math.abs(kernel.escala - upstream.escala) / kernel.escala) * 1000;
	return Math.max(centro, escala);
}

test('o upstream desenha só o que o kernel não modela, na mesma vista (MT-KV-13a)', async ({
	page
}) => {
	test.setTimeout(60_000);
	await abrirDesenho(page, path.join(process.cwd(), 'e2e/fixtures/with-unsupported.dxf'));

	const tela = telaDoKernel(page);
	const upstream = page.locator('[data-tela="upstream"]');
	// A LINE é do kernel; a SOLID, que o kernel não modela, fica com o upstream.
	await expect(tela).toHaveAttribute('data-entidades', '1', { timeout: 30_000 });
	await expect(upstream).toHaveAttribute('data-entidades', '1');
	// A tela própria deixa ver o upstream por baixo.
	await expect(tela).toHaveAttribute('data-fundo', 'transparente');
	await expect.poll(() => diferencaDeVistas(page)).toBeLessThanOrEqual(0.5);

	// Pan e zoom pela tela própria levam a vista do upstream junto.
	const antes = (await lerVista(tela)).texto;
	const caixa = await tela.boundingBox();
	expect(caixa).not.toBeNull();
	const cx = (caixa?.x ?? 0) + (caixa?.width ?? 0) / 2;
	const cy = (caixa?.y ?? 0) + (caixa?.height ?? 0) / 2;
	await page.mouse.move(cx, cy);
	await page.mouse.down({ button: 'middle' });
	await page.mouse.move(cx + 80, cy + 30, { steps: 5 });
	await page.mouse.up({ button: 'middle' });
	await page.mouse.move(cx - 100, cy + 40);
	await page.mouse.wheel(0, -360);

	await expect.poll(async () => (await lerVista(tela)).texto).not.toBe(antes);
	await expect.poll(() => diferencaDeVistas(page)).toBeLessThanOrEqual(0.5);
});

test('a área de mensagens avisa o que a tela não exibe (V6)', async ({ page }) => {
	test.setTimeout(60_000);
	await abrirSoPeloKernel(page, path.join(process.cwd(), 'e2e/fixtures/with-viewport.dxf'));

	const tela = telaDoKernel(page);
	await expect(tela).toHaveAttribute('data-entidades', '2', { timeout: 30_000 });
	const aviso = page.getByText('A tela não exibe 1 entidade(s) de Prancha A1: 1 VIEWPORT.');
	// O espaço-modelo não tem nada recusado: nenhum aviso ainda.
	await expect(page.getByText(/^A tela não exibe/)).toHaveCount(0);

	await page
		.getByRole('tablist', { name: 'Espaços do desenho' })
		.getByRole('tab', { name: 'Prancha A1' })
		.click();
	// A janela não vira traço; as duas linhas da prancha, sim.
	await expect(tela).toHaveAttribute('data-entidades', '2');
	await expect(aviso).toHaveCount(1);
});

/** Pixel da página onde fica o ponto (x, y) do desenho, pela vista exposta em `data-vista`. */
async function pixel(page: Page, x: number, y: number): Promise<{ x: number; y: number }> {
	const tela = telaDoKernel(page);
	const [cx, cy, escala] = ((await tela.getAttribute('data-vista')) ?? '').split(' ').map(Number);
	const caixa = await tela.boundingBox();
	if (caixa == null) throw new Error('a tela não tem área visível');
	return {
		x: caixa.x + caixa.width / 2 + (x - cx) * escala,
		y: caixa.y + caixa.height / 2 - (y - cy) * escala
	};
}

/** Clique esquerdo no ponto (x, y) do desenho. */
async function clicarNoDesenho(page: Page, x: number, y: number, shift = false): Promise<void> {
	// Só depois da carga do upstream a tela para de mudar de tamanho; antes disso, um
	// clique calculado em pixels pode errar o alvo.
	await expect(page.getByText('Desenho carregado com sucesso')).toBeVisible({ timeout: 30_000 });
	await expect(page.locator('.viewer-status-row')).toHaveCount(0);
	const p = await pixel(page, x, y);
	if (shift) await page.keyboard.down('Shift');
	await page.mouse.click(p.x, p.y);
	if (shift) await page.keyboard.up('Shift');
}

test('clique, janela e cruzada selecionam; Esc esvazia (S1, S2, S5)', async ({ page }) => {
	test.setTimeout(60_000);
	const tela = await abrirComSonda(page);
	await expect(tela).toHaveAttribute('data-selecao', '0');

	// A fixture: LINE de (0,0) a (100,100) e CIRCLE de raio 40 em (50,50).
	await clicarNoDesenho(page, 10, 10);
	await expect(tela).toHaveAttribute('data-selecao', '1');
	await clicarNoDesenho(page, 10, 10, true);
	await expect(tela).toHaveAttribute('data-selecao', '0');

	// Janela, da esquerda para a direita, em volta de tudo.
	await clicarNoDesenho(page, -3, -3);
	await clicarNoDesenho(page, 103, 103);
	await expect(tela).toHaveAttribute('data-selecao', '2');
	await page.keyboard.press('Escape');
	await expect(tela).toHaveAttribute('data-selecao', '0');

	// Cruzada, da direita para a esquerda: toca só o fundo do círculo.
	await clicarNoDesenho(page, 60, 0);
	await clicarNoDesenho(page, 40, 20);
	await expect(tela).toHaveAttribute('data-selecao', '1');

	// Esc na linha de comando, sem comando em curso, também esvazia.
	const linha = page.getByRole('textbox', { name: 'Linha de comando' });
	await linha.focus();
	await linha.press('Escape');
	await expect(tela).toHaveAttribute('data-selecao', '0');
});

test('a apagada sai da seleção e a movida continua nela (S4)', async ({ page }) => {
	test.setTimeout(60_000);
	const tela = await abrirComSonda(page);

	const id = await page.evaluate(() =>
		(window as unknown as JanelaComSonda).__neocadSonda!.desenharLinha(0, 90, 30, 90)
	);
	await clicarNoDesenho(page, 15, 90);
	await expect(tela).toHaveAttribute('data-selecao', '1');

	await page.evaluate(
		(linha) => (window as unknown as JanelaComSonda).__neocadSonda!.mover([linha], 0, 5),
		id
	);
	await expect(tela).toHaveAttribute('data-caixa', '0 0 100 100');
	await expect(tela).toHaveAttribute('data-selecao', '1');

	await page.evaluate(
		(linha) => (window as unknown as JanelaComSonda).__neocadSonda!.apagar(linha),
		id
	);
	await expect(tela).toHaveAttribute('data-entidades', '2');
	await expect(tela).toHaveAttribute('data-selecao', '0');
});
