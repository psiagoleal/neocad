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

use neocad_geometry::{
    snap, tessellate, text_strokes, Aabb, AnyCurve, Point2, SnapMode, SpatialIndex, Tolerance,
};
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
    /// Índice das entidades do espaço, para o recorte pela vista e a seleção.
    index: Option<SpatialIndex<EntityId>>,
    /// Entidades mexidas desde que o índice foi feito: a consulta as confere à parte,
    /// para uma edição não custar a reconstrução do índice inteiro.
    stale: BTreeSet<EntityId>,
}

/// Quantas entidades, as mais perto do cursor, entram no prendimento (spec 10, P4).
const SNAP_LIMIT: usize = 32;

/// Até quantas selecionadas o destaque procura as camadas delas, em vez de
/// percorrer todas.
const FEW_SELECTED: usize = 64;

/// Mexidas além disto fazem o índice ser refeito na próxima consulta.
const STALE_LIMIT: usize = 1024;

impl SpaceScreen {
    fn mark_stale(&mut self, id: EntityId) {
        if self.index.is_none() {
            return;
        }
        self.stale.insert(id);
        if self.stale.len() > STALE_LIMIT {
            self.index = None;
            self.stale.clear();
        }
    }
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
    /// Entidades que não viraram segmentos: espaço e camada (V6).
    refused: BTreeMap<EntityId, (usize, LayerId)>,
    /// Seleção (spec 10, S1–S6), em ordem de id. Vive num espaço só, como a da
    /// tela, que se esvazia ao trocar de espaço (S3).
    selection: BTreeSet<EntityId>,
    /// Espaço da seleção.
    selection_space: usize,
}

/// Tipo DXF da geometria, como o aviso das não desenhadas o mostra.
fn dxf_type(geometry: &Geometry) -> &'static str {
    match geometry {
        Geometry::Line(_) => "LINE",
        Geometry::Circle(_) => "CIRCLE",
        Geometry::Arc(_) => "ARC",
        Geometry::Polyline(_) => "LWPOLYLINE",
        Geometry::Text(_) => "TEXT",
        Geometry::Viewport(_) => "VIEWPORT",
    }
}

/// Tipos de entidade DXF que esta tela desenha.
///
/// Quem ainda desenha o resto (o upstream, até o MT-KV-13b) usa esta lista para
/// não desenhar de novo o que já está aqui. `VIEWPORT` é lido, mas não vira traço;
/// `INSERT` e os demais ainda não são modelados.
pub const SCREEN_ENTITY_TYPES: &[&str] =
    &["LINE", "CIRCLE", "ARC", "LWPOLYLINE", "POLYLINE", "TEXT"];

/// Segmentos `x1, y1, x2, y2` da entidade, relativos à origem.
///
/// O texto vira os traços da fonte (ADR 0010). A janela de layout não vira
/// segmentos: é moldura de outra vista.
fn segments(entity: &Entity, origin: Point2, chord_error: f64) -> Option<Vec<f32>> {
    // A subtração é feita em f64; só a diferença, pequena, vira f32.
    #[allow(clippy::cast_possible_truncation)]
    let relative = |p: Point2| [(p.x - origin.x) as f32, (p.y - origin.y) as f32];

    let curve = match &entity.geometry {
        Geometry::Line(line) => AnyCurve::Line(*line),
        Geometry::Circle(circle) => AnyCurve::Circle(*circle),
        Geometry::Arc(arc) => AnyCurve::Arc(*arc),
        Geometry::Polyline(polyline) => AnyCurve::Polyline(polyline.clone()),
        Geometry::Text(text) => {
            let strokes = text_strokes(&text.content, text.position, text.height, text.rotation);
            if strokes.is_empty() {
                return None;
            }
            return Some(
                strokes
                    .into_iter()
                    .flat_map(|(a, b)| relative(a).into_iter().chain(relative(b)))
                    .collect(),
            );
        }
        Geometry::Viewport(_) => return None,
    };
    let points = tessellate(&curve, chord_error)?;

    let mut out = Vec::with_capacity(points.len().saturating_sub(1) * 4);
    for pair in points.windows(2) {
        out.extend(relative(pair[0]));
        out.extend(relative(pair[1]));
    }

    Some(out)
}

/// Distância do ponto ao segmento `a`–`b`.
fn distance_to_segment(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length2 = dx * dx + dy * dy;
    let t = if length2 > 0.0 {
        (((p.x - a.x) * dx + (p.y - a.y) * dy) / length2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (cx, cy) = (a.x + t * dx, a.y + t * dy);
    (p.x - cx).hypot(p.y - cy)
}

/// O segmento `a`–`b` toca a caixa (recorte de Liang–Barsky).
fn segment_touches_box(a: Point2, b: Point2, area: Aabb) -> bool {
    let (min, max) = (area.min(), area.max());
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let mut t0: f64 = 0.0;
    let mut t1: f64 = 1.0;
    for (p, q) in [
        (-dx, a.x - min.x),
        (dx, max.x - a.x),
        (-dy, a.y - min.y),
        (dy, max.y - a.y),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
            if t0 > t1 {
                return false;
            }
        }
    }
    true
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
            refused: BTreeMap::new(),
            selection: BTreeSet::new(),
            selection_space: 0,
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
                stale: BTreeSet::new(),
            })
            .collect();
        self.place.clear();
        self.refused.clear();

        for space in 0..self.spaces.len() {
            let block = self.spaces[space].block;
            for (id, entity) in document.entities_in_block(block) {
                self.add(space, id, entity);
            }
            // Feito já aqui, para o primeiro clique não pagar a construção.
            let target = &mut self.spaces[space];
            target.stale.clear();
            target.index = Some(SpatialIndex::build(
                document
                    .entities_in_block(block)
                    .map(|(id, entity)| (entity.bounding_box(), id)),
            ));
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
        } else {
            self.refused.insert(id, (space, entity.layer));
        }
        target.mark_stale(id);

        entity.layer
    }

    /// Tira os segmentos da entidade; devolve onde estavam.
    fn remove(&mut self, id: EntityId) -> Option<(usize, LayerId)> {
        if let Some((space, _)) = self.refused.remove(&id) {
            self.spaces[space].mark_stale(id);
        }
        let (space, layer) = self.place.remove(&id)?;
        let target = &mut self.spaces[space];
        if let Some(entities) = target.layers.get_mut(&layer) {
            entities.remove(&id);
            if entities.is_empty() {
                target.layers.remove(&layer);
            }
        }
        target.mark_stale(id);

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

    /// Quantas entidades do espaço estão desenhadas: as que têm segmentos em
    /// camada visível.
    #[must_use]
    pub fn drawn_count(&self, document: &Document, space: &str) -> Option<usize> {
        let found = &self.spaces[self.space(space)?];
        let visible = self.layers(document, space)?;

        Some(
            visible
                .iter()
                .filter_map(|layer| found.layers.get(layer))
                .map(BTreeMap::len)
                .sum(),
        )
    }

    /// Entidades do espaço, em camada visível, que a tela não desenha: tipo DXF e
    /// quantidade, em ordem de tipo. `None` se o espaço é desconhecido.
    #[must_use]
    pub fn refused(&self, document: &Document, space: &str) -> Option<Vec<(String, usize)>> {
        let found = self.space(space)?;
        // Seis tipos no máximo: um vetor basta, e não traz outro mapa ao `.wasm`.
        let mut counts: Vec<(String, usize)> = Vec::new();
        for (id, (refused_space, layer)) in &self.refused {
            let visible = document
                .layers()
                .get(*layer)
                .is_some_and(LayerRecord::is_visible);
            if *refused_space != found || !visible {
                continue;
            }
            let Some(kind) = document.entity(*id).map(|e| dxf_type(&e.geometry)) else {
                continue;
            };
            match counts.iter_mut().find(|(k, _)| k == kind) {
                Some((_, count)) => *count += 1,
                None => counts.push((kind.to_owned(), 1)),
            }
        }
        counts.sort_unstable();

        Some(counts)
    }

    /// Entidades desenhadas, em camada visível, cujo traço passa a até `radius` do
    /// ponto, da mais perto à mais longe (spec 10, S1). `None` se o espaço é
    /// desconhecido.
    pub fn pick(
        &mut self,
        document: &Document,
        space: &str,
        point: Point2,
        radius: f64,
    ) -> Option<Vec<EntityId>> {
        let found = self.space(space)?;
        let reach = Aabb::new(
            Point2::new(point.x - radius, point.y - radius),
            Point2::new(point.x + radius, point.y + radius),
        );
        let candidates = self.candidates(document, found, reach);

        let mut hits: Vec<(f64, EntityId)> = candidates
            .into_iter()
            .filter_map(|id| {
                let distance = self
                    .visible_segments(document, found, id)?
                    .map(|(a, b)| distance_to_segment(point, a, b))
                    .fold(f64::INFINITY, f64::min);
                (distance <= radius).then_some((distance, id))
            })
            .collect();
        hits.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));

        Some(hits.into_iter().map(|(_, id)| id).collect())
    }

    /// Entidades desenhadas, em camada visível, inteiramente dentro da área
    /// (janela) ou, com `crossing`, que a área toca ou contém (cruzada) (spec 10,
    /// S2). `None` se o espaço é desconhecido.
    pub fn select_area(
        &mut self,
        document: &Document,
        space: &str,
        area: Aabb,
        crossing: bool,
    ) -> Option<Vec<EntityId>> {
        let found = self.space(space)?;
        let mut candidates = self.candidates(document, found, area);
        candidates.sort();

        Some(
            candidates
                .into_iter()
                .filter(|id| {
                    // A candidata já é do espaço; desenhada é a que não foi recusada.
                    let Some(entity) = document.entity(*id) else {
                        return false;
                    };
                    let visible = document
                        .layers()
                        .get(entity.layer)
                        .is_some_and(LayerRecord::is_visible);
                    if !visible || self.refused.contains_key(id) {
                        return false;
                    }
                    // O traço de reta, círculo, arco e polilinha tem os vértices sobre a
                    // curva, dentro da caixa dela: com a caixa dentro da área, entra sem
                    // buscar nem percorrer os segmentos (S6). A caixa do texto é
                    // estimada; ele segue conferido traço a traço.
                    if !matches!(entity.geometry, Geometry::Text(_)) {
                        let caixa = entity.bounding_box();
                        if area.contains(caixa.min()) && area.contains(caixa.max()) {
                            return true;
                        }
                    }
                    let Some(mut segments) = self.visible_segments(document, found, *id) else {
                        return false;
                    };
                    if crossing {
                        segments.any(|(a, b)| segment_touches_box(a, b, area))
                    } else {
                        segments.all(|(a, b)| area.contains(a) && area.contains(b))
                    }
                })
                .collect(),
        )
    }

    /// Põe na seleção (ou, com `remove`, tira dela) as entidades da área, como
    /// `select_area` as escolhe (spec 10, S2); devolve quantas ficam selecionadas
    /// no espaço. `None` se o espaço é desconhecido.
    pub fn select_area_into(
        &mut self,
        document: &Document,
        space: &str,
        area: Aabb,
        crossing: bool,
        remove: bool,
    ) -> Option<usize> {
        let ids = self.select_area(document, space, area, crossing)?;
        // Já conferidas como desenhadas no espaço: entram sem buscar onde estão (S6).
        self.apply_selection(space, ids.into_iter(), remove)
    }

    /// Põe na seleção (ou tira dela) as entidades dadas, como o clique (spec 10,
    /// S1); as que não estão desenhadas no espaço são ignoradas. Devolve quantas
    /// ficam selecionadas no espaço; `None` se o espaço é desconhecido.
    pub fn change_selection(
        &mut self,
        space: &str,
        ids: &[EntityId],
        remove: bool,
    ) -> Option<usize> {
        let found = self.space(space)?;
        let place = &self.place;
        let drawn: Vec<EntityId> = ids
            .iter()
            .copied()
            .filter(|id| place.get(id).is_some_and(|(s, _)| *s == found))
            .collect();
        self.apply_selection(space, drawn.into_iter(), remove)
    }

    /// Põe (ou tira) as entidades, já conferidas, na seleção do espaço; pôr
    /// noutro espaço esvazia a do anterior.
    fn apply_selection(
        &mut self,
        space: &str,
        ids: impl Iterator<Item = EntityId>,
        remove: bool,
    ) -> Option<usize> {
        let found = self.space(space)?;
        if remove {
            for id in ids {
                self.selection.remove(&id);
            }
        } else {
            if self.selection_space != found {
                self.selection.clear();
                self.selection_space = found;
            }
            self.selection.extend(ids);
        }
        self.selection_count(space)
    }

    /// Esvazia a seleção.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Selecionadas no espaço, em ordem de id.
    #[must_use]
    pub fn selection(&self, space: &str) -> Option<Vec<EntityId>> {
        let found = self.space(space)?;
        Some(if found == self.selection_space {
            self.selection.iter().copied().collect()
        } else {
            Vec::new()
        })
    }

    /// Quantas estão selecionadas no espaço.
    #[must_use]
    pub fn selection_count(&self, space: &str) -> Option<usize> {
        let found = self.space(space)?;
        Some(if found == self.selection_space {
            self.selection.len()
        } else {
            0
        })
    }

    /// O destaque da seleção na camada do espaço, como faixas do bloco dela
    /// (`block`): os primeiros vértices de cada trecho contíguo de selecionadas e
    /// depois as contagens, para a placa redesenhar o que já tem. `None` se o
    /// espaço é desconhecido.
    #[must_use]
    pub fn selection_ranges(&self, space: &str, layer: LayerId) -> Option<Vec<u32>> {
        let found = self.space(space)?;
        let mut firsts: Vec<u32> = Vec::new();
        let mut counts: Vec<u32> = Vec::new();
        let Some(entities) = self.spaces[found].layers.get(&layer) else {
            return Some(firsts);
        };
        // Com poucas selecionadas (o clique), só se percorre a camada que tem alguma,
        // e só até a última dela.
        let last = if self.selection.len() <= FEW_SELECTED {
            let Some(last) = self
                .selection
                .iter()
                .rev()
                .find(|id| entities.contains_key(*id))
            else {
                return Some(firsts);
            };
            Some(*last)
        } else {
            None
        };
        // O bloco é a concatenação dos segmentos, na ordem dos ids (`block`).
        let mut at: u32 = 0;
        let mut open = false;
        for (id, segments) in entities {
            if last.is_some_and(|last| *id > last) {
                break;
            }
            let vertices = u32::try_from(segments.len() / 2).unwrap_or(u32::MAX);
            match (self.selection.contains(id), open, counts.last_mut()) {
                (true, true, Some(count)) => *count += vertices,
                (true, _, _) => {
                    firsts.push(at);
                    counts.push(vertices);
                    open = true;
                }
                (false, _, _) => open = false,
            }
            at += vertices;
        }
        firsts.append(&mut counts);
        Some(firsts)
    }

    /// Tira da seleção o que deixou de estar desenhado no espaço dela (spec 10,
    /// S4); `ids` limita a conferência, `None` confere todas.
    fn prune_selection(&mut self, ids: Option<&[EntityId]>) {
        let place = &self.place;
        let space = self.selection_space;
        let checked: Vec<EntityId> = match ids {
            Some(ids) => ids
                .iter()
                .copied()
                .filter(|id| self.selection.contains(id))
                .collect(),
            None => self.selection.iter().copied().collect(),
        };
        let gone: Vec<EntityId> = checked
            .into_iter()
            .filter(|id| place.get(id).is_none_or(|(s, _)| *s != space))
            .collect();
        for id in gone {
            self.selection.remove(&id);
        }
    }

    /// O ponto notável mais prioritário ao alcance do cursor, entre as entidades
    /// desenhadas em camada visível no espaço (spec 10, P2): modo, ponto e a
    /// entidade que o produziu. `None` se o espaço é desconhecido; `Some(None)`
    /// se nada prende.
    pub fn snap(
        &mut self,
        document: &Document,
        space: &str,
        cursor: Point2,
        radius: f64,
        modes: &[SnapMode],
    ) -> Option<Option<(SnapMode, Point2, EntityId)>> {
        let found = self.space(space)?;
        let reach = Aabb::new(
            Point2::new(cursor.x - radius, cursor.y - radius),
            Point2::new(cursor.x + radius, cursor.y + radius),
        );
        // Só o que o cursor alcança: o traço ao alcance, ou o centro (de um círculo
        // grande, longe do traço). Dos que sobram, os mais perto, para a interseção,
        // que testa pares, não crescer com a densidade do desenho.
        let mut near: Vec<(f64, EntityId, AnyCurve)> = self
            .candidates(document, found, reach)
            .into_iter()
            .filter_map(|id| {
                let distance = self
                    .visible_segments(document, found, id)?
                    .map(|(a, b)| distance_to_segment(cursor, a, b))
                    .fold(f64::INFINITY, f64::min);
                let entity = document.entity(id)?;
                let center_in_reach = match &entity.geometry {
                    Geometry::Circle(c) => c.center.distance_to(cursor) <= radius,
                    Geometry::Arc(a) => a.center.distance_to(cursor) <= radius,
                    _ => false,
                };
                if distance > radius && !center_in_reach {
                    return None;
                }
                let curve = crate::curve_of_geometry(&entity.geometry)?;
                Some((distance, id, curve))
            })
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        near.truncate(SNAP_LIMIT);
        let (ids, curves): (Vec<EntityId>, Vec<AnyCurve>) =
            near.into_iter().map(|(_, id, curve)| (id, curve)).unzip();

        Some(
            snap(&curves, cursor, radius, modes, Tolerance::default())
                .map(|found| (found.mode, found.point, ids[found.curve])),
        )
    }

    /// Entidades do espaço cuja caixa toca a área: as do índice que não mudaram,
    /// mais as mexidas desde ele, conferidas pela caixa de agora.
    fn candidates(&mut self, document: &Document, found: usize, area: Aabb) -> Vec<EntityId> {
        let space = &mut self.spaces[found];
        let block = space.block;
        if space.index.is_none() {
            space.stale.clear();
        }
        let index = space.index.get_or_insert_with(|| {
            SpatialIndex::build(
                document
                    .entities_in_block(block)
                    .map(|(id, entity)| (entity.bounding_box(), id)),
            )
        });

        let mut found: Vec<EntityId> = index
            .query_box(area)
            .into_iter()
            .filter(|id| !space.stale.contains(id))
            .collect();
        found.extend(space.stale.iter().copied().filter(|id| {
            document
                .entity_placement(*id)
                .is_some_and(|placement| placement.block == block)
                && document
                    .entity(*id)
                    .is_some_and(|entity| entity.bounding_box().intersects(area))
        }));
        found
    }

    /// Segmentos da entidade em coordenadas do desenho, se ela está desenhada no
    /// espaço e em camada visível.
    fn visible_segments<'a>(
        &'a self,
        document: &Document,
        found: usize,
        id: EntityId,
    ) -> Option<impl Iterator<Item = (Point2, Point2)> + 'a> {
        let (place_space, layer) = self.place.get(&id)?;
        if *place_space != found
            || !document
                .layers()
                .get(*layer)
                .is_some_and(LayerRecord::is_visible)
        {
            return None;
        }
        let space = &self.spaces[found];
        let origin = space.origin;
        let point =
            move |x: f32, y: f32| Point2::new(origin.x + f64::from(x), origin.y + f64::from(y));

        Some(
            space
                .layers
                .get(layer)?
                .get(&id)?
                .chunks_exact(4)
                .map(move |s| (point(s[0], s[1]), point(s[2], s[3]))),
        )
    }

    /// Caixa envolvente das entidades do espaço; `None` se o espaço é
    /// desconhecido, `Some(None)` se está vazio.
    #[must_use]
    pub fn extents(&self, document: &Document, space: &str) -> Option<Option<Aabb>> {
        let found = &self.spaces[self.space(space)?];

        Some(document.block_bounding_box(found.block))
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
            let mexidas: Vec<EntityId> = effect
                .removed
                .iter()
                .chain(&effect.modified)
                .copied()
                .collect();
            self.prune_selection(Some(&mexidas));
        } else {
            // Alguma operação passou sem ser vista: refaz tudo.
            dirty = self.all_blocks();
            self.fill(document);
            dirty.extend(self.all_blocks());
            self.prune_selection(None);
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
        Some(self.candidates(document, found, area))
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

    /// Lê a seção `ENTITIES` de um DXF mínimo com o corpo dado.
    fn entidades_do_dxf(corpo: &str) -> Vec<Entity> {
        let texto = format!("0\nSECTION\n2\nENTITIES\n{corpo}0\nENDSEC\n0\nEOF\n");
        neocad_io::read_dxf(texto.as_bytes())
            .entities
            .into_iter()
            .map(|lida| lida.entity)
            .collect()
    }

    #[test]
    fn cada_tipo_da_lista_vira_tracos_e_o_resto_fica_fora() {
        let amostras = [
            ("LINE", "0\nLINE\n8\n0\n10\n0\n20\n0\n11\n1\n21\n1\n"),
            ("CIRCLE", "0\nCIRCLE\n8\n0\n10\n0\n20\n0\n40\n1\n"),
            ("ARC", "0\nARC\n8\n0\n10\n0\n20\n0\n40\n1\n50\n0\n51\n90\n"),
            (
                "LWPOLYLINE",
                "0\nLWPOLYLINE\n8\n0\n90\n2\n10\n0\n20\n0\n10\n1\n20\n1\n",
            ),
            (
                "POLYLINE",
                "0\nPOLYLINE\n8\n0\n66\n1\n0\nVERTEX\n8\n0\n10\n0\n20\n0\n\
                 0\nVERTEX\n8\n0\n10\n1\n20\n1\n0\nSEQEND\n8\n0\n",
            ),
            ("TEXT", "0\nTEXT\n8\n0\n10\n0\n20\n0\n40\n1\n1\nA\n"),
        ];

        for (tipo, corpo) in amostras {
            assert!(SCREEN_ENTITY_TYPES.contains(&tipo), "{tipo} fora da lista");
            let lidas = entidades_do_dxf(corpo);
            assert_eq!(lidas.len(), 1, "{tipo}: {} entidades lidas", lidas.len());
            assert!(
                segments(&lidas[0], Point2::new(0.0, 0.0), ERRO).is_some(),
                "{tipo} não vira traço"
            );
        }
        assert_eq!(SCREEN_ENTITY_TYPES.len(), amostras.len());

        for fora in ["VIEWPORT", "INSERT", "SOLID", "TRACE", "HATCH", "MTEXT"] {
            assert!(!SCREEN_ENTITY_TYPES.contains(&fora), "{fora} na lista");
        }
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

    fn janela() -> Geometry {
        Geometry::Viewport(neocad_model::Viewport {
            center: Point2::new(100.0, 100.0),
            width: 100.0,
            height: 50.0,
            view_center: Point2::ORIGIN,
            view_height: 25.0,
            twist: 0.0,
            clip: neocad_model::ViewportClip::Window,
            is_on: true,
            frozen_layers: BTreeSet::new(),
        })
    }

    fn texto(conteudo: &str) -> Geometry {
        Geometry::Text(Text {
            content: conteudo.to_owned(),
            position: Point2::ORIGIN,
            height: 1.0,
            rotation: 0.0,
        })
    }

    #[test]
    fn o_que_nao_vira_traco_e_contado_por_tipo_e_espaco() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let oculta = doc.create_layer("Oculta").expect("camada");
        let layout = doc.create_layout("Prancha A1").expect("layout");
        let bloco = doc.layouts().get(layout).expect("registro").block();
        insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 1.0));
        insere(&mut doc, &mut pilha, zero, texto("  "));
        for (camada, g) in [
            (zero, janela()),
            (zero, janela()),
            (oculta, janela()),
            (zero, reta(5.0, 5.0, 6.0, 5.0)),
        ] {
            pilha
                .edit(&mut doc, "Inserir", |editor| {
                    editor.insert_entity(Entity::new(camada, g), bloco)
                })
                .expect("insere no papel");
        }
        let mut registro = doc.layers().get(oculta).expect("registro").clone();
        registro.set_off(true);
        pilha
            .edit(&mut doc, "Desligar", |editor| {
                editor.set_layer_record(oculta, registro)
            })
            .expect("desliga");

        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        assert_eq!(
            tela.refused(&doc, MODEL_LAYOUT_NAME),
            Some(vec![("TEXT".to_owned(), 1)])
        );
        // A janela em camada desligada não seria vista de qualquer jeito.
        assert_eq!(
            tela.refused(&doc, "Prancha A1"),
            Some(vec![("VIEWPORT".to_owned(), 2)])
        );
        assert_eq!(tela.refused(&doc, "Inexistente"), None);

        // Apagar a recusada tira a contagem; a edição é acompanhada.
        let vazio = doc
            .entities_in_block(
                doc.layouts()
                    .get(doc.layouts().model_layout())
                    .expect("modelo")
                    .block(),
            )
            .find(|(_, e)| matches!(e.geometry, Geometry::Text(_)))
            .map(|(id, _)| id)
            .expect("texto vazio");
        pilha
            .edit(&mut doc, "Apagar", |editor| editor.remove_entity(vazio))
            .expect("apaga");
        tela.update(&doc, pilha.revision(), pilha.last_effect());
        assert_eq!(tela.refused(&doc, MODEL_LAYOUT_NAME), Some(vec![]));
    }

    /// Um desenho para a seleção: duas retas e um círculo no modelo, uma reta em
    /// camada desligada, uma janela de layout e uma reta no papel.
    fn desenho_para_selecao() -> (Document, Screen, [EntityId; 4]) {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let oculta = doc.create_layer("Oculta").expect("camada");
        let embaixo = insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 10.0, 0.0));
        let em_cima = insere(&mut doc, &mut pilha, zero, reta(0.0, 5.0, 10.0, 5.0));
        let circulo = insere(
            &mut doc,
            &mut pilha,
            zero,
            Geometry::Circle(Circle {
                center: Point2::new(30.0, 0.0),
                radius: 5.0,
            }),
        );
        let escondida = insere(&mut doc, &mut pilha, oculta, reta(0.0, 1.0, 10.0, 1.0));
        insere(&mut doc, &mut pilha, zero, texto("   "));
        let layout = doc.create_layout("Prancha A1").expect("layout");
        let bloco = doc.layouts().get(layout).expect("registro").block();
        for g in [janela(), reta(0.0, 0.0, 10.0, 0.0)] {
            pilha
                .edit(&mut doc, "Inserir", |editor| {
                    editor.insert_entity(Entity::new(zero, g), bloco)
                })
                .expect("insere no papel");
        }
        let mut registro = doc.layers().get(oculta).expect("registro").clone();
        registro.set_off(true);
        pilha
            .edit(&mut doc, "Desligar", |editor| {
                editor.set_layer_record(oculta, registro)
            })
            .expect("desliga");
        let tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");
        (doc, tela, [embaixo, em_cima, circulo, escondida])
    }

    #[test]
    fn o_clique_pega_o_traco_mais_perto_primeiro_so_no_que_se_ve() {
        let (doc, mut tela, [embaixo, em_cima, _, escondida]) = desenho_para_selecao();
        let m = MODEL_LAYOUT_NAME;

        assert_eq!(
            tela.pick(&doc, m, Point2::new(5.0, 0.4), 0.5),
            Some(vec![embaixo])
        );
        assert_eq!(tela.pick(&doc, m, Point2::new(5.0, 2.5), 0.5), Some(vec![]));
        assert_eq!(
            tela.pick(&doc, m, Point2::new(5.0, 3.0), 3.5),
            Some(vec![em_cima, embaixo])
        );
        // A reta de camada desligada passa em y = 1 e não é pega.
        let perto_da_escondida = tela.pick(&doc, m, Point2::new(5.0, 1.0), 0.2);
        assert_eq!(perto_da_escondida, Some(vec![]));
        assert!(!tela
            .pick(&doc, m, Point2::new(5.0, 1.0), 2.0)
            .expect("modelo")
            .contains(&escondida));
        // No papel, só a reta do papel; a janela de layout não vira traço.
        assert_eq!(
            tela.pick(&doc, "Prancha A1", Point2::new(5.0, 0.0), 0.5)
                .map(|ids| ids.len()),
            Some(1)
        );
        assert_eq!(tela.pick(&doc, "Inexistente", Point2::ORIGIN, 1.0), None);
    }

    #[test]
    fn janela_pega_o_que_esta_dentro_e_cruzada_o_que_ela_toca() {
        let (doc, mut tela, [embaixo, em_cima, circulo, _]) = desenho_para_selecao();
        let m = MODEL_LAYOUT_NAME;
        let area =
            |a: (f64, f64), b: (f64, f64)| Aabb::new(Point2::new(a.0, a.1), Point2::new(b.0, b.1));

        // Envolve a reta de baixo inteira (e passa pela escondida, que não conta).
        let baixo = area((-1.0, -1.0), (11.0, 2.0));
        assert_eq!(tela.select_area(&doc, m, baixo, false), Some(vec![embaixo]));
        // Pega metade das duas retas: a janela não pega nenhuma; a cruzada, as duas.
        let metade = area((5.0, -1.0), (20.0, 6.0));
        assert_eq!(tela.select_area(&doc, m, metade, false), Some(vec![]));
        assert_eq!(
            tela.select_area(&doc, m, metade, true),
            Some(vec![embaixo, em_cima])
        );
        // Dentro do círculo, sem tocar o traço: a caixa dele cruza a área, o traço não.
        let miolo = area((29.0, -1.0), (31.0, 1.0));
        assert_eq!(tela.select_area(&doc, m, miolo, true), Some(vec![]));
        let em_volta = area((24.0, -6.0), (36.0, 6.0));
        assert_eq!(
            tela.select_area(&doc, m, em_volta, false),
            Some(vec![circulo])
        );
        assert_eq!(tela.select_area(&doc, "Inexistente", miolo, true), None);
    }

    #[test]
    fn a_selecao_fica_no_kernel_num_espaco_so() {
        let (doc, mut tela, [embaixo, em_cima, circulo, escondida]) = desenho_para_selecao();
        let m = MODEL_LAYOUT_NAME;
        let area = |(x0, y0): (f64, f64), (x1, y1): (f64, f64)| {
            Aabb::new(Point2::new(x0, y0), Point2::new(x1, y1))
        };

        // O clique põe uma; a janela põe as que estão dentro, sem repetir.
        assert_eq!(tela.change_selection(m, &[em_cima], false), Some(1));
        let duas_retas = area((-1.0, -1.0), (11.0, 6.0));
        assert_eq!(
            tela.select_area_into(&doc, m, duas_retas, false, false),
            Some(2)
        );
        // Em ordem de id, não de entrada.
        assert_eq!(tela.selection(m), Some(vec![embaixo, em_cima]));
        assert_eq!(tela.selection_count(m), Some(2));

        // A cruzada põe o círculo; com `remove`, tira as duas retas.
        let toca_o_circulo = area((34.0, -1.0), (40.0, 1.0));
        assert_eq!(
            tela.select_area_into(&doc, m, toca_o_circulo, true, false),
            Some(3)
        );
        assert_eq!(
            tela.select_area_into(&doc, m, duas_retas, true, true),
            Some(1)
        );
        assert_eq!(tela.selection(m), Some(vec![circulo]));

        // O que não está desenhado no espaço é ignorado.
        let do_papel = tela
            .pick(&doc, "Prancha A1", Point2::new(5.0, 0.0), 0.5)
            .expect("papel");
        assert_eq!(tela.change_selection(m, &do_papel, false), Some(1));
        // A seleção vive num espaço só: selecionar noutro esvazia a do anterior.
        assert_eq!(
            tela.change_selection("Prancha A1", &do_papel, false),
            Some(1)
        );
        assert_eq!(tela.selection_count(m), Some(0));
        assert_eq!(tela.selection(m), Some(vec![]));
        assert_eq!(tela.selection("Prancha A1"), Some(do_papel.clone()));
        // O clique com Shift tira.
        assert_eq!(
            tela.change_selection("Prancha A1", &do_papel, true),
            Some(0)
        );

        // A janela não pega a de camada desligada.
        let tudo = area((-100.0, -100.0), (100.0, 100.0));
        tela.select_area_into(&doc, m, tudo, false, false);
        assert!(!tela.selection(m).expect("modelo").contains(&escondida));
        assert_eq!(tela.selection_count(m), Some(3));

        tela.clear_selection();
        assert_eq!(tela.selection_count(m), Some(0));
        assert_eq!(tela.selection_count("Prancha A1"), Some(0));
        assert_eq!(
            tela.change_selection("Inexistente", &[embaixo], false),
            None
        );
        assert_eq!(
            tela.select_area_into(&doc, "Inexistente", tudo, false, false),
            None
        );
        assert_eq!(tela.selection("Inexistente"), None);
    }

    #[test]
    fn o_destaque_sao_faixas_do_bloco_da_camada() {
        let (doc, mut tela, [embaixo, em_cima, circulo, _]) = desenho_para_selecao();
        let m = MODEL_LAYOUT_NAME;
        let zero = doc.layers().default_layer();
        // O bloco da camada 0: embaixo (2 vértices), em cima (2) e o círculo.
        let vertices = u32::try_from(tela.block(m, zero).expect("bloco").len() / 2).expect("cabe");
        let do_circulo = vertices - 4;
        assert_eq!(tela.selection_ranges(m, zero), Some(vec![]));

        // Primeiros vértices e depois as contagens, uma faixa por trecho contíguo.
        tela.change_selection(m, &[em_cima], false);
        assert_eq!(tela.selection_ranges(m, zero), Some(vec![2, 2]));
        tela.change_selection(m, &[embaixo], false);
        assert_eq!(tela.selection_ranges(m, zero), Some(vec![0, 4]));
        tela.change_selection(m, &[em_cima], true);
        tela.change_selection(m, &[circulo], false);
        assert_eq!(
            tela.selection_ranges(m, zero),
            Some(vec![0, 4, 2, do_circulo])
        );
        tela.change_selection(m, &[em_cima], false);
        assert_eq!(tela.selection_ranges(m, zero), Some(vec![0, vertices]));

        assert_eq!(tela.selection_ranges("Prancha A1", zero), Some(vec![]));
        assert_eq!(tela.selection_ranges("Inexistente", zero), None);
    }

    #[test]
    fn com_muitas_selecionadas_as_faixas_saem_do_bloco_inteiro() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let outra = doc.create_layer("Outra").expect("camada");
        let retas: Vec<EntityId> = (0..100)
            .map(|i| {
                let y = f64::from(i);
                insere(&mut doc, &mut pilha, zero, reta(0.0, y, 1.0, y))
            })
            .collect();
        insere(&mut doc, &mut pilha, outra, reta(5.0, 0.0, 6.0, 0.0));
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("tela");
        let m = MODEL_LAYOUT_NAME;
        // Fora a cada dez, a partir da quarta: faixas de 2 vértices por reta.
        let escolhidas: Vec<EntityId> = retas
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 10 != 3)
            .map(|(_, id)| *id)
            .collect();
        assert!(escolhidas.len() > FEW_SELECTED);
        tela.change_selection(m, &escolhidas, false);

        let mut inicios = vec![0];
        let mut contagens = vec![6];
        for dezena in 0..10 {
            inicios.push(2 * (dezena * 10 + 4));
            contagens.push(if dezena == 9 { 12 } else { 18 });
        }
        inicios.extend(contagens);
        assert_eq!(tela.selection_ranges(m, zero), Some(inicios));
        assert_eq!(tela.selection_ranges(m, outra), Some(vec![]));
    }

    #[test]
    fn a_edicao_tira_da_selecao_a_apagada_e_o_destaque_segue_a_movida() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let fica = insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 10.0, 0.0));
        let sai = insere(&mut doc, &mut pilha, zero, reta(0.0, 5.0, 10.0, 5.0));
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("tela");
        let m = MODEL_LAYOUT_NAME;
        tela.change_selection(m, &[sai, fica], false);

        pilha
            .edit(&mut doc, "Apagar", |editor| editor.remove_entity(sai))
            .expect("apaga");
        tela.update(&doc, pilha.revision(), pilha.last_effect());
        assert_eq!(tela.selection(m), Some(vec![fica]));

        pilha
            .edit(&mut doc, "Mover", |editor| {
                editor.replace_entity(fica, Entity::new(zero, reta(0.0, 20.0, 10.0, 20.0)))
            })
            .expect("move");
        tela.update(&doc, pilha.revision(), pilha.last_effect());
        assert_eq!(tela.selection(m), Some(vec![fica]));
        // A faixa aponta para onde a movida está agora no bloco.
        let origem = tela.origin(m).expect("origem");
        let faixa = tela.selection_ranges(m, zero).expect("modelo");
        assert_eq!(faixa.len(), 2);
        let bloco = tela.block(m, zero).expect("bloco");
        let y = bloco[faixa[0] as usize * 2 + 1];
        assert!((origem.y + f64::from(y) - 20.0).abs() < 1e-4);

        // Desfazer a apagada a traz de volta à tela, mas não à seleção.
        pilha.undo(&mut doc).expect("desfaz");
        pilha.undo(&mut doc).expect("desfaz");
        tela.update(&doc, pilha.revision(), pilha.last_effect());
        assert_eq!(tela.selection(m), Some(vec![fica]));
    }

    #[test]
    fn o_indice_acompanha_muitas_edicoes_sem_errar_a_consulta() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        insere(&mut doc, &mut pilha, zero, reta(0.0, 0.0, 1.0, 0.0));
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");
        let m = MODEL_LAYOUT_NAME;
        let longe = Aabb::new(Point2::new(500.0, -1.0), Point2::new(600.0, 1.0));

        // Passa do limite das mexidas guardadas à parte, para o índice ser refeito.
        let mut ultima = None;
        for i in 0..1100 {
            let x = 500.0 + f64::from(i) * 0.05;
            ultima = Some(insere(
                &mut doc,
                &mut pilha,
                zero,
                reta(x, 0.0, x + 0.01, 0.0),
            ));
            tela.update(&doc, pilha.revision(), pilha.last_effect());
            if i == 10 {
                assert_eq!(tela.query(&doc, m, longe).map(|v| v.len()), Some(11));
            }
        }
        assert_eq!(tela.query(&doc, m, longe).map(|v| v.len()), Some(1100));

        let ultima = ultima.expect("inserida");
        pilha
            .edit(&mut doc, "Apagar", |editor| editor.remove_entity(ultima))
            .expect("apaga");
        tela.update(&doc, pilha.revision(), pilha.last_effect());
        let achadas = tela.query(&doc, m, longe).expect("modelo");
        assert_eq!(achadas.len(), 1099);
        assert!(!achadas.contains(&ultima));
    }

    #[test]
    fn o_prendimento_olha_so_o_que_esta_perto_e_se_ve_no_espaco() {
        let (doc, mut tela, [embaixo, _, circulo, escondida]) = desenho_para_selecao();
        let m = MODEL_LAYOUT_NAME;
        let todos = [
            SnapMode::Endpoint,
            SnapMode::Midpoint,
            SnapMode::Center,
            SnapMode::Quadrant,
            SnapMode::Intersection,
        ];

        // A reta de baixo vai de (0,0) a (10,0).
        let fim = tela.snap(&doc, m, Point2::new(9.8, 0.1), 0.5, &todos);
        assert_eq!(
            fim,
            Some(Some((SnapMode::Endpoint, Point2::new(10.0, 0.0), embaixo)))
        );
        let meio = tela.snap(&doc, m, Point2::new(5.1, -0.2), 0.5, &todos);
        assert_eq!(
            meio,
            Some(Some((SnapMode::Midpoint, Point2::new(5.0, 0.0), embaixo)))
        );
        // O círculo de raio 5 em (30,0): centro e quadrante.
        let centro = tela.snap(&doc, m, Point2::new(30.2, 0.1), 0.5, &todos);
        assert_eq!(
            centro,
            Some(Some((SnapMode::Center, Point2::new(30.0, 0.0), circulo)))
        );
        let quadrante = tela.snap(&doc, m, Point2::new(30.1, 5.2), 0.5, &todos);
        assert_eq!(
            quadrante,
            Some(Some((SnapMode::Quadrant, Point2::new(30.0, 5.0), circulo)))
        );
        // A reta escondida (em y = 1) não prende; longe de tudo, nada.
        let perto_da_escondida = tela.snap(&doc, m, Point2::new(10.0, 1.1), 0.3, &todos);
        assert!(perto_da_escondida
            .flatten()
            .is_none_or(|(_, _, id)| id != escondida));
        assert_eq!(
            tela.snap(&doc, m, Point2::new(50.0, 50.0), 0.5, &todos),
            Some(None)
        );
        // No papel, a reta do papel prende; o espaço desconhecido não existe.
        assert!(matches!(
            tela.snap(&doc, "Prancha A1", Point2::new(9.9, 0.0), 0.5, &todos),
            Some(Some((SnapMode::Endpoint, _, _)))
        ));
        assert_eq!(
            tela.snap(&doc, "Inexistente", Point2::ORIGIN, 1.0, &todos),
            None
        );
    }

    #[test]
    fn o_prendimento_num_desenho_denso_acha_o_mais_perto() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        // Muitas retas que se cruzam perto do cursor, e uma extremidade nele.
        for i in 0..400 {
            let t = f64::from(i) * 0.01;
            insere(
                &mut doc,
                &mut pilha,
                zero,
                reta(-50.0, -50.0 + t, 50.0, 50.0 - t),
            );
        }
        let alvo = insere(&mut doc, &mut pilha, zero, reta(10.0, 10.0, 20.0, 15.0));
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        let achado = tela.snap(
            &doc,
            MODEL_LAYOUT_NAME,
            Point2::new(10.05, 10.02),
            0.5,
            &[SnapMode::Endpoint, SnapMode::Intersection],
        );
        assert_eq!(
            achado,
            Some(Some((SnapMode::Endpoint, Point2::new(10.0, 10.0), alvo)))
        );
    }

    #[test]
    fn texto_vira_os_tracos_da_fonte_no_bloco_da_camada() {
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

        // A reta e os três traços do `A` (duas pernas e a barra).
        let tracos = neocad_geometry::text_strokes("A", Point2::ORIGIN, 1.0, 0.0);
        assert_eq!(tracos.len(), 3);
        let segmentos = absolutos(&tela, MODEL_LAYOUT_NAME, zero);
        assert_eq!(segmentos.len(), 4);
        for (a, b) in tracos {
            assert!(segmentos
                .iter()
                .any(|s| perto(*s, [a.x, a.y, b.x, b.y], 1e-5)));
        }
        assert_eq!(tela.drawn_count(&doc, MODEL_LAYOUT_NAME), Some(2));
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
    fn contagem_desenhada_e_caixa_de_cada_espaco() {
        let mut doc = Document::new();
        let mut pilha = CommandStack::new();
        let zero = doc.layers().default_layer();
        let paredes = doc.create_layer("Paredes").expect("camada");
        let layout = doc.create_layout("Prancha A1").expect("layout");
        let bloco = doc.layouts().get(layout).expect("registro").block();
        insere(&mut doc, &mut pilha, zero, reta(-2.0, 1.0, 3.0, 1.0));
        insere(&mut doc, &mut pilha, paredes, reta(0.0, -4.0, 0.0, 6.0));
        insere(
            &mut doc,
            &mut pilha,
            zero,
            Geometry::Text(Text {
                position: Point2::new(1.0, 1.0),
                content: "A".to_owned(),
                height: 1.0,
                rotation: 0.0,
            }),
        );
        pilha
            .edit(&mut doc, "Inserir", |editor| {
                editor.insert_entity(Entity::new(zero, reta(5.0, 5.0, 6.0, 5.0)), bloco)
            })
            .expect("insere no papel");
        let mut tela = Screen::prepare(&doc, pilha.revision(), ERRO).expect("erro válido");

        assert_eq!(tela.drawn_count(&doc, MODEL_LAYOUT_NAME), Some(3));
        assert_eq!(tela.drawn_count(&doc, "Prancha A1"), Some(1));
        assert_eq!(tela.drawn_count(&doc, "Nenhum"), None);

        let caixa = tela
            .extents(&doc, MODEL_LAYOUT_NAME)
            .expect("espaço")
            .expect("não vazio");
        assert!(perto(
            [caixa.min().x, caixa.min().y, caixa.max().x, caixa.max().y],
            [-2.0, -4.0, 3.0, 6.0],
            1e-9
        ));
        let papel = tela
            .extents(&doc, "Prancha A1")
            .expect("espaço")
            .expect("não vazio");
        assert!(perto(
            [papel.min().x, papel.min().y, papel.max().x, papel.max().y],
            [5.0, 5.0, 6.0, 5.0],
            1e-9
        ));
        assert_eq!(tela.extents(&doc, "Nenhum"), None);

        // Camada desligada some da contagem.
        let mut registro = doc.layers().get(paredes).expect("registro").clone();
        registro.set_off(true);
        pilha
            .edit(&mut doc, "Desligar", |editor| {
                editor.set_layer_record(paredes, registro)
            })
            .expect("desliga");
        tela.update(&doc, pilha.revision(), pilha.last_effect());
        assert_eq!(tela.drawn_count(&doc, MODEL_LAYOUT_NAME), Some(2));
    }

    #[test]
    fn espaco_vazio_nao_tem_caixa() {
        let doc = Document::new();
        let tela = Screen::prepare(&doc, 0, ERRO).expect("erro válido");

        assert_eq!(tela.extents(&doc, MODEL_LAYOUT_NAME), Some(None));
        assert_eq!(tela.drawn_count(&doc, MODEL_LAYOUT_NAME), Some(0));
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
