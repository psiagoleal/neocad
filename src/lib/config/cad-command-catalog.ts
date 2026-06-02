// Caminho relativo: src/lib/config/cad-command-catalog.ts

/**
 * \file src/lib/config/cad-command-catalog.ts
 * \brief Metadados de apresentação dos comandos CAD e montagem do catálogo.
 * \author Iago Leal
 * \date 2026-06-02
 *
 * Conforme o ADR 0001, a fonte de verdade sobre QUAIS comandos existem é o
 * command stack do upstream, lido em tempo de execução. Este arquivo guarda
 * apenas metadados de APRESENTAÇÃO (rótulo PT-BR, categoria, observações) e os
 * funde com os descritores de runtime. Comandos sem metadados conhecidos ainda
 * aparecem no catálogo, na categoria "other".
 */

import type {
	CadCommandCatalogItem,
	CadCommandCategory,
	CadCommandDescriptor
} from '$lib/types/cad';

interface CommandPresentation {
	label: string;
	category: CadCommandCategory;
	notes?: string;
}

/** Ordem de exibição das categorias no catálogo. */
export const CAD_COMMAND_CATEGORY_ORDER: readonly CadCommandCategory[] = [
	'navigation',
	'draw',
	'modify',
	'layer',
	'system',
	'other'
];

/** Rótulos PT-BR das categorias para os cabeçalhos do catálogo. */
export const CAD_COMMAND_CATEGORY_LABELS: Record<CadCommandCategory, string> = {
	navigation: 'Navegação e seleção',
	draw: 'Desenho',
	modify: 'Edição',
	layer: 'Camadas',
	system: 'Sistema',
	other: 'Outros'
};

/**
 * Metadados de apresentação indexados pelo nome global (em maiúsculas) do
 * comando upstream. Os nomes seguem o inventário do spike
 * (docs/upstream-capabilities-spike.md); nomes não mapeados caem no fallback.
 */
const COMMAND_PRESENTATION: Record<string, CommandPresentation> = {
	// Navegação e seleção
	ZOOM: {
		label: 'Zoom',
		category: 'navigation',
		notes: 'Aceita opções como All, Window e Previous.'
	},
	PAN: { label: 'Mover vista (Pan)', category: 'navigation' },
	SELECT: { label: 'Selecionar entidades', category: 'navigation' },
	// Sistema
	OPEN: { label: 'Abrir desenho', category: 'system' },
	QNEW: { label: 'Novo desenho', category: 'system' },
	REGEN: { label: 'Regenerar desenho', category: 'system' },
	SWITCHBG: { label: 'Alternar fundo do canvas', category: 'system' },
	SYSVAR: { label: 'Variável de sistema', category: 'system' },
	LOG: { label: 'Log de depuração', category: 'system' },
	// Desenho
	LINE: { label: 'Linha', category: 'draw' },
	CIRCLE: { label: 'Círculo', category: 'draw' },
	ARC: { label: 'Arco', category: 'draw' },
	ELLIPSE: { label: 'Elipse', category: 'draw' },
	RECT: { label: 'Retângulo', category: 'draw' },
	RECTANG: { label: 'Retângulo', category: 'draw' },
	PLINE: { label: 'Polilinha', category: 'draw' },
	POLYGON: { label: 'Polígono', category: 'draw' },
	SPLINE: { label: 'Spline', category: 'draw' },
	POINT: { label: 'Ponto', category: 'draw' },
	RAY: { label: 'Semirreta (Ray)', category: 'draw' },
	XLINE: { label: 'Linha de construção', category: 'draw' },
	MLINE: { label: 'Multilinha', category: 'draw' },
	MTEXT: { label: 'Texto multilinha', category: 'draw' },
	DIMLINEAR: { label: 'Cota linear', category: 'draw' },
	HATCH: { label: 'Hachura', category: 'draw' },
	// Edição
	ERASE: { label: 'Apagar', category: 'modify' },
	MOVE: { label: 'Mover', category: 'modify' },
	COPY: { label: 'Copiar', category: 'modify' },
	ROTATE: { label: 'Rotacionar', category: 'modify' },
	// Camadas
	LAYER: { label: 'Gerenciar camadas', category: 'layer' },
	LAYCUR: { label: 'Definir camada atual', category: 'layer' },
	LAYON: { label: 'Ligar camada', category: 'layer' },
	LAYOFF: { label: 'Desligar camada', category: 'layer' },
	LAYFRZ: { label: 'Congelar camada', category: 'layer' },
	LAYTHW: { label: 'Descongelar camada', category: 'layer' },
	LAYLCK: { label: 'Bloquear camada', category: 'layer' },
	LAYULK: { label: 'Desbloquear camada', category: 'layer' },
	LAYISO: { label: 'Isolar camada', category: 'layer' },
	LAYUNISO: { label: 'Remover isolamento de camada', category: 'layer' },
	LAYP: { label: 'Camada anterior', category: 'layer' },
	LAYDEL: { label: 'Excluir camada', category: 'layer' },
	LAYCLOSE: { label: 'Fechar diálogo de camadas', category: 'layer' }
};

/** Gera um rótulo legível a partir de um nome de comando desconhecido. */
function fallbackLabel(rawLabel: string): string {
	const cleaned = rawLabel.trim();
	if (cleaned.length === 0) {
		return cleaned;
	}

	return cleaned.charAt(0).toUpperCase() + cleaned.slice(1).toLowerCase();
}

/**
 * Funde descritores de runtime com os metadados de apresentação, produzindo o
 * catálogo ordenado por categoria e, dentro de cada categoria, por rótulo.
 */
export function buildCadCommandCatalog(
	descriptors: readonly CadCommandDescriptor[]
): CadCommandCatalogItem[] {
	const items = descriptors.map((descriptor): CadCommandCatalogItem => {
		const id = descriptor.globalName.toUpperCase();
		const presentation = COMMAND_PRESENTATION[id];

		return {
			id,
			command: descriptor.globalName,
			label: presentation?.label ?? fallbackLabel(descriptor.localName || descriptor.globalName),
			category: presentation?.category ?? 'other',
			group: descriptor.group,
			notes: presentation?.notes
		};
	});

	const categoryRank = new Map<CadCommandCategory, number>(
		CAD_COMMAND_CATEGORY_ORDER.map((category, index) => [category, index])
	);

	return items.sort((left, right) => {
		const rankDelta =
			(categoryRank.get(left.category) ?? Number.MAX_SAFE_INTEGER) -
			(categoryRank.get(right.category) ?? Number.MAX_SAFE_INTEGER);

		if (rankDelta !== 0) {
			return rankDelta;
		}

		return left.label.localeCompare(right.label, 'pt-BR');
	});
}
