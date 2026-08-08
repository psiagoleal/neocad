// Caminho relativo: scripts/sync-workers.mjs

/**
 * \file scripts/sync-workers.mjs
 * \brief Sincroniza os workers do upstream @mlightcad de node_modules para static/workers.
 * \author Iago Leal
 * \date 2026-08-06
 *
 * Antes, os três workers eram cópias commitadas no repositório (~9,7 MB), o que
 * criava dois problemas: divergência silenciosa em relação à versão instalada do
 * upstream, e distribuição de binário GPL-3.0 (LibreDWG) sem proveniência
 * registrada. Agora eles são derivados de node_modules em tempo de build, e o
 * manifesto versionado `static/workers/workers.manifest.json` registra versão de
 * origem e hash de cada arquivo.
 *
 * Uso:
 *   node scripts/sync-workers.mjs           # copia e regrava o manifesto
 *   node scripts/sync-workers.mjs --check   # falha se o resultado divergir do manifesto
 */

import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const TARGET_DIR = resolve(ROOT_DIR, 'static/workers');
const MANIFEST_PATH = join(TARGET_DIR, 'workers.manifest.json');

const require = createRequire(join(ROOT_DIR, 'package.json'));

/**
 * Os pacotes publicam `exports` restritivos, então não é possível resolver os
 * workers por subcaminho. Resolvemos o entrypoint e navegamos até o `dist/`
 * irmão, que é onde o upstream publica os workers.
 */
const WORKERS = [
	{ file: 'dxf-parser-worker.js', package: '@mlightcad/data-model' },
	{ file: 'libredwg-parser-worker.js', package: '@mlightcad/cad-simple-viewer' },
	{ file: 'mtext-renderer-worker.js', package: '@mlightcad/cad-simple-viewer' }
];

function resolvePackageDistDir(packageName) {
	try {
		return dirname(require.resolve(packageName));
	} catch (error) {
		throw new Error(`Não foi possível resolver "${packageName}". Rode "pnpm install" antes.`, {
			cause: error
		});
	}
}

function resolvePackageVersion(packageName) {
	// `require.resolve` do package.json é bloqueado pelo campo `exports`; subimos
	// a partir do dist até encontrar o manifesto do pacote.
	let current = resolvePackageDistDir(packageName);

	for (let depth = 0; depth < 5; depth += 1) {
		try {
			return JSON.parse(readFileSync(join(current, 'package.json'), 'utf8')).version;
		} catch {
			current = dirname(current);
		}
	}

	return 'desconhecida';
}

function sha256(filePath) {
	return createHash('sha256').update(readFileSync(filePath)).digest('hex');
}

function buildManifest() {
	mkdirSync(TARGET_DIR, { recursive: true });

	const entries = WORKERS.map((worker) => {
		const sourcePath = join(resolvePackageDistDir(worker.package), worker.file);
		const targetPath = join(TARGET_DIR, worker.file);

		copyFileSync(sourcePath, targetPath);

		return {
			file: worker.file,
			sourcePackage: worker.package,
			sourceVersion: resolvePackageVersion(worker.package),
			sha256: sha256(targetPath)
		};
	});

	return {
		$comment:
			'Gerado por scripts/sync-workers.mjs. Os arquivos .js correspondentes não são versionados; ' +
			'são derivados de node_modules no build. Ver THIRD-PARTY-LICENSES.md.',
		generatedFrom: 'node_modules (@mlightcad)',
		workers: entries
	};
}

function main() {
	const isCheck = process.argv.includes('--check');
	const manifest = buildManifest();

	if (isCheck) {
		let previous;

		try {
			previous = JSON.parse(readFileSync(MANIFEST_PATH, 'utf8'));
		} catch {
			console.error(
				`✗ Manifesto ausente ou ilegível em ${MANIFEST_PATH}. Rode "pnpm workers:sync".`
			);
			process.exit(1);
		}

		const divergences = manifest.workers.filter((entry) => {
			const match = previous.workers?.find((item) => item.file === entry.file);
			return match == null || match.sha256 !== entry.sha256;
		});

		if (divergences.length > 0) {
			console.error('✗ Workers do upstream divergem do manifesto versionado:');
			for (const entry of divergences) {
				console.error(`  · ${entry.file} (${entry.sourcePackage}@${entry.sourceVersion})`);
			}
			console.error(
				'\nIsso indica atualização do upstream. Rode "pnpm workers:sync", revise a mudança\n' +
					'e commite o novo static/workers/workers.manifest.json.'
			);
			process.exit(1);
		}

		console.log(`Workers conferem com o manifesto (${manifest.workers.length} arquivos).`);
		return;
	}

	writeFileSync(MANIFEST_PATH, `${JSON.stringify(manifest, null, '\t')}\n`);

	console.log(`Workers sincronizados em ${TARGET_DIR}:`);
	for (const entry of manifest.workers) {
		console.log(`  · ${entry.file} <- ${entry.sourcePackage}@${entry.sourceVersion}`);
	}
}

main();
