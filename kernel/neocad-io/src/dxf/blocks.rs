// Caminho relativo: kernel/neocad-io/src/dxf/blocks.rs
//! \file kernel/neocad-io/src/dxf/blocks.rs
//! \brief Leitura das definições de bloco de um arquivo DXF.
//! \author Iago Leal
//! \date 2026-08-12

use std::collections::BTreeMap;

use neocad_geometry::Point2;
use neocad_model::{Entity, LayerTable};

use super::entities::{ler_registros, EntitiesReading, RejectedEntity};
use super::pairs::DxfPair;
use super::sections::{Section, SectionKind};

/// Bit do código `70` que marca a definição como referência externa (xref).
const XREF: i64 = 4;

/// Uma definição de bloco, com as entidades que moram dentro dela.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDefinition {
    /// Nome do bloco.
    pub name: String,
    /// Ponto-base, a partir do qual cada inserção é posicionada.
    pub base_point: Point2,
    /// Entidades do bloco, na ordem do arquivo.
    pub entities: Vec<Entity>,
    /// Caminho da referência externa, quando o bloco é um xref.
    ///
    /// Um xref não tem conteúdo local: as entidades vivem no arquivo apontado.
    /// Guardar o caminho é o que impede que a referência desapareça em silêncio
    /// só porque não temos o outro arquivo em mãos.
    pub xref_path: Option<String>,
}

impl BlockDefinition {
    /// Indica se o bloco é uma referência externa.
    #[must_use]
    pub const fn is_xref(&self) -> bool {
        self.xref_path.is_some()
    }
}

/// Resultado da leitura da seção `BLOCKS`.
#[derive(Debug, Clone, PartialEq)]
pub struct BlocksReading {
    /// Definições lidas, na ordem do arquivo.
    pub blocks: Vec<BlockDefinition>,
    /// Tipos de entidade que o modelo não representa, somados sobre todos os
    /// blocos, com quantas vezes apareceram.
    pub unsupported: BTreeMap<String, usize>,
    /// Camadas citadas dentro de bloco e ausentes da tabela, criadas na leitura.
    pub created_layers: Vec<String>,
    /// Entidades de bloco que não puderam ser criadas.
    pub rejected: Vec<RejectedEntity>,
}

impl BlocksReading {
    /// Procura uma definição pelo nome, ignorando caixa como os formatos CAD.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&BlockDefinition> {
        self.blocks
            .iter()
            .find(|bloco| bloco.name.eq_ignore_ascii_case(name))
    }

    /// Total de entidades somadas sobre todas as definições.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.blocks.iter().map(|bloco| bloco.entities.len()).sum()
    }
}

/// Lê as definições de bloco de uma seção `BLOCKS`.
///
/// # O ticket que justifica a fase
///
/// O parser DXF do upstream **não abre** arquivo cuja seção `BLOCKS` contenha
/// bloco com entidades dentro — defeito isolado por bissecção e medido em cerca
/// de 11% de um acervo real, justamente a fatia dos desenhos acabados, com
/// carimbo e simbologia. A fixture `e2e/fixtures/block-with-entities.dxf`
/// registra o caso; esta função é a que passa a lê-la.
///
/// # Os espaços aparecem aqui
///
/// Arquivos reais declaram `*Model_Space` e `*Paper_Space` como blocos. Eles são
/// entregues como definições comuns, o que é fiel ao formato e ao ADR 0005, que
/// modela layout exatamente como registro de bloco. Normalmente vêm vazios — as
/// entidades correspondentes estão na seção `ENTITIES` —, mas quando trazem
/// conteúdo ele não se perde.
///
/// # Exemplo
///
/// ```
/// use neocad_io::{read_blocks, sections};
/// use neocad_model::LayerTable;
///
/// let arquivo = b"  0\nSECTION\n  2\nBLOCKS\n\
///                 0\nBLOCK\n  2\nMARCO\n 10\n0.0\n 20\n0.0\n\
///                 0\nLINE\n  8\n0\n 10\n0.0\n 20\n0.0\n 11\n5.0\n 21\n5.0\n\
///                 0\nENDBLK\n  0\nENDSEC\n  0\nEOF\n";
/// let secao = sections(arquivo).next().expect("há seção")?;
/// let mut camadas = LayerTable::new();
/// let leitura = read_blocks(&secao, &mut camadas);
///
/// assert_eq!(leitura.blocks.len(), 1);
/// assert_eq!(leitura.get("MARCO").expect("lido").entities.len(), 1);
/// # Ok::<(), neocad_io::DxfSectionError>(())
/// ```
pub fn read_blocks(section: &Section, layers: &mut LayerTable) -> BlocksReading {
    let mut leitura = BlocksReading {
        blocks: Vec::new(),
        unsupported: BTreeMap::new(),
        created_layers: Vec::new(),
        rejected: Vec::new(),
    };

    if section.kind != SectionKind::Blocks {
        return leitura;
    }

    let mut atual: Option<BlocoEmCurso> = None;

    for par in &section.pairs {
        if par.code != 0 {
            if let Some(bloco) = atual.as_mut() {
                bloco.absorver(par);
            }

            continue;
        }

        match marcador(par) {
            Some("BLOCK") => {
                // Um `BLOCK` novo fecha o anterior: `ENDBLK` faltando é arquivo
                // torto, não motivo para um bloco engolir o seguinte.
                if let Some(anterior) = atual.take() {
                    concluir(anterior, layers, &mut leitura);
                }

                atual = Some(BlocoEmCurso::nova());
            }
            Some("ENDBLK") => {
                if let Some(bloco) = atual.take() {
                    concluir(bloco, layers, &mut leitura);
                }
            }
            _ => {
                if let Some(bloco) = atual.as_mut() {
                    bloco.no_cabecalho = false;
                    bloco.corpo.push(par.clone());
                }
            }
        }
    }

    // Seção que termina sem `ENDBLK` não custa o último bloco.
    if let Some(bloco) = atual.take() {
        concluir(bloco, layers, &mut leitura);
    }

    leitura
}

/// Definição em construção.
///
/// O cabeçalho do bloco e o corpo compartilham códigos — `8` e `10`/`20`
/// aparecem nos dois —, então a separação é posicional: tudo antes do primeiro
/// registro `0` é cabeçalho.
#[derive(Debug)]
struct BlocoEmCurso {
    cabecalho: Vec<DxfPair>,
    corpo: Vec<DxfPair>,
    no_cabecalho: bool,
}

impl BlocoEmCurso {
    fn nova() -> Self {
        Self {
            cabecalho: Vec::new(),
            corpo: Vec::new(),
            no_cabecalho: true,
        }
    }

    fn absorver(&mut self, par: &DxfPair) {
        if self.no_cabecalho {
            self.cabecalho.push(par.clone());
        } else {
            self.corpo.push(par.clone());
        }
    }
}

/// Traduz o bloco acumulado e o acrescenta à leitura.
fn concluir(bloco: BlocoEmCurso, layers: &mut LayerTable, leitura: &mut BlocksReading) {
    let flags = inteiro(&bloco.cabecalho, 70).unwrap_or(0);
    let xref_path =
        (flags & XREF != 0).then(|| texto(&bloco.cabecalho, 1).unwrap_or_default().to_owned());

    let mut entidades = EntitiesReading::vazia();
    ler_registros(&bloco.corpo, layers, &BTreeMap::new(), &mut entidades);

    let definicao = BlockDefinition {
        name: texto(&bloco.cabecalho, 2).unwrap_or_default().to_owned(),
        base_point: Point2::new(
            real(&bloco.cabecalho, 10).unwrap_or(0.0),
            real(&bloco.cabecalho, 20).unwrap_or(0.0),
        ),
        entities: entidades
            .entities
            .iter()
            .map(|lida| lida.entity.clone())
            .collect(),
        xref_path,
    };

    leitura.created_layers.extend(entidades.created_layers);
    leitura.rejected.extend(entidades.rejected);

    for (tipo, quantidade) in entidades.unsupported {
        *leitura.unsupported.entry(tipo).or_insert(0) += quantidade;
    }

    leitura.blocks.push(definicao);
}

/// Texto de um par, aparado, quando o par é textual.
fn marcador(par: &DxfPair) -> Option<&str> {
    par.value.as_text().map(str::trim)
}

/// Primeiro valor textual de um código, aparado.
fn texto(pares: &[DxfPair], code: u16) -> Option<&str> {
    pares
        .iter()
        .find(|par| par.code == code)
        .and_then(|par| par.value.as_text())
        .map(str::trim)
}

/// Primeiro valor inteiro de um código.
fn inteiro(pares: &[DxfPair], code: u16) -> Option<i64> {
    pares
        .iter()
        .find(|par| par.code == code)
        .and_then(|par| par.value.as_integer())
}

/// Primeiro valor real de um código.
fn real(pares: &[DxfPair], code: u16) -> Option<f64> {
    pares
        .iter()
        .find(|par| par.code == code)
        .and_then(|par| par.value.as_real())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sections, SectionKind};
    use neocad_model::Geometry;

    /// Monta uma seção `BLOCKS` a partir de pares.
    fn secao(pares: &[(u16, &str)]) -> Section {
        let mut texto = String::from("  0\nSECTION\n  2\nBLOCKS\n");

        for (codigo, valor) in pares {
            texto.push_str(&format!("{codigo:>3}\n{valor}\n"));
        }

        texto.push_str("  0\nENDSEC\n  0\nEOF\n");

        sections(texto.as_bytes())
            .next()
            .expect("há seção")
            .expect("bem formada")
    }

    fn ler(pares: &[(u16, &str)]) -> BlocksReading {
        let mut camadas = LayerTable::new();

        read_blocks(&secao(pares), &mut camadas)
    }

    /// Lê a seção `BLOCKS` de uma fixture sintética do E2E.
    fn ler_fixture(nome: &str) -> BlocksReading {
        let caminho = format!("{}/../../e2e/fixtures/{nome}", env!("CARGO_MANIFEST_DIR"));
        let bytes = std::fs::read(&caminho).expect("fixture existe");
        let mut camadas = LayerTable::new();

        for secao in sections(&bytes) {
            let secao = secao.expect("fixture bem formada");

            if secao.kind == SectionKind::Blocks {
                return read_blocks(&secao, &mut camadas);
            }
        }

        BlocksReading {
            blocks: Vec::new(),
            unsupported: BTreeMap::new(),
            created_layers: Vec::new(),
            rejected: Vec::new(),
        }
    }

    #[test]
    fn le_bloco_com_entidades_dentro() {
        // É o construto que o parser do upstream não lê, e o motivo da fase.
        let leitura = ler(&[
            (0, "BLOCK"),
            (8, "0"),
            (70, "0"),
            (2, "MARCO"),
            (3, "MARCO"),
            (10, "1.0"),
            (20, "2.0"),
            (0, "LINE"),
            (8, "0"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "5.0"),
            (21, "5.0"),
            (0, "ENDBLK"),
        ]);

        assert_eq!(leitura.blocks.len(), 1);
        let bloco = leitura.get("MARCO").expect("lido");
        assert_eq!(bloco.base_point, Point2::new(1.0, 2.0));
        assert_eq!(bloco.entities.len(), 1);
        assert!(matches!(bloco.entities[0].geometry, Geometry::Line(_)));
        assert!(!bloco.is_xref());
    }

    #[test]
    fn le_varios_blocos_preservando_a_ordem() {
        let leitura = ler(&[
            (0, "BLOCK"),
            (2, "PRIMEIRO"),
            (0, "ENDBLK"),
            (0, "BLOCK"),
            (2, "SEGUNDO"),
            (0, "CIRCLE"),
            (10, "0.0"),
            (20, "0.0"),
            (40, "1.0"),
            (0, "ENDBLK"),
        ]);

        assert_eq!(
            leitura
                .blocks
                .iter()
                .map(|bloco| bloco.name.as_str())
                .collect::<Vec<_>>(),
            ["PRIMEIRO", "SEGUNDO"]
        );
        assert_eq!(leitura.entity_count(), 1);
    }

    #[test]
    fn bloco_vazio_e_definicao_valida() {
        let leitura = ler(&[(0, "BLOCK"), (2, "VAZIO"), (0, "ENDBLK")]);

        assert_eq!(leitura.blocks.len(), 1);
        assert!(leitura.get("VAZIO").expect("lido").entities.is_empty());
    }

    #[test]
    fn cabecalho_e_corpo_nao_se_confundem() {
        // O `10`/`20` do cabeçalho é o ponto-base; o do corpo é da entidade.
        // Separar por posição, e não por código, é o que impede um virar o outro.
        let leitura = ler(&[
            (0, "BLOCK"),
            (2, "COM_BASE"),
            (10, "100.0"),
            (20, "200.0"),
            (0, "CIRCLE"),
            (10, "1.0"),
            (20, "2.0"),
            (40, "3.0"),
            (0, "ENDBLK"),
        ]);

        let bloco = leitura.get("COM_BASE").expect("lido");
        assert_eq!(bloco.base_point, Point2::new(100.0, 200.0));

        let Geometry::Circle(circulo) = &bloco.entities[0].geometry else {
            panic!("é círculo");
        };
        assert_eq!(circulo.center, Point2::new(1.0, 2.0));
    }

    #[test]
    fn referencia_externa_guarda_o_caminho() {
        let leitura = ler(&[
            (0, "BLOCK"),
            (2, "CARIMBO"),
            (70, "4"),
            (1, "../comum/carimbo.dwg"),
            (0, "ENDBLK"),
        ]);

        let bloco = leitura.get("CARIMBO").expect("lido");
        assert!(bloco.is_xref());
        assert_eq!(bloco.xref_path.as_deref(), Some("../comum/carimbo.dwg"));
    }

    #[test]
    fn endblk_ausente_nao_custa_o_bloco() {
        let leitura = ler(&[
            (0, "BLOCK"),
            (2, "SEM_FIM"),
            (0, "LINE"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
        ]);

        assert_eq!(leitura.blocks.len(), 1);
        assert_eq!(leitura.get("SEM_FIM").expect("lido").entities.len(), 1);
    }

    #[test]
    fn bloco_novo_fecha_o_anterior_sem_engoli_lo() {
        let leitura = ler(&[
            (0, "BLOCK"),
            (2, "PRIMEIRO"),
            (0, "LINE"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
            (0, "BLOCK"),
            (2, "SEGUNDO"),
            (0, "ENDBLK"),
        ]);

        assert_eq!(leitura.blocks.len(), 2);
        assert_eq!(leitura.get("PRIMEIRO").expect("lido").entities.len(), 1);
        assert!(leitura.get("SEGUNDO").expect("lido").entities.is_empty());
    }

    #[test]
    fn os_espacos_aparecem_como_blocos() {
        // Arquivo real declara os dois. Entregá-los como definições comuns é
        // fiel ao formato e ao ADR 0005, que modela layout como bloco.
        let leitura = ler(&[
            (0, "BLOCK"),
            (2, "*Model_Space"),
            (0, "ENDBLK"),
            (0, "BLOCK"),
            (2, "*Paper_Space"),
            (0, "ENDBLK"),
        ]);

        assert_eq!(leitura.blocks.len(), 2);
        assert!(leitura.get("*MODEL_SPACE").is_some());
        assert!(leitura.get("*Paper_Space").is_some());
    }

    #[test]
    fn polilinha_antiga_dentro_de_bloco_e_montada() {
        let leitura = ler(&[
            (0, "BLOCK"),
            (2, "COM_POLILINHA"),
            (0, "POLYLINE"),
            (66, "1"),
            (0, "VERTEX"),
            (10, "0.0"),
            (20, "0.0"),
            (0, "VERTEX"),
            (10, "3.0"),
            (20, "4.0"),
            (0, "SEQEND"),
            (0, "ENDBLK"),
        ]);

        let bloco = leitura.get("COM_POLILINHA").expect("lido");
        assert_eq!(bloco.entities.len(), 1);
        assert!(matches!(bloco.entities[0].geometry, Geometry::Polyline(_)));
    }

    #[test]
    fn tipo_nao_modelado_dentro_de_bloco_e_contado() {
        let leitura = ler(&[
            (0, "BLOCK"),
            (2, "UM"),
            (0, "HATCH"),
            (0, "ENDBLK"),
            (0, "BLOCK"),
            (2, "OUTRO"),
            (0, "HATCH"),
            (0, "ENDBLK"),
        ]);

        assert_eq!(leitura.unsupported.get("HATCH"), Some(&2));
        assert_eq!(leitura.entity_count(), 0);
    }

    #[test]
    fn camada_citada_dentro_de_bloco_e_criada() {
        let mut camadas = LayerTable::new();
        let leitura = read_blocks(
            &secao(&[
                (0, "BLOCK"),
                (2, "UM"),
                (0, "LINE"),
                (8, "Simbologia"),
                (10, "0.0"),
                (20, "0.0"),
                (11, "1.0"),
                (21, "1.0"),
                (0, "ENDBLK"),
            ]),
            &mut camadas,
        );

        assert_eq!(leitura.created_layers, ["Simbologia"]);
        assert!(camadas.get_by_name("Simbologia").is_some());
    }

    #[test]
    fn secao_que_nao_e_blocks_devolve_leitura_vazia() {
        let texto =
            "  0\nSECTION\n  2\nENTITIES\n  0\nBLOCK\n  2\nX\n  0\nENDBLK\n  0\nENDSEC\n  0\nEOF\n";
        let secao = sections(texto.as_bytes())
            .next()
            .expect("há seção")
            .expect("bem formada");
        let mut camadas = LayerTable::new();

        assert!(read_blocks(&secao, &mut camadas).blocks.is_empty());
    }

    #[test]
    fn fixture_que_o_upstream_nao_le_e_lida_aqui() {
        // O critério de aceite do MT-K2-05, e a razão de existir da fase K2.
        let leitura = ler_fixture("block-with-entities.dxf");

        assert_eq!(leitura.blocks.len(), 1);
        let bloco = leitura
            .get("MARCO")
            .expect("o bloco que derruba o upstream");
        assert_eq!(bloco.entities.len(), 1);
        assert!(matches!(bloco.entities[0].geometry, Geometry::Line(_)));
        assert!(leitura.rejected.is_empty());
    }

    #[test]
    fn fixture_sem_secao_blocks_nao_inventa_bloco() {
        let leitura = ler_fixture("block-reference.dxf");

        assert!(leitura.blocks.is_empty());
    }
}
