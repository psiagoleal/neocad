// Caminho relativo: src/lib/command-line/selecao-etapa.ts

/**
 * \file src/lib/command-line/selecao-etapa.ts
 * \brief A etapa "Selecione objetos:" que os comandos de edição têm em comum.
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Com seleção, o comando segue direto; sem ela, a linha pede objetos, o usuário
 * seleciona na tela e confirma o vazio. Sem nada selecionado, o comando encerra
 * sem mudar o documento (spec 10, M4). Quando o comando termina, a seleção é
 * esvaziada, como no AutoCAD.
 */

import type { CadEntityId } from '$lib/types/cad';
import type { ComandoEmCurso, ContextoDeComando, Resposta } from './motor';

export function sobreASelecao(
	contexto: ContextoDeComando,
	continuar: (ids: CadEntityId[]) => ComandoEmCurso
): ComandoEmCurso {
	const { selecao } = contexto;
	let depois: ComandoEmCurso | null = null;

	/** Ao terminar o comando, a seleção sai, como no AutoCAD. */
	function terminar(resposta: Resposta): Resposta {
		if (resposta.tipo === 'fim') selecao.esvaziar();
		return resposta;
	}

	function comecar(): Resposta {
		const ids = selecao.ids();
		if (ids.length === 0) return { tipo: 'fim' };
		depois = continuar(ids);
		return terminar(depois.iniciar?.() ?? { tipo: 'continua' });
	}

	return {
		iniciar: () => (selecao.ids().length > 0 ? comecar() : { tipo: 'continua' }),
		prompt: () => depois?.prompt() ?? 'Selecione objetos:',
		responder(texto) {
			if (depois != null) return terminar(depois.responder(texto));
			if (texto.trim() === '') return comecar();
			return {
				tipo: 'invalida',
				mensagem: 'Selecione objetos na tela; Enter segue com o que estiver selecionado.'
			};
		},
		encerrar: () => depois?.encerrar()
	};
}
