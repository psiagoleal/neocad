// Caminho relativo: kernel/neocad-transaction/src/commands.rs
//! \file kernel/neocad-transaction/src/commands.rs
//! \brief Operações de edição que movem, copiam, giram, espelham e escalam.
//! \author Iago Leal
//! \date 2026-09-08
//!
//! São as cinco operações que respondem pela maior parte do tempo de quem edita
//! um desenho, e todas as demais do bloco de edição pressupõem que geometria
//! possa ser transformada.
//!
//! # Como se encaixam na reversibilidade
//!
//! Nenhuma delas guarda o seu próprio desfazer. Todas trabalham sobre um
//! [`DocumentEditor`], que já registra a inversa de cada mudança, e é
//! [`CommandStack::edit`](crate::CommandStack::edit) que as transforma em
//! transação nomeada. Inventar aqui um segundo caminho de desfazer criaria duas
//! versões da mesma verdade, que é o defeito que o ADR 0003 fecha ao proibir
//! mutação fora de transação.
//!
//! ```
//! use neocad_geometry::{Line, Point2};
//! use neocad_model::{Document, Entity, Geometry};
//! use neocad_transaction::{commands, CommandStack};
//!
//! let mut document = Document::new();
//! let layer = document.layers().default_layer();
//! let mut stack = CommandStack::new();
//!
//! let id = stack.edit(&mut document, "Desenhar", |editor| {
//!     editor.insert_in_model_space(Entity::new(
//!         layer,
//!         Geometry::Line(Line { start: Point2::ORIGIN, end: Point2::new(4.0, 0.0) }),
//!     ))
//! })?;
//!
//! stack.edit(&mut document, "Mover", |editor| {
//!     commands::move_by(editor, &[id], 10.0, 0.0)
//! })?;
//!
//! stack.undo(&mut document)?;
//!
//! let Geometry::Line(line) = &document.entity(id).unwrap().geometry else {
//!     panic!("continua sendo uma reta");
//! };
//!
//! assert_eq!(line.start, Point2::ORIGIN, "desfazer devolveu a reta ao lugar");
//! # Ok::<(), Box<dyn core::error::Error>>(())
//! ```

use core::fmt;

use neocad_geometry::{
    chamfer as chamfer_curves, extend as extend_curve, fillet as fillet_curves,
    offset as parallel_to, trim as trim_curve, AnyCurve, Corner, CornerError, OffsetError, Point2,
    Tolerance, Transform, TrimError,
};
use neocad_model::{DocumentEditor, DocumentError, Entity, EntityId, Geometry};

/// Por que uma operação de edição não pôde ser feita.
///
/// Existe porque a paralela falha por motivo que o documento não sabe nomear: o
/// raio que a operação anularia não é um defeito do desenho, e devolver
/// [`DocumentError`] para ele obrigaria a inventar uma variante de documento
/// para cada geometria que não fecha.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    /// Falha vinda do documento.
    Document(DocumentError),
    /// A entidade não tem geometria que se possa paralelar.
    ///
    /// Texto e janela de prancha são entidades, mas não são curvas.
    NotACurve,
    /// Falha vinda da construção da paralela.
    Offset(OffsetError),
    /// Fator de escala zero, negativo ou não finito.
    ///
    /// O AutoCAD exige fator positivo: zero colapsaria a geometria num ponto, e
    /// o sinal negativo não tem leitura de escala que o usuário espere.
    InvalidScaleFactor,
    /// Falha vinda do aparo ou da extensão.
    Trim(TrimError),
    /// Falha vinda da concordância ou do chanfro.
    Corner(CornerError),
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document(error) => write!(formatter, "{error}"),
            Self::NotACurve => write!(formatter, "a entidade não é uma curva"),
            Self::Offset(error) => write!(formatter, "{error}"),
            Self::Trim(error) => write!(formatter, "{error}"),
            Self::Corner(error) => write!(formatter, "{error}"),
            Self::InvalidScaleFactor => {
                write!(formatter, "o fator de escala tem de ser positivo e finito")
            }
        }
    }
}

impl core::error::Error for CommandError {}

impl From<DocumentError> for CommandError {
    fn from(error: DocumentError) -> Self {
        Self::Document(error)
    }
}

impl From<CornerError> for CommandError {
    fn from(error: CornerError) -> Self {
        Self::Corner(error)
    }
}

impl From<TrimError> for CommandError {
    fn from(error: TrimError) -> Self {
        Self::Trim(error)
    }
}

impl From<OffsetError> for CommandError {
    fn from(error: OffsetError) -> Self {
        Self::Offset(error)
    }
}

/// Aplica a transformação às entidades, no lugar.
///
/// # Errors
///
/// Falha se alguma entidade não existir. A verificação é feita antes de a
/// primeira mudança ser aplicada, para que um identificador inválido no meio da
/// lista não deixe metade do desenho transformado.
pub fn transform(
    editor: &mut DocumentEditor<'_>,
    entities: &[EntityId],
    transform: &Transform,
) -> Result<(), DocumentError> {
    let transformed = prepare(editor, entities, transform)?;

    for (id, entity) in transformed {
        editor.replace_entity(id, entity)?;
    }

    Ok(())
}

/// Duplica as entidades, aplicando a transformação às cópias.
///
/// As cópias ficam no mesmo bloco das originais — copiar do espaço-modelo para
/// uma prancha é mudar de espaço, que é outra operação.
///
/// # Errors
///
/// Falha se alguma entidade não existir ou não estiver em bloco algum.
pub fn duplicate(
    editor: &mut DocumentEditor<'_>,
    entities: &[EntityId],
    transform: &Transform,
) -> Result<Vec<EntityId>, DocumentError> {
    let transformed = prepare(editor, entities, transform)?;
    let mut blocks = Vec::with_capacity(transformed.len());

    for (id, _) in &transformed {
        let placement = editor
            .document()
            .entity_placement(*id)
            .ok_or(DocumentError::UnknownEntity)?;

        blocks.push(placement.block);
    }

    let mut created = Vec::with_capacity(transformed.len());

    for ((_, entity), block) in transformed.into_iter().zip(blocks) {
        created.push(editor.insert_entity(entity, block)?);
    }

    Ok(created)
}

/// Lê e transforma antes de escrever qualquer coisa.
///
/// Separar a leitura da escrita é o que dá à operação o tudo-ou-nada: se uma
/// entidade da lista não existe, ninguém foi alterado ainda.
fn prepare(
    editor: &DocumentEditor<'_>,
    entities: &[EntityId],
    transform: &Transform,
) -> Result<Vec<(EntityId, neocad_model::Entity)>, DocumentError> {
    entities
        .iter()
        .map(|id| {
            let entity = editor
                .document()
                .entity(*id)
                .ok_or(DocumentError::UnknownEntity)?;

            Ok((*id, entity.transformed(transform)))
        })
        .collect()
}

/// Move as entidades por um deslocamento.
///
/// # Errors
///
/// Ver [`transform`].
pub fn move_by(
    editor: &mut DocumentEditor<'_>,
    entities: &[EntityId],
    dx: f64,
    dy: f64,
) -> Result<(), DocumentError> {
    transform(editor, entities, &Transform::translation(dx, dy))
}

/// Gira as entidades em torno de um ponto, em radianos e no anti-horário.
///
/// # Errors
///
/// Ver [`transform`].
pub fn rotate(
    editor: &mut DocumentEditor<'_>,
    entities: &[EntityId],
    pivot: Point2,
    angle: f64,
) -> Result<(), DocumentError> {
    transform(editor, entities, &Transform::rotation_around(pivot, angle))
}

/// Espelha as entidades por uma reta.
///
/// # Errors
///
/// Ver [`transform`].
pub fn mirror(
    editor: &mut DocumentEditor<'_>,
    entities: &[EntityId],
    through: Point2,
    angle: f64,
) -> Result<(), DocumentError> {
    transform(editor, entities, &Transform::mirror(through, angle))
}

/// Escala as entidades em torno de um ponto.
///
/// # Errors
///
/// [`CommandError::InvalidScaleFactor`] se o fator não for positivo e finito,
/// antes de qualquer mudança; os demais casos, ver [`transform`].
pub fn scale(
    editor: &mut DocumentEditor<'_>,
    entities: &[EntityId],
    pivot: Point2,
    factor: f64,
) -> Result<(), CommandError> {
    if !(factor.is_finite() && factor > 0.0) {
        return Err(CommandError::InvalidScaleFactor);
    }

    Ok(transform(
        editor,
        entities,
        &Transform::scaling_around(pivot, factor),
    )?)
}

/// Copia as entidades, deslocando as cópias.
///
/// # Errors
///
/// Ver [`duplicate`].
pub fn copy(
    editor: &mut DocumentEditor<'_>,
    entities: &[EntityId],
    dx: f64,
    dy: f64,
) -> Result<Vec<EntityId>, DocumentError> {
    duplicate(editor, entities, &Transform::translation(dx, dy))
}

/// Cria a paralela de uma entidade, à distância dada, do lado onde está `side`.
///
/// A entidade original **permanece**: paralelar é acrescentar, não deslocar —
/// e é por isso que a operação devolve identificadores novos em vez de nada.
/// As cópias herdam camada e cor do original e ficam no mesmo bloco.
///
/// A paralela de uma polilinha pode se partir em mais de um pedaço quando parte
/// dela é engolida pela distância pedida, daí a lista.
///
/// # Errors
///
/// Ver [`CommandError`].
///
/// # Exemplo
///
/// ```
/// use neocad_geometry::{Line, Point2};
/// use neocad_model::{Document, Entity, Geometry};
/// use neocad_transaction::{commands, CommandStack};
///
/// let mut document = Document::new();
/// let layer = document.layers().default_layer();
/// let mut stack = CommandStack::new();
///
/// let id = stack.edit(&mut document, "Desenhar", |editor| {
///     editor.insert_in_model_space(Entity::new(
///         layer,
///         Geometry::Line(Line { start: Point2::ORIGIN, end: Point2::new(10.0, 0.0) }),
///     ))
/// })?;
///
/// let paralelas = stack.edit(&mut document, "Paralela", |editor| {
///     commands::offset(editor, id, 2.0, Point2::new(5.0, 1.0))
/// })?;
///
/// assert_eq!(paralelas.len(), 1);
/// assert_eq!(document.entity_count(), 2, "a original continua lá");
///
/// stack.undo(&mut document)?;
///
/// assert_eq!(document.entity_count(), 1, "desfazer tira só a paralela");
/// # Ok::<(), Box<dyn core::error::Error>>(())
/// ```
pub fn offset(
    editor: &mut DocumentEditor<'_>,
    entity: EntityId,
    distance: f64,
    side: Point2,
) -> Result<Vec<EntityId>, CommandError> {
    let source = editor
        .document()
        .entity(entity)
        .ok_or(DocumentError::UnknownEntity)?;
    let block = editor
        .document()
        .entity_placement(entity)
        .ok_or(DocumentError::UnknownEntity)?
        .block;
    let layer = source.layer;
    let color = source.color;

    let curve = as_curve(&source.geometry).ok_or(CommandError::NotACurve)?;
    let parallels = parallel_to(&curve, distance, side, Tolerance::default())?;
    let mut created = Vec::with_capacity(parallels.len());

    for parallel in parallels {
        let entity = Entity {
            layer,
            color,
            geometry: as_geometry(parallel),
        };

        created.push(editor.insert_entity(entity, block)?);
    }

    Ok(created)
}

/// Apara a entidade contra as arestas de corte, removendo o trecho indicado.
///
/// O que sobra fica no lugar da original: o primeiro pedaço mantém o
/// identificador dela e o segundo, quando há, vira entidade nova com a mesma
/// camada e cor. Se nada sobra, a entidade é removida. Arestas que não são
/// curvas (texto, janela de prancha) não cortam e são ignoradas. Devolve os
/// identificadores do que sobrou.
///
/// # Errors
///
/// [`CommandError::NotACurve`] se a entidade não for curva;
/// [`CommandError::Trim`] se o ponto não estiver sobre ela ou nenhuma aresta a
/// dividir; erro de documento se alguma entidade não existir.
pub fn trim(
    editor: &mut DocumentEditor<'_>,
    entity: EntityId,
    cutting_edges: &[EntityId],
    pick: Point2,
    capture: f64,
) -> Result<Vec<EntityId>, CommandError> {
    let (source, curve) = curve_of(editor, entity)?;
    let edges = curves_of(editor, cutting_edges)?;
    let block = editor
        .document()
        .entity_placement(entity)
        .ok_or(DocumentError::UnknownEntity)?
        .block;
    let mut pieces = trim_curve(&curve, &edges, pick, capture, Tolerance::default())?.into_iter();

    let Some(first) = pieces.next() else {
        editor.remove_entity(entity)?;
        return Ok(Vec::new());
    };

    editor.replace_entity(
        entity,
        Entity {
            geometry: as_geometry(first),
            ..source.clone()
        },
    )?;
    let mut kept = vec![entity];

    for piece in pieces {
        let created = Entity {
            geometry: as_geometry(piece),
            ..source.clone()
        };
        kept.push(editor.insert_entity(created, block)?);
    }

    Ok(kept)
}

/// Estende a entidade, pela ponta mais próxima do ponto, até a primeira
/// fronteira, no lugar.
///
/// # Errors
///
/// [`CommandError::NotACurve`] se a entidade não for curva;
/// [`CommandError::Trim`] se o ponto não estiver sobre ela, se ela for fechada
/// ou se nenhuma fronteira for alcançada; erro de documento se alguma entidade
/// não existir.
pub fn extend(
    editor: &mut DocumentEditor<'_>,
    entity: EntityId,
    boundaries: &[EntityId],
    pick: Point2,
    capture: f64,
) -> Result<(), CommandError> {
    let (source, curve) = curve_of(editor, entity)?;
    let fences = curves_of(editor, boundaries)?;
    let extended = extend_curve(&curve, &fences, pick, capture, Tolerance::default())?;

    editor.replace_entity(
        entity,
        Entity {
            geometry: as_geometry(extended),
            ..source
        },
    )?;

    Ok(())
}

/// Concorda duas entidades com um arco de raio dado.
///
/// As duas são aparadas ou estendidas no lugar; o arco entra como entidade
/// nova, com a camada e a cor da primeira. Devolve o identificador do arco, ou
/// `None` com raio zero.
///
/// # Errors
///
/// [`CommandError::NotACurve`] se alguma não for curva;
/// [`CommandError::Corner`] se a concordância for recusada; erro de documento
/// se alguma entidade não existir.
pub fn fillet(
    editor: &mut DocumentEditor<'_>,
    first: EntityId,
    first_pick: Point2,
    second: EntityId,
    second_pick: Point2,
    radius: f64,
) -> Result<Option<EntityId>, CommandError> {
    let (source, curve_a) = curve_of(editor, first)?;
    let (_, curve_b) = curve_of(editor, second)?;
    let corner = fillet_curves(
        &curve_a,
        first_pick,
        &curve_b,
        second_pick,
        radius,
        Tolerance::default(),
    )?;

    apply_corner(editor, first, &source, second, corner)
}

/// Chanfra duas entidades com as distâncias dadas, a partir do vértice.
///
/// As duas são aparadas ou estendidas no lugar; o segmento entra como entidade
/// nova, com a camada e a cor da primeira. Devolve o identificador do segmento,
/// ou `None` com as duas distâncias zero.
///
/// # Errors
///
/// [`CommandError::NotACurve`] se alguma não for curva;
/// [`CommandError::Corner`] se o chanfro for recusado; erro de documento se
/// alguma entidade não existir.
pub fn chamfer(
    editor: &mut DocumentEditor<'_>,
    first: EntityId,
    first_pick: Point2,
    second: EntityId,
    second_pick: Point2,
    distances: (f64, f64),
) -> Result<Option<EntityId>, CommandError> {
    let (source, curve_a) = curve_of(editor, first)?;
    let (_, curve_b) = curve_of(editor, second)?;
    let corner = chamfer_curves(
        &curve_a,
        first_pick,
        &curve_b,
        second_pick,
        distances.0,
        distances.1,
        Tolerance::default(),
    )?;

    apply_corner(editor, first, &source, second, corner)
}

/// Grava o canto: as duas curvas no lugar e a junção como entidade nova, na
/// camada, cor e bloco da primeira.
fn apply_corner(
    editor: &mut DocumentEditor<'_>,
    first: EntityId,
    source: &Entity,
    second: EntityId,
    corner: Corner,
) -> Result<Option<EntityId>, CommandError> {
    let block = editor
        .document()
        .entity_placement(first)
        .ok_or(DocumentError::UnknownEntity)?
        .block;
    let other = editor
        .document()
        .entity(second)
        .ok_or(DocumentError::UnknownEntity)?
        .clone();

    editor.replace_entity(
        first,
        Entity {
            geometry: as_geometry(corner.first),
            ..source.clone()
        },
    )?;
    editor.replace_entity(
        second,
        Entity {
            geometry: as_geometry(corner.second),
            ..other
        },
    )?;

    let Some(joint) = corner.joint else {
        return Ok(None);
    };

    let created = Entity {
        geometry: as_geometry(joint),
        ..source.clone()
    };

    Ok(Some(editor.insert_entity(created, block)?))
}

/// A entidade e a sua geometria como curva.
fn curve_of(
    editor: &DocumentEditor<'_>,
    entity: EntityId,
) -> Result<(Entity, AnyCurve), CommandError> {
    let source = editor
        .document()
        .entity(entity)
        .ok_or(DocumentError::UnknownEntity)?
        .clone();
    let curve = as_curve(&source.geometry).ok_or(CommandError::NotACurve)?;

    Ok((source, curve))
}

/// As curvas das entidades dadas; as que não são curva ficam de fora.
fn curves_of(
    editor: &DocumentEditor<'_>,
    entities: &[EntityId],
) -> Result<Vec<AnyCurve>, CommandError> {
    let mut curves = Vec::with_capacity(entities.len());

    for &entity in entities {
        let source = editor
            .document()
            .entity(entity)
            .ok_or(DocumentError::UnknownEntity)?;
        curves.extend(as_curve(&source.geometry));
    }

    Ok(curves)
}

/// A geometria vista como curva, quando ela for uma.
fn as_curve(geometry: &Geometry) -> Option<AnyCurve> {
    match geometry {
        Geometry::Line(line) => Some(AnyCurve::Line(*line)),
        Geometry::Circle(circle) => Some(AnyCurve::Circle(*circle)),
        Geometry::Arc(arc) => Some(AnyCurve::Arc(*arc)),
        Geometry::Polyline(polyline) => Some(AnyCurve::Polyline(polyline.clone())),
        Geometry::Text(_) | Geometry::Viewport(_) => None,
    }
}

/// A curva de volta à geometria que o modelo guarda.
fn as_geometry(curve: AnyCurve) -> Geometry {
    match curve {
        AnyCurve::Line(line) => Geometry::Line(line),
        AnyCurve::Circle(circle) => Geometry::Circle(circle),
        AnyCurve::Arc(arc) => Geometry::Arc(arc),
        AnyCurve::Polyline(polyline) => Geometry::Polyline(polyline),
    }
}
