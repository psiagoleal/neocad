// Caminho relativo: src/lib/command-line/resolver.ts

/**
 * \file src/lib/command-line/resolver.ts
 * \brief Resolve o texto digitado na linha de comando em comando (spec 08).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * O catálogo vem de `docs/specs/08-comandos-autocad.tsv`: nome em inglês, nome em
 * português e apelidos, com a grafia oficial da Autodesk em cada idioma. Nada aqui
 * traduz um nome para o outro idioma. O módulo é puro: não sabe o que cada comando
 * faz, só qual comando o texto pede (C1–C5) e quais começam por ele (C9).
 */

/** Um comando do catálogo, com a grafia oficial da Autodesk em cada idioma. */
export type ComandoDoCatalogo = { en: string; ptbr: string; apelidos: string[] };

export type Resolucao =
	| { tipo: 'vazio' }
	| { tipo: 'comando'; comando: ComandoDoCatalogo; semDialogo: boolean }
	| { tipo: 'indisponivel'; comando: ComandoDoCatalogo; digitado: string }
	| { tipo: 'desconhecido'; digitado: string };

const CABECALHO = 'apelido\tcomando_en\tcomando_ptbr';

/** Caixa alta, sem acento e sem espaço nas pontas (C2). */
export function normalizar(texto: string): string {
	return texto.normalize('NFD').replace(/\p{M}/gu, '').trim().toUpperCase();
}

/**
 * Lê a tabela da spec 08. Cada linha é um apelido; as linhas de um mesmo comando
 * se juntam num item, e precisam concordar no nome em português.
 */
export function lerTabela(tsv: string): ComandoDoCatalogo[] {
	const linhas = tsv.replace(/\r\n/g, '\n').trimEnd().split('\n');
	if (linhas[0] !== CABECALHO) {
		throw new Error(`Tabela de comandos com cabeçalho inesperado: "${linhas[0]}".`);
	}

	const porNome = new Map<string, ComandoDoCatalogo>();
	linhas.slice(1).forEach((linha, indice) => {
		const colunas = linha.split('\t');
		if (colunas.length !== 3 || colunas.some((c) => c.trim() === '')) {
			throw new Error(`Tabela de comandos: a linha ${indice + 2} não tem três colunas.`);
		}

		const [apelido, en, ptbr] = colunas.map((c) => c.trim());
		const existente = porNome.get(en);
		if (existente == null) {
			porNome.set(en, { en, ptbr, apelidos: [apelido] });
		} else if (existente.ptbr !== ptbr) {
			throw new Error(`Tabela de comandos: ${en} aparece como ${existente.ptbr} e como ${ptbr}.`);
		} else {
			existente.apelidos.push(apelido);
		}
	});

	return [...porNome.values()];
}

/** O que a linha responde quando o texto não inicia comando (C5); `null` nos demais casos. */
export function mensagemDaResolucao(resolucao: Resolucao): string | null {
	switch (resolucao.tipo) {
		case 'indisponivel':
			return `${resolucao.digitado}: ainda não disponível.`;
		case 'desconhecido':
			return `${resolucao.digitado}: comando desconhecido.`;
		default:
			return null;
	}
}

/** Texto digitado sem os prefixos `_` e `-`, que valem uma vez cada, em qualquer ordem (C3). */
function separarPrefixos(
	texto: string
): { nome: string; soIngles: boolean; semDialogo: boolean } | null {
	let soIngles = false;
	let semDialogo = false;
	let resto = texto.trim();

	for (;;) {
		if (resto.startsWith('_') && !soIngles) soIngles = true;
		else if (resto.startsWith('-') && !semDialogo) semDialogo = true;
		else break;
		resto = resto.slice(1);
	}

	const nome = normalizar(resto);
	if (nome === '' || nome.startsWith('_') || nome.startsWith('-')) return null;
	return { nome, soIngles, semDialogo };
}

export class ResolvedorDeComandos {
	/** Todo texto digitável (nome, nome em português, apelido), normalizado. */
	private readonly porTexto = new Map<string, ComandoDoCatalogo>();
	/** Só os nomes em inglês, normalizados: o que o prefixo `_` aceita. */
	private readonly porNomeIngles = new Map<string, ComandoDoCatalogo>();
	private readonly implementados: Set<string>;

	/**
	 * @param catalogo comandos reconhecidos (a tabela e os que as specs acrescentam)
	 * @param implementados nomes em inglês dos comandos que já funcionam
	 * @throws se dois comandos diferentes respondem ao mesmo texto (C4)
	 */
	constructor(
		private readonly catalogo: ComandoDoCatalogo[],
		implementados: Iterable<string>
	) {
		this.implementados = new Set([...implementados].map(normalizar));

		for (const comando of catalogo) {
			this.porNomeIngles.set(normalizar(comando.en), comando);
			for (const texto of [comando.en, comando.ptbr, ...comando.apelidos]) {
				const chave = normalizar(texto);
				const outro = this.porTexto.get(chave);
				if (outro != null && outro !== comando) {
					throw new Error(
						`Catálogo de comandos: ${chave} resolve para ${outro.en} e para ${comando.en}.`
					);
				}
				this.porTexto.set(chave, comando);
			}
		}
	}

	/** Quantos textos digitáveis o catálogo reconhece, já normalizados. */
	get quantidadeDeTextos(): number {
		return this.porTexto.size;
	}

	/** Qual comando o texto pede, e se está disponível (C1–C5). */
	resolver(texto: string): Resolucao {
		if (texto.trim() === '') return { tipo: 'vazio' };

		const digitado = texto.trim();
		const partes = separarPrefixos(digitado);
		const comando =
			partes == null
				? undefined
				: (partes.soIngles ? this.porNomeIngles : this.porTexto).get(partes.nome);

		if (partes == null || comando == null) return { tipo: 'desconhecido', digitado };
		if (!this.implementados.has(normalizar(comando.en))) {
			return { tipo: 'indisponivel', comando, digitado };
		}
		return { tipo: 'comando', comando, semDialogo: partes.semDialogo };
	}

	/**
	 * Comandos cujo nome, nome em português ou apelido começa pelo texto (C9), uma
	 * vez cada; o de texto correspondente mais curto vem primeiro.
	 */
	sugerir(texto: string, limite = 8): ComandoDoCatalogo[] {
		const partes = separarPrefixos(texto);
		if (partes == null) return [];

		const encontrados: Array<{ comando: ComandoDoCatalogo; tamanho: number }> = [];
		for (const comando of this.catalogo) {
			const textos = partes.soIngles
				? [comando.en]
				: [comando.en, comando.ptbr, ...comando.apelidos];
			const tamanhos = textos
				.map(normalizar)
				.filter((t) => t.startsWith(partes.nome))
				.map((t) => t.length);
			if (tamanhos.length > 0) encontrados.push({ comando, tamanho: Math.min(...tamanhos) });
		}

		return encontrados
			.sort((a, b) => a.tamanho - b.tamanho || a.comando.en.localeCompare(b.comando.en))
			.slice(0, limite)
			.map(({ comando }) => comando);
	}
}
