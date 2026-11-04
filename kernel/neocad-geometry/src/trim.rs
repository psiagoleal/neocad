// Caminho relativo: kernel/neocad-geometry/src/trim.rs
//! \file kernel/neocad-geometry/src/trim.rs
//! \brief Aparar uma curva contra arestas de corte e estendê-la até uma fronteira.
//! \author Iago Leal
//! \date 2026-10-29
//!
//! Spec 05, critérios E7 a E9. A escolha de **qual pedaço** sai ou de **qual
//! ponta** cresce é feita pelo ponto que o usuário indica, como no AutoCAD.

use core::f64::consts::TAU;
use core::fmt;

use crate::curve::PolylineSegment;
use crate::offset::{polyline_from, sub_piece};
use crate::{intersect, AnyCurve, Arc, Curve, Line, Point2, Tolerance};

/// Por que um aparo ou uma extensão não pôde ser feito.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimError {
    /// O ponto indicado está mais longe da curva do que a tolerância de captura.
    PickOffCurve,
    /// Nenhuma aresta de corte cruza a curva num ponto que a divida.
    NoCuttingEdge,
    /// Prolongada pela ponta indicada, a curva não alcança fronteira alguma.
    NoBoundary,
    /// A curva é fechada (circunferência, polilinha fechada) e não tem ponta.
    NotExtendable,
}

impl fmt::Display for TrimError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PickOffCurve => "o ponto indicado não está sobre a curva",
            Self::NoCuttingEdge => "nenhuma aresta de corte divide a curva",
            Self::NoBoundary => "a curva prolongada não alcança fronteira alguma",
            Self::NotExtendable => "curva fechada não tem ponta para estender",
        })
    }
}

impl core::error::Error for TrimError {}

/// Apara a curva, removendo o trecho indicado por `pick`.
///
/// O trecho removido vai da interseção com as arestas de corte mais próxima de
/// cada lado do ponto; sem interseção de um lado, vai até a ponta livre. O que
/// sobra volta como lista: nenhuma, uma ou duas curvas.
///
/// # Errors
///
/// [`TrimError::PickOffCurve`] se `pick` estiver a mais de `capture` da curva;
/// [`TrimError::NoCuttingEdge`] se nenhuma aresta dividir a curva.
pub fn trim(
    curve: &AnyCurve,
    cutting_edges: &[AnyCurve],
    pick: Point2,
    capture: f64,
    tolerance: Tolerance,
) -> Result<Vec<AnyCurve>, TrimError> {
    let picked = curve.closest_point(pick);

    if picked.distance > capture {
        return Err(TrimError::PickOffCurve);
    }

    let closed = is_closed(curve);
    let cuts = cut_parameters(curve, cutting_edges, closed, tolerance);
    let t = picked.parameter;

    if closed {
        if cuts.len() < 2 {
            return Err(TrimError::NoCuttingEdge);
        }

        // Os vizinhos do ponto, dando a volta quando não há corte de um lado.
        let low = cuts
            .iter()
            .copied()
            .rev()
            .find(|&s| s < t)
            .unwrap_or(cuts[cuts.len() - 1]);
        let high = cuts.iter().copied().find(|&s| s > t).unwrap_or(cuts[0]);

        return Ok(remaining_of_closed(curve, high, low, tolerance)
            .into_iter()
            .collect());
    }

    let low = cuts.iter().copied().rev().find(|&s| s < t);
    let high = cuts.iter().copied().find(|&s| s > t);

    if low.is_none() && high.is_none() {
        return Err(TrimError::NoCuttingEdge);
    }

    let mut kept = Vec::new();

    if let Some(low) = low {
        kept.extend(rebuild(curve, &between(curve, 0.0, low, tolerance)));
    }

    if let Some(high) = high {
        kept.extend(rebuild(curve, &between(curve, high, 1.0, tolerance)));
    }

    Ok(kept)
}

/// Estende a curva, pela ponta mais próxima de `pick`, até a primeira fronteira.
///
/// # Errors
///
/// [`TrimError::PickOffCurve`] se `pick` estiver a mais de `capture` da curva;
/// [`TrimError::NotExtendable`] se a curva for fechada;
/// [`TrimError::NoBoundary`] se nenhuma fronteira for alcançada.
pub fn extend(
    curve: &AnyCurve,
    boundaries: &[AnyCurve],
    pick: Point2,
    capture: f64,
    tolerance: Tolerance,
) -> Result<AnyCurve, TrimError> {
    let picked = curve.closest_point(pick);

    if picked.distance > capture {
        return Err(TrimError::PickOffCurve);
    }

    if is_closed(curve) {
        return Err(TrimError::NotExtendable);
    }

    let at_end = picked.parameter >= 0.5;

    match curve {
        AnyCurve::Line(line) => {
            extend_line(*line, at_end, boundaries, tolerance).map(AnyCurve::Line)
        }
        AnyCurve::Arc(arc) => extend_arc(*arc, at_end, boundaries, tolerance).map(AnyCurve::Arc),
        AnyCurve::Polyline(polyline) => {
            let mut pieces: Vec<PolylineSegment> = polyline.segments().collect();
            let index = if at_end {
                pieces.len().checked_sub(1)
            } else {
                Some(0)
            };
            let Some(piece) = index.and_then(|index| pieces.get_mut(index)) else {
                return Err(TrimError::NoBoundary);
            };

            *piece = match *piece {
                PolylineSegment::Straight(line) => {
                    PolylineSegment::Straight(extend_line(line, at_end, boundaries, tolerance)?)
                }
                // Num trecho percorrido ao contrário, a ponta final da polilinha
                // é o começo do arco.
                PolylineSegment::Curved { arc, reversed } => PolylineSegment::Curved {
                    arc: extend_arc(arc, at_end != reversed, boundaries, tolerance)?,
                    reversed,
                },
            };

            Ok(AnyCurve::Polyline(polyline_from(&pieces, false)))
        }
        AnyCurve::Circle(_) => Err(TrimError::NotExtendable),
    }
}

/// Circunferência e polilinha fechada não têm ponta.
fn is_closed(curve: &AnyCurve) -> bool {
    match curve {
        AnyCurve::Circle(_) => true,
        AnyCurve::Polyline(polyline) => polyline.closed,
        AnyCurve::Line(_) | AnyCurve::Arc(_) => false,
    }
}

/// Parâmetros, em ordem crescente e sem repetição, onde as arestas cortam a
/// curva. Numa curva aberta, um corte sobre a ponta não divide nada e é
/// descartado; numa fechada, o parâmetro `1` é o mesmo ponto que o `0`.
fn cut_parameters(
    curve: &AnyCurve,
    edges: &[AnyCurve],
    closed: bool,
    tolerance: Tolerance,
) -> Vec<f64> {
    let start = curve.point_at(0.0);
    let end = curve.point_at(1.0);
    let mut cuts: Vec<f64> = Vec::new();

    for edge in edges {
        for point in intersect(curve, edge, tolerance).points {
            if !closed
                && (tolerance.points_equal(point, start) || tolerance.points_equal(point, end))
            {
                continue;
            }

            let mut parameter = curve.closest_point(point).parameter;

            if closed && parameter >= 1.0 {
                parameter = 0.0;
            }

            cuts.push(parameter);
        }
    }

    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| tolerance.points_equal(curve.point_at(*a), curve.point_at(*b)));
    cuts
}

/// A curva repartida em trechos, com o parâmetro global repartido igualmente
/// entre eles — a mesma convenção de [`crate::Polyline`].
fn pieces_of(curve: &AnyCurve) -> Vec<PolylineSegment> {
    match curve {
        AnyCurve::Line(line) => vec![PolylineSegment::Straight(*line)],
        AnyCurve::Arc(arc) => vec![PolylineSegment::Curved {
            arc: *arc,
            reversed: false,
        }],
        AnyCurve::Polyline(polyline) => polyline.segments().collect(),
        AnyCurve::Circle(_) => Vec::new(),
    }
}

/// Os trechos da curva entre dois parâmetros globais, `from < to`.
fn between(curve: &AnyCurve, from: f64, to: f64, tolerance: Tolerance) -> Vec<PolylineSegment> {
    let pieces = pieces_of(curve);
    let count = pieces.len() as f64;
    let mut kept = Vec::new();

    for (index, piece) in pieces.iter().enumerate() {
        let low = index as f64 / count;
        let high = (index + 1) as f64 / count;
        let a = from.max(low);
        let b = to.min(high);

        if b > a {
            kept.extend(sub_piece(
                piece,
                (a - low) * count,
                (b - low) * count,
                tolerance,
            ));
        }
    }

    kept
}

/// Remonta, no tipo da curva original, a curva aberta feita dos trechos.
fn rebuild(curve: &AnyCurve, pieces: &[PolylineSegment]) -> Option<AnyCurve> {
    match (curve, pieces) {
        (_, []) => None,
        (AnyCurve::Line(_), [PolylineSegment::Straight(line)]) => Some(AnyCurve::Line(*line)),
        (AnyCurve::Arc(_), [PolylineSegment::Curved { arc, .. }]) => Some(AnyCurve::Arc(*arc)),
        _ => Some(AnyCurve::Polyline(polyline_from(pieces, false))),
    }
}

/// O que sobra da curva fechada: o percurso de `from` até `to`, dando a volta
/// pelo parâmetro zero quando `to` vem antes.
fn remaining_of_closed(
    curve: &AnyCurve,
    from: f64,
    to: f64,
    tolerance: Tolerance,
) -> Option<AnyCurve> {
    if let AnyCurve::Circle(circle) = curve {
        return Some(AnyCurve::Arc(Arc {
            center: circle.center,
            radius: circle.radius,
            start_angle: from * TAU,
            end_angle: to * TAU,
        }));
    }

    let mut pieces = if from < to {
        between(curve, from, to, tolerance)
    } else {
        between(curve, from, 1.0, tolerance)
    };

    if from >= to {
        pieces.extend(between(curve, 0.0, to, tolerance));
    }

    rebuild(curve, &pieces)
}

/// Prolonga a reta pela ponta indicada até a primeira fronteira.
fn extend_line(
    line: Line,
    at_end: bool,
    boundaries: &[AnyCurve],
    tolerance: Tolerance,
) -> Result<Line, TrimError> {
    let (fixed, tip) = if at_end {
        (line.start, line.end)
    } else {
        (line.end, line.start)
    };
    let length = fixed.distance_to(tip);

    if tolerance.is_zero_length(length) {
        return Err(TrimError::NoBoundary);
    }

    // O raio só precisa ir além de toda fronteira.
    let reach = boundaries
        .iter()
        .map(|boundary| {
            let bounds = boundary.bounding_box();
            tip.distance_to(bounds.min())
                .max(tip.distance_to(bounds.max()))
        })
        .fold(0.0, f64::max)
        + 1.0;
    let ray = AnyCurve::Line(Line {
        start: tip,
        end: Point2::new(
            tip.x + (tip.x - fixed.x) / length * reach,
            tip.y + (tip.y - fixed.y) / length * reach,
        ),
    });

    let hit = boundaries
        .iter()
        .flat_map(|boundary| intersect(&ray, boundary, tolerance).points)
        .filter(|point| !tolerance.points_equal(*point, tip))
        .min_by(|a, b| tip.distance_to(*a).total_cmp(&tip.distance_to(*b)))
        .ok_or(TrimError::NoBoundary)?;

    Ok(if at_end {
        Line {
            start: fixed,
            end: hit,
        }
    } else {
        Line {
            start: hit,
            end: fixed,
        }
    })
}

/// Prolonga o arco, sobre a mesma circunferência, pela ponta indicada até a
/// primeira fronteira; o prolongamento é o arco complementar.
fn extend_arc(
    arc: Arc,
    at_end: bool,
    boundaries: &[AnyCurve],
    tolerance: Tolerance,
) -> Result<Arc, TrimError> {
    let gap = AnyCurve::Arc(Arc {
        start_angle: arc.end_angle,
        end_angle: arc.start_angle,
        ..arc
    });
    let tip = gap.point_at(if at_end { 0.0 } else { 1.0 });

    let delta = boundaries
        .iter()
        .flat_map(|boundary| intersect(&gap, boundary, tolerance).points)
        .filter(|point| !tolerance.points_equal(*point, tip))
        .map(|point| {
            let angle = (point.y - arc.center.y).atan2(point.x - arc.center.x);

            if at_end {
                (angle - arc.end_angle).rem_euclid(TAU)
            } else {
                (arc.start_angle - angle).rem_euclid(TAU)
            }
        })
        .min_by(f64::total_cmp)
        .ok_or(TrimError::NoBoundary)?;

    Ok(if at_end {
        Arc {
            end_angle: arc.end_angle + delta,
            ..arc
        }
    } else {
        Arc {
            start_angle: arc.start_angle - delta,
            ..arc
        }
    })
}

#[cfg(test)]
mod tests {
    // Testes escritos em contexto limpo a partir dos critérios E7–E9 da spec 05.
    use super::*;
    use crate::{point_on_circle, Arc, Circle, Line, Polyline};

    const CAPTURA: f64 = 0.5;

    fn p(x: f64, y: f64) -> Point2 {
        Point2::new(x, y)
    }

    fn reta(ax: f64, ay: f64, bx: f64, by: f64) -> AnyCurve {
        AnyCurve::Line(Line {
            start: p(ax, ay),
            end: p(bx, by),
        })
    }

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    fn perto(a: Point2, b: Point2) -> bool {
        a.distance_to(b) < 1e-9
    }

    fn pontas(curva: &AnyCurve) -> (Point2, Point2) {
        match curva {
            AnyCurve::Line(line) => (line.start, line.end),
            AnyCurve::Arc(arc) => (
                point_on_circle(arc.center, arc.radius, arc.start_angle),
                point_on_circle(arc.center, arc.radius, arc.end_angle),
            ),
            outra => panic!("curva sem pontas: {outra:?}"),
        }
    }

    #[test]
    fn trim_reta_cortada_por_duas_retas_verticais_ponto_no_meio() {
        let linha = reta(0.0, 0.0, 10.0, 0.0);
        let corte1 = reta(3.0, -1.0, 3.0, 1.0);
        let corte2 = reta(7.0, -1.0, 7.0, 1.0);
        let resultado = trim(&linha, &[corte1, corte2], p(5.0, 0.0), CAPTURA, tol()).unwrap();
        assert_eq!(resultado.len(), 2);
        assert!(resultado.iter().all(|c| matches!(c, AnyCurve::Line(_))));
        let (a1, b1) = pontas(&resultado[0]);
        let (a2, b2) = pontas(&resultado[1]);
        assert!(perto(a1, p(0.0, 0.0)) && perto(b1, p(3.0, 0.0)));
        assert!(perto(a2, p(7.0, 0.0)) && perto(b2, p(10.0, 0.0)));
    }

    #[test]
    fn trim_reta_cortada_por_duas_retas_verticais_ponto_proximo_a_uma_ponta() {
        let linha = reta(0.0, 0.0, 10.0, 0.0);
        let corte1 = reta(3.0, -1.0, 3.0, 1.0);
        let corte2 = reta(7.0, -1.0, 7.0, 1.0);
        let resultado = trim(&linha, &[corte1, corte2], p(1.0, 0.0), CAPTURA, tol()).unwrap();
        assert_eq!(resultado.len(), 1);
        assert!(matches!(resultado[0], AnyCurve::Line(_)));
        let (a, b) = pontas(&resultado[0]);
        assert!(perto(a, p(3.0, 0.0)) && perto(b, p(10.0, 0.0)));
    }

    #[test]
    fn trim_reta_cortada_por_retas_verticais_em_ordem_diferente() {
        let linha = reta(0.0, 0.0, 10.0, 0.0);
        let corte1 = reta(8.0, -1.0, 8.0, 1.0);
        let corte2 = reta(2.0, -1.0, 2.0, 1.0);
        let corte3 = reta(6.0, -1.0, 6.0, 1.0);
        let corte4 = reta(4.0, -1.0, 4.0, 1.0);
        let resultado = trim(
            &linha,
            &[corte1, corte2, corte3, corte4],
            p(5.0, 0.0),
            CAPTURA,
            tol(),
        )
        .unwrap();
        assert_eq!(resultado.len(), 2);
        assert!(resultado.iter().all(|c| matches!(c, AnyCurve::Line(_))));
        let (a1, b1) = pontas(&resultado[0]);
        let (a2, b2) = pontas(&resultado[1]);
        assert!(perto(a1, p(0.0, 0.0)) && perto(b1, p(4.0, 0.0)));
        assert!(perto(a2, p(6.0, 0.0)) && perto(b2, p(10.0, 0.0)));
    }

    #[test]
    fn trim_reta_cortada_por_uma_aresta_vertical_ponto_proximo_a_uma_ponta() {
        let linha = reta(0.0, 0.0, 10.0, 0.0);
        let corte = reta(4.0, -1.0, 4.0, 1.0);
        let resultado = trim(&linha, &[corte], p(8.0, 0.0), CAPTURA, tol()).unwrap();
        assert_eq!(resultado.len(), 1);
        assert!(matches!(resultado[0], AnyCurve::Line(_)));
        let (a, b) = pontas(&resultado[0]);
        assert!(perto(a, p(0.0, 0.0)) && perto(b, p(4.0, 0.0)));
    }

    #[test]
    fn trim_reta_com_ponto_longe_da_curva() {
        let linha = reta(0.0, 0.0, 10.0, 0.0);
        let corte = reta(5.0, -10.0, 5.0, 10.0);
        let resultado = trim(&linha, &[corte], p(2.0, 3.0), CAPTURA, tol());
        assert_eq!(resultado, Err(TrimError::PickOffCurve));
    }

    #[test]
    fn trim_reta_sem_corte_na_curva() {
        let linha = reta(0.0, 0.0, 10.0, 0.0);
        let corte = reta(0.0, 5.0, 10.0, 5.0);
        let resultado = trim(&linha, &[corte], p(2.0, 0.0), CAPTURA, tol());
        assert_eq!(resultado, Err(TrimError::NoCuttingEdge));
    }

    #[test]
    fn trim_circunferencia_com_arco_sobrando() {
        let circ = AnyCurve::Circle(Circle {
            center: p(0.0, 0.0),
            radius: 5.0,
        });
        let corte = reta(0.0, -10.0, 0.0, 10.0);
        let resultado = trim(&circ, &[corte], p(5.0, 0.0), CAPTURA, tol()).unwrap();
        assert_eq!(resultado.len(), 1);
        match &resultado[0] {
            AnyCurve::Arc(arc) => {
                assert_eq!(arc.center, p(0.0, 0.0));
                assert_eq!(arc.radius, 5.0);
                let (inicio, fim) = pontas(&resultado[0]);
                assert!(perto(inicio, p(0.0, 5.0)));
                assert!(perto(fim, p(0.0, -5.0)));
            }
            _ => panic!("Esperava Arc"),
        }
    }

    #[test]
    fn trim_arco_com_corte_vertical() {
        let arco = AnyCurve::Arc(Arc {
            center: p(0.0, 0.0),
            radius: 5.0,
            start_angle: 0.0,
            end_angle: core::f64::consts::PI,
        });
        let corte = reta(0.0, -10.0, 0.0, 10.0);
        let ponto = p(
            5.0 * (core::f64::consts::PI / 4.0).cos(),
            5.0 * (core::f64::consts::PI / 4.0).sin(),
        );
        let resultado = trim(&arco, &[corte], ponto, CAPTURA, tol()).unwrap();
        assert_eq!(resultado.len(), 1);
        match &resultado[0] {
            AnyCurve::Arc(arc) => {
                assert_eq!(arc.center, p(0.0, 0.0));
                assert_eq!(arc.radius, 5.0);
                let (inicio, fim) = pontas(&resultado[0]);
                assert!(perto(inicio, p(0.0, 5.0)));
                assert!(perto(fim, p(-5.0, 0.0)));
            }
            _ => panic!("Esperava Arc"),
        }
    }

    #[test]
    fn estender_reta_fronteira_vertical_primeira() {
        let linha = reta(0.0, 0.0, 5.0, 0.0);
        let fronteira = reta(10.0, -5.0, 10.0, 5.0);
        let resultado = extend(&linha, &[fronteira], p(4.0, 0.0), CAPTURA, tol());
        let Ok(curva) = resultado else {
            panic!("Expected Ok");
        };
        match curva {
            AnyCurve::Line(l) => {
                let (inicio, fim) = pontas(&AnyCurve::Line(l));
                assert!(perto(inicio, p(0.0, 0.0)));
                assert!(perto(fim, p(10.0, 0.0)));
            }
            _ => panic!("Expected Line"),
        }
    }

    #[test]
    fn estender_reta_fronteira_vertical_segunda() {
        let linha = reta(0.0, 0.0, 5.0, 0.0);
        let fronteira1 = reta(12.0, -5.0, 12.0, 5.0);
        let fronteira2 = reta(8.0, -5.0, 8.0, 5.0);
        let resultado = extend(
            &linha,
            &[fronteira1, fronteira2],
            p(4.0, 0.0),
            CAPTURA,
            tol(),
        );
        let Ok(curva) = resultado else {
            panic!("Expected Ok");
        };
        match curva {
            AnyCurve::Line(l) => {
                let (inicio, fim) = pontas(&AnyCurve::Line(l));
                assert!(perto(inicio, p(0.0, 0.0)));
                assert!(perto(fim, p(8.0, 0.0)));
            }
            _ => panic!("Expected Line"),
        }
    }

    #[test]
    fn estender_reta_fronteira_vertical_ponta_inicial() {
        let linha = reta(2.0, 0.0, 5.0, 0.0);
        let fronteira = reta(-3.0, -5.0, -3.0, 5.0);
        let resultado = extend(&linha, &[fronteira], p(2.5, 0.0), CAPTURA, tol());
        let Ok(curva) = resultado else {
            panic!("Expected Ok");
        };
        match curva {
            AnyCurve::Line(l) => {
                let (inicio, fim) = pontas(&AnyCurve::Line(l));
                assert!(perto(inicio, p(-3.0, 0.0)));
                assert!(perto(fim, p(5.0, 0.0)));
            }
            _ => panic!("Expected Line"),
        }
    }

    #[test]
    fn estender_reta_fronteira_nao_alcancada_ou_fora_do_limite() {
        let linha = reta(0.0, 0.0, 5.0, 0.0);
        let fronteira = reta(10.0, 1.0, 10.0, 5.0);
        let resultado = extend(&linha, &[fronteira], p(4.0, 0.0), CAPTURA, tol());
        assert_eq!(resultado, Err(TrimError::NoBoundary));

        let fronteira2 = reta(10.0, -5.0, 10.0, 5.0);
        let resultado2 = extend(&linha, &[fronteira2], p(4.0, 3.0), CAPTURA, tol());
        assert_eq!(resultado2, Err(TrimError::PickOffCurve));
    }

    #[test]
    fn estender_circunferencia_ou_arco() {
        let circ = AnyCurve::Circle(Circle {
            center: p(0.0, 0.0),
            radius: 5.0,
        });
        let fronteira = reta(0.0, -10.0, 0.0, 10.0);
        let resultado = extend(&circ, &[fronteira], p(5.0, 0.0), CAPTURA, tol());
        assert_eq!(resultado, Err(TrimError::NotExtendable));

        let arco = AnyCurve::Arc(Arc {
            center: p(0.0, 0.0),
            radius: 5.0,
            start_angle: 0.0,
            end_angle: core::f64::consts::PI / 4.0,
        });
        let fronteira2 = reta(0.0, 0.0, 0.0, 10.0);
        let resultado2 = extend(
            &arco,
            &[fronteira2],
            p(
                5.0 * (core::f64::consts::PI / 5.0).cos(),
                5.0 * (core::f64::consts::PI / 5.0).sin(),
            ),
            CAPTURA,
            tol(),
        );
        let Ok(curva) = resultado2 else {
            panic!("Expected Ok");
        };
        match curva {
            AnyCurve::Arc(a) => {
                let (inicio, fim) = pontas(&AnyCurve::Arc(a));
                assert!(perto(inicio, p(5.0, 0.0)));
                assert!(perto(fim, p(0.0, 5.0)));
            }
            _ => panic!("Expected Arc"),
        }
    }

    #[test]
    fn trim_polilinha_aberta_corte_vertical_ponto_esquerdo() {
        let polyline = AnyCurve::Polyline(Polyline::from_points(
            [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0)],
            false,
        ));
        let cutting_edge = reta(5.0, -5.0, 5.0, 5.0);
        let result = trim(&polyline, &[cutting_edge], p(2.0, 0.0), CAPTURA, tol()).unwrap();
        assert_eq!(result.len(), 1);
        let trimmed = &result[0];
        match trimmed {
            AnyCurve::Polyline(poly) => {
                let points: Vec<Point2> = poly.points().collect();
                assert_eq!(points.len(), 3);
                assert!(perto(points[0], p(5.0, 0.0)));
                assert!(perto(points[1], p(10.0, 0.0)));
                assert!(perto(points[2], p(10.0, 10.0)));
            }
            _ => panic!("Esperava Polyline"),
        }
    }

    #[test]
    fn trim_polilinha_fechada_quadrada_corte_vertical_ponto_direito() {
        let polyline = AnyCurve::Polyline(Polyline::from_points(
            [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)],
            true,
        ));
        let cutting_edge = reta(5.0, -5.0, 5.0, 15.0);
        let result = trim(&polyline, &[cutting_edge], p(10.0, 5.0), CAPTURA, tol()).unwrap();
        assert_eq!(result.len(), 1);
        let trimmed = &result[0];
        match trimmed {
            AnyCurve::Polyline(poly) => {
                let points: Vec<Point2> = poly.points().collect();
                assert_eq!(points.len(), 4);
                assert!(perto(points[0], p(5.0, 10.0)));
                assert!(perto(points[1], p(0.0, 10.0)));
                assert!(perto(points[2], p(0.0, 0.0)));
                assert!(perto(points[3], p(5.0, 0.0)));
            }
            _ => panic!("Esperava Polyline"),
        }
    }

    #[test]
    fn extend_polilinha_aberta_fronteira_horizontal_ponto_final() {
        let polyline = AnyCurve::Polyline(Polyline::from_points(
            [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0)],
            false,
        ));
        let boundary = reta(0.0, 15.0, 20.0, 15.0);
        let result = extend(
            &polyline,
            core::slice::from_ref(&boundary),
            p(10.0, 9.0),
            CAPTURA,
            tol(),
        )
        .unwrap();
        match &result {
            AnyCurve::Polyline(poly) => {
                let points: Vec<Point2> = poly.points().collect();
                assert_eq!(points.len(), 3);
                assert!(perto(points[0], p(0.0, 0.0)));
                assert!(perto(points[1], p(10.0, 0.0)));
                assert!(perto(points[2], p(10.0, 15.0)));
            }
            _ => panic!("Esperava Polyline"),
        }

        let closed_polyline = AnyCurve::Polyline(Polyline::from_points(
            [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)],
            true,
        ));
        let err = extend(&closed_polyline, &[boundary], p(10.0, 5.0), CAPTURA, tol()).unwrap_err();
        assert_eq!(err, TrimError::NotExtendable);
    }
}
