// Caminho relativo: scripts/size-budget.spec.mjs

import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { checkBuildTotal, checkKernelWasm, measureDir } from './size-budget.mjs';

const budget = { kernelWasmLimit: 600, buildTotal: { record: 1000, slack: 100 } };

describe('orçamento de tamanho (ADR 0009)', () => {
	it('o kernel no teto passa; um byte acima falha citando os dois números', () => {
		expect(checkKernelWasm(600, budget)).toBeNull();
		const error = checkKernelWasm(601, budget);
		expect(error).toContain('601');
		expect(error).toContain('600');
	});

	it('o total passa até registro + folga e falha um byte acima, citando os números', () => {
		expect(checkBuildTotal(1100, budget)).toBeNull();
		const error = checkBuildTotal(1101, budget);
		expect(error).toContain('1101');
		expect(error).toContain('1000');
		expect(error).toContain('100');
	});

	it('o total abaixo do registro passa', () => {
		expect(checkBuildTotal(900, budget)).toBeNull();
	});

	it('mede a soma dos arquivos, inclusive em subpastas', () => {
		const dir = mkdtempSync(join(tmpdir(), 'orcamento-'));
		writeFileSync(join(dir, 'a.js'), 'x'.repeat(10));
		mkdirSync(join(dir, 'sub'));
		writeFileSync(join(dir, 'sub', 'b.wasm'), 'y'.repeat(32));
		expect(measureDir(dir)).toBe(42);
	});

	it('a linha de comando falha acima do orçamento e passa com ele', () => {
		const dir = mkdtempSync(join(tmpdir(), 'orcamento-'));
		writeFileSync(join(dir, 'a.js'), 'x'.repeat(50));
		const script = resolve(import.meta.dirname, 'size-budget.mjs');
		const run = (record, slack) => {
			const file = join(dir, `budget-${record}-${slack}.json`);
			writeFileSync(file, JSON.stringify({ kernelWasmLimit: 1, buildTotal: { record, slack } }));
			return spawnSync('node', [script, '--dir', join(dir), '--budget', file], {
				encoding: 'utf8'
			});
		};
		// O arquivo de orçamento também está na pasta: a medida inclui os dois.
		const tight = run(10, 0);
		expect(tight.status).toBe(1);
		expect(tight.stderr).toContain('ADR 0009');
		expect(run(10_000, 0).status).toBe(0);
	});
});
