// Caminho relativo: kernel/neocad-wasm/src/tela.rs
//! \file kernel/neocad-wasm/src/tela.rs
//! \brief Geometria pronta para a tela: blocos de segmentos e índice espacial.
//! \author Iago Leal
//! \date 2026-10-31
//!
//! O ADR 0008 deixa no kernel tudo o que é cálculo da tela; o TypeScript só
//! repassa os blocos à GPU com a matriz da vista. Este módulo transforma as
//! entidades de cada espaço (modelo e cada layout) em blocos contínuos de
//! segmentos `x1, y1, x2, y2` em `f32`, um por camada, e mantém o índice
//! espacial do recorte pela vista.
//!
//! # Por que uma origem por espaço
//!
//! `f32` tem 24 bits de mantissa: numa coordenada UTM (milhões de metros) o
//! passo entre dois valores representáveis passa de meio metro. Os segmentos
//! saem relativos a uma origem própria do espaço, em `f64`, fixada ao preparar;
//! a vista soma essa origem na matriz.
//!
//! # Como acompanha as edições
//!
//! Cada operação informa o que mudou ([`Effect`]) e a pilha conta as operações
//! ([`CommandStack::revision`]). Se a tela recebe a revisão seguinte à que
//! conhece, refaz só as entidades informadas; se perdeu alguma no caminho,
//! refaz tudo. Assim nenhuma sequência de chamadas a deixa velha.
//!
//! [`CommandStack::revision`]: neocad_transaction::CommandStack::revision

use std::collections::{BTreeMap, BTreeSet};

use neocad_geometry::{tessellate, Aabb, AnyCurve, Point2, SpatialIndex};
use neocad_model::{BlockId, Document, Entity, EntityId, Geometry, LayerId, LayerRecord};
use neocad_transaction::Effect;

/// Um bloco de segmentos que precisa ser enviado de novo à GPU.
pub type DirtyBlock = (String, LayerId);

/// Geometria de um espaço: segmentos por camada e por entidade.
#[derive(Debug)]
struct SpaceScreen {
    name: String,
    block: BlockId,
    origin: Point2,
    /// Segmentos de cada entidade, agrupados por camada.
    layers: BTreeMap<LayerId, BTreeMap<EntityId, Vec<f32>>>,
    /// Índice do recorte pela vista; refeito na primeira consulta após edição.
    index: Option<SpatialIndex<EntityId>>,
}

/// Geometria de todos os espaços, pronta para a tela.
#[derive(Debug)]
pub struct Screen {
    chord_error: f64,
    /// Revisão da pilha que a tela reflete.
    revision: u64,
    spaces: Vec<SpaceScreen>,
    /// Onde estão os segmentos de cada entidade: espaço e camada.
    place: BTreeMap<EntityId, (usize, LayerId)>,
}

/// Segmentos `x1, y1, x2, y2` da entidade, relativos à origem.
///
/// Texto e janela de layout não viram segmentos aqui: o texto tem caminho
/// próprio (MT-KV-11) e a janela é moldura de outra vista.
fn segments(entity: &Entity, origin: Point2, chord_error: f64) -> Option<Vec<f32>> {
    let curve = match &entity.geometry {
        Geometry::Line(line) => AnyCurve::Line(*line),
        Geometry::Circle(circle) => AnyCurve::Circle(*circle),
        Geometry::Arc(arc) => AnyCurve::Arc(*arc),
        Geometry::Polyline(polyline) => AnyCurve::Polyline(polyline.clone()),
        Geometry::Text(_) | Geometry::Viewport(_) => return None,
    };
    let points = tessellate(&curve, chord_error)?;

    // A subtração é feita em f64; só a diferença, pequena, vira f32.
    #[allow(clippy::cast_possible_truncation)]
    let relative = |p: Point2| [(p.x - origin.x) as f32, (p.y - origin.y) as f32];
    let mut out = Vec::with_capacity(points.len().saturating_sub(1) * 4);
    for pair in points.windows(2) {
        out.extend(relative(pair[0]));
        out.extend(relative(pair[1]));
    }

    Some(out)
}

impl Screen {
    /// Prepara todos os espaços com o erro de corda informado.
    ///
    /// Devolve `None` se o erro de corda não for finito e positivo.
    #[must_use]
    pub fn prepare(document: &Document, revision: u64, chord_error: f64) -> Option<Self> {
        if !(chord_error.is_finite() && chord_error > 0.0) {
            return None;
        }

        let mut screen = Self {
            chord_error,
            revision,
            spaces: Vec::new(),
            place: BTreeMap::new(),
        };
        screen.fill(document);

        Some(screen)
    }

    /// Refaz todos os espaços a partir do documento.
    fn fill(&mut self, document: &Document) {
        let layouts = document.layouts();
        let model = layouts.model_layout();
        let ordered = layouts
            .get(model)
            .map(|record| (model, record))
            .into_iter()
            .chain(
                layouts
                    .in_tab_order()
                    .into_iter()
                    .filter(|(id, _)| *id != model),
            );

        self.spaces = ordered
            .map(|(_, record)| SpaceScreen {
                name: record.name().to_owned(),
                block: record.block(),
                origin: document
                    .block_bounding_box(record.block())
                    .map_or(Point2::ORIGIN, Aabb::center),
                layers: BTreeMap::new(),
                index: None,
            })
            .collect();
        self.place.clear();

        for space in 0..self.spaces.len() {
            let block = self.spaces[space].block;
            for (id, entity) in document.entities_in_block(block) {
                self.add(space, id, entity);
            }
        }
    }

    /// Guarda os segmentos da entidade no espaço; devolve a camada.
    fn add(&mut self, space: usize, id: EntityId, entity: &Entity) -> LayerId {
        let target = &mut self.spaces[space];
        if let Some(segments) = segments(entity, target.origin, self.chord_error) {
            target
                .layers
                .entry(entity.layer)
                .or_default()
                .insert(id, segments);
            self.place.insert(id, (space, entity.layer));
        }
        target.index = None;

        entity.layer
    }

    /// Tira os segmentos da entidade; devolve onde estavam.
    fn remove(&mut self, id: EntityId) -> Option<(usize, LayerId)> {
        let (space, layer) = self.place.remove(&id)?;
        let target = &mut self.spaces[space];
        if let Some(entities) = target.layers.get_mut(&layer) {
            entities.remove(&id);
            if entities.is_empty() {
                target.layers.remove(&layer);
            }
        }
        target.index = None;

        Some((space, layer))
    }

    /// Todos os blocos que existem agora.
    fn all_blocks(&self) -> BTreeSet<(usize, LayerId)> {
        self.spaces
            .iter()
            .enumerate()
            .flat_map(|(space, s)| s.layers.keys().map(move |layer| (space, *layer)))
            .collect()
    }

    fn space(&self, name: &str) -> Option<usize> {
        self.spaces.iter().position(|space| space.name == name)
    }

    /// Nomes dos espaços, o modelo primeiro e os layouts na ordem das abas.
    #[must_use]
    pub fn spaces(&self) -> Vec<String> {
        self.spaces.iter().map(|space| space.name.clone()).collect()
    }

    /// Origem dos segmentos do espaço.
    #[must_use]
    pub fn origin(&self, space: &str) -> Option<Point2> {
        Some(self.spaces[self.space(space)?].origin)
    }

    /// Camadas visíveis que têm segmentos no espaço, em ordem crescente.
    #[must_use]
    pub fn layers(&self, document: &Document, space: &str) -> Option<Vec<LayerId>> {
        let space = &self.spaces[self.space(space)?];

        Some(
            space
                .layers
                .keys()
                .copied()
                .filter(|layer| {
                    document
                        .layers()
                        .get(*layer)
                        .is_some_and(LayerRecord::is_visible)
                })
                .collect(),
        )
    }

    /// Segmentos da camada no espaço, relativos à origem do espaço.
    ///
    /// Camada sem segmentos no espaço dá bloco vazio.
    #[must_use]
    pub fn block(&self, space: &str, layer: LayerId) -> Option<Vec<f32>> {
        let space = &self.spaces[self.space(space)?];

        Some(
            space
                .layers
                .get(&layer)
                .map(|entities| entities.values().flatten().copied().collect())
                .unwrap_or_default(),
        )
    }

    /// Acompanha a operação de revisão `revision`, que teve o efeito informado.
    ///
    /// Devolve os blocos que mudaram, em ordem.
    pub fn update(
        &mut self,
        document: &Document,
        revision: u64,
        effect: &Effect,
    ) -> Vec<DirtyBlock> {
        if revision == self.revision {
            return Vec::new();
        }

        let mut dirty: BTreeSet<(usize, LayerId)> = BTreeSet::new();

        if revision == self.revision + 1 {
            for id in effect.removed.iter().chain(&effect.modified) {
                dirty.extend(self.remove(*id));
            }
            for id in effect.created.iter().chain(&effect.modified) {
                let Some(entity) = document.entity(*id) else {
                    continue;
                };
                let Some(placement) = document.entity_placement(*id) else {
                    continue;
                };
                // Entidade de definição de bloco não está em nenhum espaço.
                if let Some(space) = self.spaces.iter().position(|s| s.block == placement.block) {
                    let layer = self.add(space, *id, entity);
                    dirty.insert((space, layer));
                }
            }
            for layer in &effect.layers {
                for (space, s) in self.spaces.iter().enumerate() {
                    if s.layers.contains_key(layer) {
                        dirty.insert((space, *layer));
                    }
                }
            }
        } else {
            // Alguma operação passou sem ser vista: refaz tudo.
            dirty = self.all_blocks();
            self.fill(document);
            dirty.extend(self.all_blocks());
        }
        self.revision = revision;

        dirty
            .into_iter()
            .map(|(space, layer)| (self.spaces[space].name.clone(), layer))
            .collect()
    }

    /// Entidades do espaço cuja caixa envolvente toca a área.
    ///
    /// São candidatas: o traço pode passar fora da área mesmo com a caixa dentro.
    pub fn query(&mut self, document: &Document, space: &str, area: Aabb) -> Option<Vec<EntityId>> {
        let found = self.space(space)?;
        let space = &mut self.spaces[found];
        let block = space.block;
        let index = space.index.get_or_insert_with(|| {
            SpatialIndex::build(
                document
                    .entities_in_block(block)
                    .map(|(id, entity)| (entity.bounding_box(), id)),
            )
        });

        Some(index.query_box(area))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neocad_geometry::{Arc, Circle, Line};
    use neocad_model::{Text, MODEL_LAYOUT_NAME};
    use neocad_transaction::CommandStack;

    const ERRO: f64 = 1e-3;

    fn reta(x1: f64, y1: f64, x2: f64, y2: f64) -> Geometry {
        Geometry::Line(Line {
            start: Point2::new(x1, y1),
            end: Point2::new(x2, y2),
        })
    }

    /// Insere no espaço-modelo pela pilha, como faria um comando.
    fn insere(
        doc: &mut Document,
        pilha: &mut CommandStack,
        layer: LayerId,
        g: Geometry,
    ) -> EntityId {
        pilha
            .edit(doc, "Inserir", |editor| {
                editor.insert_in_model_space(Entity::new(layer, g))
            })
            .expect("insere")
    }

    /// Volta os segmentos do bloco às coordenadas do desenho.
    fn absolutos(tela: &Screen, espaco: &str, camada: LayerId) -> Vec<[f64; 4]> {
        let origem = tela.origin(espaco).expect("origem");
        let bloco = tela.block(espaco, camada).expect("bloco");
        assert_eq!(bloco.len() % 4, 0, "bloco com {} floats", bloco.len());
        bloco
            .chunks(4)
            .map(|s| {
                [
                    origem.x + f64::from(s[0]),
                    origem.y + f64::from(s[1]),
                    origem.x + f64::from(s[2]),
                    origem.y + f64::from(s[3]),
                ]
            })
            .collect()
    }

    fn perto(a: [f64; 4], b: [f64; 4], tol: f64) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() <= tol)
    }

    #[test]
    fn a_reta_do_modelo_vira_um_segmento_no_bloco_da_camada() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        insere(&mut doc, &mut pilha, zero, reta(1.0, 2.0, 3.0, 5.0));

        let tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        assert_eq!(
            tela.spaces().first().map(String::as_str),
            Some(MODEL_LAYOUT_NAME)
        );
        let segmentos = absolutos(&tela, MODEL_LAYOUT_NAME, zero);
        assert_eq!(segmentos.len(), 1);
        assert!(
            perto(segmentos[0], [1.0, 2.0, 3.0, 5.0], 1e-6),
            "{:?}",
            segmentos[0]
        );
    }

    #[test]
    fn o_circulo_vira_cordas_dentro_do_erro() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let centro = Point2::new(10.0, 10.0);
        insere(
            &mut doc,
            &mut pilha,
            zero,
            Geometry::Circle(Circle {
                center: centro,
                radius: 2.0,
            }),
        );

        let tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        let segmentos = absolutos(&tela, MODEL_LAYOUT_NAME, zero);
        assert!(segmentos.len() >= 3, "{} cordas", segmentos.len());
        for s in &segmentos {
            let meio = Point2::new((s[0] + s[2]) / 2.0, (s[1] + s[3]) / 2.0);
            assert!(
                2.0 - centro.distance_to(meio) <= ERRO + 1e-5,
                "corda fora do erro: {s:?}"
            );
        }
        // As cordas se encadeiam: o fim de uma é o começo da seguinte.
        for par in segmentos.windows(2) {
            assert!((par[0][2] - par[1][0]).abs() < 1e-5 && (par[0][3] - par[1][1]).abs() < 1e-5);
        }
    }

    #[test]
    fn longe_da_origem_os_segmentos_nao_perdem_precisao() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let (x, y) = (5_000_000.0, 7_500_000.0);
        insere(
            &mut doc,
            &mut pilha,
            zero,
            reta(x + 1.25, y + 3.5, x + 2.75, y + 3.5),
        );
        insere(
            &mut doc,
            &mut pilha,
            zero,
            reta(x + 100.0, y + 100.0, x + 101.0, y + 100.0),
        );

        let tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        let segmentos = absolutos(&tela, MODEL_LAYOUT_NAME, zero);
        // Em f32 puro, 5 000 001,25 vira 5 000 001,0: erro de 25 cm.
        assert!(
            perto(segmentos[0], [x + 1.25, y + 3.5, x + 2.75, y + 3.5], 1e-4),
            "{:?}",
            segmentos[0]
        );
    }

    #[test]
    fn cada_camada_tem_o_seu_bloco_e_a_desligada_nao_aparece() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let paredes = doc.create_layer("Paredes").expect("camada");
        insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 0.0));
        insere(&mut doc, &mut pilha, paredes, reta(0.0, 1.0, 1.0, 1.0));
        insere(&mut doc, &mut pilha, paredes, reta(0.0, 2.0, 1.0, 2.0));
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        let mut esperadas = vec![zero, paredes];
        esperadas.sort();
        assert_eq!(tela.layers(&doc, MODEL_LAYOUT_NAME), Some(esperadas));
        assert_eq!(absolutos(&tela, MODEL_LAYOUT_NAME, paredes).len(), 2);

        let mut registro = doc.layers().get(paredes).expect("registro").clone();
        registro.set_off(true);
        pilha
            .edit(&mut doc, "Desligar", |editor| {
                editor.set_layer_record(paredes, registro)
            })
            .expect("desliga");
        let sujos = tela.update(&doc, pilha.revision(), pilha.last_effect());

        assert_eq!(tela.layers(&doc, MODEL_LAYOUT_NAME), Some(vec![zero]));
        assert!(
            sujos.contains(&(MODEL_LAYOUT_NAME.to_owned(), paredes)),
            "{sujos:?}"
        );
    }

    #[test]
    fn o_layout_tem_espaco_proprio() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let layout = doc.create_layout("Prancha A1").expect("layout");
        let bloco = doc.layouts().get(layout).expect("registro").block();
        insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 1.0));
        pilha
            .edit(&mut doc, "Inserir", |editor| {
                editor.insert_entity(Entity::new(zero, reta(5.0, 5.0, 6.0, 5.0)), bloco)
            })
            .expect("insere no papel");

        let tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        assert_eq!(
            tela.spaces(),
            vec![MODEL_LAYOUT_NAME.to_owned(), "Prancha A1".to_owned()]
        );
        assert_eq!(absolutos(&tela, MODEL_LAYOUT_NAME, zero).len(), 1);
        let papel = absolutos(&tela, "Prancha A1", zero);
        assert_eq!(papel.len(), 1);
        assert!(perto(papel[0], [5.0, 5.0, 6.0, 5.0], 1e-6));
    }

    #[test]
    fn texto_nao_entra_nos_blocos_de_segmentos() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        insere(
            &mut doc,
            &mut pilha,
            zero,
            Geometry::Text(Text {
                position: Point2::ORIGIN,
                content: "A".to_owned(),
                height: 1.0,
                rotation: 0.0,
            }),
        );
        insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 0.0));

        let tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        assert_eq!(absolutos(&tela, MODEL_LAYOUT_NAME, zero).len(), 1);
    }

    #[test]
    fn inserir_alterar_e_apagar_atualizam_so_o_bloco_afetado() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let outra = doc.create_layer("Outra").expect("camada");
        insere(&mut doc, &mut pilha, outra, reta(9.0, 9.0, 10.0, 9.0));
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        let id = insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 0.0));
        let sujos = tela.update(&doc, pilha.revision(), pilha.last_effect());
        assert_eq!(sujos, vec![(MODEL_LAYOUT_NAME.to_owned(), zero)]);
        assert_eq!(absolutos(&tela, MODEL_LAYOUT_NAME, zero).len(), 1);

        pilha
            .edit(&mut doc, "Substituir", |editor| {
                editor.replace_entity(id, Entity::new(zero, reta(0.0, 5.0, 1.0, 5.0)))
            })
            .expect("substitui");
        let sujos = tela.update(&doc, pilha.revision(), pilha.last_effect());
        assert_eq!(sujos, vec![(MODEL_LAYOUT_NAME.to_owned(), zero)]);
        assert!(perto(
            absolutos(&tela, MODEL_LAYOUT_NAME, zero)[0],
            [0.0, 5.0, 1.0, 5.0],
            1e-6
        ));

        pilha
            .edit(&mut doc, "Apagar", |editor| editor.remove_entity(id))
            .expect("apaga");
        let sujos = tela.update(&doc, pilha.revision(), pilha.last_effect());
        assert_eq!(sujos, vec![(MODEL_LAYOUT_NAME.to_owned(), zero)]);
        assert!(tela
            .block(MODEL_LAYOUT_NAME, zero)
            .unwrap_or_default()
            .is_empty());
        assert_eq!(absolutos(&tela, MODEL_LAYOUT_NAME, outra).len(), 1);
    }

    #[test]
    fn mudar_a_entidade_de_camada_suja_as_duas() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let outra = doc.create_layer("Outra").expect("camada");
        let id = insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 0.0));
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        pilha
            .edit(&mut doc, "Mudar camada", |editor| {
                editor.replace_entity(id, Entity::new(outra, reta(0.0, 0.0, 1.0, 0.0)))
            })
            .expect("substitui");
        let mut sujos = tela.update(&doc, pilha.revision(), pilha.last_effect());
        sujos.sort();

        let mut esperado = vec![
            (MODEL_LAYOUT_NAME.to_owned(), zero),
            (MODEL_LAYOUT_NAME.to_owned(), outra),
        ];
        esperado.sort();
        assert_eq!(sujos, esperado);
        assert!(tela
            .block(MODEL_LAYOUT_NAME, zero)
            .unwrap_or_default()
            .is_empty());
        assert_eq!(absolutos(&tela, MODEL_LAYOUT_NAME, outra).len(), 1);
    }

    #[test]
    fn revisao_pulada_refaz_tudo() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 0.0));
        insere(&mut doc, &mut pilha, zero, reta(0.0, 1.0, 1.0, 1.0));
        // Só o efeito da segunda chega; a primeira foi perdida.
        let sujos = tela.update(&doc, pilha.revision(), pilha.last_effect());

        assert_eq!(absolutos(&tela, MODEL_LAYOUT_NAME, zero).len(), 2);
        assert!(sujos.contains(&(MODEL_LAYOUT_NAME.to_owned(), zero)));
    }

    #[test]
    fn a_mesma_revisao_nao_muda_nada() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 0.0));
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        let sujos = tela.update(&doc, pilha.revision(), pilha.last_effect());

        assert!(sujos.is_empty(), "{sujos:?}");
        assert_eq!(absolutos(&tela, MODEL_LAYOUT_NAME, zero).len(), 1);
    }

    #[test]
    fn o_recorte_pela_vista_acompanha_as_edicoes() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let dentro = insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 1.0));
        insere(&mut doc, &mut pilha, zero, reta(50.0, 50.0, 51.0, 51.0));
        insere(
            &mut doc,
            &mut pilha,
            zero,
            Geometry::Arc(Arc {
                center: Point2::new(100.0, 0.0),
                radius: 1.0,
                start_angle: 0.0,
                end_angle: 1.0,
            }),
        );
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");
        let vista = Aabb::new(Point2::new(-1.0, -1.0), Point2::new(2.0, 2.0));

        assert_eq!(
            tela.query(&doc, MODEL_LAYOUT_NAME, vista),
            Some(vec![dentro])
        );

        let nova = insere(&mut doc, &mut pilha, zero, reta(0.5, 0.5, 1.5, 0.5));
        tela.update(&doc, pilha.revision(), pilha.last_effect());

        let mut achadas = tela.query(&doc, MODEL_LAYOUT_NAME, vista).expect("espaço");
        achadas.sort();
        let mut esperadas = vec![dentro, nova];
        esperadas.sort();
        assert_eq!(achadas, esperadas);
    }

    #[test]
    fn espaco_desconhecido_e_erro_de_corda_invalido() {
        let doc = Document::new();

        for erro in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(
                Screen::prepare(&doc, 0, erro).is_none(),
                "erro {erro} aceito"
            );
        }
        let mut tela = Screen::prepare(&doc, 0, ERRO).expect("erro válido");
        let zero = doc.layers().default_layer();
        assert_eq!(tela.origin("Não existe"), None);
        assert_eq!(tela.layers(&doc, "Não existe"), None);
        assert_eq!(tela.block("Não existe", zero), None);
        let vista = Aabb::new(Point2::ORIGIN, Point2::new(1.0, 1.0));
        assert_eq!(tela.query(&doc, "Não existe", vista), None);
        // O espaço-modelo existe mesmo vazio, com origem e sem camadas.
        assert_eq!(tela.origin(MODEL_LAYOUT_NAME), Some(Point2::ORIGIN));
        assert_eq!(tela.layers(&doc, MODEL_LAYOUT_NAME), Some(Vec::new()));
    }
}
