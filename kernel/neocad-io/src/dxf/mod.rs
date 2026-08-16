// Caminho relativo: kernel/neocad-io/src/dxf/mod.rs
//! \file kernel/neocad-io/src/dxf/mod.rs
//! \brief Leitura e escrita do formato DXF.
//! \author Iago Leal
//! \date 2026-08-11
//!
//! O DXF é lido em camadas, de baixo para cima: o fluxo de pares código/valor
//! ([`pairs`]), depois as seções, depois as tabelas e entidades. Cada camada só
//! conhece a anterior, o que permite testá-las isoladamente e trocar uma sem
//! mexer nas outras. [`read_dxf`] é o ponto onde elas se juntam.
//!
//! Esta é a leitura **própria**, que substitui a do upstream. A motivação está
//! registrada em `docs/tickets/k2-dxf-nativo.md`: o parser upstream não lê
//! arquivos cuja seção `BLOCKS` contenha bloco com entidades.

mod blocks;
mod entities;
mod pairs;
mod report;
mod sections;
mod tables;
mod writer;

pub use blocks::{read_blocks, BlockDefinition, BlocksReading};
pub use entities::{
    read_entities, EntitiesReading, EntitySpace, ReadEntity, RejectedEntity, DEFAULT_PAPER_SPACE,
};
pub use pairs::{pairs, DxfPair, DxfPairError, DxfPairs, DxfValue};
pub use report::DxfReport;
pub use sections::{sections, DxfSectionError, Section, SectionKind, Sections};
pub use tables::{read_layer_table, LayerTableReading, RejectedLayer};
pub use writer::{formatar_real, write_dxf, DxfContents, ACAD_VERSION};

use neocad_model::LayerTable;

/// Um arquivo DXF lido por inteiro.
///
/// # Por que ainda não é um `Document`
///
/// Montar um [`neocad_model::Document`] exige colocar cada entidade num registro
/// de bloco, e as entidades de espaço-papel precisam dos blocos `*Paper_Space*`,
/// que a `BlockTable` **recusa criar** — nomes iniciados por `*` são reservados.
/// Abrir essa via é o MT-KL-04, na fase de layouts.
///
/// A alternativa seria descartar as entidades de papel para montar o documento
/// agora, e isso a diretriz de conformidade do ADR 0005 proíbe: 70% dos desenhos
/// do acervo têm conteúdo lá. Entre entregar um documento incompleto e entregar
/// a leitura completa, a leitura completa é a que não perde trabalho alheio.
#[derive(Debug)]
pub struct DxfReading {
    /// Tabela de camadas, incluindo as criadas por citação de entidade.
    pub layers: LayerTable,
    /// Entidades dos espaços, na ordem do arquivo, cada uma com o seu espaço.
    pub entities: Vec<ReadEntity>,
    /// Definições de bloco, na ordem do arquivo.
    pub blocks: Vec<BlockDefinition>,
    /// O que a leitura não compreendeu.
    pub report: DxfReport,
}

impl DxfReading {
    /// Quantidade de entidades do espaço-modelo.
    #[must_use]
    pub fn model_space_count(&self) -> usize {
        self.entities
            .iter()
            .filter(|lida| lida.space == EntitySpace::Model)
            .count()
    }

    /// Nomes dos layouts de espaço-papel que têm ao menos uma entidade, em ordem
    /// alfabética.
    ///
    /// Serve à pergunta que o acervo tornou urgente: *este desenho está montado
    /// no papel?* Um arquivo com `model_space_count()` zero e esta lista não
    /// vazia é exatamente o caso que hoje abre mostrando nada.
    #[must_use]
    pub fn paper_space_layouts(&self) -> Vec<&str> {
        let mut nomes: Vec<&str> = self
            .entities
            .iter()
            .filter_map(|lida| match &lida.space {
                EntitySpace::Paper(aba) => Some(aba.as_str()),
                EntitySpace::Model => None,
            })
            .collect();

        nomes.sort_unstable();
        nomes.dedup();
        nomes
    }
}

/// Lê um arquivo DXF inteiro: camadas, entidades, blocos e o relatório.
///
/// # Nada interrompe a abertura
///
/// Tipo de entidade desconhecido é contado e ignorado; seção que ainda não
/// consumimos é registrada com o tamanho; falha local de percurso entra no
/// relatório e a leitura segue. Só a perda de sincronismo no fluxo de pares
/// encerra, porque a partir dali não há como distinguir código de valor.
///
/// # A ordem das seções não importa
///
/// As seções são colhidas antes de interpretadas, e as `TABLES` são processadas
/// primeiro, para que as entidades encontrem suas camadas já criadas. Depender da
/// ordem do arquivo faria a leitura falhar em desenho de ferramenta de terceiro,
/// que costuma gravar noutra sequência.
///
/// # Exemplo
///
/// ```
/// use neocad_io::read_dxf;
///
/// let arquivo = b"  0\nSECTION\n  2\nENTITIES\n\
///                 0\nLINE\n  8\n0\n 10\n0.0\n 20\n0.0\n 11\n1.0\n 21\n1.0\n\
///                 0\nHATCH\n  8\n0\n\
///                 0\nENDSEC\n  0\nEOF\n";
/// let leitura = read_dxf(arquivo);
///
/// assert_eq!(leitura.model_space_count(), 1);
/// assert_eq!(leitura.report.unsupported.get("HATCH"), Some(&1));
/// ```
#[must_use]
pub fn read_dxf(input: &[u8]) -> DxfReading {
    let mut report = DxfReport::default();
    let mut colhidas = Vec::new();

    for resultado in sections(input) {
        match resultado {
            Ok(secao) => colhidas.push(secao),
            Err(erro) => report.section_errors.push(erro),
        }
    }

    // As tabelas primeiro: as entidades referenciam camadas por nome, e criá-las
    // por citação só deve acontecer quando o arquivo de fato as omitiu.
    let mut layers = LayerTable::new();
    let primeira_tables = colhidas.iter().position(|s| s.kind == SectionKind::Tables);

    if let Some(indice) = primeira_tables {
        let leitura = read_layer_table(&colhidas[indice]);

        layers = leitura.table;
        report.rejected_layers.extend(leitura.rejected);

        for (codigo, quantidade) in leitura.unread_codes {
            *report.unread_layer_codes.entry(codigo).or_insert(0) += quantidade;
        }
    }

    let mut entities = Vec::new();
    let mut blocks = Vec::new();

    for (indice, secao) in colhidas.iter().enumerate() {
        match secao.kind {
            // Um arquivo bem formado tem uma `TABLES`. Havendo mais — o que
            // concatenação produz —, a segunda não sobrescreve a primeira em
            // silêncio: fica registrada como não consumida, para a perda
            // aparecer em vez de acontecer.
            SectionKind::Tables if primeira_tables == Some(indice) => {}
            SectionKind::Entities => {
                let leitura = read_entities(secao, &mut layers);

                entities.extend(leitura.entities);
                report.created_layers.extend(leitura.created_layers);
                report.rejected_entities.extend(leitura.rejected);

                for (tipo, quantidade) in leitura.unsupported {
                    report.contar_nao_representado(tipo, quantidade);
                }
            }
            SectionKind::Blocks => {
                let leitura = read_blocks(secao, &mut layers);

                blocks.extend(leitura.blocks);
                report.created_layers.extend(leitura.created_layers);
                report.rejected_entities.extend(leitura.rejected);

                for (tipo, quantidade) in leitura.unsupported {
                    report.contar_nao_representado(tipo, quantidade);
                }
            }
            _ => {
                *report
                    .skipped_sections
                    .entry(secao.name.clone())
                    .or_insert(0) += secao.pairs.len();
            }
        }
    }

    DxfReading {
        layers,
        entities,
        blocks,
        report,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Monta um arquivo DXF a partir de seções nomeadas e seus pares.
    fn arquivo(secoes: &[(&str, &[(u16, &str)])]) -> Vec<u8> {
        let mut texto = String::new();

        for (nome, pares) in secoes {
            texto.push_str(&format!("  0\nSECTION\n  2\n{nome}\n"));

            for (codigo, valor) in *pares {
                texto.push_str(&format!("{codigo:>3}\n{valor}\n"));
            }

            texto.push_str("  0\nENDSEC\n");
        }

        texto.push_str("  0\nEOF\n");
        texto.into_bytes()
    }

    const RETA: &[(u16, &str)] = &[
        (0, "LINE"),
        (8, "0"),
        (10, "0.0"),
        (20, "0.0"),
        (11, "1.0"),
        (21, "1.0"),
    ];

    #[test]
    fn arquivo_com_tipos_desconhecidos_abre_e_reporta_os_tres() {
        // O critério de aceite do MT-K2-06, `SPLICE` inventado incluído.
        let bytes = arquivo(&[(
            "ENTITIES",
            &[
                (0, "LINE"),
                (8, "0"),
                (10, "0.0"),
                (20, "0.0"),
                (11, "1.0"),
                (21, "1.0"),
                (0, "HATCH"),
                (8, "0"),
                (0, "DIMENSION"),
                (8, "0"),
                (0, "SPLICE"),
                (8, "0"),
                (0, "HATCH"),
                (8, "0"),
            ],
        )]);

        let leitura = read_dxf(&bytes);

        assert_eq!(leitura.model_space_count(), 1);
        assert_eq!(leitura.report.unsupported.get("HATCH"), Some(&2));
        assert_eq!(leitura.report.unsupported.get("DIMENSION"), Some(&1));
        assert_eq!(leitura.report.unsupported.get("SPLICE"), Some(&1));
        assert_eq!(leitura.report.unsupported_count(), 4);
        assert!(leitura.report.section_errors.is_empty());
    }

    #[test]
    fn camadas_da_tabela_chegam_as_entidades() {
        let bytes = arquivo(&[
            (
                "TABLES",
                &[
                    (0, "TABLE"),
                    (2, "LAYER"),
                    (0, "LAYER"),
                    (2, "Eixos"),
                    (62, "1"),
                ],
            ),
            (
                "ENTITIES",
                &[
                    (0, "LINE"),
                    (8, "Eixos"),
                    (10, "0.0"),
                    (20, "0.0"),
                    (11, "1.0"),
                    (21, "1.0"),
                ],
            ),
        ]);

        let leitura = read_dxf(&bytes);

        // A camada veio da tabela, então não foi criada por citação.
        assert!(leitura.report.created_layers.is_empty());
        assert!(leitura.layers.get_by_name("Eixos").is_some());
    }

    #[test]
    fn tabelas_sao_lidas_antes_mesmo_vindo_depois_no_arquivo() {
        // Ferramenta de terceiro grava fora da ordem canônica, e depender dela
        // faria a camada ser criada por citação em vez de vir com suas cores.
        let bytes = arquivo(&[
            (
                "ENTITIES",
                &[
                    (0, "LINE"),
                    (8, "Eixos"),
                    (10, "0.0"),
                    (20, "0.0"),
                    (11, "1.0"),
                    (21, "1.0"),
                ],
            ),
            (
                "TABLES",
                &[
                    (0, "TABLE"),
                    (2, "LAYER"),
                    (0, "LAYER"),
                    (2, "Eixos"),
                    (62, "5"),
                ],
            ),
        ]);

        let leitura = read_dxf(&bytes);

        assert!(leitura.report.created_layers.is_empty());
        assert_eq!(
            leitura
                .layers
                .get_by_name("Eixos")
                .expect("veio da tabela")
                .color(),
            neocad_model::Color::Index(5)
        );
    }

    #[test]
    fn blocos_e_entidades_convivem_no_mesmo_arquivo() {
        let bytes = arquivo(&[
            (
                "BLOCKS",
                &[
                    (0, "BLOCK"),
                    (2, "MARCO"),
                    (0, "CIRCLE"),
                    (8, "0"),
                    (10, "0.0"),
                    (20, "0.0"),
                    (40, "1.0"),
                    (0, "ENDBLK"),
                ],
            ),
            ("ENTITIES", RETA),
        ]);

        let leitura = read_dxf(&bytes);

        assert_eq!(leitura.blocks.len(), 1);
        assert_eq!(leitura.blocks[0].entities.len(), 1);
        assert_eq!(leitura.model_space_count(), 1);
    }

    #[test]
    fn nao_modelado_dentro_de_bloco_soma_com_o_de_fora() {
        let bytes = arquivo(&[
            (
                "BLOCKS",
                &[(0, "BLOCK"), (2, "UM"), (0, "HATCH"), (0, "ENDBLK")],
            ),
            ("ENTITIES", &[(0, "HATCH"), (8, "0")]),
        ]);

        let leitura = read_dxf(&bytes);

        assert_eq!(leitura.report.unsupported.get("HATCH"), Some(&2));
    }

    #[test]
    fn segunda_tabela_nao_sobrescreve_a_primeira_em_silencio() {
        let bytes = arquivo(&[
            (
                "TABLES",
                &[(0, "TABLE"), (2, "LAYER"), (0, "LAYER"), (2, "Primeira")],
            ),
            (
                "TABLES",
                &[(0, "TABLE"), (2, "LAYER"), (0, "LAYER"), (2, "Segunda")],
            ),
        ]);

        let leitura = read_dxf(&bytes);

        assert!(leitura.layers.get_by_name("Primeira").is_some());
        assert!(leitura.layers.get_by_name("Segunda").is_none());
        // A perda aparece em vez de acontecer.
        assert!(leitura.report.skipped_sections.contains_key("TABLES"));
    }

    #[test]
    fn secao_nao_consumida_e_registrada_com_o_tamanho() {
        // A `OBJECTS` é onde moram os `LAYOUT`: esta contagem é a medida do que
        // falta para a fase KL.
        let bytes = arquivo(&[
            ("HEADER", &[(9, "$ACADVER"), (1, "AC1015")]),
            (
                "OBJECTS",
                &[(0, "LAYOUT"), (1, "Prancha A1"), (0, "LAYOUT")],
            ),
            ("ENTITIES", RETA),
        ]);

        let leitura = read_dxf(&bytes);

        assert_eq!(leitura.report.skipped_sections.get("HEADER"), Some(&2));
        assert_eq!(leitura.report.skipped_sections.get("OBJECTS"), Some(&3));
        // Lacuna conhecida não é sujeira do arquivo.
        assert!(leitura.report.is_clean());
    }

    #[test]
    fn desenho_montado_no_papel_e_reconhecivel() {
        // O caso dos 8% do acervo: nada no espaço-modelo, tudo na prancha.
        let bytes = arquivo(&[(
            "ENTITIES",
            &[
                (0, "LINE"),
                (8, "0"),
                (67, "1"),
                (410, "Prancha A1"),
                (10, "0.0"),
                (20, "0.0"),
                (11, "1.0"),
                (21, "1.0"),
            ],
        )]);

        let leitura = read_dxf(&bytes);

        assert_eq!(leitura.model_space_count(), 0);
        assert_eq!(leitura.paper_space_layouts(), ["Prancha A1"]);
    }

    #[test]
    fn falha_local_de_secao_nao_impede_a_leitura() {
        // `ENDSEC` faltando na primeira seção: relatado, e a segunda é lida.
        let bytes = b"  0\nSECTION\n  2\nHEADER\n  9\n$ACADVER\n\
                      0\nSECTION\n  2\nENTITIES\n\
                      0\nLINE\n  8\n0\n 10\n0.0\n 20\n0.0\n 11\n1.0\n 21\n1.0\n\
                      0\nENDSEC\n  0\nEOF\n";

        let leitura = read_dxf(bytes);

        assert_eq!(leitura.report.section_errors.len(), 1);
        assert_eq!(leitura.model_space_count(), 1);
        assert!(!leitura.report.is_clean());
    }

    #[test]
    fn arquivo_vazio_nao_entra_em_panico() {
        let leitura = read_dxf(b"");

        assert!(leitura.entities.is_empty());
        assert!(leitura.blocks.is_empty());
        assert!(leitura.report.is_clean());
        // A camada `0` existe em todo documento.
        assert_eq!(leitura.layers.iter().count(), 1);
    }

    #[test]
    fn le_a_fixture_que_o_upstream_nao_abre() {
        let caminho = format!(
            "{}/../../e2e/fixtures/block-with-entities.dxf",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(&caminho).expect("fixture existe");

        let leitura = read_dxf(&bytes);

        assert_eq!(leitura.model_space_count(), 2);
        assert_eq!(leitura.blocks.len(), 1);
        assert_eq!(leitura.blocks[0].entities.len(), 1);
        assert!(leitura.report.is_clean());
    }

    #[test]
    fn le_a_fixture_com_entidade_nao_modelada() {
        let caminho = format!(
            "{}/../../e2e/fixtures/with-unsupported.dxf",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(&caminho).expect("fixture existe");

        let leitura = read_dxf(&bytes);

        assert_eq!(leitura.model_space_count(), 1);
        assert_eq!(leitura.report.unsupported_by_frequency(), [("SOLID", 1)]);
        assert_eq!(
            leitura.report.to_string(),
            "1 entidade(s) de 1 tipo(s) não representada(s)"
        );
    }
}
