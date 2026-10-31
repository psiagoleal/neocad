// Caminho relativo: e2e/dxf-sintetico.e2e.ts

/**
 * \file e2e/dxf-sintetico.e2e.ts
 * \brief Gerador de DXF sintético e a contagem que o kernel informa ao abri-lo.
 * \author Iago Leal
 * \date 2026-10-30
 *
 * O desenho de 100 mil entidades que os spikes de KV medem não pode depender de
 * arquivo de terceiros (spec 00, T10): sai deste gerador. Os três primeiros testes
 * conferem o gerador sem navegador; o quarto prova que o kernel lê o arquivo com a
 * contagem exata; o quinto só roda com `NEOCAD_SINTETICO_N` e mede a abertura.
 */

import { writeFile } from 'node:fs/promises';
import { expect, test, type Page } from '@playwright/test';
import { gerarDxfSintetico } from '../scripts/gerar-dxf-sintetico.mjs';

/** Abre um DXF pelo diálogo de arquivo, sem esperar nada depois do clique. */
async function abrirSintetico(page: Page, caminho: string): Promise<void> {
	await page.goto('/');
	const chooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Abrir desenho CAD' }).first().click();
	await (await chooser).setFiles(caminho);
}

test('Contagem coerente', () => {
	const opcoes = { entidades: 1000, camadas: 6, semente: 7 };
	const { contagem } = gerarDxfSintetico(opcoes);

	expect(contagem.total).toBe(1000);
	expect(contagem.modelo + Object.values(contagem.papel).reduce((a, b) => a + b, 0)).toBe(1000);
	expect(Object.keys(contagem.papel).sort()).toEqual(['Prancha A1', 'Prancha A2']);
	expect(contagem.papel['Prancha A1']).toBeGreaterThan(0);
	expect(contagem.papel['Prancha A2']).toBeGreaterThan(0);
	expect(contagem.modelo).toBeGreaterThan(0);
	expect(Object.values(contagem.porTipo).reduce((a, b) => a + b, 0)).toBe(1000);
	expect(contagem.porTipo.LINE).toBeGreaterThan(0);
	expect(contagem.porTipo.CIRCLE).toBeGreaterThan(0);
	expect(contagem.porTipo.ARC).toBeGreaterThan(0);
	expect(contagem.porTipo.LWPOLYLINE).toBeGreaterThan(0);
	expect(contagem.porTipo.TEXT).toBeGreaterThan(0);
	expect(contagem.camadas).toBe(7);
});

test('Texto bate com contagem', () => {
	const opcoes = { entidades: 1000, camadas: 6, semente: 7 };
	const { texto, contagem } = gerarDxfSintetico(opcoes);

	const tipos = ['LINE', 'CIRCLE', 'ARC', 'LWPOLYLINE', 'TEXT'];
	for (const tipo of tipos) {
		const regex = new RegExp('^\\s*0\\r?\\n' + tipo + '\\r?$', 'gm');
		const ocorrencias = (texto.match(regex) ?? []).length;
		expect(contagem.porTipo[tipo]).toBeGreaterThan(0);
		expect(ocorrencias).toBe(contagem.porTipo[tipo]);
	}
});

test('cada layout recebe os cinco tipos', () => {
	const { texto } = gerarDxfSintetico({ entidades: 1000, camadas: 6, semente: 7 });

	// O espaço-papel é medido pelos spikes de KV como o modelo: um layout só com
	// polilinha e texto deixaria a medida cega para as demais primitivas.
	for (const layout of ['Prancha A1', 'Prancha A2']) {
		for (const tipo of ['LINE', 'CIRCLE', 'ARC', 'LWPOLYLINE', 'TEXT']) {
			const regex = new RegExp(
				`^\\s*0\\r?\\n${tipo}\\r?\\n\\s*5\\r?\\n[0-9A-F]+\\r?\\n\\s*100\\r?\\nAcDbEntity\\r?\\n\\s*67\\r?\\n1\\r?\\n\\s*410\\r?\\n${layout}\\r?$`,
				'gm'
			);
			expect((texto.match(regex) ?? []).length, `${tipo} em ${layout}`).toBeGreaterThan(0);
		}
	}
});

test('Determinismo', () => {
	const opcoes = { entidades: 1000, camadas: 6, semente: 7 };
	const resultado1 = gerarDxfSintetico(opcoes);
	const resultado2 = gerarDxfSintetico(opcoes);

	expect(resultado1.texto).toBe(resultado2.texto);

	const opcoes2 = { entidades: 1000, camadas: 6, semente: 8 };
	const resultado3 = gerarDxfSintetico(opcoes2);
	expect(resultado1.texto).not.toBe(resultado3.texto);
});

test('Kernel informa contagem exata', async ({ page }) => {
	test.setTimeout(120_000);

	const opcoes = { entidades: 2000, camadas: 8, semente: 1 };
	const { texto, contagem } = gerarDxfSintetico(opcoes);
	const caminho = test.info().outputPath('sintetico.dxf');
	await writeFile(caminho, texto);

	await abrirSintetico(page, caminho);

	await expect(page.getByText(/^Kernel: /).first()).toBeVisible({ timeout: 60_000 });

	// A mensagem do kernel conta o espaço-modelo; as entidades dos dois layouts
	// são conferidas no texto pelos testes acima. "Compreendido por inteiro"
	// diz que nada ficou sem representação.
	await expect(page.getByText(/^Kernel: /).first()).toHaveText(
		`Kernel: ${contagem.modelo} entidade(s) em ${contagem.camadas} camada(s). arquivo compreendido por inteiro.`
	);

	// Desde o MT-KL-09 o espaço-papel é gravado: salvar não pode acusar perda.
	const download = page.waitForEvent('download');
	await page.getByRole('button', { name: 'Arquivo' }).click();
	await page.getByRole('button', { name: 'Salvar', exact: true }).click();
	await download;

	await expect(page.getByText('Desenho gravado em sintetico.dxf.', { exact: true })).toBeVisible();
});

test('Escala', async ({ page }) => {
	test.skip(!process.env.NEOCAD_SINTETICO_N, 'defina NEOCAD_SINTETICO_N para medir em escala');
	test.setTimeout(600_000);

	const n = Number(process.env.NEOCAD_SINTETICO_N);
	const opcoes = { entidades: n, camadas: 20, semente: 1 };
	const { texto, contagem } = gerarDxfSintetico(opcoes);
	const caminho = test.info().outputPath('sintetico.dxf');
	await writeFile(caminho, texto);

	const inicio = Date.now();
	await abrirSintetico(page, caminho);
	await expect(page.getByText(/^Kernel: /).first()).toBeVisible({ timeout: 300_000 });
	const fim = Date.now();

	const mensagem = await page
		.getByText(/^Kernel: /)
		.first()
		.textContent();
	expect(mensagem).toContain(
		`Kernel: ${contagem.modelo} entidade(s) em ${contagem.camadas} camada(s)`
	);

	const ms = fim - inicio;
	test.info().annotations.push({ type: 'abertura-ms', description: String(ms) });
});
