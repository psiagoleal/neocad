// Caminho relativo: src/lib/config/app.ts

/**
 * \file src/lib/config/app.ts
 * \brief Metadados e constantes de interface do aplicativo NeoCAD.
 * \author Iago Leal
 * \date 2026-05-12
 */

export const appMetadata = {
	name: 'NeoCAD',
	tagline:
		'Wrapper desktop open-source para CAD com SvelteKit, Tauri 2 e integração ao upstream MLightCAD.',
	status: 'Fase 2 — integração inicial do viewer',
	license: 'MIT'
} as const;

export const supportedTargets = ['Windows', 'Linux'] as const;

export const primaryStack = [
	'SvelteKit',
	'Svelte 5',
	'Tauri 2',
	'TypeScript',
	'Rust',
	'@mlightcad/cad-simple-viewer'
] as const;

export const nextMilestones = [
	'Adicionar arquivos recentes e drag-and-drop para abertura de desenhos.',
	'Expandir a UI para camadas, propriedades e fluxo de edição básica assistida.',
	'Planejar exportação e persistência local para próximos incrementos do MVP.'
] as const;

export function isSupportedDesktopTarget(target: string): boolean {
	return supportedTargets.some((supportedTarget) => supportedTarget === target);
}
