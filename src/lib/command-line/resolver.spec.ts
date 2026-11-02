// Caminho relativo: src/lib/command-line/resolver.spec.ts

/**
 * \file src/lib/command-line/resolver.spec.ts
 * \brief Resolução do texto digitado em comando (spec 08: C1, C2, C3, C4, C5, C9).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * A tabela é a da spec (`08-comandos-autocad.tsv`), lida do disco: os casos
 * parametrizados passam por todas as linhas dela.
 */

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import {
	lerTabela,
	mensagemDaResolucao,
	normalizar,
	ResolvedorDeComandos,
	type ComandoDoCatalogo
} from './resolver';

const TSV = readFileSync(path.join(process.cwd(), 'docs/specs/08-comandos-autocad.tsv'), 'utf-8');
const LINHAS = TSV.trim()
	.split('\n')
	.slice(1)
	.map((linha) => linha.split('\t'));
const CATALOGO = lerTabela(TSV);
const TODOS = CATALOGO.map((c) => c.en);

describe('lerTabela', () => {
	it('agrupa os apelidos de cada comando e mantém a grafia oficial', () => {
		const hachura = CATALOGO.find((c) => c.en === 'HATCH');
		expect(hachura).toEqual({ en: 'HATCH', ptbr: 'HACHURA', apelidos: ['BH', 'H'] });
		expect(CATALOGO.find((c) => c.en === 'POLYSOLID')?.ptbr).toBe('POLISSÓLIDO');
		expect(new Set(TODOS).size).toBe(TODOS.length);
		expect(CATALOGO.flatMap((c) => c.apelidos)).toHaveLength(LINHAS.length);
	});

	it('recusa tabela com cabeçalho ou colunas fora do formato', () => {
		expect(() => lerTabela('alias\ten\tpt\nL\tLINE\tLINHA\n')).toThrow(/cabeçalho/);
		expect(() => lerTabela('apelido\tcomando_en\tcomando_ptbr\nL\tLINE\n')).toThrow(/linha 2/);
	});

	it('recusa o mesmo comando com dois nomes em português', () => {
		const tsv = 'apelido\tcomando_en\tcomando_ptbr\nL\tLINE\tLINHA\nLI\tLINE\tRETA\n';
		expect(() => lerTabela(tsv)).toThrow(/LINE/);
	});
});

describe('C1: nome em inglês, em português e apelido iniciam o comando', () => {
	const resolvedor = new ResolvedorDeComandos(CATALOGO, TODOS);

	it.each(LINHAS)('%s / %s / %s', (apelido, en, ptbr) => {
		for (const texto of [apelido, en, ptbr]) {
			const r = resolvedor.resolver(texto);
			expect(r.tipo, texto).toBe('comando');
			expect(r.tipo === 'comando' && r.comando.en, texto).toBe(en);
			expect(r.tipo === 'comando' && r.semDialogo, texto).toBe(false);
		}
	});
});

describe('C2: caixa e acento não importam', () => {
	const resolvedor = new ResolvedorDeComandos(CATALOGO, TODOS);

	it('POLISSÓLIDO, polissolido e Polissólido são o mesmo', () => {
		for (const texto of ['POLISSÓLIDO', 'polissolido', 'Polissólido', '  psolid ']) {
			const r = resolvedor.resolver(texto);
			expect(r.tipo === 'comando' && r.comando.en, texto).toBe('POLYSOLID');
		}
	});

	it('normalizar tira acento, espaço das pontas e caixa', () => {
		expect(normalizar('  Polissólido ')).toBe('POLISSOLIDO');
		expect(normalizar('ação')).toBe('ACAO');
	});
});

describe('C3: prefixos _ e -', () => {
	const resolvedor = new ResolvedorDeComandos(CATALOGO, TODOS);

	it('_ aceita só o nome em inglês', () => {
		const r = resolvedor.resolver('_line');
		expect(r.tipo === 'comando' && r.comando.en).toBe('LINE');
		expect(resolvedor.resolver('_LINHA')).toEqual({ tipo: 'desconhecido', digitado: '_LINHA' });
		expect(resolvedor.resolver('_L')).toEqual({ tipo: 'desconhecido', digitado: '_L' });
	});

	it('- pede a variante sem diálogo, nos dois idiomas e no apelido', () => {
		for (const texto of ['-LAYER', '-CAMADA', '-LA']) {
			const r = resolvedor.resolver(texto);
			expect(r.tipo === 'comando' && r.comando.en, texto).toBe('LAYER');
			expect(r.tipo === 'comando' && r.semDialogo, texto).toBe(true);
		}
	});

	it('os dois prefixos juntos, em qualquer ordem', () => {
		for (const texto of ['_-LAYER', '-_LAYER', '_-layer']) {
			const r = resolvedor.resolver(texto);
			expect(r.tipo === 'comando' && r.comando.en, texto).toBe('LAYER');
			expect(r.tipo === 'comando' && r.semDialogo, texto).toBe(true);
		}
		expect(resolvedor.resolver('-_CAMADA').tipo).toBe('desconhecido');
	});

	it('prefixo repetido ou sozinho não é comando', () => {
		for (const texto of ['__LINE', '--LINE', '_', '-', '_-']) {
			expect(resolvedor.resolver(texto).tipo, texto).toBe('desconhecido');
		}
	});
});

describe('C4: nenhum texto resolve para dois comandos', () => {
	it('a tabela inteira: 634 textos, nenhuma colisão', () => {
		const resolvedor = new ResolvedorDeComandos(CATALOGO, TODOS);
		expect(resolvedor.quantidadeDeTextos).toBe(634);
	});

	it('um catálogo com colisão é recusado, com o texto e os dois comandos', () => {
		const comColisao: ComandoDoCatalogo[] = [
			{ en: 'LINE', ptbr: 'LINHA', apelidos: ['L'] },
			{ en: 'LENGTHEN', ptbr: 'LÍNHA', apelidos: ['LEN'] }
		];
		expect(() => new ResolvedorDeComandos(comColisao, [])).toThrow(/LINHA.*LINE.*LENGTHEN/);
	});
});

describe('C5: ainda não disponível e desconhecido', () => {
	const resolvedor = new ResolvedorDeComandos(CATALOGO, ['LINE']);

	it('comando do catálogo não implementado é "ainda não disponível", com o nome digitado', () => {
		const r = resolvedor.resolver('hachura');
		expect(r.tipo).toBe('indisponivel');
		expect(r.tipo === 'indisponivel' && r.comando.en).toBe('HATCH');
		expect(mensagemDaResolucao(r)).toBe('hachura: ainda não disponível.');
	});

	it('texto fora do catálogo é "comando desconhecido"', () => {
		const r = resolvedor.resolver('XYZZY');
		expect(r).toEqual({ tipo: 'desconhecido', digitado: 'XYZZY' });
		expect(mensagemDaResolucao(r)).toBe('XYZZY: comando desconhecido.');
	});

	it('texto vazio não é erro e não tem mensagem', () => {
		expect(resolvedor.resolver('   ')).toEqual({ tipo: 'vazio' });
		expect(mensagemDaResolucao({ tipo: 'vazio' })).toBeNull();
		expect(mensagemDaResolucao(resolvedor.resolver('L'))).toBeNull();
	});

	it('implementado se reconhece pelo nome em inglês, sem importar a caixa', () => {
		const r = new ResolvedorDeComandos(CATALOGO, ['line']).resolver('LINHA');
		expect(r.tipo).toBe('comando');
	});
});

describe('C9: sugestões pelo começo do nome ou do apelido, nos dois idiomas', () => {
	const resolvedor = new ResolvedorDeComandos(CATALOGO, TODOS);
	const nomes = (texto: string, limite?: number) =>
		resolvedor.sugerir(texto, limite).map((c) => c.en);

	it('o português sugere pelo nome em português', () => {
		expect(nomes('hachu')).toContain('HATCH');
		expect(nomes('polissó')).toContain('POLYSOLID');
	});

	it('o inglês e o apelido sugerem também', () => {
		expect(nomes('HATC')).toContain('HATCH');
		expect(nomes('psol')).toContain('POLYSOLID');
	});

	it('cada comando aparece uma vez, o texto mais curto primeiro, até o limite', () => {
		const lista = nomes('L', 50);
		expect(new Set(lista).size).toBe(lista.length);
		expect(lista[0]).toBe('LINE');
		expect(nomes('L', 3)).toHaveLength(3);
	});

	it('com _ só sugere pelo nome em inglês; vazio não sugere nada', () => {
		expect(nomes('_LINH')).toEqual([]);
		expect(nomes('_LIN')).toContain('LINE');
		expect(nomes('')).toEqual([]);
		expect(nomes('-')).toEqual([]);
	});
});
