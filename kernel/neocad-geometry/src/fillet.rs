// Caminho relativo: kernel/neocad-geometry/src/fillet.rs
//! \file kernel/neocad-geometry/src/fillet.rs
//! \brief Concordância e chanfro entre duas curvas.
//! \author Iago Leal
//! \date 2026-10-29
//!
//! Spec 05, critérios E10 a E13. Das soluções possíveis, a escolhida é a que os
//! pontos indicados pelo usuário apontam, e não a que a ordem dos argumentos
//! sugere — como no AutoCAD.

use core::f64::consts::{PI, TAU};
use core::fmt;

use crate::{AnyCurve, Arc, Line, Point2, Tolerance};

/// As duas curvas depois do canto, e o que as une.
#[derive(Debug, Clone, PartialEq)]
pub struct Corner {
    /// A primeira curva, aparada ou estendida até o canto.
    pub first: AnyCurve,
    /// A segunda curva, aparada ou estendida até o canto.
    pub second: AnyCurve,
    /// O arco da concordância ou o segmento do chanfro; `None` quando o canto
    /// é vivo (raio ou distâncias zero).
    pub joint: Option<AnyCurve>,
}

/// Por que uma concordância ou um chanfro não pôde ser feito.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerError {
    /// As retas são paralelas e não têm vértice.
    ParallelLines,
    /// O raio não cabe entre as curvas.
    RadiusTooLarge,
    /// A distância do chanfro passa do trecho disponível da reta.
    DistanceTooLarge,
    /// Raio ou distância negativo ou não finito.
    InvalidValue,
    /// Tipo de curva que a operação ainda não trata.
    UnsupportedCurve,
}

impl fmt::Display for CornerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ParallelLines => "as retas são paralelas e não têm vértice",
            Self::RadiusTooLarge => "o raio não cabe entre as curvas",
            Self::DistanceTooLarge => "a distância passa do trecho disponível da reta",
            Self::InvalidValue => "o valor tem de ser zero ou positivo, e finito",
            Self::UnsupportedCurve => "a operação ainda não trata este tipo de curva",
        })
    }
}

impl core::error::Error for CornerError {}

/// Concorda duas curvas (reta ou arco) com um arco de raio dado.
///
/// # Errors
///
/// Ver [`CornerError`].
pub fn fillet(
    first: &AnyCurve,
    first_pick: Point2,
    second: &AnyCurve,
    second_pick: Point2,
    radius: f64,
    tolerance: Tolerance,
) -> Result<Corner, CornerError> {
    if !(radius.is_finite() && radius >= 0.0) {
        return Err(CornerError::InvalidValue);
    }

    let support_a = support_of(first)?;
    let support_b = support_of(second)?;

    if let (AnyCurve::Line(a), AnyCurve::Line(b)) = (first, second) {
        return fillet_lines(*a, first_pick, *b, second_pick, radius, tolerance);
    }

    // Com arco envolvido, as soluções são os centros a `radius` das duas
    // curvas, e vence a de pontos de tangência mais perto dos indicados.
    let mut best: Option<(f64, Point2, Point2, Point2)> = None;

    for offset_a in offsets(support_a, radius) {
        for offset_b in offsets(support_b, radius) {
            for center in crossings(offset_a, offset_b, tolerance) {
                let (Some(touch_a), Some(touch_b)) = (
                    foot(support_a, center, radius),
                    foot(support_b, center, radius),
                ) else {
                    continue;
                };
                let cost = touch_a.distance_to(first_pick) + touch_b.distance_to(second_pick);

                if best.is_none_or(|(least, ..)| cost < least) {
                    best = Some((cost, center, touch_a, touch_b));
                }
            }
        }
    }

    let (_, center, touch_a, touch_b) = best.ok_or(CornerError::RadiusTooLarge)?;

    Ok(Corner {
        first: cut_at(first, touch_a, first_pick, tolerance)?,
        second: cut_at(second, touch_b, second_pick, tolerance)?,
        joint: joint_arc(center, radius, touch_a, touch_b),
    })
}

/// Concordância entre duas retas: o quadrante é o dos pontos indicados.
fn fillet_lines(
    a: Line,
    pick_a: Point2,
    b: Line,
    pick_b: Point2,
    radius: f64,
    tolerance: Tolerance,
) -> Result<Corner, CornerError> {
    let vertex = vertex_of(a, b, tolerance)?;
    let ray_a = ray_toward(a, vertex, pick_a);
    let ray_b = ray_toward(b, vertex, pick_b);

    // Meio ângulo entre os dois raios que formam o canto.
    let half = dot(ray_a, ray_b).clamp(-1.0, 1.0).acos() / 2.0;
    let reach = radius / half.tan();
    let touch_a = shifted(vertex, ray_a, reach);
    let touch_b = shifted(vertex, ray_b, reach);
    let bisector = unit((ray_a.0 + ray_b.0, ray_a.1 + ray_b.1));
    let center = shifted(vertex, bisector, radius / half.sin());

    Ok(Corner {
        first: AnyCurve::Line(
            keep_ray(a, touch_a, ray_a, tolerance).ok_or(CornerError::RadiusTooLarge)?,
        ),
        second: AnyCurve::Line(
            keep_ray(b, touch_b, ray_b, tolerance).ok_or(CornerError::RadiusTooLarge)?,
        ),
        joint: joint_arc(center, radius, touch_a, touch_b),
    })
}

/// Chanfra duas retas com as distâncias dadas, medidas a partir do vértice.
///
/// # Errors
///
/// Ver [`CornerError`].
pub fn chamfer(
    first: &AnyCurve,
    first_pick: Point2,
    second: &AnyCurve,
    second_pick: Point2,
    first_distance: f64,
    second_distance: f64,
    tolerance: Tolerance,
) -> Result<Corner, CornerError> {
    let valid = |value: f64| value.is_finite() && value >= 0.0;

    if !(valid(first_distance) && valid(second_distance)) {
        return Err(CornerError::InvalidValue);
    }

    let (AnyCurve::Line(a), AnyCurve::Line(b)) = (first, second) else {
        return Err(CornerError::UnsupportedCurve);
    };

    let vertex = vertex_of(*a, *b, tolerance)?;
    let ray_a = ray_toward(*a, vertex, first_pick);
    let ray_b = ray_toward(*b, vertex, second_pick);
    let point_a = shifted(vertex, ray_a, first_distance);
    let point_b = shifted(vertex, ray_b, second_distance);
    let kept_a = keep_ray(*a, point_a, ray_a, tolerance).ok_or(CornerError::DistanceTooLarge)?;
    let kept_b = keep_ray(*b, point_b, ray_b, tolerance).ok_or(CornerError::DistanceTooLarge)?;
    let joint = (!tolerance.points_equal(point_a, point_b)).then_some(AnyCurve::Line(Line {
        start: point_a,
        end: point_b,
    }));

    Ok(Corner {
        first: AnyCurve::Line(kept_a),
        second: AnyCurve::Line(kept_b),
        joint,
    })
}

type Vector = (f64, f64);

fn dot(a: Vector, b: Vector) -> f64 {
    a.0 * b.0 + a.1 * b.1
}

fn cross(a: Vector, b: Vector) -> f64 {
    a.0 * b.1 - a.1 * b.0
}

fn between(from: Point2, to: Point2) -> Vector {
    (to.x - from.x, to.y - from.y)
}

fn unit(vector: Vector) -> Vector {
    let length = vector.0.hypot(vector.1);
    (vector.0 / length, vector.1 / length)
}

fn shifted(point: Point2, direction: Vector, amount: f64) -> Point2 {
    Point2::new(
        point.x + direction.0 * amount,
        point.y + direction.1 * amount,
    )
}

fn angle_of(center: Point2, point: Point2) -> f64 {
    (point.y - center.y).atan2(point.x - center.x)
}

/// O suporte infinito de uma curva: a reta inteira ou a circunferência inteira.
#[derive(Debug, Clone, Copy)]
enum Support {
    Straight { origin: Point2, direction: Vector },
    Round { center: Point2, radius: f64 },
}

fn support_of(curve: &AnyCurve) -> Result<Support, CornerError> {
    match curve {
        AnyCurve::Line(line) => Ok(Support::Straight {
            origin: line.start,
            direction: unit(between(line.start, line.end)),
        }),
        AnyCurve::Arc(arc) => Ok(Support::Round {
            center: arc.center,
            radius: arc.radius,
        }),
        AnyCurve::Circle(_) | AnyCurve::Polyline(_) => Err(CornerError::UnsupportedCurve),
    }
}

/// Os lugares dos centros a `distance` do suporte.
fn offsets(support: Support, distance: f64) -> Vec<Support> {
    if distance == 0.0 {
        return vec![support];
    }

    match support {
        Support::Straight { origin, direction } => {
            let normal = (-direction.1, direction.0);
            vec![
                Support::Straight {
                    origin: shifted(origin, normal, distance),
                    direction,
                },
                Support::Straight {
                    origin: shifted(origin, normal, -distance),
                    direction,
                },
            ]
        }
        Support::Round { center, radius } => {
            let mut rounds = vec![Support::Round {
                center,
                radius: radius + distance,
            }];
            let inner = (radius - distance).abs();

            if inner > 0.0 {
                rounds.push(Support::Round {
                    center,
                    radius: inner,
                });
            }

            rounds
        }
    }
}

/// Os pontos comuns a dois suportes.
fn crossings(a: Support, b: Support, tolerance: Tolerance) -> Vec<Point2> {
    match (a, b) {
        (
            Support::Straight {
                origin: oa,
                direction: da,
            },
            Support::Straight {
                origin: ob,
                direction: db,
            },
        ) => {
            let denominator = cross(da, db);

            if denominator.abs() <= tolerance.angular() {
                return Vec::new();
            }

            vec![shifted(oa, da, cross(between(oa, ob), db) / denominator)]
        }
        (Support::Straight { origin, direction }, Support::Round { center, radius })
        | (Support::Round { center, radius }, Support::Straight { origin, direction }) => {
            let offset = between(center, origin);
            let half_b = dot(offset, direction);
            let discriminant = half_b * half_b - (dot(offset, offset) - radius * radius);

            if discriminant < 0.0 {
                return Vec::new();
            }

            let root = discriminant.sqrt();
            vec![
                shifted(origin, direction, -half_b - root),
                shifted(origin, direction, -half_b + root),
            ]
        }
        (
            Support::Round {
                center: ca,
                radius: ra,
            },
            Support::Round {
                center: cb,
                radius: rb,
            },
        ) => {
            let gap = ca.distance_to(cb);

            if tolerance.is_zero_length(gap) || gap > ra + rb || gap < (ra - rb).abs() {
                return Vec::new();
            }

            let along = (ra * ra - rb * rb + gap * gap) / (2.0 * gap);
            let height = (ra * ra - along * along).max(0.0).sqrt();
            let axis = unit(between(ca, cb));
            let base = shifted(ca, axis, along);
            let normal = (-axis.1, axis.0);
            vec![
                shifted(base, normal, height),
                shifted(base, normal, -height),
            ]
        }
    }
}

/// Onde a concordância de centro `point` e raio `reach` toca o suporte.
///
/// Na circunferência, o toque está na reta dos centros, mas de que lado
/// depende de a concordância envolver ou não o arco: é o dos dois pontos que
/// fica a `reach` do centro dela.
fn foot(support: Support, point: Point2, reach: f64) -> Option<Point2> {
    match support {
        Support::Straight { origin, direction } => Some(shifted(
            origin,
            direction,
            dot(between(origin, point), direction),
        )),
        Support::Round { center, radius } => {
            let away = between(center, point);

            if away.0.hypot(away.1) == 0.0 {
                return None;
            }

            let near = shifted(center, unit(away), radius);
            let far = shifted(center, unit(away), -radius);
            let miss = |touch: Point2| (touch.distance_to(point) - reach).abs();

            Some(if miss(near) <= miss(far) { near } else { far })
        }
    }
}

/// O vértice de duas retas, ou a recusa por serem paralelas.
fn vertex_of(a: Line, b: Line, tolerance: Tolerance) -> Result<Point2, CornerError> {
    crossings(
        support_of(&AnyCurve::Line(a))?,
        support_of(&AnyCurve::Line(b))?,
        tolerance,
    )
    .first()
    .copied()
    .ok_or(CornerError::ParallelLines)
}

/// A direção, ao longo da reta, que vai de `from` para o lado do ponto indicado.
fn ray_toward(line: Line, from: Point2, pick: Point2) -> Vector {
    let direction = unit(between(line.start, line.end));

    if dot(between(from, pick), direction) >= 0.0 {
        direction
    } else {
        (-direction.0, -direction.1)
    }
}

/// A reta reduzida (ou prolongada) ao trecho que vai de `from` até a ponta mais
/// distante no sentido `ray`, no sentido original. `None` se não sobra trecho.
fn keep_ray(line: Line, from: Point2, ray: Vector, tolerance: Tolerance) -> Option<Line> {
    let reach = |point: Point2| dot(between(from, point), ray);
    let far = if reach(line.end) >= reach(line.start) {
        line.end
    } else {
        line.start
    };

    if reach(far) <= 0.0 || tolerance.points_equal(far, from) {
        return None;
    }

    Some(if dot(between(line.start, line.end), ray) >= 0.0 {
        Line {
            start: from,
            end: far,
        }
    } else {
        Line {
            start: far,
            end: from,
        }
    })
}

/// A curva aparada ou prolongada até `touch`, do lado do ponto indicado.
fn cut_at(
    curve: &AnyCurve,
    touch: Point2,
    pick: Point2,
    tolerance: Tolerance,
) -> Result<AnyCurve, CornerError> {
    match curve {
        AnyCurve::Line(line) => keep_ray(*line, touch, ray_toward(*line, touch, pick), tolerance)
            .map(AnyCurve::Line)
            .ok_or(CornerError::RadiusTooLarge),
        AnyCurve::Arc(arc) => {
            let at = angle_of(arc.center, touch);
            let picked = angle_of(arc.center, pick);
            let sweep = |from: f64, to: f64| (to - from).rem_euclid(TAU);
            // Dos dois arcos que vão até o toque, o que contém o ponto indicado;
            // se os dois contêm (o toque está fora do arco), o menor.
            let options = [(arc.start_angle, at), (at, arc.end_angle)];
            let chosen = options
                .into_iter()
                .filter(|&(from, to)| sweep(from, picked) <= sweep(from, to))
                .min_by(|x, y| sweep(x.0, x.1).total_cmp(&sweep(y.0, y.1)))
                .ok_or(CornerError::RadiusTooLarge)?;

            if sweep(chosen.0, chosen.1) <= tolerance.angular() {
                return Err(CornerError::RadiusTooLarge);
            }

            Ok(AnyCurve::Arc(Arc {
                start_angle: chosen.0,
                end_angle: chosen.1,
                ..*arc
            }))
        }
        AnyCurve::Circle(_) | AnyCurve::Polyline(_) => Err(CornerError::UnsupportedCurve),
    }
}

/// O arco de concordância: anti-horário e menor que meia volta. Nenhum, com
/// raio zero.
fn joint_arc(center: Point2, radius: f64, a: Point2, b: Point2) -> Option<AnyCurve> {
    if radius == 0.0 {
        return None;
    }

    let from = angle_of(center, a);
    let to = angle_of(center, b);
    let (start_angle, end_angle) = if (to - from).rem_euclid(TAU) < PI {
        (from, to)
    } else {
        (to, from)
    };

    Some(AnyCurve::Arc(Arc {
        center,
        radius,
        start_angle,
        end_angle,
    }))
}

#[cfg(test)]
mod tests {
    // Testes escritos em contexto limpo a partir dos critérios E10–E13 da spec 05.
    use super::*;
    use crate::{point_on_circle, Arc, Circle, Line};
    use core::f64::consts::{FRAC_PI_2, FRAC_PI_4};

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

    fn confere_reta(curva: &AnyCurve, inicio: Point2, fim: Point2) {
        let AnyCurve::Line(line) = curva else {
            panic!("esperava reta, veio {curva:?}");
        };
        assert!(
            perto(line.start, inicio) && perto(line.end, fim),
            "reta {line:?}, esperada {inicio:?} → {fim:?}"
        );
    }

    fn confere_arco(curva: &AnyCurve, centro: Point2, raio: f64, inicio: Point2, fim: Point2) {
        let AnyCurve::Arc(arc) = curva else {
            panic!("esperava arco, veio {curva:?}");
        };
        let a = point_on_circle(arc.center, arc.radius, arc.start_angle);
        let b = point_on_circle(arc.center, arc.radius, arc.end_angle);
        assert!(
            perto(arc.center, centro) && (arc.radius - raio).abs() < 1e-9 && perto(a, inicio) && perto(b, fim),
            "arco {arc:?} ({a:?} → {b:?}), esperado centro {centro:?}, raio {raio}, {inicio:?} → {fim:?}"
        );
    }

    #[test]
    fn concordancia_raio_zero() {
        let first = reta(0.0, 0.0, 8.0, 0.0);
        let second = reta(10.0, 2.0, 10.0, 10.0);
        let result = fillet(&first, p(2.0, 0.0), &second, p(10.0, 8.0), 0.0, tol()).unwrap();
        confere_reta(&result.first, p(0.0, 0.0), p(10.0, 0.0));
        confere_reta(&result.second, p(10.0, 0.0), p(10.0, 10.0));
        assert_eq!(result.joint, None);
    }

    #[test]
    fn concordancia_raio_dois() {
        let first = reta(0.0, 0.0, 10.0, 0.0);
        let second = reta(10.0, 0.0, 10.0, 10.0);
        let result = fillet(&first, p(2.0, 0.0), &second, p(10.0, 8.0), 2.0, tol()).unwrap();
        confere_reta(&result.first, p(0.0, 0.0), p(8.0, 0.0));
        confere_reta(&result.second, p(10.0, 2.0), p(10.0, 10.0));
        confere_arco(
            result.joint.as_ref().expect("joint should be Some"),
            p(8.0, 2.0),
            2.0,
            p(8.0, 0.0),
            p(10.0, 2.0),
        );
    }

    #[test]
    fn concordancia_raio_dois_trocado() {
        let first = reta(10.0, 0.0, 10.0, 10.0);
        let second = reta(0.0, 0.0, 10.0, 0.0);
        let result = fillet(&first, p(10.0, 8.0), &second, p(2.0, 0.0), 2.0, tol()).unwrap();
        confere_reta(&result.first, p(10.0, 2.0), p(10.0, 10.0));
        confere_reta(&result.second, p(0.0, 0.0), p(8.0, 0.0));
        confere_arco(
            result.joint.as_ref().expect("joint should be Some"),
            p(8.0, 2.0),
            2.0,
            p(8.0, 0.0),
            p(10.0, 2.0),
        );
    }

    #[test]
    fn concordancia_quadrante() {
        let first = reta(-10.0, 0.0, 10.0, 0.0);
        let second = reta(0.0, -10.0, 0.0, 10.0);
        let result = fillet(&first, p(5.0, 0.0), &second, p(0.0, 5.0), 1.0, tol()).unwrap();
        confere_reta(&result.first, p(1.0, 0.0), p(10.0, 0.0));
        confere_reta(&result.second, p(0.0, 1.0), p(0.0, 10.0));
        confere_arco(
            result.joint.as_ref().expect("joint should be Some"),
            p(1.0, 1.0),
            1.0,
            p(0.0, 1.0),
            p(1.0, 0.0),
        );

        let result2 = fillet(&first, p(-5.0, 0.0), &second, p(0.0, -5.0), 1.0, tol()).unwrap();
        confere_reta(&result2.first, p(-10.0, 0.0), p(-1.0, 0.0));
        confere_reta(&result2.second, p(0.0, -10.0), p(0.0, -1.0));
        confere_arco(
            result2.joint.as_ref().expect("joint should be Some"),
            p(-1.0, -1.0),
            1.0,
            p(0.0, -1.0),
            p(-1.0, 0.0),
        );
    }

    #[test]
    fn concordancia_paralelas() {
        let r1 = reta(0.0, 0.0, 10.0, 0.0);
        let r2 = reta(0.0, 5.0, 10.0, 5.0);
        let resultado = fillet(&r1, p(2.0, 0.0), &r2, p(2.0, 5.0), 1.0, tol());
        assert_eq!(resultado, Err(CornerError::ParallelLines));
    }

    #[test]
    fn concordancia_raio_grande() {
        let r1 = reta(0.0, 0.0, 10.0, 0.0);
        let r2 = reta(10.0, 0.0, 10.0, 3.0);
        let resultado = fillet(&r1, p(2.0, 0.0), &r2, p(10.0, 2.0), 5.0, tol());
        assert_eq!(resultado, Err(CornerError::RadiusTooLarge));
    }

    #[test]
    fn concordancia_raio_negativo() {
        let r1 = reta(0.0, 0.0, 10.0, 0.0);
        let r2 = reta(10.0, 0.0, 10.0, 3.0);
        let resultado = fillet(&r1, p(2.0, 0.0), &r2, p(10.0, 2.0), -1.0, tol());
        assert_eq!(resultado, Err(CornerError::InvalidValue));
    }

    #[test]
    fn concordancia_circunferencia_nao_suportada() {
        let r1 = reta(10.0, 0.0, 10.0, 10.0);
        let circ = AnyCurve::Circle(Circle {
            center: p(0.0, 0.0),
            radius: 5.0,
        });
        let resultado = fillet(&circ, p(5.0, 0.0), &r1, p(10.0, 5.0), 1.0, tol());
        assert_eq!(resultado, Err(CornerError::UnsupportedCurve));
    }

    #[test]
    fn concordancia_reta_e_arco() {
        let r1 = reta(5.0, 0.0, 15.0, 0.0);
        let a1 = Arc {
            center: p(0.0, 0.0),
            radius: 5.0,
            start_angle: 0.0,
            end_angle: FRAC_PI_2,
        };
        let r35 = 35f64.sqrt();
        let resultado = fillet(
            &r1,
            p(10.0, 0.0),
            &AnyCurve::Arc(a1),
            p(5.0 * FRAC_PI_4.cos(), 5.0 * FRAC_PI_4.sin()),
            1.0,
            tol(),
        );
        let corner = resultado.expect("Deveria ter sucesso");
        confere_reta(&corner.first, p(r35, 0.0), p(15.0, 0.0));
        confere_arco(
            &corner.second,
            p(0.0, 0.0),
            5.0,
            p(5.0 * r35 / 6.0, 5.0 / 6.0),
            p(0.0, 5.0),
        );
        confere_arco(
            corner.joint.as_ref().expect("joint deve existir"),
            p(r35, 1.0),
            1.0,
            p(5.0 * r35 / 6.0, 5.0 / 6.0),
            p(r35, 0.0),
        );
    }

    #[test]
    fn chanfro_distancias_dois_e_tres() {
        let first = reta(0.0, 0.0, 10.0, 0.0);
        let second = reta(10.0, 0.0, 10.0, 10.0);
        let result = chamfer(&first, p(2.0, 0.0), &second, p(10.0, 8.0), 2.0, 3.0, tol());
        let corner = result.expect("Deveria ter sucesso");

        confere_reta(&corner.first, p(0.0, 0.0), p(8.0, 0.0));
        confere_reta(&corner.second, p(10.0, 3.0), p(10.0, 10.0));
        confere_reta(
            corner.joint.as_ref().expect("joint não deve ser None"),
            p(8.0, 0.0),
            p(10.0, 3.0),
        );
    }

    #[test]
    fn chanfro_retas_que_nao_se_cruzam() {
        let first = reta(0.0, 0.0, 6.0, 0.0);
        let second = reta(10.0, 4.0, 10.0, 10.0);
        let result = chamfer(&first, p(2.0, 0.0), &second, p(10.0, 8.0), 2.0, 2.0, tol());
        let corner = result.expect("Deveria ter sucesso");

        confere_reta(&corner.first, p(0.0, 0.0), p(8.0, 0.0));
        confere_reta(&corner.second, p(10.0, 2.0), p(10.0, 10.0));
        confere_reta(
            corner.joint.as_ref().expect("joint não deve ser None"),
            p(8.0, 0.0),
            p(10.0, 2.0),
        );
    }

    #[test]
    fn chanfro_distancias_zero_estende_ao_vertice() {
        let first = reta(0.0, 0.0, 8.0, 0.0);
        let second = reta(10.0, 2.0, 10.0, 10.0);
        let result = chamfer(&first, p(2.0, 0.0), &second, p(10.0, 8.0), 0.0, 0.0, tol());
        let corner = result.expect("Deveria ter sucesso");

        confere_reta(&corner.first, p(0.0, 0.0), p(10.0, 0.0));
        confere_reta(&corner.second, p(10.0, 0.0), p(10.0, 10.0));
        assert_eq!(corner.joint, None);
    }

    #[test]
    fn chanfro_recusas() {
        let first = reta(0.0, 0.0, 10.0, 0.0);
        let second = reta(10.0, 0.0, 10.0, 10.0);

        // Teste de valor inválido
        let result = chamfer(&first, p(2.0, 0.0), &second, p(10.0, 8.0), -1.0, 2.0, tol());
        assert_eq!(result, Err(CornerError::InvalidValue));

        // Teste de distância muito grande
        let result = chamfer(&first, p(2.0, 0.0), &second, p(10.0, 8.0), 12.0, 2.0, tol());
        assert_eq!(result, Err(CornerError::DistanceTooLarge));

        // Teste de retas paralelas
        let second_paralela = reta(0.0, 5.0, 10.0, 5.0);
        let result = chamfer(
            &first,
            p(2.0, 0.0),
            &second_paralela,
            p(2.0, 5.0),
            1.0,
            1.0,
            tol(),
        );
        assert_eq!(result, Err(CornerError::ParallelLines));
    }

    // Achado da crítica independente: quando a concordância envolve o arco
    // (raio maior que o dele), o toque fica do lado oposto ao centro dela.
    #[test]
    fn concordancia_que_envolve_o_arco_toca_do_lado_oposto() {
        let suporte = Support::Round {
            center: p(0.0, 0.0),
            radius: 1.0,
        };
        let toque = foot(suporte, p(0.0, 1.0), 2.0).expect("há toque");
        assert!(perto(toque, p(0.0, -1.0)), "toque em {toque:?}");

        let por_fora = foot(suporte, p(0.0, 3.0), 2.0).expect("há toque");
        assert!(perto(por_fora, p(0.0, 1.0)), "toque em {por_fora:?}");
    }
}
