// Caminho relativo: kernel/neocad-geometry/src/intersect.rs
//! \file kernel/neocad-geometry/src/intersect.rs
//! \brief Interseção entre as curvas 2D do kernel.
//! \author Iago Leal
//! \date 2026-09-07
//!
//! Aparo, extensão, concordância e o snap de interseção são todos consumidores
//! deste módulo — é a operação de que o resto da fase K3 depende.
//!
//! # Onde entra a tolerância e onde entra o sinal
//!
//! As duas perguntas do ADR 0006 aparecem aqui lado a lado. **Se as curvas se
//! tocam** é pergunta de tolerância: uma reta tangente a uma circunferência
//! passa a um zero de distância que nenhum cálculo em `f64` produz exatamente.
//! **De que lado um extremo está** é pergunta de sinal, e vai para o predicado
//! exato: é ele que decide se dois segmentos se cruzam, porque um epsilon ali
//! deixaria a resposta depender da ordem dos argumentos.

use core::f64::consts::TAU;

use crate::{orientation, Arc, Circle, Curve, Line, Orientation, Point2, Polyline, Tolerance};

/// Resultado de uma interseção.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Intersections {
    /// Pontos de cruzamento ou de tangência, sem repetição, ordenados pelo
    /// parâmetro ao longo da **primeira** curva.
    ///
    /// Quando há sobreposição, traz os extremos do trecho comum — que é o mais
    /// próximo de uma resposta finita para um conjunto infinito de pontos.
    pub points: Vec<Point2>,
    /// Verdadeiro quando as duas curvas coincidem num trecho de comprimento não
    /// nulo.
    ///
    /// Duas circunferências idênticas coincidem sem terem extremos, e nesse caso
    /// `points` fica vazio: não há ponto que resuma a coincidência.
    pub overlapping: bool,
}

impl Intersections {
    /// Nenhuma interseção.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Se não há ponto nem sobreposição.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty() && !self.overlapping
    }
}

/// Qualquer curva do kernel, para que a interseção possa despachar por par.
///
/// O traço [`Curve`] responde por uma curva de cada vez; interseção é sobre
/// duas, e casar dois traços não se faz sem enumerar os casos.
#[derive(Debug, Clone, PartialEq)]
pub enum AnyCurve {
    /// Segmento de reta.
    Line(Line),
    /// Circunferência.
    Circle(Circle),
    /// Arco.
    Arc(Arc),
    /// Polilinha.
    Polyline(Polyline),
}

impl From<Line> for AnyCurve {
    fn from(curve: Line) -> Self {
        Self::Line(curve)
    }
}

impl From<Circle> for AnyCurve {
    fn from(curve: Circle) -> Self {
        Self::Circle(curve)
    }
}

impl From<Arc> for AnyCurve {
    fn from(curve: Arc) -> Self {
        Self::Arc(curve)
    }
}

impl From<Polyline> for AnyCurve {
    fn from(curve: Polyline) -> Self {
        Self::Polyline(curve)
    }
}

impl Curve for AnyCurve {
    fn point_at(&self, parameter: f64) -> Point2 {
        match self {
            Self::Line(curve) => curve.point_at(parameter),
            Self::Circle(curve) => curve.point_at(parameter),
            Self::Arc(curve) => curve.point_at(parameter),
            Self::Polyline(curve) => curve.point_at(parameter),
        }
    }

    fn tangent_at(&self, parameter: f64) -> Option<f64> {
        match self {
            Self::Line(curve) => curve.tangent_at(parameter),
            Self::Circle(curve) => curve.tangent_at(parameter),
            Self::Arc(curve) => curve.tangent_at(parameter),
            Self::Polyline(curve) => curve.tangent_at(parameter),
        }
    }

    fn length(&self) -> f64 {
        match self {
            Self::Line(curve) => curve.length(),
            Self::Circle(curve) => curve.length(),
            Self::Arc(curve) => curve.length(),
            Self::Polyline(curve) => curve.length(),
        }
    }

    fn closest_point(&self, target: Point2) -> crate::ClosestPoint {
        match self {
            Self::Line(curve) => curve.closest_point(target),
            Self::Circle(curve) => curve.closest_point(target),
            Self::Arc(curve) => curve.closest_point(target),
            Self::Polyline(curve) => curve.closest_point(target),
        }
    }

    fn bounding_box(&self) -> crate::Aabb {
        match self {
            Self::Line(curve) => curve.bounding_box(),
            Self::Circle(curve) => curve.bounding_box(),
            Self::Arc(curve) => curve.bounding_box(),
            Self::Polyline(curve) => curve.bounding_box(),
        }
    }
}

/// Interseção entre duas curvas.
///
/// # Exemplo
///
/// ```
/// use neocad_geometry::{intersect, AnyCurve, Line, Point2, Tolerance};
///
/// let horizontal = Line {
///     start: Point2::new(-1.0, 0.0),
///     end: Point2::new(1.0, 0.0),
/// };
/// let vertical = Line {
///     start: Point2::new(0.0, -1.0),
///     end: Point2::new(0.0, 1.0),
/// };
///
/// let encontro = intersect(
///     &AnyCurve::from(horizontal),
///     &AnyCurve::from(vertical),
///     Tolerance::default(),
/// );
///
/// assert_eq!(encontro.points, [Point2::ORIGIN]);
/// assert!(!encontro.overlapping);
/// ```
#[must_use]
pub fn intersect(a: &AnyCurve, b: &AnyCurve, tolerance: Tolerance) -> Intersections {
    let mut found = dispatch(a, b, tolerance);

    dedup(&mut found.points, tolerance);
    sort_along(&mut found.points, a);

    found
}

/// Encaminha o par para o cálculo que lhe cabe.
fn dispatch(a: &AnyCurve, b: &AnyCurve, tolerance: Tolerance) -> Intersections {
    match (a, b) {
        (AnyCurve::Polyline(polyline), other) => {
            against_polyline(polyline, other, tolerance, false)
        }
        (other, AnyCurve::Polyline(polyline)) => against_polyline(polyline, other, tolerance, true),
        (AnyCurve::Line(first), AnyCurve::Line(second)) => line_line(first, second, tolerance),
        (AnyCurve::Line(line), AnyCurve::Circle(circle)) => line_circle(line, circle, tolerance),
        (AnyCurve::Circle(circle), AnyCurve::Line(line)) => line_circle(line, circle, tolerance),
        (AnyCurve::Line(line), AnyCurve::Arc(arc)) => on_arcs(
            line_circle(line, &circle_of(arc), tolerance),
            &[arc],
            tolerance,
        ),
        (AnyCurve::Arc(arc), AnyCurve::Line(line)) => on_arcs(
            line_circle(line, &circle_of(arc), tolerance),
            &[arc],
            tolerance,
        ),
        (AnyCurve::Circle(first), AnyCurve::Circle(second)) => {
            circle_circle(first, second, tolerance)
        }
        (AnyCurve::Circle(circle), AnyCurve::Arc(arc)) => on_arcs(
            circle_circle(circle, &circle_of(arc), tolerance),
            &[arc],
            tolerance,
        ),
        (AnyCurve::Arc(arc), AnyCurve::Circle(circle)) => on_arcs(
            circle_circle(&circle_of(arc), circle, tolerance),
            &[arc],
            tolerance,
        ),
        (AnyCurve::Arc(first), AnyCurve::Arc(second)) => arc_arc(first, second, tolerance),
    }
}

/// A circunferência de suporte de um arco.
fn circle_of(arc: &Arc) -> Circle {
    Circle {
        center: arc.center,
        radius: arc.radius.abs(),
    }
}

/// Descarta os pontos que caem fora da varredura dos arcos informados.
///
/// A pertinência é medida pela distância ao arco, e não pelo ângulo, porque
/// distância é o que a tolerância do kernel sabe julgar (ADR 0006).
fn on_arcs(found: Intersections, arcs: &[&Arc], tolerance: Tolerance) -> Intersections {
    let points = found
        .points
        .into_iter()
        .filter(|point| {
            arcs.iter().all(|arc| {
                let magnitude = magnitude_of(*point).max(magnitude_of(arc.center));

                arc.closest_point(*point).distance <= tolerance.effective_linear(magnitude)
            })
        })
        .collect();

    Intersections {
        points,
        overlapping: found.overlapping,
    }
}

/// Maior coordenada em módulo, que é a magnitude que a tolerância usa.
fn magnitude_of(point: Point2) -> f64 {
    point.x.abs().max(point.y.abs())
}

/// Interseção de uma polilinha com outra curva, trecho a trecho.
fn against_polyline(
    polyline: &Polyline,
    other: &AnyCurve,
    tolerance: Tolerance,
    reversed: bool,
) -> Intersections {
    let mut points = Vec::new();
    let mut overlapping = false;

    for segment in polyline.segments() {
        let as_curve = match segment {
            crate::PolylineSegment::Straight(line) => AnyCurve::Line(line),
            crate::PolylineSegment::Curved { arc, .. } => AnyCurve::Arc(arc),
        };

        let found = if reversed {
            dispatch(other, &as_curve, tolerance)
        } else {
            dispatch(&as_curve, other, tolerance)
        };

        points.extend(found.points);
        overlapping |= found.overlapping;
    }

    Intersections {
        points,
        overlapping,
    }
}

/// Remove pontos repetidos, dentro da tolerância.
///
/// Trechos vizinhos de uma polilinha compartilham um vértice, e uma curva que
/// passe por ele apareceria duas vezes.
fn dedup(points: &mut Vec<Point2>, tolerance: Tolerance) {
    let mut unique: Vec<Point2> = Vec::with_capacity(points.len());

    for point in points.drain(..) {
        if !unique
            .iter()
            .any(|kept| tolerance.points_equal(*kept, point))
        {
            unique.push(point);
        }
    }

    *points = unique;
}

/// Ordena os pontos pelo parâmetro ao longo da curva informada.
///
/// Sem uma ordem declarada, o resultado dependeria da ordem em que os cálculos
/// produziram os pontos — e um aparo teria de adivinhar qual pedaço é qual.
fn sort_along(points: &mut [Point2], along: &AnyCurve) {
    points.sort_by(|first, second| {
        along
            .closest_point(*first)
            .parameter
            .total_cmp(&along.closest_point(*second).parameter)
    });
}

/// Interseção entre dois segmentos de reta.
fn line_line(a: &Line, b: &Line, tolerance: Tolerance) -> Intersections {
    // Segmento degenerado é um ponto, e a pergunta vira "o ponto está na outra".
    if a.start == a.end {
        return point_on_curve(a.start, b, tolerance);
    }

    if b.start == b.end {
        return point_on_curve(b.start, a, tolerance);
    }

    let first = orientation(a.start, a.end, b.start);
    let second = orientation(a.start, a.end, b.end);
    let third = orientation(b.start, b.end, a.start);
    let fourth = orientation(b.start, b.end, a.end);

    // Quatro extremos colineares: as duas retas são a mesma, e o que resta é
    // achar o trecho comum.
    if [first, second, third, fourth]
        .iter()
        .all(|side| *side == Orientation::Collinear)
    {
        return collinear_overlap(a, b, tolerance);
    }

    // Cruzamento próprio: cada segmento separa os extremos do outro. É aqui que
    // o predicado exato paga o seu preço — a decisão não usa epsilon nenhum.
    if first != second && third != fourth {
        if let Some(point) = crossing_point(a, b) {
            return Intersections {
                points: vec![point],
                overlapping: false,
            };
        }
    }

    // Encosto: um extremo de um está sobre o outro sem que haja cruzamento.
    let mut points = Vec::new();

    for (point, host) in [(b.start, a), (b.end, a), (a.start, b), (a.end, b)] {
        let magnitude = magnitude_of(point);

        if host.closest_point(point).distance <= tolerance.effective_linear(magnitude) {
            points.push(point);
        }
    }

    Intersections {
        points,
        overlapping: false,
    }
}

/// Ponto de cruzamento de duas retas que sabidamente se cruzam.
fn crossing_point(a: &Line, b: &Line) -> Option<Point2> {
    let (adx, ady) = (a.end.x - a.start.x, a.end.y - a.start.y);
    let (bdx, bdy) = (b.end.x - b.start.x, b.end.y - b.start.y);
    let denominator = adx * bdy - ady * bdx;

    if denominator == 0.0 || !denominator.is_finite() {
        return None;
    }

    let (dx, dy) = (b.start.x - a.start.x, b.start.y - a.start.y);
    let parameter = (dx * bdy - dy * bdx) / denominator;

    Some(Point2::new(
        a.start.x + adx * parameter,
        a.start.y + ady * parameter,
    ))
}

/// Trecho comum a dois segmentos que estão sobre a mesma reta.
fn collinear_overlap(a: &Line, b: &Line, tolerance: Tolerance) -> Intersections {
    let (dx, dy) = (a.end.x - a.start.x, a.end.y - a.start.y);
    let squared = dx * dx + dy * dy;
    let projected =
        |point: Point2| ((point.x - a.start.x) * dx + (point.y - a.start.y) * dy) / squared;

    let (mut low, mut high) = (projected(b.start), projected(b.end));

    if low > high {
        core::mem::swap(&mut low, &mut high);
    }

    let start = low.max(0.0);
    let end = high.min(1.0);
    let length = a.length();

    // A tolerância é linear; aqui compara-se parâmetro, então ela é convertida
    // pela extensão do segmento.
    let slack = if length > 0.0 {
        tolerance.effective_linear(magnitude_of(a.start).max(magnitude_of(a.end))) / length
    } else {
        0.0
    };

    if end < start - slack {
        return Intersections::none();
    }

    if end - start <= slack {
        // Encostam num ponto só.
        return Intersections {
            points: vec![a.point_at((start + end) / 2.0)],
            overlapping: false,
        };
    }

    Intersections {
        points: vec![a.point_at(start), a.point_at(end)],
        overlapping: true,
    }
}

/// Resposta para "este ponto está sobre aquela curva?".
fn point_on_curve(point: Point2, host: &Line, tolerance: Tolerance) -> Intersections {
    let magnitude = magnitude_of(point).max(magnitude_of(host.start));

    if host.closest_point(point).distance <= tolerance.effective_linear(magnitude) {
        Intersections {
            points: vec![point],
            overlapping: false,
        }
    } else {
        Intersections::none()
    }
}

/// Interseção entre um segmento de reta e uma circunferência.
fn line_circle(line: &Line, circle: &Circle, tolerance: Tolerance) -> Intersections {
    let radius = circle.radius.abs();
    let (dx, dy) = (line.end.x - line.start.x, line.end.y - line.start.y);
    let squared = dx * dx + dy * dy;

    if squared == 0.0 {
        let distance = (line.start.distance_to(circle.center) - radius).abs();
        let magnitude = magnitude_of(line.start).max(magnitude_of(circle.center));

        return if distance <= tolerance.effective_linear(magnitude) {
            Intersections {
                points: vec![line.start],
                overlapping: false,
            }
        } else {
            Intersections::none()
        };
    }

    let length = squared.sqrt();
    let magnitude = magnitude_of(line.start)
        .max(magnitude_of(line.end))
        .max(magnitude_of(circle.center));
    let linear = tolerance.effective_linear(magnitude);

    // Pé da perpendicular do centro sobre a reta de suporte.
    let foot =
        ((circle.center.x - line.start.x) * dx + (circle.center.y - line.start.y) * dy) / squared;
    let closest = Point2::new(line.start.x + dx * foot, line.start.y + dy * foot);
    let gap = closest.distance_to(circle.center);

    if gap > radius + linear {
        return Intersections::none();
    }

    // Tangência é pergunta de coincidência, e portanto de tolerância: a
    // distância exata nunca sai zero de um cálculo em ponto flutuante.
    let parameters = if (gap - radius).abs() <= linear {
        vec![foot]
    } else {
        let half = (radius * radius - gap * gap).max(0.0).sqrt() / length;

        vec![foot - half, foot + half]
    };

    let slack = linear / length;
    let points = parameters
        .into_iter()
        .filter(|parameter| *parameter >= -slack && *parameter <= 1.0 + slack)
        .map(|parameter| Point2::new(line.start.x + dx * parameter, line.start.y + dy * parameter))
        .collect();

    Intersections {
        points,
        overlapping: false,
    }
}

/// Interseção entre duas circunferências.
fn circle_circle(a: &Circle, b: &Circle, tolerance: Tolerance) -> Intersections {
    let (first, second) = (a.radius.abs(), b.radius.abs());
    let distance = a.center.distance_to(b.center);
    let magnitude = magnitude_of(a.center).max(magnitude_of(b.center));
    let linear = tolerance.effective_linear(magnitude);

    if distance <= linear {
        // Concêntricas: ou são a mesma, e coincidem em toda a volta, ou não se
        // encontram em ponto algum.
        return if (first - second).abs() <= linear {
            Intersections {
                points: Vec::new(),
                overlapping: true,
            }
        } else {
            Intersections::none()
        };
    }

    if distance > first + second + linear || distance < (first - second).abs() - linear {
        return Intersections::none();
    }

    let (ux, uy) = (
        (b.center.x - a.center.x) / distance,
        (b.center.y - a.center.y) / distance,
    );

    // Tangência externa: o ponto fica entre os dois centros.
    if (distance - (first + second)).abs() <= linear {
        return Intersections {
            points: vec![Point2::new(
                a.center.x + ux * first,
                a.center.y + uy * first,
            )],
            overlapping: false,
        };
    }

    // Tangência interna: uma circunferência está dentro da outra e a toca. O
    // ponto fica do lado do centro menor, que é o que troca o sinal.
    if (distance - (first - second).abs()).abs() <= linear {
        let along = if first > second { first } else { -first };

        return Intersections {
            points: vec![Point2::new(
                a.center.x + ux * along,
                a.center.y + uy * along,
            )],
            overlapping: false,
        };
    }

    let along = (distance * distance + first * first - second * second) / (2.0 * distance);
    let height = (first * first - along * along).max(0.0).sqrt();
    let base = Point2::new(a.center.x + ux * along, a.center.y + uy * along);

    Intersections {
        points: vec![
            Point2::new(base.x - uy * height, base.y + ux * height),
            Point2::new(base.x + uy * height, base.y - ux * height),
        ],
        overlapping: false,
    }
}

/// Interseção entre dois arcos.
fn arc_arc(a: &Arc, b: &Arc, tolerance: Tolerance) -> Intersections {
    let magnitude = magnitude_of(a.center).max(magnitude_of(b.center));
    let linear = tolerance.effective_linear(magnitude);
    let same_circle = a.center.distance_to(b.center) <= linear
        && (a.radius.abs() - b.radius.abs()).abs() <= linear;

    if same_circle {
        return coincident_arcs(a, b, tolerance);
    }

    on_arcs(
        circle_circle(&circle_of(a), &circle_of(b), tolerance),
        &[a, b],
        tolerance,
    )
}

/// Trecho comum a dois arcos da mesma circunferência.
///
/// A conta é feita no referencial do primeiro arco: o segundo entra como um
/// intervalo deslocado, e a volta completa obriga a testar também a cópia desse
/// intervalo uma volta atrás.
fn coincident_arcs(a: &Arc, b: &Arc, tolerance: Tolerance) -> Intersections {
    let radius = a.radius.abs();
    let (first, second) = (a.sweep(), b.sweep());

    // Arco de varredura nula é um ponto, e a pergunta vira de pertinência.
    if first == 0.0 || second == 0.0 {
        let (point, host) = if first == 0.0 {
            (a.point_at(0.0), b)
        } else {
            (b.point_at(0.0), a)
        };
        let magnitude = magnitude_of(point).max(magnitude_of(host.center));

        return if host.closest_point(point).distance <= tolerance.effective_linear(magnitude) {
            Intersections {
                points: vec![point],
                overlapping: false,
            }
        } else {
            Intersections::none()
        };
    }

    let offset = (b.start_angle - a.start_angle).rem_euclid(TAU);

    // Tolerância angular derivada do raio: é a distância que se está julgando,
    // e não a direção (ADR 0006).
    let magnitude = magnitude_of(a.center).max(radius);
    let slack = if radius > 0.0 {
        tolerance.effective_linear(magnitude) / radius
    } else {
        0.0
    };

    let mut points = Vec::new();
    let mut overlapping = false;

    for shifted in [offset, offset - TAU] {
        let start = shifted.max(0.0);
        let end = (shifted + second).min(first);

        if end < start - slack {
            continue;
        }

        if end - start <= slack {
            points.push(a.point_at(((start + end) / 2.0) / first));
            continue;
        }

        overlapping = true;
        points.push(a.point_at(start / first));
        points.push(a.point_at(end / first));
    }

    Intersections {
        points,
        overlapping,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Polyline, PolylineVertex};
    use core::f64::consts::{FRAC_PI_2, PI};

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    /// Roda o mesmo caso perto da origem e em coordenada da ordem de `1e6`.
    fn nas_duas_magnitudes(caso: impl Fn(f64)) {
        for base in [0.0, 7.0e6] {
            caso(base);
        }
    }

    fn reta(base: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> Line {
        Line {
            start: Point2::new(base + x1, base + y1),
            end: Point2::new(base + x2, base + y2),
        }
    }

    fn encontro(a: impl Into<AnyCurve>, b: impl Into<AnyCurve>) -> Intersections {
        intersect(&a.into(), &b.into(), tol())
    }

    #[test]
    fn duas_retas_que_se_cruzam_dao_um_ponto() {
        nas_duas_magnitudes(|base| {
            let achado = encontro(
                reta(base, -1.0, 0.0, 1.0, 0.0),
                reta(base, 0.0, -1.0, 0.0, 1.0),
            );

            assert_eq!(achado.points.len(), 1, "base {base}");
            assert!(tol().points_equal(achado.points[0], Point2::new(base, base)));
            assert!(!achado.overlapping);
        });
    }

    #[test]
    fn retas_paralelas_nao_se_encontram() {
        nas_duas_magnitudes(|base| {
            let achado = encontro(
                reta(base, 0.0, 0.0, 10.0, 0.0),
                reta(base, 0.0, 3.0, 10.0, 3.0),
            );

            assert!(achado.is_empty(), "base {base}: {achado:?}");
        });
    }

    #[test]
    fn segmentos_que_nao_se_alcancam_nao_se_encontram() {
        // As retas de suporte se cruzam; os segmentos, não. É o caso que separa
        // interseção de reta de interseção de segmento.
        let achado = encontro(
            Line {
                start: Point2::ORIGIN,
                end: Point2::new(1.0, 0.0),
            },
            Line {
                start: Point2::new(5.0, -1.0),
                end: Point2::new(5.0, 1.0),
            },
        );

        assert!(achado.is_empty(), "{achado:?}");
    }

    #[test]
    fn colineares_sobrepostos_devolvem_o_trecho_comum() {
        nas_duas_magnitudes(|base| {
            let achado = encontro(
                reta(base, 0.0, 0.0, 10.0, 0.0),
                reta(base, 4.0, 0.0, 20.0, 0.0),
            );

            assert!(achado.overlapping, "base {base}: {achado:?}");
            assert_eq!(achado.points.len(), 2);
            assert!(tol().points_equal(achado.points[0], Point2::new(base + 4.0, base)));
            assert!(tol().points_equal(achado.points[1], Point2::new(base + 10.0, base)));
        });
    }

    #[test]
    fn colineares_que_apenas_se_encostam_dao_um_ponto_sem_sobreposicao() {
        let achado = encontro(
            Line {
                start: Point2::ORIGIN,
                end: Point2::new(10.0, 0.0),
            },
            Line {
                start: Point2::new(10.0, 0.0),
                end: Point2::new(20.0, 0.0),
            },
        );

        assert!(!achado.overlapping, "{achado:?}");
        assert_eq!(achado.points.len(), 1);
        assert!(tol().points_equal(achado.points[0], Point2::new(10.0, 0.0)));
    }

    #[test]
    fn colineares_separados_nao_se_encontram() {
        let achado = encontro(
            Line {
                start: Point2::ORIGIN,
                end: Point2::new(1.0, 0.0),
            },
            Line {
                start: Point2::new(5.0, 0.0),
                end: Point2::new(6.0, 0.0),
            },
        );

        assert!(achado.is_empty(), "{achado:?}");
    }

    #[test]
    fn a_reta_tangente_toca_a_circunferencia_num_ponto_so() {
        nas_duas_magnitudes(|base| {
            let circunferencia = Circle {
                center: Point2::new(base, base),
                radius: 2.0,
            };
            let tangente = reta(base, -5.0, 2.0, 5.0, 2.0);
            let achado = encontro(tangente, circunferencia);

            assert_eq!(achado.points.len(), 1, "base {base}: {achado:?}");
            assert!(tol().points_equal(achado.points[0], Point2::new(base, base + 2.0)));
        });
    }

    #[test]
    fn a_reta_secante_corta_a_circunferencia_em_dois_pontos_ordenados() {
        let circunferencia = Circle {
            center: Point2::ORIGIN,
            radius: 2.0,
        };
        let secante = Line {
            start: Point2::new(-5.0, 0.0),
            end: Point2::new(5.0, 0.0),
        };
        let achado = encontro(secante, circunferencia);

        assert_eq!(achado.points.len(), 2);
        // A ordem segue o percurso da primeira curva, da esquerda para a direita.
        assert!(tol().points_equal(achado.points[0], Point2::new(-2.0, 0.0)));
        assert!(tol().points_equal(achado.points[1], Point2::new(2.0, 0.0)));
    }

    #[test]
    fn a_reta_que_passa_longe_nao_encontra_a_circunferencia() {
        let achado = encontro(
            Line {
                start: Point2::new(-5.0, 9.0),
                end: Point2::new(5.0, 9.0),
            },
            Circle {
                center: Point2::ORIGIN,
                radius: 2.0,
            },
        );

        assert!(achado.is_empty(), "{achado:?}");
    }

    #[test]
    fn circunferencias_tangentes_por_fora_tocam_entre_os_centros() {
        let achado = encontro(
            Circle {
                center: Point2::ORIGIN,
                radius: 2.0,
            },
            Circle {
                center: Point2::new(5.0, 0.0),
                radius: 3.0,
            },
        );

        assert_eq!(achado.points.len(), 1, "{achado:?}");
        assert!(tol().points_equal(achado.points[0], Point2::new(2.0, 0.0)));
    }

    #[test]
    fn circunferencias_tangentes_por_dentro_tocam_do_lado_do_menor() {
        // A menor está dentro da maior; o toque fica do lado oposto ao
        // deslocamento, e é aí que o sinal do cálculo precisa estar certo.
        let maior = Circle {
            center: Point2::ORIGIN,
            radius: 5.0,
        };
        let menor = Circle {
            center: Point2::new(2.0, 0.0),
            radius: 3.0,
        };

        let direto = encontro(maior, menor);
        let invertido = encontro(menor, maior);

        assert_eq!(direto.points.len(), 1, "{direto:?}");
        assert!(tol().points_equal(direto.points[0], Point2::new(5.0, 0.0)));

        assert_eq!(invertido.points.len(), 1, "{invertido:?}");
        assert!(tol().points_equal(invertido.points[0], Point2::new(5.0, 0.0)));
    }

    #[test]
    fn circunferencias_secantes_dao_dois_pontos() {
        let achado = encontro(
            Circle {
                center: Point2::ORIGIN,
                radius: 5.0,
            },
            Circle {
                center: Point2::new(6.0, 0.0),
                radius: 5.0,
            },
        );

        assert_eq!(achado.points.len(), 2, "{achado:?}");
        assert!(achado
            .points
            .iter()
            .all(|p| tol().lengths_equal(p.x, 3.0) && tol().lengths_equal(p.y.abs(), 4.0)));
    }

    #[test]
    fn circunferencias_iguais_coincidem_sem_pontos_de_referencia() {
        let uma = Circle {
            center: Point2::new(1.0, 1.0),
            radius: 4.0,
        };
        let achado = encontro(uma, uma);

        assert!(achado.overlapping, "{achado:?}");
        assert!(achado.points.is_empty());
    }

    #[test]
    fn circunferencias_concentricas_de_raios_diferentes_nao_se_encontram() {
        let achado = encontro(
            Circle {
                center: Point2::ORIGIN,
                radius: 2.0,
            },
            Circle {
                center: Point2::ORIGIN,
                radius: 5.0,
            },
        );

        assert!(achado.is_empty(), "{achado:?}");
    }

    #[test]
    fn o_cruzamento_fora_da_varredura_do_arco_nao_conta() {
        let arco = Arc {
            center: Point2::ORIGIN,
            radius: 2.0,
            start_angle: 0.0,
            end_angle: FRAC_PI_2,
        };
        let secante = Line {
            start: Point2::new(-5.0, 0.0),
            end: Point2::new(5.0, 0.0),
        };

        // A reta corta a circunferência em (−2,0) e (2,0); só o segundo está no
        // primeiro quadrante, que é onde o arco existe.
        let achado = encontro(secante, arco);

        assert_eq!(achado.points.len(), 1, "{achado:?}");
        assert!(tol().points_equal(achado.points[0], Point2::new(2.0, 0.0)));
    }

    #[test]
    fn arcos_do_mesmo_circulo_que_se_sobrepoem_devolvem_o_trecho_comum() {
        let primeiro = Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle: 0.0,
            end_angle: PI,
        };
        let segundo = Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle: FRAC_PI_2,
            end_angle: 3.0 * PI / 2.0,
        };
        let achado = encontro(primeiro, segundo);

        assert!(achado.overlapping, "{achado:?}");
        assert_eq!(achado.points.len(), 2);
        assert!(tol().points_equal(achado.points[0], Point2::new(0.0, 1.0)));
        assert!(tol().points_equal(achado.points[1], Point2::new(-1.0, 0.0)));
    }

    #[test]
    fn arcos_do_mesmo_circulo_que_so_se_encostam_dao_um_ponto() {
        let primeiro = Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle: 0.0,
            end_angle: FRAC_PI_2,
        };
        let segundo = Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle: FRAC_PI_2,
            end_angle: PI,
        };
        let achado = encontro(primeiro, segundo);

        assert!(!achado.overlapping, "{achado:?}");
        assert_eq!(achado.points.len(), 1);
        assert!(tol().points_equal(achado.points[0], Point2::new(0.0, 1.0)));
    }

    #[test]
    fn arcos_do_mesmo_circulo_sem_trecho_comum_nao_se_encontram() {
        let primeiro = Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle: 0.0,
            end_angle: FRAC_PI_2,
        };
        let segundo = Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle: PI,
            end_angle: 3.0 * PI / 2.0,
        };

        assert!(encontro(primeiro, segundo).is_empty());
    }

    #[test]
    fn a_polilinha_encontra_a_reta_em_cada_trecho_atravessado() {
        // Uma escada de dois degraus cortada por uma horizontal.
        let polilinha = Polyline::from_points(
            [
                Point2::new(0.0, -1.0),
                Point2::new(0.0, 1.0),
                Point2::new(2.0, 1.0),
                Point2::new(2.0, -1.0),
            ],
            false,
        );
        let corte = Line {
            start: Point2::new(-1.0, 0.0),
            end: Point2::new(3.0, 0.0),
        };
        let achado = encontro(polilinha, corte);

        assert_eq!(achado.points.len(), 2, "{achado:?}");
        assert!(tol().points_equal(achado.points[0], Point2::new(0.0, 0.0)));
        assert!(tol().points_equal(achado.points[1], Point2::new(2.0, 0.0)));
    }

    #[test]
    fn o_vertice_compartilhado_nao_conta_duas_vezes() {
        // A reta passa exatamente pelo vértice que os dois trechos dividem.
        let polilinha = Polyline::from_points(
            [
                Point2::new(-1.0, 1.0),
                Point2::new(0.0, 0.0),
                Point2::new(1.0, 1.0),
            ],
            false,
        );
        let corte = Line {
            start: Point2::new(-2.0, 0.0),
            end: Point2::new(2.0, 0.0),
        };
        let achado = encontro(polilinha, corte);

        assert_eq!(achado.points.len(), 1, "{achado:?}");
        assert!(tol().points_equal(achado.points[0], Point2::ORIGIN));
    }

    #[test]
    fn o_trecho_curvo_da_polilinha_tambem_e_atravessado() {
        // Meia-volta abaixo da corda, cortada por uma horizontal que só a
        // encontra por causa do abaulamento.
        let polilinha = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 1.0),
                PolylineVertex::straight(Point2::new(2.0, 0.0)),
            ],
            closed: false,
        };
        let corte = Line {
            start: Point2::new(-1.0, -0.5),
            end: Point2::new(3.0, -0.5),
        };
        let achado = encontro(polilinha, corte);

        assert_eq!(achado.points.len(), 2, "{achado:?}");
        assert!(achado.points.iter().all(|p| tol().lengths_equal(p.y, -0.5)));
    }

    #[test]
    fn a_ordem_dos_pontos_segue_a_primeira_curva() {
        let circunferencia = Circle {
            center: Point2::ORIGIN,
            radius: 2.0,
        };
        let da_direita = Line {
            start: Point2::new(5.0, 0.0),
            end: Point2::new(-5.0, 0.0),
        };
        let achado = encontro(da_direita, circunferencia);

        // Percorrida da direita para a esquerda, a reta encontra (2,0) antes.
        assert!(tol().points_equal(achado.points[0], Point2::new(2.0, 0.0)));
        assert!(tol().points_equal(achado.points[1], Point2::new(-2.0, 0.0)));
    }
}
