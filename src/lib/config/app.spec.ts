// Caminho relativo: src/lib/config/app.spec.ts

/**
 * \file src/lib/config/app.spec.ts
 * \brief Testes unitários para contratos básicos de configuração do NeoCAD.
 * \author Iago Leal
 * \date 2026-05-12
 */

import { describe, expect, it } from 'vitest';
import { appMetadata, isSupportedDesktopTarget, primaryStack, supportedTargets } from './app';

describe('app config', () => {
	it('expõe os metadados principais do NeoCAD', () => {
		expect(appMetadata.name).toBe('NeoCAD');
		expect(appMetadata.license).toBe('MIT');
		expect(primaryStack).toContain('Tauri 2');
	});

	it('reconhece apenas plataformas desktop suportadas no MVP', () => {
		expect(supportedTargets).toEqual(['Windows', 'Linux']);
		expect(isSupportedDesktopTarget('Windows')).toBe(true);
		expect(isSupportedDesktopTarget('Linux')).toBe(true);
		expect(isSupportedDesktopTarget('macOS')).toBe(false);
	});
});
