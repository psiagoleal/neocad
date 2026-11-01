// Caminho relativo: scripts/size-budget.mjs

/**
 * \file scripts/size-budget.mjs
 * \brief Orçamento de tamanho do aplicativo (ADR 0009): teto do kernel e catraca do total.
 * \author Iago Leal
 * \date 2026-11-01
 *
 * Os números moram em `scripts/size-budget.json`:
 *   - `kernelWasmLimit`: teto do `.wasm` de release do kernel, em bytes (spec 06, W5);
 *   - `buildTotal.record`: soma de `build/` medida da última vez que o registro mudou;
 *   - `buildTotal.slack`: folga fixa acima do registro, para a variação do build e o
 *     crescimento do código próprio.
 *
 * O registro desce quando uma parte do upstream sai; subir o registro, a folga ou o
 * teto exige ADR.
 *
 * Uso:
 *   node scripts/size-budget.mjs                              # confere build/
 *   node scripts/size-budget.mjs --dir <pasta> --budget <json>
 */

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const BUDGET_PATH = resolve(ROOT_DIR, 'scripts/size-budget.json');

/** Lê o orçamento versionado. */
export function readBudget(path = BUDGET_PATH) {
	return JSON.parse(readFileSync(path, 'utf8'));
}

/** Mensagem de erro se o `.wasm` do kernel passou do teto; senão `null`. */
export function checkKernelWasm(size, budget) {
	const limit = budget.kernelWasmLimit;
	if (size <= limit) {
		return null;
	}
	return `o .wasm de release tem ${size} bytes, acima do teto de ${limit} (spec 06, W5; ADR 0009).`;
}

/** Mensagem de erro se o total passou de registro + folga; senão `null`. */
export function checkBuildTotal(total, budget) {
	const { record, slack } = budget.buildTotal;
	if (total <= record + slack) {
		return null;
	}
	return (
		`build/ soma ${total} bytes, acima do registro de ${record} mais a folga de ${slack} ` +
		`(${record + slack}; ADR 0009). Subir o registro ou a folga exige ADR.`
	);
}

/** Soma o tamanho de todos os arquivos da pasta, em bytes. */
export function measureDir(dir) {
	let total = 0;
	for (const entry of readdirSync(dir, { withFileTypes: true })) {
		const path = join(dir, entry.name);
		total += entry.isDirectory() ? measureDir(path) : statSync(path).size;
	}
	return total;
}

function argument(args, name, fallback) {
	const index = args.indexOf(name);
	return index >= 0 ? resolve(args[index + 1]) : fallback;
}

function main() {
	const args = process.argv.slice(2);
	const dir = argument(args, '--dir', resolve(ROOT_DIR, 'build'));
	const budget = readBudget(argument(args, '--budget', BUDGET_PATH));
	const total = measureDir(dir);
	const error = checkBuildTotal(total, budget);

	if (error) {
		console.error(`\nerro: ${error}`);
		process.exit(1);
	}

	const { record, slack } = budget.buildTotal;
	console.log(
		`Orçamento de tamanho: build/ soma ${total} bytes (registro ${record}, folga ${slack}).`
	);
	if (record - total > slack) {
		console.log(
			'  O total caiu bem abaixo do registro: baixe-o em scripts/size-budget.json (ADR 0009).'
		);
	}
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
	main();
}
