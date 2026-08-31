// Caminho relativo: scripts/check-licenses.mjs

/**
 * \file scripts/check-licenses.mjs
 * \brief Verifica as licenças das dependências de runtime contra scripts/license-policy.json.
 * \author Iago Leal
 * \date 2026-08-06
 *
 * Motivação: o NeoCAD é GPL-3.0-or-later (ADR 0002) e distribui binários que
 * embutem dependências copyleft (LibreDWG e dxf-json). O objetivo deste
 * verificador é impedir que entre na árvore de runtime uma licença incompatível
 * com a GPL-3.0 — ou que uma dependência acompanhada mude de termos no upstream
 * sem que ninguém perceba.
 *
 * Uso:
 *   node scripts/check-licenses.mjs          # falha se houver desvio
 *   node scripts/check-licenses.mjs --list   # apenas lista o inventário
 */

import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const POLICY_PATH = resolve(ROOT_DIR, 'scripts/license-policy.json');

/** Lê o inventário de licenças de produção a partir do pnpm. */
function readProductionLicenses() {
	const raw = execFileSync('pnpm', ['licenses', 'list', '--prod', '--json'], {
		cwd: ROOT_DIR,
		encoding: 'utf8',
		maxBuffer: 32 * 1024 * 1024
	});

	/** @type {Record<string, Array<{ name: string; versions: string[]; license: string }>>} */
	const grouped = JSON.parse(raw);

	return Object.entries(grouped).flatMap(([license, packages]) =>
		packages.flatMap((entry) =>
			entry.versions.map((version) => ({
				name: entry.name,
				version,
				license: entry.license ?? license
			}))
		)
	);
}

function loadPolicy() {
	return JSON.parse(readFileSync(POLICY_PATH, 'utf8'));
}

function formatEntry(entry) {
	return `${entry.name}@${entry.version} (${entry.license})`;
}

function main() {
	const policy = loadPolicy();
	const inventory = readProductionLicenses().sort((a, b) => a.name.localeCompare(b.name));

	if (process.argv.includes('--list')) {
		for (const entry of inventory) {
			console.log(formatEntry(entry));
		}
		return;
	}

	const allowed = new Set(policy.allowed);
	const denied = new Set(policy.denied);
	const tracked = new Map(policy.tracked.map((item) => [`${item.package}@${item.version}`, item]));
	const seenTracked = new Set();

	const violations = [];

	for (const entry of inventory) {
		const key = `${entry.name}@${entry.version}`;

		if (denied.has(entry.license)) {
			violations.push(
				`${key}: licença "${entry.license}" é incompatível com ${policy.projectLicense} ` +
					`e está na lista "denied".`
			);
			continue;
		}

		if (tracked.has(key)) {
			seenTracked.add(key);

			const declared = tracked.get(key).license;
			if (declared !== entry.license) {
				violations.push(
					`${key}: licença mudou de "${declared}" (política) para "${entry.license}" ` +
						`(árvore atual). Reavalie e atualize THIRD-PARTY-LICENSES.md.`
				);
				continue;
			}
		}

		if (!allowed.has(entry.license)) {
			violations.push(
				`${key}: licença "${entry.license}" não consta em "allowed". Avalie a ` +
					`compatibilidade com ${policy.projectLicense} antes de aceitar.`
			);
		}
	}

	const stale = [...tracked.keys()].filter((key) => !seenTracked.has(key));

	console.log(`Licença do projeto: ${policy.projectLicense}`);
	console.log(`Dependências de runtime inspecionadas: ${inventory.length}`);
	console.log(`Dependências copyleft acompanhadas: ${tracked.size}`);

	for (const key of seenTracked) {
		const item = tracked.get(key);
		console.log(`  · ${key} — ${item.license} — via ${item.reachedVia}`);
	}

	if (stale.length > 0) {
		console.log(
			`\nAviso: entradas acompanhadas que já não aparecem na árvore (limpe a política): ` +
				stale.join(', ')
		);
	}

	if (violations.length > 0) {
		console.error(`\nFalha na política de licenças (${violations.length}):`);
		for (const violation of violations) {
			console.error(`  ✗ ${violation}`);
		}
		console.error(`\nConsulte THIRD-PARTY-LICENSES.md e scripts/license-policy.json.`);
		process.exit(1);
	}

	console.log('\nPolítica de licenças satisfeita.');
}

main();
