// Caminho relativo: src/lib/command-line/catalogo.ts

/**
 * \file src/lib/command-line/catalogo.ts
 * \brief O catálogo de comandos da spec 08, embutido no build.
 * \author Iago Leal
 * \date 2026-11-02
 */

import tabela from '../../../docs/specs/08-comandos-autocad.tsv?raw';
import { lerTabela, type ComandoDoCatalogo } from './resolver';

/**
 * A tabela entra no bundle como texto e é lida uma vez; a spec continua sendo o
 * único lugar onde os nomes são escritos.
 */
export const CATALOGO_DA_SPEC: ComandoDoCatalogo[] = lerTabela(tabela);
