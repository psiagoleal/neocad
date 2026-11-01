// Caminho relativo: src/lib/render/webgl2.ts

/**
 * \file src/lib/render/webgl2.ts
 * \brief Caminho pela GPU: os blocos do kernel em WebGL2 (ADR 0008, MT-KV-09).
 * \author Iago Leal
 * \date 2026-11-01
 *
 * Um buffer por bloco, enviado com o `Float32Array` que o kernel devolve, sem
 * cópia; cada quadro é uma chamada `drawArrays(LINES)` por bloco com a
 * transformação da vista num `uniform`.
 */

import type { Rgb } from './cor';
import type { Desenhista } from './desenhista';
import type { Transformacao } from './vista';

const VERTICE = `#version 300 es
in vec2 a_pos;
uniform vec4 u_t;
void main() { gl_Position = vec4(a_pos * u_t.xy + u_t.zw, 0.0, 1.0); }`;

const FRAGMENTO = `#version 300 es
precision mediump float;
uniform vec3 u_cor;
out vec4 cor;
void main() { cor = vec4(u_cor, 1.0); }`;

type Bloco = { vao: WebGLVertexArrayObject; buffer: WebGLBuffer; vertices: number; cor: Rgb };

function compilar(gl: WebGL2RenderingContext, tipo: number, fonte: string): WebGLShader {
	const shader = gl.createShader(tipo);
	if (shader == null) throw new Error('WebGL2: não foi possível criar o shader.');
	gl.shaderSource(shader, fonte);
	gl.compileShader(shader);
	if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
		throw new Error(`WebGL2: shader recusado: ${gl.getShaderInfoLog(shader)}`);
	}
	return shader;
}

export class DesenhistaWebgl2 implements Desenhista {
	readonly caminho = 'webgl2' as const;
	private readonly blocos = new Map<string, Bloco>();
	private readonly programa: WebGLProgram;
	private readonly uT: WebGLUniformLocation | null;
	private readonly uCor: WebGLUniformLocation | null;
	private readonly aPos: number;

	constructor(private readonly gl: WebGL2RenderingContext) {
		const programa = gl.createProgram();
		gl.attachShader(programa, compilar(gl, gl.VERTEX_SHADER, VERTICE));
		gl.attachShader(programa, compilar(gl, gl.FRAGMENT_SHADER, FRAGMENTO));
		gl.linkProgram(programa);
		if (!gl.getProgramParameter(programa, gl.LINK_STATUS)) {
			throw new Error(`WebGL2: programa recusado: ${gl.getProgramInfoLog(programa)}`);
		}
		this.programa = programa;
		this.uT = gl.getUniformLocation(programa, 'u_t');
		this.uCor = gl.getUniformLocation(programa, 'u_cor');
		this.aPos = gl.getAttribLocation(programa, 'a_pos');
	}

	definirBloco(chave: string, segmentos: Float32Array, cor: Rgb): void {
		const gl = this.gl;
		let bloco = this.blocos.get(chave);
		if (bloco == null) {
			const vao = gl.createVertexArray();
			const buffer = gl.createBuffer();
			gl.bindVertexArray(vao);
			gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
			gl.enableVertexAttribArray(this.aPos);
			gl.vertexAttribPointer(this.aPos, 2, gl.FLOAT, false, 0, 0);
			gl.bindVertexArray(null);
			bloco = { vao, buffer, vertices: 0, cor };
			this.blocos.set(chave, bloco);
		}
		gl.bindBuffer(gl.ARRAY_BUFFER, bloco.buffer);
		gl.bufferData(gl.ARRAY_BUFFER, segmentos, gl.STATIC_DRAW);
		bloco.vertices = segmentos.length / 2;
		bloco.cor = cor;
	}

	removerBloco(chave: string): void {
		const bloco = this.blocos.get(chave);
		if (bloco == null) return;
		this.gl.deleteBuffer(bloco.buffer);
		this.gl.deleteVertexArray(bloco.vao);
		this.blocos.delete(chave);
	}

	desenhar(t: Transformacao, largura: number, altura: number, fundo: Rgb | null): void {
		const gl = this.gl;
		gl.viewport(0, 0, largura, altura);
		if (fundo == null) gl.clearColor(0, 0, 0, 0);
		else gl.clearColor(fundo[0] / 255, fundo[1] / 255, fundo[2] / 255, 1);
		gl.clear(gl.COLOR_BUFFER_BIT);
		gl.useProgram(this.programa);
		gl.uniform4f(this.uT, t[0], t[1], t[2], t[3]);

		for (const bloco of this.blocos.values()) {
			if (bloco.vertices === 0) continue;
			gl.uniform3f(this.uCor, bloco.cor[0] / 255, bloco.cor[1] / 255, bloco.cor[2] / 255);
			gl.bindVertexArray(bloco.vao);
			gl.drawArrays(gl.LINES, 0, bloco.vertices);
		}
		gl.bindVertexArray(null);
	}

	destruir(): void {
		for (const chave of [...this.blocos.keys()]) this.removerBloco(chave);
		this.gl.deleteProgram(this.programa);
	}
}
