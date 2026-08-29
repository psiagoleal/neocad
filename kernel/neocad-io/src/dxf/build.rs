// Caminho relativo: kernel/neocad-io/src/dxf/build.rs
//! \file kernel/neocad-io/src/dxf/build.rs
//! \brief Montagem de um documento a partir de uma leitura DXF.
//! \author Iago Leal
//! \date 2026-08-29

use std::collections::BTreeMap;

use neocad_model::{BlockId, Document, DocumentError, Entity, LayerTable};

use super::entities::{EntitySpace, DEFAULT_PAPER_SPACE};
use super::DxfReading;

/// Nome da aba a que uma entidade de papel sem `410` pertence.
///
/// O código `67` diz que a entidade está no papel, e não **em qual** papel: o
/// DXF antigo só tem um espaço-papel e não nomeia aba. `Layout1` é como o
/// AutoCAD chama essa aba, e usar outro nome faria a mesma prancha aparecer com
/// nomes diferentes conforme a versão do arquivo de origem.
pub const DEFAULT_PAPER_LAYOUT: &str = "Layout1";

/// Documento montado, com o que a montagem precisou improvisar.
#[derive(Debug)]
pub struct DocumentBuild {
    /// Documento pronto.
    pub document: Document,
    /// Abas criadas porque uma entidade as citou sem que o arquivo as
    /// declarasse.
    ///
    /// Não é defeito da leitura: a seção `OBJECTS`, que declara os layouts,
    /// ainda não é consumida (MT-KL-10). Até lá, é a citação que revela a aba —
    /// e criar é a única alternativa a perder a entidade.
    pub created_layouts: Vec<String>,
    /// Abas cujo nome o modelo recusou, e cujo conteúdo foi para a aba padrão.
    pub relocated_layouts: Vec<String>,
    /// Entidades recusadas por referenciarem camada que o modelo não aceita.
    pub skipped_count: usize,
}

/// Monta um [`Document`] a partir de uma leitura DXF.
///
/// # O bloco de destino de cada entidade
///
/// É o que este passo decide, e o que estava bloqueado até a fase de layouts:
/// entidade de espaço-modelo vai para `*Model_Space`; entidade de papel vai para
/// o bloco da **aba** que o código `410` nomeia, ou para a aba padrão quando só
/// há o sinalizador `67`. A aba é criada se não existir, porque a alternativa
/// seria perder a entidade — e perder desenho por causa de uma declaração
/// ausente é exatamente o que o ADR 0005 proíbe.
///
/// # Identificadores de camada são refeitos
///
/// A leitura traz a sua própria tabela de camadas, com identificadores que só
/// valem nela. O documento tem outra, e as entidades são religadas **pelo
/// nome** — usar o identificador cru faria a entidade cair em camada errada, ou
/// em nenhuma.
///
/// # Errors
///
/// Falha quando o documento recusa uma operação estrutural — nome de camada
/// impossível, bloco duplicado. Entidade solta não derruba a montagem: ela é
/// contada em [`DocumentBuild::skipped_count`].
pub fn build_document(reading: &DxfReading) -> Result<DocumentBuild, DocumentError> {
    let mut document = Document::new();
    let mut build = DocumentBuild {
        document: Document::new(),
        created_layouts: Vec::new(),
        relocated_layouts: Vec::new(),
        skipped_count: 0,
    };

    copiar_camadas(&mut document, &reading.layers)?;

    let mut abas: BTreeMap<String, BlockId> = BTreeMap::new();
    let mut destino_de = |document: &mut Document,
                          espaco: &EntitySpace,
                          criadas: &mut Vec<String>,
                          realocadas: &mut Vec<String>|
     -> Result<BlockId, DocumentError> {
        let aba = match espaco {
            EntitySpace::Model => return Ok(document.model_space()),
            EntitySpace::Paper(nome) if nome == DEFAULT_PAPER_SPACE => {
                String::from(DEFAULT_PAPER_LAYOUT)
            }
            EntitySpace::Paper(nome) => nome.clone(),
        };

        if let Some(&bloco) = abas.get(&aba) {
            return Ok(bloco);
        }

        let bloco = match criar_aba(document, &aba) {
            Ok(bloco) => {
                criadas.push(aba.clone());
                bloco
            }
            Err(_) => {
                // Nome de aba que o modelo recusa não pode custar o desenho: o
                // conteúdo vai para a aba padrão, e o nome recusado é relatado.
                realocadas.push(aba.clone());
                let padrao = String::from(DEFAULT_PAPER_LAYOUT);

                match abas.get(&padrao) {
                    Some(&bloco) => bloco,
                    None => {
                        let bloco = criar_aba(document, &padrao)?;
                        criadas.push(padrao.clone());
                        abas.insert(padrao, bloco);
                        bloco
                    }
                }
            }
        };

        abas.insert(aba, bloco);

        Ok(bloco)
    };

    for lida in &reading.entities {
        let bloco = destino_de(
            &mut document,
            &lida.space,
            &mut build.created_layouts,
            &mut build.relocated_layouts,
        )?;

        if !inserir(&mut document, &reading.layers, &lida.entity, bloco)? {
            build.skipped_count += 1;
        }
    }

    for definicao in &reading.blocks {
        // Os blocos de espaço não são blocos do desenho: o conteúdo deles já
        // veio pela seção de entidades, com o espaço marcado.
        if definicao.name.starts_with('*') {
            continue;
        }

        let bloco = document.create_block(definicao.name.as_str())?;
        document.set_block_origin(bloco, definicao.base_point)?;

        for entidade in &definicao.entities {
            if !inserir(&mut document, &reading.layers, entidade, bloco)? {
                build.skipped_count += 1;
            }
        }
    }

    build.document = document;

    Ok(build)
}

/// Cria uma aba e devolve o bloco dela.
fn criar_aba(document: &mut Document, nome: &str) -> Result<BlockId, DocumentError> {
    let layout = document.create_layout(nome)?;

    document
        .layouts()
        .get(layout)
        .map(|registro| registro.block())
        .ok_or(DocumentError::UnknownBlock)
}

/// Copia as camadas da leitura para o documento, preservando as propriedades.
fn copiar_camadas(document: &mut Document, layers: &LayerTable) -> Result<(), DocumentError> {
    for (_, registro) in layers.iter() {
        // A camada `0` existe em todo documento; o arquivo apenas redefine as
        // propriedades dela.
        let id = match document.layers().id_of(registro.name()) {
            Some(existente) => existente,
            None => document.create_layer(registro.name())?,
        };

        document.edit().set_layer_record(id, registro.clone())?;
    }

    Ok(())
}

/// Insere uma entidade num bloco, religando a camada pelo nome.
///
/// Devolve `false` quando a camada não existe no documento — o que só acontece
/// se a tabela da leitura e a do documento divergirem, e é contado em vez de
/// derrubar a montagem.
fn inserir(
    document: &mut Document,
    layers: &LayerTable,
    entidade: &Entity,
    bloco: BlockId,
) -> Result<bool, DocumentError> {
    let Some(nome) = layers
        .get(entidade.layer)
        .map(|camada| camada.name().to_owned())
    else {
        return Ok(false);
    };

    let Some(layer) = document.layers().id_of(&nome) else {
        return Ok(false);
    };

    let mut copia = entidade.clone();
    copia.layer = layer;

    let mut editor = document.edit();
    let resultado = editor.insert_entity(copia, bloco);
    let _ = editor.finish();
    resultado?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_dxf;
    use neocad_model::MODEL_LAYOUT_NAME;

    /// Monta um arquivo com uma seção `ENTITIES` a partir de pares.
    fn arquivo(pares: &[(u16, &str)]) -> Vec<u8> {
        let mut texto = String::from("  0\nSECTION\n  2\nENTITIES\n");

        for (codigo, valor) in pares {
            texto.push_str(&format!("{codigo:>3}\n{valor}\n"));
        }

        texto.push_str("  0\nENDSEC\n  0\nEOF\n");
        texto.into_bytes()
    }

    /// Uma reta, com os pares extras que o teste quiser antes da geometria.
    fn reta<'a>(extras: &[(u16, &'a str)]) -> Vec<(u16, &'a str)> {
        let mut pares = vec![(0, "LINE"), (8, "0")];
        pares.extend_from_slice(extras);
        pares.extend_from_slice(&[(10, "0.0"), (20, "0.0"), (11, "1.0"), (21, "1.0")]);

        pares
    }

    fn montar(pares: &[(u16, &str)]) -> DocumentBuild {
        build_document(&read_dxf(&arquivo(pares))).expect("montagem válida")
    }

    /// Entidades de uma aba, pelo nome.
    fn na_aba(build: &DocumentBuild, aba: &str) -> usize {
        build
            .document
            .layouts()
            .get_by_name(aba)
            .map_or(0, |registro| {
                build.document.entities_in_block(registro.block()).count()
            })
    }

    #[test]
    fn entidade_de_modelo_vai_para_o_espaco_modelo() {
        let build = montar(&reta(&[]));
        let modelo = build.document.model_space();

        assert_eq!(build.document.entities_in_block(modelo).count(), 1);
        assert!(build.created_layouts.is_empty());
        assert_eq!(build.document.layouts().len(), 1);
    }

    #[test]
    fn entidade_de_papel_pelo_67_vai_para_a_aba_padrao() {
        // O `67` diz que está no papel, e não em qual: o DXF antigo só tem um
        // espaço-papel e não nomeia aba.
        let build = montar(&reta(&[(67, "1")]));

        assert_eq!(na_aba(&build, DEFAULT_PAPER_LAYOUT), 1);
        assert_eq!(build.created_layouts, [DEFAULT_PAPER_LAYOUT]);
        assert_eq!(
            build
                .document
                .entities_in_block(build.document.model_space())
                .count(),
            0
        );
    }

    #[test]
    fn entidade_de_papel_pelo_410_vai_para_a_aba_nomeada() {
        let build = montar(&reta(&[(67, "1"), (410, "Prancha A1")]));

        assert_eq!(na_aba(&build, "Prancha A1"), 1);
        assert_eq!(build.created_layouts, ["Prancha A1"]);
    }

    #[test]
    fn a_aba_inexistente_e_criada_em_vez_de_a_entidade_se_perder() {
        // O critério de aceite do MT-KL-09. A seção `OBJECTS`, que declara os
        // layouts, ainda não é consumida — então **toda** aba citada é
        // inexistente, e criar é a única alternativa a perder desenho.
        let build = montar(&reta(&[(410, "Aba Que Ninguem Declarou")]));

        assert_eq!(na_aba(&build, "Aba Que Ninguem Declarou"), 1);
        assert_eq!(build.skipped_count, 0);
    }

    #[test]
    fn abas_diferentes_recebem_blocos_diferentes() {
        let mut pares = reta(&[(67, "1"), (410, "Prancha A1")]);
        pares.extend(reta(&[(67, "1"), (410, "Prancha A2")]));
        pares.extend(reta(&[(67, "1"), (410, "Prancha A1")]));

        let build = montar(&pares);

        assert_eq!(na_aba(&build, "Prancha A1"), 2);
        assert_eq!(na_aba(&build, "Prancha A2"), 1);
        // Modelo mais as duas pranchas.
        assert_eq!(build.document.layouts().len(), 3);
    }

    #[test]
    fn a_aba_chamada_model_e_o_espaco_modelo_e_nao_uma_prancha() {
        let build = montar(&reta(&[(410, "Model")]));

        assert_eq!(
            build
                .document
                .entities_in_block(build.document.model_space())
                .count(),
            1
        );
        assert_eq!(build.document.layouts().len(), 1);
        assert!(build.created_layouts.is_empty());
    }

    #[test]
    fn aba_de_nome_impossivel_nao_custa_a_entidade() {
        // Nome com barra é recusado pelo modelo. O conteúdo vai para a aba
        // padrão e o nome recusado é relatado — perder o desenho por causa do
        // nome da aba seria trocar um problema pequeno por um grande.
        let build = montar(&reta(&[(67, "1"), (410, "Prancha/A1")]));

        assert_eq!(build.relocated_layouts, ["Prancha/A1"]);
        assert_eq!(na_aba(&build, DEFAULT_PAPER_LAYOUT), 1);
        assert_eq!(build.skipped_count, 0);
    }

    #[test]
    fn a_aba_do_espaco_modelo_continua_sendo_a_primeira() {
        let build = montar(&reta(&[(67, "1"), (410, "Prancha")]));
        let abas: Vec<&str> = build
            .document
            .layouts()
            .in_tab_order()
            .iter()
            .map(|(_, registro)| registro.name())
            .collect();

        assert_eq!(abas, [MODEL_LAYOUT_NAME, "Prancha"]);
    }

    #[test]
    fn as_camadas_sao_religadas_pelo_nome_e_nao_pelo_identificador() {
        // A leitura traz a própria tabela, com identificadores que só valem
        // nela. Usar o identificador cru faria a entidade cair em camada errada.
        let mut pares = vec![
            (0, "LINE"),
            (8, "Fiação"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
        ];
        pares.extend(reta(&[]));

        let build = montar(&pares);
        let fiacao = build
            .document
            .layers()
            .id_of("Fiação")
            .expect("camada criada por citação");

        assert_eq!(build.document.entities_in_layer(fiacao).count(), 1);
        assert_eq!(build.skipped_count, 0);
    }

    #[test]
    fn a_definicao_de_bloco_atravessa_com_o_conteudo() {
        let texto = concat!(
            "  0\nSECTION\n  2\nBLOCKS\n",
            "  0\nBLOCK\n  2\nMARCO\n 10\n1.0\n 20\n2.0\n",
            "  0\nLINE\n  8\n0\n 10\n0.0\n 20\n0.0\n 11\n5.0\n 21\n5.0\n",
            "  0\nENDBLK\n  0\nENDSEC\n  0\nEOF\n"
        );
        let build = build_document(&read_dxf(texto.as_bytes())).expect("montagem válida");

        let bloco = build
            .document
            .blocks()
            .id_of("MARCO")
            .expect("bloco montado");
        assert_eq!(build.document.entities_in_block(bloco).count(), 1);
    }

    #[test]
    fn arquivo_vazio_produz_documento_valido() {
        let build = build_document(&read_dxf(b"")).expect("montagem válida");

        assert_eq!(build.document.entity_count(), 0);
        assert_eq!(build.document.layouts().len(), 1);
        assert_eq!(build.document.layers().len(), 1);
    }

    #[test]
    fn a_fixture_de_dois_layouts_chega_inteira_ao_documento() {
        let caminho = format!(
            "{}/../../e2e/fixtures/two-layouts.dxf",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(&caminho).expect("fixture existe");
        let build = build_document(&read_dxf(&bytes)).expect("montagem válida");

        assert_eq!(
            build
                .document
                .entities_in_block(build.document.model_space())
                .count(),
            2
        );
        assert_eq!(na_aba(&build, "Prancha A1"), 2);
        assert_eq!(na_aba(&build, "Prancha A2"), 1);
        assert_eq!(build.skipped_count, 0);
        assert!(build.relocated_layouts.is_empty());
    }
}
