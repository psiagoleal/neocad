// Caminho relativo: scripts/build-kernel.mjs

/**
 * \file scripts/build-kernel.mjs
 * \brief Compila o kernel CAD para WebAssembly e o entrega ao frontend.
 * \author Iago Leal
 * \date 2026-08-07
 *
 * Segue o mesmo padrão de `scripts/sync-workers.mjs`: a saída é derivada, não
 * versionada, e regenerada por `pnpm dev` e `pnpm build`. O que o repositório
 * guarda é a fonte em `kernel/`, nunca o artefato.
 *
 * Uso:
 *   node scripts/build-kernel.mjs           # perfil release
 *   node scripts/build-kernel.mjs --dev     # perfil de desenvolvimento, mais rápido
 *   node scripts/build-kernel.mjs --force   # recompila mesmo sem mudança na fonte
 *   node scripts/build-kernel.mjs --check   # apenas verifica pré-requisitos
 */

import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const CRATE_DIR = resolve(ROOT_DIR, 'kernel/neocad-wasm');
const KERNEL_DIR = resolve(ROOT_DIR, 'kernel');
const OUT_DIR = resolve(ROOT_DIR, 'src/lib/kernel/pkg');
const WASM_FILE = join(OUT_DIR, 'neocad_wasm_bg.wasm');
const WASM_TARGET = 'wasm32-unknown-unknown';

function run(command, args, options = {}) {
	return spawnSync(command, args, { encoding: 'utf8', ...options });
}

function hasWasmPack() {
	return run('wasm-pack', ['--version']).status === 0;
}

function hasWasmTarget() {
	const result = run('rustup', ['target', 'list', '--installed'], { cwd: KERNEL_DIR });

	return result.status === 0 && result.stdout.split('\n').includes(WASM_TARGET);
}

function reportMissingPrerequisites() {
	const missing = [];

	if (!hasWasmPack()) {
		missing.push('wasm-pack        → cargo install wasm-pack');
	}
	if (!hasWasmTarget()) {
		missing.push(`${WASM_TARGET} → rustup target add ${WASM_TARGET}`);
	}

	if (missing.length > 0) {
		console.error('\nerro: faltam pré-requisitos para compilar o kernel WebAssembly:');
		for (const item of missing) {
			console.error(`  - ${item}`);
		}
		return true;
	}

	return false;
}

/** Instante da modificação mais recente entre as fontes do kernel. */
function newestSourceTime(directory) {
	let newest = 0;

	for (const entry of readdirSync(directory, { withFileTypes: true })) {
		// `target` guarda artefatos de compilação e não conta como fonte.
		if (entry.name === 'target' || entry.name === 'pkg') {
			continue;
		}

		const path = join(directory, entry.name);

		if (entry.isDirectory()) {
			newest = Math.max(newest, newestSourceTime(path));
			continue;
		}

		if (entry.name.endsWith('.rs') || entry.name.endsWith('.toml')) {
			newest = Math.max(newest, statSync(path).mtimeMs);
		}
	}

	return newest;
}

/**
 * Indica se o artefato já reflete a fonte.
 *
 * Sem isso, cada `pnpm build` — inclusive o que o Playwright dispara antes dos
 * testes E2E — recompilaria o kernel inteiro sem necessidade.
 */
function isUpToDate() {
	if (!existsSync(WASM_FILE)) {
		return false;
	}

	return statSync(WASM_FILE).mtimeMs >= newestSourceTime(KERNEL_DIR);
}

function formatSize(bytes) {
	return `${(bytes / 1024).toFixed(1)} KB`;
}

function main() {
	const args = process.argv.slice(2);

	if (reportMissingPrerequisites()) {
		process.exit(1);
	}

	if (args.includes('--check')) {
		console.log('Pré-requisitos do kernel WebAssembly presentes.');
		return;
	}

	if (!args.includes('--force') && isUpToDate()) {
		console.log(`Kernel WebAssembly já atualizado (${formatSize(statSync(WASM_FILE).size)}).`);
		return;
	}

	const profile = args.includes('--dev') ? '--dev' : '--release';
	const result = run(
		'wasm-pack',
		['build', CRATE_DIR, '--target', 'web', '--out-dir', OUT_DIR, profile],
		{ cwd: ROOT_DIR, stdio: 'inherit' }
	);

	if (result.status !== 0) {
		console.error('\nerro: a compilação do kernel WebAssembly falhou.');
		process.exit(result.status ?? 1);
	}

	// O tamanho entra no bundle distribuído; convém acompanhá-lo desde o início.
	console.log(
		`Kernel WebAssembly em ${OUT_DIR} (${formatSize(statSync(WASM_FILE).size)}, perfil ${profile.replace('--', '')}).`
	);
}

main();
