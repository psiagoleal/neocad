// Caminho relativo: src/lib/command-line/motor.ts

/**
 * \file src/lib/command-line/motor.ts
 * \brief Conduz o comando em curso pelos prompts (spec 08: C1, C1b, C5–C8, C10).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { CadKernelOperationError, type CadDocument } from '$lib/services/cad-document';
import type { CadEntityId } from '$lib/types/cad';
import {
	mensagemDaResolucao,
	normalizar,
	ResolvedorDeComandos,
	type ComandoDoCatalogo
} from './resolver';

/** Uma opção de prompt: palavra-chave e abreviação em cada idioma (spec 08, C8). */
export type Opcao = { pt: string; abreviacaoPt: string; en: string; abreviacaoEn: string };

/** O que o comando responde a um texto confirmado. */
export type Resposta =
	| { tipo: 'continua' }
	| { tipo: 'fim' }
	| { tipo: 'invalida'; mensagem: string };

export interface ComandoEmCurso {
	/**
	 * Chamado logo depois de criado; um comando que já tem o que precisa (a
	 * seleção, no ERASE) responde aqui e pode encerrar sem prompt.
	 */
	iniciar?(): Resposta;
	/** O prompt de agora, já com as opções entre colchetes. */
	prompt(): string;
	/** A seleção da tela mudou por um clique; o comando pode tomá-la como resposta. */
	aoSelecionar?(ids: CadEntityId[]): Resposta;
	responder(texto: string): Resposta;
	/** `Esc`: o comando para; o que já confirmou permanece. */
	encerrar(): void;
}

/** O que os comandos usam do documento; o resto não lhes diz respeito. */
export type DocumentoDosComandos = Pick<
	CadDocument,
	| 'drawLine'
	| 'eraseEntity'
	| 'eraseEntities'
	| 'moveEntities'
	| 'copyEntities'
	| 'rotateEntities'
	| 'mirrorEntities'
	| 'mirrorCopyEntities'
	| 'scaleEntities'
	| 'offsetEntity'
	| 'listLayers'
	| 'getHistory'
	| 'undo'
>;

/** A seleção da tela, como os comandos a veem. */
export interface SelecaoDosComandos {
	ids(): CadEntityId[];
	esvaziar(): void;
}

export type ContextoDeComando = {
	documento: DocumentoDosComandos;
	/** Avisa a página que o kernel mudou (menu, tela). */
	aposEditar(): void;
	selecao: SelecaoDosComandos;
};

export type FabricaDeComando = (contexto: ContextoDeComando) => ComandoEmCurso;

/** Uma linha do histórico da linha de comando. */
export type Mensagem = { tipo: 'eco' | 'info' | 'erro'; texto: string };

export type OpcoesDoMotor = {
	catalogo: ComandoDoCatalogo[];
	/** Comandos implementados, pelo nome em inglês. */
	comandos: Record<string, FabricaDeComando>;
	documento(): DocumentoDosComandos | null;
	aposEditar(): void;
	selecao: SelecaoDosComandos;
	mensagem(m: Mensagem): void;
};

/**
 * Qual opção o texto escolhe (C8): a palavra em português, a abreviação dela e a
 * palavra em inglês; com `_`, só o inglês, e aí também a abreviação inglesa. Na
 * colisão vale o português, por isso ele é procurado antes.
 */
export function lerOpcao<T extends Opcao>(texto: string, opcoes: T[]): T | null {
	const bruto = texto.trim();
	const soIngles = bruto.startsWith('_');
	const t = normalizar(soIngles ? bruto.slice(1) : bruto);
	if (t === '') return null;

	if (soIngles) {
		return opcoes.find((o) => normalizar(o.en) === t || normalizar(o.abreviacaoEn) === t) ?? null;
	}
	return (
		opcoes.find((o) => normalizar(o.pt) === t || normalizar(o.abreviacaoPt) === t) ??
		opcoes.find((o) => normalizar(o.en) === t) ??
		null
	);
}

const PROMPT_LIVRE = 'Comando:';

export class MotorDeComandos {
	private readonly resolvedor: ResolvedorDeComandos;
	private emCursoAgora: { nome: string; comando: ComandoEmCurso } | null = null;
	/** Último comando iniciado, concluído ou cancelado: o que o vazio repete (C6). */
	private ultimoIniciado: string | null = null;

	constructor(private readonly opcoes: OpcoesDoMotor) {
		this.resolvedor = new ResolvedorDeComandos(opcoes.catalogo, Object.keys(opcoes.comandos));
	}

	get prompt(): string {
		return this.emCursoAgora?.comando.prompt() ?? PROMPT_LIVRE;
	}

	/** Nome em inglês do comando em curso, ou `null`. */
	get emCurso(): string | null {
		return this.emCursoAgora?.nome ?? null;
	}

	/** Sugestões para o texto sendo digitado (C9). */
	sugerir(texto: string): ComandoDoCatalogo[] {
		return this.emCursoAgora == null ? this.resolvedor.sugerir(texto) : [];
	}

	/** Avisa o comando em curso que a seleção da tela mudou por um clique. */
	selecaoMudou(): void {
		const atual = this.emCursoAgora;
		const aoSelecionar = atual?.comando.aoSelecionar;
		if (atual == null || aoSelecionar == null) return;
		this.executar(atual, () => aoSelecionar.call(atual.comando, this.opcoes.selecao.ids()));
	}

	/** O texto que o usuário confirmou (`Enter`). */
	confirmar(texto: string): void {
		const atual = this.emCursoAgora;
		if (atual != null) {
			this.opcoes.mensagem({ tipo: 'eco', texto: `${atual.comando.prompt()} ${texto}`.trimEnd() });
			this.responder(atual, texto);
			return;
		}

		if (texto.trim() === '') {
			if (this.ultimoIniciado != null) this.iniciar(this.ultimoIniciado);
			return;
		}

		this.opcoes.mensagem({ tipo: 'eco', texto: `${PROMPT_LIVRE} ${texto}` });
		const resolucao = this.resolvedor.resolver(texto);
		if (resolucao.tipo === 'comando') {
			this.iniciar(resolucao.comando.en);
			return;
		}
		const mensagem = mensagemDaResolucao(resolucao);
		if (mensagem != null) this.opcoes.mensagem({ tipo: 'erro', texto: mensagem });
	}

	/** `Esc`: encerra o comando em curso; sem comando, nada a fazer aqui. */
	cancelar(): void {
		const atual = this.emCursoAgora;
		if (atual == null) return;
		this.emCursoAgora = null;
		atual.comando.encerrar();
		this.opcoes.mensagem({ tipo: 'info', texto: '*Cancelado*' });
	}

	private iniciar(nome: string): void {
		const fabrica = this.opcoes.comandos[nome];
		const documento = this.opcoes.documento();
		if (fabrica == null) return;
		if (documento == null) {
			this.opcoes.mensagem({ tipo: 'erro', texto: `Abra um desenho antes de usar ${nome}.` });
			return;
		}

		this.ultimoIniciado = nome;
		const atual = (this.emCursoAgora = {
			nome,
			comando: fabrica({
				documento,
				aposEditar: () => this.opcoes.aposEditar(),
				selecao: this.opcoes.selecao
			})
		});
		if (atual.comando.iniciar != null) {
			const comando = atual.comando;
			this.executar(atual, () => comando.iniciar?.() ?? { tipo: 'continua' });
		}
	}

	private responder(atual: { nome: string; comando: ComandoEmCurso }, texto: string): void {
		this.executar(atual, () => atual.comando.responder(texto));
	}

	/** Roda um passo do comando e trata a resposta e a recusa do kernel. */
	private executar(atual: { nome: string; comando: ComandoEmCurso }, passo: () => Resposta): void {
		let resposta: Resposta;
		try {
			resposta = passo();
		} catch (erro) {
			// Spec 07, V5b: a recusa do kernel vira mensagem, e o comando continua.
			if (!(erro instanceof CadKernelOperationError)) throw erro;
			this.opcoes.mensagem({ tipo: 'erro', texto: `O kernel recusou: ${erro.kernelMessage}.` });
			return;
		}

		if (resposta.tipo === 'invalida') {
			this.opcoes.mensagem({ tipo: 'erro', texto: resposta.mensagem });
		} else if (resposta.tipo === 'fim') {
			this.emCursoAgora = null;
		}
	}
}
