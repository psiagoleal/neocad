// Caminho relativo: scripts/gerar-dxf-sintetico.mjs

/**
 * \file scripts/gerar-dxf-sintetico.mjs
 * \brief Gera um DXF sintético grande, para medir abertura e edição sem arquivo de terceiros.
 * \author Iago Leal
 * \date 2026-10-30
 *
 * Os spikes da fase KV medem o NeoCAD num desenho de 100 mil entidades (spec 07), e a
 * spec 00 (T10) não deixa o corpus depender de arquivo de origem desconhecida. Este
 * script gera esse desenho a partir de uma semente: o mesmo pedido dá o mesmo arquivo.
 *
 * O arquivo usa só as primitivas que o kernel modela (LINE, CIRCLE, ARC, LWPOLYLINE e
 * TEXT), espalhadas em camadas, no espaço-modelo e em dois layouts de espaço-papel. A
 * estrutura segue a fixture `e2e/fixtures/two-layouts.dxf`.
 *
 * Uso: node scripts/gerar-dxf-sintetico.mjs --entidades 100000 --camadas 20 \
 *        [--semente 1] --saida desenho.dxf
 * A contagem esperada sai em JSON na saída padrão.
 */

import { writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';

const TIPOS = ['LINE', 'CIRCLE', 'ARC', 'LWPOLYLINE', 'TEXT'];

/** Layouts de espaço-papel, na ordem das abas, com o registro de bloco de cada um. */
const LAYOUTS = [
	{ nome: 'Prancha A1', bloco: '*Paper_Space' },
	{ nome: 'Prancha A2', bloco: '*Paper_Space0' }
];

/** A cada 20 entidades, uma vai para cada layout; as demais ficam no espaço-modelo. */
const CICLO_DE_ESPACO = 20;

/** Lado do quadrado, em unidades de desenho, onde o espaço-modelo é preenchido. */
const LADO_DO_MODELO = 10_000;

/** Gerador pseudoaleatório pequeno e determinístico (mulberry32). */
function aleatorio(semente) {
	let estado = semente >>> 0;

	return () => {
		estado = (estado + 0x6d2b79f5) >>> 0;
		let t = estado;
		t = Math.imul(t ^ (t >>> 15), t | 1);
		t ^= t + Math.imul(t ^ (t >>> 7), t | 61);

		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

/** Formata um número com precisão fixa, como o DXF espera. */
function num(valor) {
	return valor.toFixed(3);
}

/**
 * Gera o DXF.
 *
 * @param {{ entidades: number, camadas: number, semente?: number }} opcoes
 * @returns {{ texto: string, contagem: {
 *   total: number, modelo: number, papel: Record<string, number>,
 *   porTipo: Record<string, number>, camadas: number } }}
 */
export function gerarDxfSintetico({ entidades, camadas, semente = 1 }) {
	if (!Number.isInteger(entidades) || entidades < 0) {
		throw new Error(`quantidade de entidades inválida: ${entidades}`);
	}

	if (!Number.isInteger(camadas) || camadas < 1) {
		throw new Error(`quantidade de camadas inválida: ${camadas}`);
	}

	if (!Number.isInteger(semente)) {
		throw new Error(`semente inválida: ${semente}`);
	}

	const sorteio = aleatorio(semente);
	const linhas = [];
	const par = (codigo, valor) => linhas.push(String(codigo).padStart(3, ' '), String(valor));

	let proximoHandle = 0x100;
	const handle = () => (proximoHandle++).toString(16).toUpperCase();

	const nomesDeCamada = Array.from(
		{ length: camadas },
		(_, i) => `S-${String(i + 1).padStart(2, '0')}`
	);

	// Os handles fixos das tabelas e dos blocos vêm antes das entidades.
	const handleDoRegistro = { '*Model_Space': '20', '*Paper_Space': '21', '*Paper_Space0': '22' };

	par(999, 'Desenho sintetico gerado por scripts/gerar-dxf-sintetico.mjs.');
	par(0, 'SECTION');
	par(2, 'HEADER');
	par(9, '$ACADVER');
	par(1, 'AC1015');
	const indiceDoHandseed = linhas.length + 1;
	par(9, '$HANDSEED');
	par(5, '0');
	par(0, 'ENDSEC');

	par(0, 'SECTION');
	par(2, 'TABLES');
	par(0, 'TABLE');
	par(2, 'LAYER');
	par(5, '2');
	par(100, 'AcDbSymbolTable');
	par(70, camadas + 1);

	for (const [i, nome] of ['0', ...nomesDeCamada].entries()) {
		par(0, 'LAYER');
		par(5, handle());
		par(100, 'AcDbSymbolTableRecord');
		par(100, 'AcDbLayerTableRecord');
		par(2, nome);
		par(70, 0);
		par(62, (i % 7) + 1);
		par(6, 'Continuous');
		par(370, -3);
	}

	par(0, 'ENDTAB');
	par(0, 'TABLE');
	par(2, 'BLOCK_RECORD');
	par(5, '3');
	par(100, 'AcDbSymbolTable');
	par(70, 3);

	for (const [nome, h] of Object.entries(handleDoRegistro)) {
		par(0, 'BLOCK_RECORD');
		par(5, h);
		par(100, 'AcDbSymbolTableRecord');
		par(100, 'AcDbBlockTableRecord');
		par(2, nome);
		par(70, 0);
	}

	par(0, 'ENDTAB');
	par(0, 'ENDSEC');

	par(0, 'SECTION');
	par(2, 'BLOCKS');

	for (const nome of Object.keys(handleDoRegistro)) {
		par(0, 'BLOCK');
		par(5, handle());
		par(100, 'AcDbEntity');
		par(8, '0');
		par(100, 'AcDbBlockBegin');
		par(2, nome);
		par(70, 0);
		par(10, '0.0');
		par(20, '0.0');
		par(30, '0.0');
		par(3, nome);
		par(1, '');
		par(0, 'ENDBLK');
		par(5, handle());
		par(100, 'AcDbEntity');
		par(8, '0');
		par(100, 'AcDbBlockEnd');
	}

	par(0, 'ENDSEC');

	const contagem = {
		total: entidades,
		modelo: 0,
		papel: Object.fromEntries(LAYOUTS.map((layout) => [layout.nome, 0])),
		porTipo: Object.fromEntries(TIPOS.map((tipo) => [tipo, 0])),
		camadas: camadas + 1
	};

	par(0, 'SECTION');
	par(2, 'ENTITIES');

	for (let i = 0; i < entidades; i++) {
		const tipo = TIPOS[i % TIPOS.length];
		const posicaoNoCiclo = i % CICLO_DE_ESPACO;
		const layout =
			posicaoNoCiclo >= CICLO_DE_ESPACO - LAYOUTS.length
				? LAYOUTS[posicaoNoCiclo - (CICLO_DE_ESPACO - LAYOUTS.length)]
				: null;
		const lado = layout == null ? LADO_DO_MODELO : 280;
		const tamanho = lado / 100;
		const x = sorteio() * lado;
		const y = sorteio() * lado;
		const camada = nomesDeCamada[Math.floor(sorteio() * camadas)];

		par(0, tipo);
		par(5, handle());
		par(100, 'AcDbEntity');

		if (layout != null) {
			par(67, 1);
			par(410, layout.nome);
			contagem.papel[layout.nome]++;
		} else {
			contagem.modelo++;
		}

		par(8, camada);
		contagem.porTipo[tipo]++;

		const raio = tamanho * (0.2 + sorteio());

		switch (tipo) {
			case 'LINE': {
				const angulo = sorteio() * 2 * Math.PI;
				par(100, 'AcDbLine');
				par(10, num(x));
				par(20, num(y));
				par(30, '0.0');
				par(11, num(x + raio * Math.cos(angulo)));
				par(21, num(y + raio * Math.sin(angulo)));
				par(31, '0.0');
				break;
			}
			case 'CIRCLE':
				par(100, 'AcDbCircle');
				par(10, num(x));
				par(20, num(y));
				par(30, '0.0');
				par(40, num(raio));
				break;
			case 'ARC': {
				const inicio = sorteio() * 360;
				par(100, 'AcDbCircle');
				par(10, num(x));
				par(20, num(y));
				par(30, '0.0');
				par(40, num(raio));
				par(100, 'AcDbArc');
				par(50, num(inicio));
				par(51, num((inicio + 30 + sorteio() * 270) % 360));
				break;
			}
			case 'LWPOLYLINE': {
				const vertices = 3 + Math.floor(sorteio() * 4);
				const fechada = sorteio() < 0.5;
				par(100, 'AcDbPolyline');
				par(90, vertices);
				par(70, fechada ? 1 : 0);

				for (let v = 0; v < vertices; v++) {
					const angulo = (v / vertices) * 2 * Math.PI;
					par(10, num(x + raio * Math.cos(angulo)));
					par(20, num(y + raio * Math.sin(angulo)));
				}
				break;
			}
			case 'TEXT':
				par(100, 'AcDbText');
				par(10, num(x));
				par(20, num(y));
				par(30, '0.0');
				par(40, num(tamanho / 2));
				par(1, `T${i}`);
				par(100, 'AcDbText');
				break;
		}
	}

	par(0, 'ENDSEC');

	par(0, 'SECTION');
	par(2, 'OBJECTS');

	const abas = [
		{ nome: 'Model', ordem: 0, bloco: '*Model_Space' },
		...LAYOUTS.map((layout, i) => ({ ...layout, ordem: i + 1 }))
	];

	for (const aba of abas) {
		par(0, 'LAYOUT');
		par(5, handle());
		par(100, 'AcDbPlotSettings');
		for (const codigo of [1, 2, 4, 6]) par(codigo, '');
		for (const codigo of [40, 41, 42, 43]) par(codigo, '0.0');
		par(44, '420.0');
		par(45, '297.0');
		for (const codigo of [46, 47, 48, 49, 140, 141]) par(codigo, '0.0');
		par(142, '1.0');
		par(143, '1.0');
		par(70, 688);
		par(72, 0);
		par(73, 1);
		par(74, 5);
		par(7, '');
		par(75, 16);
		par(147, '1.0');
		par(100, 'AcDbLayout');
		par(1, aba.nome);
		par(70, 1);
		par(71, aba.ordem);
		for (const codigo of [12, 22, 32, 14, 24, 34]) par(codigo, '0.0');
		par(15, '420.0');
		par(25, '297.0');
		for (const codigo of [35, 146, 13, 23, 33]) par(codigo, '0.0');
		par(16, '1.0');
		par(26, '0.0');
		par(36, '0.0');
		par(17, '0.0');
		par(27, '1.0');
		par(37, '0.0');
		par(76, 0);
		par(330, handleDoRegistro[aba.bloco]);
	}

	par(0, 'ENDSEC');
	par(0, 'EOF');

	linhas[indiceDoHandseed + 2] = handle();

	return { texto: linhas.join('\n') + '\n', contagem };
}

/** Lê as opções da linha de comando, grava o arquivo e imprime a contagem. */
function principal() {
	const { values } = parseArgs({
		options: {
			entidades: { type: 'string' },
			camadas: { type: 'string', default: '20' },
			semente: { type: 'string', default: '1' },
			saida: { type: 'string' }
		}
	});

	if (values.entidades == null || values.saida == null) {
		console.error(
			'uso: node scripts/gerar-dxf-sintetico.mjs --entidades N [--camadas L] [--semente S] --saida arquivo.dxf'
		);
		process.exit(2);
	}

	let resultado;

	try {
		resultado = gerarDxfSintetico({
			entidades: Number(values.entidades),
			camadas: Number(values.camadas),
			semente: Number(values.semente)
		});
	} catch (error) {
		console.error(error instanceof Error ? error.message : String(error));
		process.exit(2);
	}

	const { texto, contagem } = resultado;

	writeFileSync(values.saida, texto);
	console.log(JSON.stringify(contagem, null, 2));
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
	principal();
}
