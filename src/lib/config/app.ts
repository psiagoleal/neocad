// Caminho relativo: src/lib/config/app.ts

/**
 * \file src/lib/config/app.ts
 * \brief Metadados e constantes de interface do aplicativo NeoCAD.
 * \author Iago Leal
 * \date 2026-05-12
 */

export const appMetadata = {
	name: 'NeoCAD',
	tagline: 'Wrapper desktop open-source para CAD com SvelteKit e Tauri 2.',
	status: 'Fase 1 — scaffold base concluído',
	license: 'MIT'
} as const;

export const supportedTargets = ['Windows', 'Linux'] as const;

export const primaryStack = ['SvelteKit', 'Svelte 5', 'Tauri 2', 'TypeScript', 'Rust'] as const;

export const nextMilestones = [
	'Inicializar o backend Tauri e validar a janela desktop.',
	'Integrar o cad-viewer com uma camada adaptadora dedicada.',
	'Habilitar abertura local de arquivos DWG/DXF no fluxo desktop.'
] as const;

export function isSupportedDesktopTarget(target: string): boolean {
	return supportedTargets.some((supportedTarget) => supportedTarget === target);
}
