// Caminho relativo: kernel/neocad-io/src/dxf/writer/entities.rs
//! \file kernel/neocad-io/src/dxf/writer/entities.rs
//! \brief Escrita das entidades e das definições de bloco de um arquivo DXF.
//! \author Iago Leal
//! \date 2026-08-16

use neocad_geometry::{Aabb, Point2};
use neocad_model::{
    Color, Entity, EntityColor, Geometry, LayerTable, MODEL_SPACE_NAME as MODEL_SPACE,
};

use super::super::entities::{EntitySpace, DEFAULT_PAPER_SPACE};
use super::{DxfContents, Handles, Saida};

/// Bit do código `70` que marca a polilinha como fechada.
const POLILINHA_FECHADA: i64 = 1;

/// Escreve a seção `BLOCKS`.
///
/// # Os dois blocos de espaço saem sempre
///
/// O formato exige que `*Model_Space` e `*Paper_Space` existam, mesmo vazios: é
/// deles que as entidades das seções seguintes dependem para ter dono. Uma
/// definição vinda da leitura com um desses nomes **não** é descartada — suas
/// entidades saem dentro do bloco correspondente, para o conteúdo não sumir por
/// causa de um nome reservado.
pub(super) fn write_blocks(saida: &mut Saida, contents: &DxfContents<'_>, handles: &mut Handles) {
    saida.par(0, "SECTION");
    saida.par(2, "BLOCKS");

    for reservado in [MODEL_SPACE, DEFAULT_PAPER_SPACE] {
        let definicao = contents
            .blocks
            .iter()
            .find(|b| nome_igual(&b.name, reservado));

        write_block(
            saida,
            reservado,
            definicao.map_or(Point2::ORIGIN, |b| b.base_point),
            definicao.map_or(&[][..], |b| &b.entities),
            definicao.and_then(|b| b.xref_path.as_deref()),
            contents.layers,
            handles,
        );
    }

    // Os blocos das demais abas — `*Paper_Space0` em diante — saem antes dos
    // blocos comuns: sem eles, uma aba declarada na `OBJECTS` apontaria para um
    // registro que o arquivo não tem.
    for nome in contents
        .layouts
        .iter()
        .filter_map(|layout| layout.block_name.as_deref())
        .filter(|nome| !eh_espaco_reservado(nome))
    {
        write_block(
            saida,
            nome,
            Point2::ORIGIN,
            &[],
            None,
            contents.layers,
            handles,
        );
    }

    for bloco in contents
        .blocks
        .iter()
        .filter(|b| !eh_bloco_de_espaco(&b.name, contents))
    {
        write_block(
            saida,
            &bloco.name,
            bloco.base_point,
            &bloco.entities,
            bloco.xref_path.as_deref(),
            contents.layers,
            handles,
        );
    }

    saida.par(0, "ENDSEC");
}

/// Escreve a seção `ENTITIES`.
///
/// Sai na ordem em que as entidades chegam, que é a ordem de desenho — trocá-la
/// mudaria o que fica por cima do quê num desenho com hachura ou máscara.
pub(super) fn write_entities(saida: &mut Saida, contents: &DxfContents<'_>, handles: &mut Handles) {
    saida.par(0, "SECTION");
    saida.par(2, "ENTITIES");

    for lida in contents.entities {
        write_entity(
            saida,
            &lida.entity,
            Some(&lida.space),
            contents.layers,
            handles,
        );
    }

    saida.par(0, "ENDSEC");
}

/// Extensão do desenho no espaço-modelo, para `$EXTMIN`/`$EXTMAX`.
///
/// `None` quando não há entidade no espaço-modelo: declarar extensão de um
/// desenho vazio seria inventar um retângulo que ninguém desenhou.
pub(super) fn model_space_extents(contents: &DxfContents<'_>) -> Option<Aabb> {
    Aabb::union_all(
        contents
            .entities
            .iter()
            .filter(|lida| lida.space == EntitySpace::Model)
            .map(|lida| lida.entity.bounding_box()),
    )
}

/// Indica se o nome é de um dos dois blocos de espaço que saem sempre.
fn eh_espaco_reservado(nome: &str) -> bool {
    nome_igual(nome, MODEL_SPACE) || nome_igual(nome, DEFAULT_PAPER_SPACE)
}

/// Indica se o nome é de um bloco de espaço — inclusive o de uma aba.
///
/// Bloco de aba é gravado uma vez, junto com o layout que o declara. Um nome
/// anônimo como `*U1` **não** entra aqui: apesar do asterisco, ele é conteúdo de
/// verdade — hachura e cota vivem nesses blocos — e excluí-lo perderia desenho.
fn eh_bloco_de_espaco(nome: &str, contents: &DxfContents<'_>) -> bool {
    eh_espaco_reservado(nome)
        || contents
            .layouts
            .iter()
            .filter_map(|layout| layout.block_name.as_deref())
            .any(|bloco| nome_igual(bloco, nome))
}

/// Compara nomes como os formatos CAD comparam: ignorando caixa.
fn nome_igual(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Escreve uma definição de bloco com o que houver dentro.
fn write_block(
    saida: &mut Saida,
    nome: &str,
    base: Point2,
    entidades: &[Entity],
    xref: Option<&str>,
    layers: &LayerTable,
    handles: &mut Handles,
) {
    saida.par(0, "BLOCK");
    saida.par(5, &handles.proximo());
    saida.par(100, "AcDbEntity");
    saida.par(8, "0");
    saida.par(100, "AcDbBlockBegin");
    saida.par(2, nome);
    saida.inteiro(70, if xref.is_some() { 4 } else { 0 });
    saida.real(10, base.x);
    saida.real(20, base.y);
    saida.real(30, 0.0);
    saida.par(3, nome);
    // O caminho da referência externa sai sempre, ainda que vazio: o código `1`
    // é onde o formato o espera, e omiti-lo transformaria um xref em bloco
    // comum e vazio ao reabrir.
    saida.par(1, xref.unwrap_or(""));

    for entidade in entidades {
        write_entity(saida, entidade, None, layers, handles);
    }

    saida.par(0, "ENDBLK");
    saida.par(5, &handles.proximo());
    saida.par(100, "AcDbEntity");
    saida.par(8, "0");
    saida.par(100, "AcDbBlockEnd");
}

/// Escreve uma entidade.
///
/// `space` é `None` dentro de definição de bloco: ali a entidade pertence ao
/// bloco, e marcá-la como de espaço-papel a mandaria para dois donos.
///
/// # Limitação declarada da escrita de `VIEWPORT`
///
/// O recorte por entidade ([`neocad_model::ViewportClip::Boundary`]) **não é
/// gravado**. O código `340` exige o handle da **entidade** que delimita, e o que
/// a escrita recebe é uma lista de entidades sem identidade: não há como saber
/// qual delas o recorte aponta.
///
/// O congelamento por janela, que sofria do mesmo sintoma, foi resolvido no
/// MT-KL-12 — mas por uma razão que não vale para o recorte: camada tem **nome**,
/// e nome é chave estável entre a tabela e a janela. Entidade não tem. Fechar o
/// recorte exige a escrita passar a receber o documento, com identificadores, em
/// vez de uma lista solta.
///
/// A leitura conta as janelas recortadas em `DxfReport::clipped_viewports`, então
/// a diferença aparece em vez de acontecer em silêncio.
fn write_entity(
    saida: &mut Saida,
    entidade: &Entity,
    space: Option<&EntitySpace>,
    layers: &LayerTable,
    handles: &mut Handles,
) {
    let tipo = match &entidade.geometry {
        Geometry::Line(_) => "LINE",
        Geometry::Circle(_) => "CIRCLE",
        Geometry::Arc(_) => "ARC",
        Geometry::Polyline(_) => "LWPOLYLINE",
        Geometry::Text(_) => "TEXT",
        Geometry::Viewport(_) => "VIEWPORT",
    };

    saida.par(0, tipo);
    saida.par(5, &handles.proximo());
    saida.par(100, "AcDbEntity");

    if let Some(EntitySpace::Paper(aba)) = space {
        saida.inteiro(67, 1);
        saida.par(410, aba);
    }

    saida.par(8, nome_da_camada(entidade, layers));
    escrever_cor(saida, entidade.color);

    match &entidade.geometry {
        Geometry::Line(line) => {
            saida.par(100, "AcDbLine");
            escrever_ponto(saida, 10, line.start);
            escrever_ponto(saida, 11, line.end);
        }
        Geometry::Circle(circle) => {
            saida.par(100, "AcDbCircle");
            escrever_ponto(saida, 10, circle.center);
            saida.real(40, circle.radius);
        }
        Geometry::Arc(arc) => {
            saida.par(100, "AcDbCircle");
            escrever_ponto(saida, 10, arc.center);
            saida.real(40, arc.radius);
            saida.par(100, "AcDbArc");
            // O modelo guarda radianos e o formato grava graus. A conversão é
            // exata só até o último bit; ver a perda declarada no MT-K2-09.
            saida.real(50, arc.start_angle.to_degrees());
            saida.real(51, arc.end_angle.to_degrees());
        }
        Geometry::Polyline(polyline) => {
            saida.par(100, "AcDbPolyline");
            saida.inteiro(
                90,
                i64::try_from(polyline.vertices.len()).unwrap_or(i64::MAX),
            );
            saida.inteiro(
                70,
                if polyline.closed {
                    POLILINHA_FECHADA
                } else {
                    0
                },
            );

            for vertice in &polyline.vertices {
                saida.real(10, vertice.x);
                saida.real(20, vertice.y);
            }
        }
        Geometry::Text(text) => {
            saida.par(100, "AcDbText");
            escrever_ponto(saida, 10, text.position);
            saida.real(40, text.height);
            saida.par(1, &text.content);

            if text.rotation != 0.0 {
                saida.real(50, text.rotation.to_degrees());
            }

            // O segundo marcador é o que o formato pede para `TEXT`, e a sua
            // ausência faz leitor estrito tratar a entidade como incompleta.
            saida.par(100, "AcDbText");
        }
        Geometry::Viewport(viewport) => {
            saida.par(100, "AcDbViewport");
            escrever_ponto(saida, 10, viewport.center);
            saida.real(40, viewport.width);
            saida.real(41, viewport.height);
            // O código `68` é o que liga e desliga a janela: zero é desligada,
            // positivo é a ordem de empilhamento. Não há sinalizador separado.
            saida.inteiro(68, i64::from(viewport.is_on));
            saida.real(12, viewport.view_center.x);
            saida.real(22, viewport.view_center.y);
            saida.real(45, viewport.view_height);
            saida.real(51, viewport.twist.to_degrees());

            // O `331` aponta a camada congelada **por handle**, e é por isso que
            // a tabela de camadas registra o seu ao ser gravada. A camada cujo
            // handle não existe é omitida: apontar para nada faria o leitor
            // congelar camada errada, que é pior do que não congelar.
            for congelada in &viewport.frozen_layers {
                let handle = layers
                    .get(*congelada)
                    .and_then(|camada| handles.camada(camada.name()))
                    .map(str::to_owned);

                if let Some(handle) = handle {
                    saida.par(331, &handle);
                }
            }
        }
    }
}

/// Escreve um ponto nos três códigos consecutivos a partir de `base`.
fn escrever_ponto(saida: &mut Saida, base: u16, ponto: Point2) {
    saida.real(base, ponto.x);
    saida.real(base + 10, ponto.y);
    saida.real(base + 20, 0.0);
}

/// Nome da camada da entidade, com a camada `0` quando o identificador não
/// resolve.
///
/// Identificador obsoleto aqui significa entidade órfã, que não deveria existir;
/// mandá-la para a camada `0` a preserva no arquivo em vez de descartá-la.
fn nome_da_camada<'a>(entidade: &Entity, layers: &'a LayerTable) -> &'a str {
    layers
        .get(entidade.layer)
        .map_or("0", |camada| camada.name())
}

/// Escreve a cor da entidade.
///
/// Herdar da camada é o padrão do formato e é gravado por **omissão**, como o
/// AutoCAD faz: um `62` em toda entidade engordaria o arquivo sem dizer nada.
fn escrever_cor(saida: &mut Saida, color: EntityColor) {
    const PADRAO: i64 = 7;

    match color {
        EntityColor::ByLayer => {}
        EntityColor::ByBlock => saida.inteiro(62, 0),
        EntityColor::Explicit(Color::ByLayer) => saida.inteiro(62, 256),
        EntityColor::Explicit(Color::ByBlock) => saida.inteiro(62, 0),
        EntityColor::Explicit(Color::Index(indice)) => saida.inteiro(62, i64::from(indice)),
        EntityColor::Explicit(Color::Rgb { red, green, blue }) => {
            // O `62` de companhia serve a quem não lê `420`; ver a aproximação
            // declarada na escrita das camadas.
            saida.inteiro(62, PADRAO);
            saida.inteiro(
                420,
                (i64::from(red) << 16) | (i64::from(green) << 8) | i64::from(blue),
            );
        }
    }
}

/// Nomes dos registros de bloco que a tabela `BLOCK_RECORD` precisa declarar.
///
/// Os dois espaços vêm primeiro e sempre; os demais na ordem em que aparecem.
pub(super) fn block_record_names<'a>(contents: &DxfContents<'a>) -> Vec<&'a str> {
    let mut nomes = vec![MODEL_SPACE, DEFAULT_PAPER_SPACE];

    // Os blocos das abas vêm antes dos comuns, na mesma ordem em que a seção
    // `BLOCKS` os grava — a tabela e a seção precisam concordar.
    nomes.extend(
        contents
            .layouts
            .iter()
            .filter_map(|layout| layout.block_name.as_deref())
            .filter(|nome| !eh_espaco_reservado(nome)),
    );

    nomes.extend(
        contents
            .blocks
            .iter()
            .filter(|b| !eh_bloco_de_espaco(&b.name, contents))
            .map(|b| b.name.as_str()),
    );

    // Bloco declarado por duas vias — uma aba e a seção `BLOCKS` — sairia
    // duplicado, e handle repetido invalida o arquivo.
    nomes.dedup_by(|a, b| nome_igual(a, b));

    nomes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{read_dxf, write_dxf, BlockDefinition, DxfContents, ReadEntity};
    use neocad_model::{Arc, Circle, Line, Polyline, Text};

    /// Monta uma tabela com a camada informada e devolve a entidade nela.
    fn entidade(layers: &mut LayerTable, camada: &str, geometry: Geometry) -> Entity {
        let id = layers
            .id_of(camada)
            .unwrap_or_else(|| layers.create(camada).expect("nome válido"));

        Entity::new(id, geometry)
    }

    /// Grava e relê, devolvendo as entidades do espaço-modelo.
    fn ida_e_volta(layers: &LayerTable, entidades: &[ReadEntity]) -> Vec<ReadEntity> {
        let bytes = write_dxf(&DxfContents {
            layers,
            entities: entidades,
            blocks: &[],
            layouts: &[],
        });

        read_dxf(&bytes).entities
    }

    fn no_modelo(entity: Entity) -> ReadEntity {
        ReadEntity {
            space: EntitySpace::Model,
            entity,
        }
    }

    #[test]
    fn reta_sobrevive_a_ida_e_volta() {
        let mut camadas = LayerTable::new();
        let reta = entidade(
            &mut camadas,
            "Eixos",
            Geometry::Line(Line {
                start: Point2::new(1.5, -2.5),
                end: Point2::new(30.25, 40.75),
            }),
        );

        let relidas = ida_e_volta(&camadas, &[no_modelo(reta.clone())]);

        assert_eq!(relidas.len(), 1);
        assert_eq!(relidas[0].entity.geometry, reta.geometry);
    }

    #[test]
    fn circunferencia_sobrevive() {
        let mut camadas = LayerTable::new();
        let circulo = entidade(
            &mut camadas,
            "0",
            Geometry::Circle(Circle {
                center: Point2::new(10.0, 20.0),
                radius: 2.5,
            }),
        );

        let relidas = ida_e_volta(&camadas, &[no_modelo(circulo.clone())]);

        assert_eq!(relidas[0].entity.geometry, circulo.geometry);
    }

    #[test]
    fn arco_sobrevive_a_conversao_de_angulo() {
        // Graus no arquivo, radianos no modelo. Os ângulos cardeais precisam
        // voltar exatos; os demais têm perda de último bit, declarada.
        let mut camadas = LayerTable::new();
        let arco = entidade(
            &mut camadas,
            "0",
            Geometry::Arc(Arc {
                center: Point2::new(1.0, 2.0),
                radius: 3.0,
                start_angle: 0.0,
                end_angle: core::f64::consts::FRAC_PI_2,
            }),
        );

        let relidas = ida_e_volta(&camadas, &[no_modelo(arco)]);
        let Geometry::Arc(relido) = &relidas[0].entity.geometry else {
            panic!("é arco");
        };

        assert_eq!(relido.start_angle, 0.0);
        assert!((relido.end_angle - core::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }

    #[test]
    fn polilinha_sobrevive_com_vertices_e_fechamento() {
        let mut camadas = LayerTable::new();
        let polilinha = entidade(
            &mut camadas,
            "0",
            Geometry::Polyline(Polyline {
                vertices: vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(5.0, 0.0),
                    Point2::new(5.0, 5.0),
                ],
                closed: true,
            }),
        );

        let relidas = ida_e_volta(&camadas, &[no_modelo(polilinha.clone())]);

        assert_eq!(relidas[0].entity.geometry, polilinha.geometry);
    }

    #[test]
    fn texto_sobrevive_com_conteudo_acentuado() {
        let mut camadas = LayerTable::new();
        let texto = entidade(
            &mut camadas,
            "0",
            Geometry::Text(Text {
                position: Point2::new(1.0, 2.0),
                content: String::from("Fiação — 3ª etapa"),
                height: 2.5,
                rotation: 0.0,
            }),
        );

        let relidas = ida_e_volta(&camadas, &[no_modelo(texto.clone())]);

        assert_eq!(relidas[0].entity.geometry, texto.geometry);
    }

    #[test]
    fn a_camada_da_entidade_sobrevive() {
        let mut camadas = LayerTable::new();
        let reta = entidade(
            &mut camadas,
            "Cotas Elétricas",
            Geometry::Line(Line {
                start: Point2::ORIGIN,
                end: Point2::new(1.0, 1.0),
            }),
        );

        let bytes = write_dxf(&DxfContents {
            layers: &camadas,
            entities: &[no_modelo(reta)],
            blocks: &[],
            layouts: &[],
        });
        let leitura = read_dxf(&bytes);

        let id = leitura
            .layers
            .id_of("Cotas Elétricas")
            .expect("camada relida");
        assert_eq!(leitura.entities[0].entity.layer, id);
        // Veio da tabela, não foi inventada por citação.
        assert!(leitura.report.created_layers.is_empty());
    }

    #[test]
    fn cor_da_entidade_sobrevive_nas_quatro_formas() {
        let mut camadas = LayerTable::new();
        let cores = [
            EntityColor::ByLayer,
            EntityColor::ByBlock,
            EntityColor::Explicit(Color::Index(3)),
            EntityColor::Explicit(Color::Rgb {
                red: 0x33,
                green: 0x66,
                blue: 0x99,
            }),
        ];

        let entidades: Vec<ReadEntity> = cores
            .iter()
            .map(|cor| {
                let mut entidade = entidade(
                    &mut camadas,
                    "0",
                    Geometry::Circle(Circle {
                        center: Point2::ORIGIN,
                        radius: 1.0,
                    }),
                );
                entidade.color = *cor;

                no_modelo(entidade)
            })
            .collect();

        let relidas = ida_e_volta(&camadas, &entidades);

        assert_eq!(
            relidas.iter().map(|r| r.entity.color).collect::<Vec<_>>(),
            cores
        );
    }

    #[test]
    fn entidade_de_espaco_papel_volta_no_mesmo_layout() {
        let mut camadas = LayerTable::new();
        let reta = entidade(
            &mut camadas,
            "0",
            Geometry::Line(Line {
                start: Point2::ORIGIN,
                end: Point2::new(1.0, 1.0),
            }),
        );

        let relidas = ida_e_volta(
            &camadas,
            &[ReadEntity {
                space: EntitySpace::Paper(String::from("Prancha A1")),
                entity: reta,
            }],
        );

        assert_eq!(
            relidas[0].space,
            EntitySpace::Paper(String::from("Prancha A1"))
        );
    }

    #[test]
    fn a_ordem_de_desenho_e_preservada() {
        // Trocar a ordem muda o que fica por cima em desenho com hachura.
        let mut camadas = LayerTable::new();
        let entidades: Vec<ReadEntity> = (0..5)
            .map(|indice| {
                no_modelo(entidade(
                    &mut camadas,
                    "0",
                    Geometry::Circle(Circle {
                        center: Point2::new(f64::from(indice), 0.0),
                        radius: 1.0,
                    }),
                ))
            })
            .collect();

        let relidas = ida_e_volta(&camadas, &entidades);

        let centros: Vec<f64> = relidas
            .iter()
            .map(|r| match &r.entity.geometry {
                Geometry::Circle(c) => c.center.x,
                _ => panic!("é círculo"),
            })
            .collect();
        assert_eq!(centros, [0.0, 1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn bloco_com_entidades_sobrevive() {
        let mut camadas = LayerTable::new();
        let dentro = entidade(
            &mut camadas,
            "Simbologia",
            Geometry::Line(Line {
                start: Point2::ORIGIN,
                end: Point2::new(5.0, 5.0),
            }),
        );
        let blocos = [BlockDefinition {
            name: String::from("MARCO"),
            base_point: Point2::new(1.0, 2.0),
            entities: vec![dentro.clone()],
            xref_path: None,
        }];

        let bytes = write_dxf(&DxfContents {
            layers: &camadas,
            entities: &[],
            blocks: &blocos,
            layouts: &[],
        });
        let leitura = read_dxf(&bytes);

        let bloco = leitura.blocks.iter().find(|b| b.name == "MARCO");
        let bloco = bloco.expect("bloco relido");
        assert_eq!(bloco.base_point, Point2::new(1.0, 2.0));
        assert_eq!(bloco.entities.len(), 1);
        assert_eq!(bloco.entities[0].geometry, dentro.geometry);
    }

    #[test]
    fn referencia_externa_sobrevive_com_o_caminho() {
        let camadas = LayerTable::new();
        let blocos = [BlockDefinition {
            name: String::from("CARIMBO"),
            base_point: Point2::ORIGIN,
            entities: Vec::new(),
            xref_path: Some(String::from("../comum/carimbo.dwg")),
        }];

        let bytes = write_dxf(&DxfContents {
            layers: &camadas,
            entities: &[],
            blocks: &blocos,
            layouts: &[],
        });
        let leitura = read_dxf(&bytes);

        let bloco = leitura
            .blocks
            .iter()
            .find(|b| b.name == "CARIMBO")
            .expect("relido");
        assert!(bloco.is_xref());
        assert_eq!(bloco.xref_path.as_deref(), Some("../comum/carimbo.dwg"));
    }

    #[test]
    fn os_blocos_de_espaco_saem_sempre_e_sem_duplicar() {
        let camadas = LayerTable::new();
        let blocos = [BlockDefinition {
            name: String::from("*Model_Space"),
            base_point: Point2::ORIGIN,
            entities: Vec::new(),
            xref_path: None,
        }];

        let bytes = write_dxf(&DxfContents {
            layers: &camadas,
            entities: &[],
            blocks: &blocos,
            layouts: &[],
        });
        let leitura = read_dxf(&bytes);

        let espacos = leitura
            .blocks
            .iter()
            .filter(|b| eh_espaco_reservado(&b.name))
            .count();
        assert_eq!(espacos, 2);
    }

    #[test]
    fn conteudo_de_bloco_reservado_nao_se_perde() {
        // Nome reservado não é motivo para descartar o que está dentro.
        let mut camadas = LayerTable::new();
        let dentro = entidade(
            &mut camadas,
            "0",
            Geometry::Circle(Circle {
                center: Point2::ORIGIN,
                radius: 1.0,
            }),
        );
        let blocos = [BlockDefinition {
            name: String::from("*Model_Space"),
            base_point: Point2::ORIGIN,
            entities: vec![dentro],
            xref_path: None,
        }];

        let bytes = write_dxf(&DxfContents {
            layers: &camadas,
            entities: &[],
            blocks: &blocos,
            layouts: &[],
        });
        let leitura = read_dxf(&bytes);

        let modelo = leitura
            .blocks
            .iter()
            .find(|b| nome_igual(&b.name, MODEL_SPACE))
            .expect("relido");
        assert_eq!(modelo.entities.len(), 1);
    }

    #[test]
    fn a_saida_com_entidades_e_deterministica() {
        let mut camadas = LayerTable::new();
        let entidades: Vec<ReadEntity> = (0..3)
            .map(|indice| {
                no_modelo(entidade(
                    &mut camadas,
                    "0",
                    Geometry::Circle(Circle {
                        center: Point2::new(f64::from(indice), 0.0),
                        radius: 1.0,
                    }),
                ))
            })
            .collect();
        let blocos = [BlockDefinition {
            name: String::from("MARCO"),
            base_point: Point2::ORIGIN,
            entities: Vec::new(),
            xref_path: None,
        }];

        let conteudo = DxfContents {
            layers: &camadas,
            entities: &entidades,
            blocks: &blocos,
            layouts: &[],
        };

        assert_eq!(write_dxf(&conteudo), write_dxf(&conteudo));
    }

    #[test]
    fn a_extensao_do_desenho_cobre_o_espaco_modelo() {
        let mut camadas = LayerTable::new();
        let entidades = [
            no_modelo(entidade(
                &mut camadas,
                "0",
                Geometry::Line(Line {
                    start: Point2::new(-5.0, -3.0),
                    end: Point2::new(1.0, 1.0),
                }),
            )),
            // A do papel não entra: a extensão é do desenho, não da prancha.
            ReadEntity {
                space: EntitySpace::Paper(String::from("Prancha")),
                entity: entidade(
                    &mut camadas,
                    "0",
                    Geometry::Line(Line {
                        start: Point2::new(500.0, 500.0),
                        end: Point2::new(900.0, 900.0),
                    }),
                ),
            },
        ];

        let extensao = model_space_extents(&DxfContents {
            layers: &camadas,
            entities: &entidades,
            blocks: &[],
            layouts: &[],
        })
        .expect("há entidade no modelo");

        assert_eq!(extensao.min(), Point2::new(-5.0, -3.0));
        assert_eq!(extensao.max(), Point2::new(1.0, 1.0));
    }

    #[test]
    fn desenho_sem_entidade_no_modelo_nao_tem_extensao() {
        let camadas = LayerTable::new();

        assert!(model_space_extents(&DxfContents {
            layers: &camadas,
            entities: &[],
            blocks: &[],
            layouts: &[],
        })
        .is_none());
    }

    #[test]
    fn a_fixture_que_o_upstream_nao_abre_sobrevive_a_regravacao() {
        // Ler, gravar e reler o arquivo que motivou a fase inteira.
        let caminho = format!(
            "{}/../../e2e/fixtures/block-with-entities.dxf",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(&caminho).expect("fixture existe");
        let original = read_dxf(&bytes);

        let regravado = write_dxf(&DxfContents {
            layers: &original.layers,
            entities: &original.entities,
            blocks: &original.blocks,
            layouts: &[],
        });
        let relido = read_dxf(&regravado);

        assert_eq!(relido.model_space_count(), original.model_space_count());
        assert!(relido.report.is_clean());

        let marco = relido
            .blocks
            .iter()
            .find(|b| b.name == "MARCO")
            .expect("o bloco atravessou");
        assert_eq!(marco.entities.len(), 1);

        // E a segunda regravação é idêntica à primeira.
        assert_eq!(
            regravado,
            write_dxf(&DxfContents {
                layers: &relido.layers,
                entities: &relido.entities,
                blocks: &relido.blocks,
                layouts: &[],
            })
        );
    }

    #[test]
    fn nenhum_tipo_escrito_volta_como_nao_representado() {
        // Se a escrita emitir uma forma que a leitura não reconhece, o relatório
        // acusa — é o que impede os dois lados de divergirem em silêncio.
        let mut camadas = LayerTable::new();
        let entidades = [
            no_modelo(entidade(
                &mut camadas,
                "0",
                Geometry::Line(Line {
                    start: Point2::ORIGIN,
                    end: Point2::new(1.0, 1.0),
                }),
            )),
            no_modelo(entidade(
                &mut camadas,
                "0",
                Geometry::Circle(Circle {
                    center: Point2::ORIGIN,
                    radius: 1.0,
                }),
            )),
            no_modelo(entidade(
                &mut camadas,
                "0",
                Geometry::Arc(Arc {
                    center: Point2::ORIGIN,
                    radius: 1.0,
                    start_angle: 0.0,
                    end_angle: 1.0,
                }),
            )),
            no_modelo(entidade(
                &mut camadas,
                "0",
                Geometry::Polyline(Polyline {
                    vertices: vec![Point2::ORIGIN, Point2::new(1.0, 1.0)],
                    closed: false,
                }),
            )),
            no_modelo(entidade(
                &mut camadas,
                "0",
                Geometry::Text(Text {
                    position: Point2::ORIGIN,
                    content: String::from("t"),
                    height: 1.0,
                    rotation: 0.5,
                }),
            )),
        ];

        let bytes = write_dxf(&DxfContents {
            layers: &camadas,
            entities: &entidades,
            blocks: &[],
            layouts: &[],
        });
        let leitura = read_dxf(&bytes);

        assert!(
            leitura.report.is_clean(),
            "relatório sujo: {:?}",
            leitura.report
        );
        assert_eq!(leitura.entities.len(), 5);
    }
}
