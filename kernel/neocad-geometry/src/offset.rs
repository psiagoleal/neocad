// Caminho relativo: kernel/neocad-geometry/src/offset.rs
//! \file kernel/neocad-geometry/src/offset.rs
//! \brief Paralela a distância dada, com o lado escolhido por um ponto.
//! \author Iago Leal
//! \date 2026-09-08
//!
//! É a operação de que o desenho técnico mais depende depois das cinco de
//! transformação: espessura de parede, faixa de servidão, contorno de peça.
//!
//! # O que distingue a paralela certa da ingênua
//!
//! Deslocar cada trecho pela normal e reemendar vértice a vértice produz uma
//! figura que **não é** a paralela: onde a curva vira para dentro mais depressa
//! do que a distância pedida, os trechos deslocados se ultrapassam e sobra um
//! laço. O laço não é um detalhe estético — ele é o pedaço da figura que está
//! *mais perto* do original do que a distância pedida, e é por isso que ele sai
//! daqui por um critério, e não por uma coleção de casos:
//!
//! > ponto da paralela que esteja mais perto do original do que a distância
//! > pedida não é ponto da paralela.
//!
//! O mesmo critério apara o laço de dois trechos vizinhos e o laço que se fecha
//! entre trechos distantes na sequência, que é o que impede esta operação de
//! virar uma lista de casos particulares.
//!
//! # O que ela não faz
//!
//! Não paralela curva de abaulamento variável (não há tal curva no modelo) e
//! não arredonda o canto convexo: o canto sai em ponta, prolongando os dois
//! trechos até se encontrarem, que é o que o AutoCAD faz por omissão.

use crate::{
    intersect, orientation, AnyCurve, Arc, Circle, Curve, Line, Orientation, Point2, Polyline,
    PolylineSegment, PolylineVertex, Tolerance,
};
use core::fmt;

/// Por que a paralela não pôde ser construída.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetError {
    /// Distância nula, negativa ou não numérica.
    ///
    /// A distância é uma medida, e o lado vem do ponto; distância negativa
    /// seria uma segunda forma de dizer o lado, e duas formas de dizer a mesma
    /// coisa acabam discordando.
    InvalidDistance,
    /// A curva não tem comprimento sobre o qual construir uma paralela.
    DegenerateCurve,
    /// O ponto informado está **sobre** a curva e não escolhe lado algum.
    AmbiguousSide,
    /// A paralela anularia ou inverteria um raio.
    ///
    /// Arco paralelado para dentro além do próprio raio não vira arco de raio
    /// negativo: vira nada.
    RadiusVanishes,
    /// Nada sobrou da paralela: todos os trechos estão mais perto do original
    /// do que a distância pedida.
    NothingLeft,
}

impl fmt::Display for OffsetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDistance => {
                write!(formatter, "a distância da paralela precisa ser positiva")
            }
            Self::DegenerateCurve => write!(formatter, "curva sem comprimento não tem paralela"),
            Self::AmbiguousSide => write!(
                formatter,
                "o ponto informado está sobre a curva e não escolhe lado"
            ),
            Self::RadiusVanishes => write!(formatter, "a paralela anularia o raio"),
            Self::NothingLeft => write!(
                formatter,
                "nada da paralela fica à distância pedida do original"
            ),
        }
    }
}

impl core::error::Error for OffsetError {}

/// Paralela à curva, à distância dada, do lado onde está `side`.
///
/// Devolve **uma lista** porque a paralela de uma polilinha pode se partir: os
/// trechos que o critério de distância descarta deixam vãos, e cada pedaço
/// contíguo que sobra é uma curva por si. Para reta, circunferência e arco a
/// lista tem sempre um elemento.
///
/// # Errors
///
/// Ver [`OffsetError`].
///
/// # Exemplo
///
/// ```
/// use neocad_geometry::{offset, AnyCurve, Line, Point2, Tolerance};
///
/// let reta = AnyCurve::Line(Line {
///     start: Point2::ORIGIN,
///     end: Point2::new(10.0, 0.0),
/// });
///
/// // O ponto está acima da reta, então a paralela sobe.
/// let acima = offset(&reta, 2.0, Point2::new(5.0, 1.0), Tolerance::default())?;
///
/// let [AnyCurve::Line(paralela)] = acima.as_slice() else {
///     panic!("a paralela de uma reta é uma reta");
/// };
///
/// assert_eq!(paralela.start, Point2::new(0.0, 2.0));
/// assert_eq!(paralela.end, Point2::new(10.0, 2.0));
/// # Ok::<(), neocad_geometry::OffsetError>(())
/// ```
pub fn offset(
    curve: &AnyCurve,
    distance: f64,
    side: Point2,
    tolerance: Tolerance,
) -> Result<Vec<AnyCurve>, OffsetError> {
    if !distance.is_finite() || distance <= 0.0 || tolerance.is_zero_length(distance) {
        return Err(OffsetError::InvalidDistance);
    }

    match curve {
        AnyCurve::Line(line) => Ok(vec![AnyCurve::Line(offset_line(
            line,
            signed_for_line(line, side, distance, tolerance)?,
            tolerance,
        )?)]),
        AnyCurve::Circle(circle) => {
            let signed =
                signed_for_circle(circle.center, circle.radius, side, distance, tolerance)?;
            let radius = shrunk(circle.radius.abs(), signed, tolerance)?;

            Ok(vec![AnyCurve::Circle(Circle {
                center: circle.center,
                radius,
            })])
        }
        AnyCurve::Arc(arc) => {
            let signed = signed_for_circle(arc.center, arc.radius, side, distance, tolerance)?;
            let radius = shrunk(arc.radius.abs(), signed, tolerance)?;

            Ok(vec![AnyCurve::Arc(Arc { radius, ..*arc })])
        }
        AnyCurve::Polyline(polyline) => offset_polyline(polyline, distance, side, tolerance),
    }
}

/// O raio que sobra depois de a paralela comer `signed` dele.
///
/// Positivo é o lado esquerdo do percurso, que numa curva anti-horária é o lado
/// de dentro — daí a subtração.
fn shrunk(radius: f64, signed: f64, tolerance: Tolerance) -> Result<f64, OffsetError> {
    let result = radius - signed;

    if result <= 0.0 || tolerance.is_zero_length(result) {
        return Err(OffsetError::RadiusVanishes);
    }

    Ok(result)
}

/// Paralela de um segmento de reta, deslocado pela normal à esquerda.
fn offset_line(line: &Line, signed: f64, tolerance: Tolerance) -> Result<Line, OffsetError> {
    let (dx, dy) = (line.end.x - line.start.x, line.end.y - line.start.y);
    let length = line.start.distance_to(line.end);

    if tolerance.is_zero_length(length) {
        return Err(OffsetError::DegenerateCurve);
    }

    let normal = (-dy / length * signed, dx / length * signed);

    Ok(Line {
        start: line.start.translated(normal.0, normal.1),
        end: line.end.translated(normal.0, normal.1),
    })
}

/// De que lado da reta está o ponto, em distância com sinal.
///
/// A decisão é do predicado exato: um ponto colinear não escolhe lado, e um
/// ponto que não é colinear escolhe um lado que nenhum arredondamento inverte
/// (ADR 0006).
fn signed_for_line(
    line: &Line,
    side: Point2,
    distance: f64,
    tolerance: Tolerance,
) -> Result<f64, OffsetError> {
    if tolerance.points_equal(line.start, line.end) {
        return Err(OffsetError::DegenerateCurve);
    }

    match orientation(line.start, line.end, side) {
        Orientation::CounterClockwise => Ok(distance),
        Orientation::Clockwise => Ok(-distance),
        Orientation::Collinear => Err(OffsetError::AmbiguousSide),
    }
}

/// De que lado da circunferência de suporte está o ponto.
///
/// Aqui não há predicado exato a que recorrer — a pergunta é sobre distância ao
/// centro, e não sobre orientação de três pontos —, então o ponto que cai
/// **sobre** a circunferência dentro da tolerância é recusado antes de o sinal
/// ser lido. O sinal que sobra já está longe da fronteira.
fn signed_for_circle(
    center: Point2,
    radius: f64,
    side: Point2,
    distance: f64,
    tolerance: Tolerance,
) -> Result<f64, OffsetError> {
    let radius = radius.abs();

    if radius <= 0.0 || tolerance.is_zero_length(radius) {
        return Err(OffsetError::DegenerateCurve);
    }

    let reach = center.distance_to(side);

    if tolerance.lengths_equal(reach, radius) {
        return Err(OffsetError::AmbiguousSide);
    }

    // Percorrida no anti-horário, a circunferência tem o centro à esquerda.
    if reach < radius {
        Ok(distance)
    } else {
        Ok(-distance)
    }
}

/// Paralela de uma polilinha, trecho a trecho e depois aparada.
fn offset_polyline(
    source: &Polyline,
    distance: f64,
    side: Point2,
    tolerance: Tolerance,
) -> Result<Vec<AnyCurve>, OffsetError> {
    let segments: Vec<PolylineSegment> = source
        .segments()
        .filter(|segment| !tolerance.is_zero_length(segment.length()))
        .collect();

    if segments.is_empty() {
        return Err(OffsetError::DegenerateCurve);
    }

    let signed = signed_for_polyline(&segments, side, distance, tolerance)?;
    let mut pieces = Vec::with_capacity(segments.len());

    for segment in &segments {
        pieces.push(offset_segment(segment, signed, tolerance)?);
    }

    join_all(&mut pieces, &segments, source.closed, tolerance);

    let kept = without_loops(&pieces, source, distance, tolerance);
    let assembled = assemble(kept, source.closed, tolerance);

    if assembled.is_empty() {
        return Err(OffsetError::NothingLeft);
    }

    Ok(assembled.into_iter().map(AnyCurve::Polyline).collect())
}

/// De que lado da polilinha está o ponto, medido no trecho mais próximo dele.
///
/// O lado é decidido uma vez e vale para a polilinha inteira: o sinal é relativo
/// ao **sentido de percurso**, e o sentido de percurso é o que dá coerência a um
/// deslocamento que muda de direção a cada vértice.
fn signed_for_polyline(
    segments: &[PolylineSegment],
    side: Point2,
    distance: f64,
    tolerance: Tolerance,
) -> Result<f64, OffsetError> {
    let nearest = segments
        .iter()
        .min_by(|first, second| {
            first
                .closest_point(side)
                .distance
                .total_cmp(&second.closest_point(side).distance)
        })
        .ok_or(OffsetError::DegenerateCurve)?;

    match nearest {
        PolylineSegment::Straight(line) => signed_for_line(line, side, distance, tolerance),
        PolylineSegment::Curved { arc, reversed } => {
            let signed = signed_for_circle(arc.center, arc.radius, side, distance, tolerance)?;

            // Percorrido ao contrário, o trecho tem o centro à direita.
            Ok(if *reversed { -signed } else { signed })
        }
    }
}

/// Paralela de um trecho, preservando o sentido em que ele é percorrido.
fn offset_segment(
    segment: &PolylineSegment,
    signed: f64,
    tolerance: Tolerance,
) -> Result<PolylineSegment, OffsetError> {
    match segment {
        PolylineSegment::Straight(line) => Ok(PolylineSegment::Straight(offset_line(
            line, signed, tolerance,
        )?)),
        PolylineSegment::Curved { arc, reversed } => {
            // Percorrido ao contrário, o lado esquerdo é o de fora, e a paralela
            // cresce onde a do trecho direto encolheria.
            let toward = if *reversed { -signed } else { signed };
            let radius = shrunk(arc.radius.abs(), toward, tolerance)?;

            Ok(PolylineSegment::Curved {
                arc: Arc { radius, ..*arc },
                reversed: *reversed,
            })
        }
    }
}

/// Reemenda cada par de trechos vizinhos no ponto em que os suportes se cruzam.
///
/// É o mesmo gesto para os dois tipos de canto: no canto que abre, os trechos
/// são **prolongados** até se encontrarem, e sai a ponta; no canto que fecha,
/// são **aparados** no mesmo ponto, e some a sobra. Quando os suportes não se
/// encontram, os trechos ficam como estão e a paralela se parte ali — inventar
/// uma emenda seria inventar geometria que o original não tem.
fn join_all(
    pieces: &mut [PolylineSegment],
    segments: &[PolylineSegment],
    closed: bool,
    tolerance: Tolerance,
) {
    let count = pieces.len();
    let joints = if closed {
        count
    } else {
        count.saturating_sub(1)
    };

    for index in 0..joints {
        let next = (index + 1) % count;

        if next == index {
            continue;
        }

        let corner = segments[next].point_at(0.0);
        let mut first = pieces[index];
        let mut second = pieces[next];

        if tolerance.points_equal(first.point_at(1.0), second.point_at(0.0)) {
            continue;
        }

        let meeting = supports_meet(&first, &second)
            .into_iter()
            .filter(|point| point.is_finite())
            .min_by(|a, b| corner.distance_to(*a).total_cmp(&corner.distance_to(*b)));

        let Some(meeting) = meeting else {
            continue;
        };

        set_end(&mut first, meeting);
        set_start(&mut second, meeting);

        pieces[index] = first;
        pieces[next] = second;
    }
}

/// Corta a corrente em toda autointerseção e descarta o que ficou perto demais.
///
/// A distância é medida do **meio** de cada pedaço à curva original. Depois do
/// corte, é o meio que diz a que lado do laço o pedaço pertence, porque as
/// pontas de um pedaço de laço estão, por construção, exatamente à distância
/// pedida.
fn without_loops(
    pieces: &[PolylineSegment],
    source: &Polyline,
    distance: f64,
    tolerance: Tolerance,
) -> Vec<PolylineSegment> {
    let mut cuts: Vec<Vec<f64>> = vec![Vec::new(); pieces.len()];

    for (index, first) in pieces.iter().enumerate() {
        for (other, second) in pieces.iter().enumerate().skip(index + 1) {
            let found = intersect(&as_curve(first), &as_curve(second), tolerance);

            for point in found.points {
                cuts[index].push(first.closest_point(point).parameter);
                cuts[other].push(second.closest_point(point).parameter);
            }
        }
    }

    let mut kept = Vec::new();

    for (piece, marks) in pieces.iter().zip(cuts) {
        for part in split(piece, &marks, tolerance) {
            if far_enough(&part, source, distance, tolerance) && !repeats(&kept, &part, tolerance) {
                kept.push(part);
            }
        }
    }

    kept
}

/// Se o pedaço repete caminho que outro já percorre.
///
/// Acontece quando a paralela engole uma reentrância: os dois trechos que
/// ladeavam a reentrância produzem paralelas que se sobrepõem, e as duas
/// guardam a distância pedida — o critério de afastamento não tem como separá-las
/// porque nenhuma das duas está errada. Errado é a paralela passar duas vezes
/// pelo mesmo lugar, e é isso que se recusa aqui.
fn repeats(kept: &[PolylineSegment], part: &PolylineSegment, tolerance: Tolerance) -> bool {
    let middle = part.point_at(0.5);

    kept.iter()
        .any(|other| tolerance.points_equal(other.closest_point(middle).point, middle))
}

/// Se o meio do pedaço guarda do original a distância pedida.
fn far_enough(
    part: &PolylineSegment,
    source: &Polyline,
    distance: f64,
    tolerance: Tolerance,
) -> bool {
    let middle = part.point_at(0.5);
    let magnitude = middle.x.abs().max(middle.y.abs()).max(distance);

    source.closest_point(middle).distance >= distance - tolerance.effective_linear(magnitude)
}

/// Reparte o trecho nos parâmetros informados, descartando pedaços sem
/// comprimento.
fn split(piece: &PolylineSegment, cuts: &[f64], tolerance: Tolerance) -> Vec<PolylineSegment> {
    let mut bounds = Vec::with_capacity(cuts.len() + 2);

    bounds.push(0.0);
    bounds.extend(
        cuts.iter()
            .copied()
            .filter(|mark| *mark > 0.0 && *mark < 1.0),
    );
    bounds.push(1.0);
    bounds.sort_by(f64::total_cmp);

    bounds
        .windows(2)
        .filter_map(|pair| sub_piece(piece, pair[0], pair[1], tolerance))
        .collect()
}

/// O pedaço do trecho entre dois parâmetros, ou nada se ele não tiver
/// comprimento.
pub(crate) fn sub_piece(
    piece: &PolylineSegment,
    from: f64,
    to: f64,
    tolerance: Tolerance,
) -> Option<PolylineSegment> {
    let result = match piece {
        PolylineSegment::Straight(_) => PolylineSegment::Straight(Line {
            start: piece.point_at(from),
            end: piece.point_at(to),
        }),
        PolylineSegment::Curved { arc, reversed } => {
            let sweep = arc.sweep();
            // O arco guardado é sempre anti-horário; num trecho percorrido ao
            // contrário, o começo do pedaço é o fim do arco.
            let (low, high) = if *reversed {
                (1.0 - to, 1.0 - from)
            } else {
                (from, to)
            };

            PolylineSegment::Curved {
                arc: Arc {
                    start_angle: arc.start_angle + low * sweep,
                    end_angle: arc.start_angle + high * sweep,
                    ..*arc
                },
                reversed: *reversed,
            }
        }
    };

    if tolerance.is_zero_length(result.length()) {
        None
    } else {
        Some(result)
    }
}

/// Junta os pedaços que sobraram em curvas contíguas.
///
/// Dois pedaços seguidos entram na mesma curva quando o fim de um é o começo do
/// outro, ainda que o que havia entre eles tenha sido descartado: é justamente
/// isso que faz uma reentrância engolida pela paralela sumir sem deixar emenda.
fn assemble(pieces: Vec<PolylineSegment>, closed: bool, tolerance: Tolerance) -> Vec<Polyline> {
    let mut runs: Vec<Vec<PolylineSegment>> = Vec::new();

    for piece in pieces {
        match runs.last_mut() {
            Some(run)
                if tolerance.points_equal(
                    run.last().map_or(Point2::ORIGIN, |last| last.point_at(1.0)),
                    piece.point_at(0.0),
                ) =>
            {
                run.push(piece);
            }
            _ => runs.push(vec![piece]),
        }
    }

    // Numa polilinha fechada a corrente pode ter sido cortada no meio de um
    // trecho, e o último pedaço então continua o primeiro.
    if closed && runs.len() >= 2 {
        let ends_where_it_starts = tolerance.points_equal(
            runs[runs.len() - 1][runs[runs.len() - 1].len() - 1].point_at(1.0),
            runs[0][0].point_at(0.0),
        );

        if ends_where_it_starts {
            let tail = runs.pop().unwrap_or_default();

            runs[0].splice(0..0, tail);
        }
    }

    runs.into_iter()
        .map(|run| {
            let round = closed && runs_around(&run, tolerance) && run.len() > 1;

            polyline_from(&run, round)
        })
        .collect()
}

/// Se a corrente de pedaços volta ao ponto de partida.
fn runs_around(run: &[PolylineSegment], tolerance: Tolerance) -> bool {
    match (run.first(), run.last()) {
        (Some(first), Some(last)) => {
            tolerance.points_equal(last.point_at(1.0), first.point_at(0.0))
        }
        _ => false,
    }
}

/// Monta a polilinha a partir dos pedaços, devolvendo o abaulamento a cada arco.
pub(crate) fn polyline_from(run: &[PolylineSegment], closed: bool) -> Polyline {
    let mut vertices = Vec::with_capacity(run.len() + 1);

    for piece in run {
        let bulge = match piece {
            PolylineSegment::Straight(_) => 0.0,
            PolylineSegment::Curved { arc, reversed } => {
                let magnitude = (arc.sweep() / 4.0).tan();

                if *reversed {
                    -magnitude
                } else {
                    magnitude
                }
            }
        };

        vertices.push(PolylineVertex::new(piece.point_at(0.0), bulge));
    }

    if !closed {
        if let Some(last) = run.last() {
            vertices.push(PolylineVertex::straight(last.point_at(1.0)));
        }
    }

    Polyline { vertices, closed }
}

/// O trecho visto como curva qualquer, para a interseção poder despachá-lo.
fn as_curve(piece: &PolylineSegment) -> AnyCurve {
    match piece {
        PolylineSegment::Straight(line) => AnyCurve::Line(*line),
        PolylineSegment::Curved { arc, .. } => AnyCurve::Arc(*arc),
    }
}

/// Move o começo do trecho para o ponto informado, prolongando ou aparando.
fn set_start(piece: &mut PolylineSegment, point: Point2) {
    match piece {
        PolylineSegment::Straight(line) => line.start = point,
        PolylineSegment::Curved { arc, reversed } => {
            let angle = angle_at(arc.center, point);

            if *reversed {
                arc.end_angle = angle;
            } else {
                arc.start_angle = angle;
            }
        }
    }
}

/// Move o fim do trecho para o ponto informado, prolongando ou aparando.
fn set_end(piece: &mut PolylineSegment, point: Point2) {
    match piece {
        PolylineSegment::Straight(line) => line.end = point,
        PolylineSegment::Curved { arc, reversed } => {
            let angle = angle_at(arc.center, point);

            if *reversed {
                arc.start_angle = angle;
            } else {
                arc.end_angle = angle;
            }
        }
    }
}

/// Ângulo do ponto visto do centro.
fn angle_at(center: Point2, point: Point2) -> f64 {
    (point.y - center.y).atan2(point.x - center.x)
}

/// Onde os suportes de dois trechos se encontram, sem limite de extensão.
///
/// É outra pergunta que a da interseção: ali as curvas terminam onde terminam,
/// aqui a reta é infinita e o arco é a circunferência inteira, porque emendar um
/// canto que abre exige prolongar os dois lados.
fn supports_meet(first: &PolylineSegment, second: &PolylineSegment) -> Vec<Point2> {
    match (first, second) {
        (PolylineSegment::Straight(a), PolylineSegment::Straight(b)) => lines_meet(a, b),
        (PolylineSegment::Straight(line), PolylineSegment::Curved { arc, .. }) => {
            line_circle_meet(line, arc)
        }
        (PolylineSegment::Curved { arc, .. }, PolylineSegment::Straight(line)) => {
            line_circle_meet(line, arc)
        }
        (PolylineSegment::Curved { arc: a, .. }, PolylineSegment::Curved { arc: b, .. }) => {
            circles_meet(a, b)
        }
    }
}

/// Encontro de duas retas de suporte, ambas infinitas.
fn lines_meet(a: &Line, b: &Line) -> Vec<Point2> {
    let first = Point2::new(a.end.x - a.start.x, a.end.y - a.start.y);
    let second = Point2::new(b.end.x - b.start.x, b.end.y - b.start.y);

    // Paralelismo decidido pelo predicado exato sobre as direções: um
    // denominador comparado com epsilon erraria justamente no canto raso, que é
    // onde a resposta mais importa (ADR 0006).
    if orientation(Point2::ORIGIN, first, second) == Orientation::Collinear {
        return Vec::new();
    }

    let denominator = first.x * second.y - first.y * second.x;
    let (dx, dy) = (b.start.x - a.start.x, b.start.y - a.start.y);
    let along = (dx * second.y - dy * second.x) / denominator;

    vec![Point2::new(
        a.start.x + first.x * along,
        a.start.y + first.y * along,
    )]
}

/// Encontro de uma reta infinita com a circunferência de suporte de um arco.
fn line_circle_meet(line: &Line, arc: &Arc) -> Vec<Point2> {
    let (dx, dy) = (line.end.x - line.start.x, line.end.y - line.start.y);
    let squared = dx * dx + dy * dy;

    if squared == 0.0 {
        return Vec::new();
    }

    let radius = arc.radius.abs();
    let along = ((arc.center.x - line.start.x) * dx + (arc.center.y - line.start.y) * dy) / squared;
    let foot = Point2::new(line.start.x + dx * along, line.start.y + dy * along);
    let gap = arc.center.distance_to(foot);

    if gap > radius {
        return Vec::new();
    }

    let half = (radius * radius - gap * gap).max(0.0).sqrt();
    let length = squared.sqrt();
    let step = (dx / length * half, dy / length * half);

    vec![
        foot.translated(step.0, step.1),
        foot.translated(-step.0, -step.1),
    ]
}

/// Encontro das circunferências de suporte de dois arcos.
fn circles_meet(a: &Arc, b: &Arc) -> Vec<Point2> {
    let (first, second) = (a.radius.abs(), b.radius.abs());
    let gap = a.center.distance_to(b.center);

    if gap == 0.0 || gap > first + second || gap < (first - second).abs() {
        return Vec::new();
    }

    let along = (gap * gap + first * first - second * second) / (2.0 * gap);
    let half = (first * first - along * along).max(0.0).sqrt();
    let unit = (
        (b.center.x - a.center.x) / gap,
        (b.center.y - a.center.y) / gap,
    );
    let foot = Point2::new(a.center.x + unit.0 * along, a.center.y + unit.1 * along);
    let normal = (-unit.1 * half, unit.0 * half);

    vec![
        foot.translated(normal.0, normal.1),
        foot.translated(-normal.0, -normal.1),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64::consts::{FRAC_PI_2, PI};

    /// Quão longe do original está o ponto mais próximo de toda a paralela.
    fn menor_afastamento(paralela: &[AnyCurve], original: &Polyline) -> f64 {
        let mut menor = f64::INFINITY;

        for curva in paralela {
            for passo in 0..=200 {
                let parametro = f64::from(passo) / 200.0;
                let ponto = curva.point_at(parametro);

                menor = menor.min(original.closest_point(ponto).distance);
            }
        }

        menor
    }

    fn reta(de: (f64, f64), ate: (f64, f64)) -> AnyCurve {
        AnyCurve::Line(Line {
            start: Point2::new(de.0, de.1),
            end: Point2::new(ate.0, ate.1),
        })
    }

    #[test]
    fn a_distancia_precisa_ser_positiva() {
        let alvo = reta((0.0, 0.0), (10.0, 0.0));
        let lado = Point2::new(5.0, 1.0);
        let tolerancia = Tolerance::default();

        for invalida in [0.0, -2.0, f64::NAN, f64::INFINITY, 1e-12] {
            assert_eq!(
                offset(&alvo, invalida, lado, tolerancia),
                Err(OffsetError::InvalidDistance),
                "distância {invalida} deveria ser recusada"
            );
        }
    }

    #[test]
    fn a_paralela_de_reta_vai_para_o_lado_do_ponto() {
        let alvo = reta((0.0, 0.0), (10.0, 0.0));
        let tolerancia = Tolerance::default();

        let acima = offset(&alvo, 2.0, Point2::new(5.0, 7.0), tolerancia).expect("lado de cima");
        let abaixo = offset(&alvo, 2.0, Point2::new(5.0, -0.5), tolerancia).expect("lado de baixo");

        let [AnyCurve::Line(sobe)] = acima.as_slice() else {
            panic!("paralela de reta é reta");
        };
        let [AnyCurve::Line(desce)] = abaixo.as_slice() else {
            panic!("paralela de reta é reta");
        };

        assert_eq!(sobe.start, Point2::new(0.0, 2.0));
        assert_eq!(desce.start, Point2::new(0.0, -2.0));
    }

    #[test]
    fn a_paralela_de_reta_guarda_a_distancia_longe_da_origem() {
        let base = 7.0e6;
        let alvo = reta((base, base), (base + 10.0, base));
        let paralela = offset(
            &alvo,
            2.0,
            Point2::new(base + 5.0, base + 1.0),
            Tolerance::default(),
        )
        .expect("a magnitude não impede a paralela");

        let [AnyCurve::Line(sobe)] = paralela.as_slice() else {
            panic!("paralela de reta é reta");
        };

        assert!(
            (sobe.start.y - base - 2.0).abs() < 1e-6,
            "a paralela deveria estar 2 acima, obtido {}",
            sobe.start.y - base
        );
    }

    #[test]
    fn o_ponto_sobre_a_reta_nao_escolhe_lado() {
        let alvo = reta((0.0, 0.0), (10.0, 0.0));

        assert_eq!(
            offset(&alvo, 2.0, Point2::new(5.0, 0.0), Tolerance::default()),
            Err(OffsetError::AmbiguousSide)
        );
    }

    #[test]
    fn a_paralela_por_dentro_encolhe_a_circunferencia() {
        let alvo = AnyCurve::Circle(Circle {
            center: Point2::ORIGIN,
            radius: 10.0,
        });
        let tolerancia = Tolerance::default();

        let dentro = offset(&alvo, 3.0, Point2::new(1.0, 0.0), tolerancia).expect("por dentro");
        let fora = offset(&alvo, 3.0, Point2::new(20.0, 0.0), tolerancia).expect("por fora");

        let [AnyCurve::Circle(menor)] = dentro.as_slice() else {
            panic!("paralela de circunferência é circunferência");
        };
        let [AnyCurve::Circle(maior)] = fora.as_slice() else {
            panic!("paralela de circunferência é circunferência");
        };

        assert_eq!(menor.radius, 7.0);
        assert_eq!(maior.radius, 13.0);
        assert_eq!(menor.center, Point2::ORIGIN);
    }

    #[test]
    fn o_ponto_sobre_a_circunferencia_nao_escolhe_lado() {
        let alvo = AnyCurve::Circle(Circle {
            center: Point2::ORIGIN,
            radius: 10.0,
        });

        assert_eq!(
            offset(&alvo, 3.0, Point2::new(10.0, 0.0), Tolerance::default()),
            Err(OffsetError::AmbiguousSide)
        );
    }

    #[test]
    fn a_paralela_que_anularia_o_raio_e_recusada() {
        let tolerancia = Tolerance::default();
        let dentro = Point2::new(0.5, 0.0);

        let circunferencia = AnyCurve::Circle(Circle {
            center: Point2::ORIGIN,
            radius: 2.0,
        });
        let arco = AnyCurve::Arc(Arc {
            center: Point2::ORIGIN,
            radius: 2.0,
            start_angle: 0.0,
            end_angle: FRAC_PI_2,
        });

        // Além do raio, e exatamente no raio: nos dois casos não sobra arco.
        for distancia in [3.0, 2.0] {
            assert_eq!(
                offset(&circunferencia, distancia, dentro, tolerancia),
                Err(OffsetError::RadiusVanishes)
            );
            assert_eq!(
                offset(&arco, distancia, dentro, tolerancia),
                Err(OffsetError::RadiusVanishes)
            );
        }
    }

    #[test]
    fn a_paralela_de_arco_guarda_os_angulos() {
        let alvo = AnyCurve::Arc(Arc {
            center: Point2::new(1.0, 2.0),
            radius: 5.0,
            start_angle: 0.0,
            end_angle: FRAC_PI_2,
        });

        let fora = offset(&alvo, 2.0, Point2::new(20.0, 2.0), Tolerance::default())
            .expect("por fora sempre cabe");

        let [AnyCurve::Arc(maior)] = fora.as_slice() else {
            panic!("paralela de arco é arco");
        };

        assert_eq!(maior.radius, 7.0);
        assert_eq!(maior.center, Point2::new(1.0, 2.0));
        assert_eq!(maior.start_angle, 0.0);
        assert_eq!(maior.end_angle, FRAC_PI_2);
    }

    #[test]
    fn a_paralela_do_retangulo_por_dentro_apara_os_quatro_cantos() {
        let original = Polyline::from_points(
            [
                Point2::ORIGIN,
                Point2::new(10.0, 0.0),
                Point2::new(10.0, 8.0),
                Point2::new(0.0, 8.0),
            ],
            true,
        );

        let paralela = offset(
            &AnyCurve::Polyline(original.clone()),
            1.0,
            Point2::new(5.0, 4.0),
            Tolerance::default(),
        )
        .expect("um metro para dentro de um retângulo 10x8 cabe");

        let [AnyCurve::Polyline(menor)] = paralela.as_slice() else {
            panic!("paralela de polilinha é polilinha");
        };

        assert!(menor.closed, "o retângulo paralelo continua fechado");
        assert_eq!(
            menor.vertices.len(),
            4,
            "os cantos foram aparados, não duplicados: {:?}",
            menor.vertices
        );

        let esperados = [
            Point2::new(1.0, 1.0),
            Point2::new(9.0, 1.0),
            Point2::new(9.0, 7.0),
            Point2::new(1.0, 7.0),
        ];

        for (obtido, esperado) in menor.points().zip(esperados) {
            assert!(
                obtido.distance_to(esperado) < 1e-9,
                "canto em {obtido:?}, esperado {esperado:?}"
            );
        }
    }

    #[test]
    fn a_paralela_do_retangulo_por_dentro_tambem_cabe_longe_da_origem() {
        let base = 3.0e6;
        let original = Polyline::from_points(
            [
                Point2::new(base, base),
                Point2::new(base + 10.0, base),
                Point2::new(base + 10.0, base + 8.0),
                Point2::new(base, base + 8.0),
            ],
            true,
        );

        let paralela = offset(
            &AnyCurve::Polyline(original.clone()),
            1.0,
            Point2::new(base + 5.0, base + 4.0),
            Tolerance::default(),
        )
        .expect("a magnitude não impede a paralela");

        let [AnyCurve::Polyline(menor)] = paralela.as_slice() else {
            panic!("paralela de polilinha é polilinha");
        };

        assert_eq!(menor.vertices.len(), 4);
        assert!(
            menor
                .points()
                .next()
                .expect("há vértices")
                .distance_to(Point2::new(base + 1.0, base + 1.0))
                < 1e-6
        );
    }

    #[test]
    fn a_paralela_de_polilinha_com_arco_continua_tendo_arco() {
        // Reta, meia-volta e reta de volta: a paralela por fora abre o arco.
        let original = Polyline {
            vertices: vec![
                PolylineVertex::straight(Point2::ORIGIN),
                PolylineVertex::new(Point2::new(10.0, 0.0), 1.0),
                PolylineVertex::straight(Point2::new(10.0, 6.0)),
            ],
            closed: false,
        };

        let paralela = offset(
            &AnyCurve::Polyline(original.clone()),
            1.0,
            Point2::new(5.0, -1.0),
            Tolerance::default(),
        )
        .expect("a paralela por fora sempre cabe");

        let [AnyCurve::Polyline(fora)] = paralela.as_slice() else {
            panic!("paralela de polilinha é polilinha");
        };

        assert!(
            fora.vertices.iter().any(|vertice| vertice.bulge != 0.0),
            "o trecho curvo continua curvo: {:?}",
            fora.vertices
        );
        assert!(
            menor_afastamento(paralela.as_slice(), &original) > 1.0 - 1e-6,
            "nenhum ponto da paralela entra na faixa de um metro"
        );
    }

    #[test]
    fn a_paralela_recusa_quando_um_trecho_de_arco_perderia_o_raio() {
        // Meia-volta de raio 1 no meio do caminho; três metros para dentro dela
        // não sobra arco algum, e a operação inteira recusa.
        let original = Polyline {
            vertices: vec![
                PolylineVertex::straight(Point2::ORIGIN),
                PolylineVertex::new(Point2::new(10.0, 0.0), 1.0),
                PolylineVertex::straight(Point2::new(10.0, 2.0)),
            ],
            closed: false,
        };

        assert_eq!(
            offset(
                &AnyCurve::Polyline(original),
                3.0,
                Point2::new(5.0, 1.0),
                Tolerance::default()
            ),
            Err(OffsetError::RadiusVanishes)
        );
    }

    #[test]
    fn a_paralela_que_engoliria_um_entalhe_vem_aparada() {
        // Um entalhe de 2 de largura por 2 de fundo numa base reta. A paralela
        // a 3 do lado de cima não cabe dentro do entalhe: a emenda ingênua
        // vértice a vértice devolveria uma espícula de ida e volta dentro dele,
        // e a resposta certa é a reta sem entalhe algum.
        let original = Polyline::from_points(
            [
                Point2::ORIGIN,
                Point2::new(4.0, 0.0),
                Point2::new(4.0, -2.0),
                Point2::new(6.0, -2.0),
                Point2::new(6.0, 0.0),
                Point2::new(10.0, 0.0),
            ],
            false,
        );

        let paralela = offset(
            &AnyCurve::Polyline(original.clone()),
            3.0,
            Point2::new(5.0, 5.0),
            Tolerance::default(),
        )
        .expect("acima do entalhe há paralela");

        let [AnyCurve::Polyline(acima)] = paralela.as_slice() else {
            panic!("sobrou um pedaço só: {paralela:?}");
        };

        assert!(
            acima.vertices.iter().all(|vertice| vertice.point.y > 0.0),
            "nenhum vértice desce para dentro do entalhe: {:?}",
            acima.vertices
        );
        assert!(
            acima.vertices.len() < original.vertices.len(),
            "o entalhe sumiu da paralela: {:?}",
            acima.vertices
        );

        let primeiro = acima.points().next().expect("há vértices");
        let ultimo = acima.points().last().expect("há vértices");

        assert!(primeiro.distance_to(Point2::new(0.0, 3.0)) < 1e-9);
        assert!(ultimo.distance_to(Point2::new(10.0, 3.0)) < 1e-9);

        // O critério que define a operação: nada da paralela está mais perto do
        // original do que a distância pedida.
        assert!(
            menor_afastamento(paralela.as_slice(), &original) > 3.0 - 1e-6,
            "algum ponto da paralela invadiu a faixa de três metros"
        );
    }

    #[test]
    fn a_paralela_que_nao_cabe_em_lugar_nenhum_e_recusada() {
        // Um canal de 4 de largura não comporta paralela a 3 de cada parede.
        let original = Polyline::from_points(
            [
                Point2::new(0.0, 4.0),
                Point2::ORIGIN,
                Point2::new(4.0, 0.0),
                Point2::new(4.0, 4.0),
            ],
            false,
        );

        assert_eq!(
            offset(
                &AnyCurve::Polyline(original),
                3.0,
                Point2::new(2.0, 2.0),
                Tolerance::default()
            ),
            Err(OffsetError::NothingLeft)
        );
    }

    #[test]
    fn polilinha_sem_comprimento_nao_tem_paralela() {
        let solto = Polyline::from_points([Point2::ORIGIN], false);

        assert_eq!(
            offset(
                &AnyCurve::Polyline(solto),
                1.0,
                Point2::new(1.0, 1.0),
                Tolerance::default()
            ),
            Err(OffsetError::DegenerateCurve)
        );
    }

    #[test]
    fn os_suportes_paralelos_nao_se_encontram() {
        let primeira = Line {
            start: Point2::ORIGIN,
            end: Point2::new(10.0, 0.0),
        };
        let segunda = Line {
            start: Point2::new(0.0, 5.0),
            end: Point2::new(10.0, 5.0),
        };

        assert!(lines_meet(&primeira, &segunda).is_empty());
        assert_eq!(
            lines_meet(
                &primeira,
                &Line {
                    start: Point2::new(3.0, -1.0),
                    end: Point2::new(3.0, 1.0),
                }
            ),
            vec![Point2::new(3.0, 0.0)]
        );
    }

    #[test]
    fn a_paralela_de_meia_volta_fechada_gira_do_lado_certo() {
        // Um "estádio": duas retas ligadas por duas meias-voltas, fechado.
        let original = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::new(0.0, -2.0), -1.0),
                PolylineVertex::straight(Point2::new(0.0, 2.0)),
                PolylineVertex::new(Point2::new(8.0, 2.0), -1.0),
                PolylineVertex::straight(Point2::new(8.0, -2.0)),
            ],
            closed: true,
        };

        let paralela = offset(
            &AnyCurve::Polyline(original.clone()),
            1.0,
            Point2::new(4.0, 0.0),
            Tolerance::default(),
        )
        .expect("um metro para dentro do estádio cabe");

        let [AnyCurve::Polyline(dentro)] = paralela.as_slice() else {
            panic!("sobrou um pedaço só: {paralela:?}");
        };

        assert!(dentro.closed, "a paralela de curva fechada é fechada");
        assert_eq!(
            dentro
                .vertices
                .iter()
                .filter(|vertice| vertice.bulge != 0.0)
                .count(),
            2,
            "as duas meias-voltas continuam curvas: {:?}",
            dentro.vertices
        );

        // Raio 2 encolhido de 1 dá 1: meia-volta de raio 1 tem comprimento π.
        let comprimento: f64 = dentro.segments().map(|trecho| trecho.length()).sum();

        assert!(
            (comprimento - (2.0 * PI + 16.0)).abs() < 1e-6,
            "duas meias-voltas de raio 1 e dois lados de 8, obtido {comprimento}"
        );
    }
}
