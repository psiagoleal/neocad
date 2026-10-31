// Caminho relativo: kernel/neocad-geometry/src/tessellate.rs
//! \file kernel/neocad-geometry/src/tessellate.rs
//! \brief Divisão de curvas em segmentos de reta para a tela.
//! \author Iago Leal
//! \date 2026-10-31
//!
//! A GPU desenha segmentos de reta; arco, círculo e polilinha com abaulamento
//! chegam a ela já divididos (ADR 0008). Quem decide quão fina é a divisão é o
//! **erro de corda**: a maior distância admitida entre uma corda e o arco que
//! ela substitui, nas unidades do desenho. A tela o deriva do tamanho do pixel
//! na vista, de modo que a divisão não aparece.

use std::f64::consts::{PI, TAU};

use crate::{point_on_circle, AnyCurve, Arc, Point2, PolylineSegment};

/// Teto de cordas por arco.
///
/// Um erro de corda minúsculo num raio enorme pediria milhões de cordas; acima
/// deste teto a corda passa do erro pedido, o que na tela é menos que um pixel
/// de diferença e muito menos memória.
const MAX_SEGMENTS: usize = 1 << 16;

/// Quantas cordas um arco de raio `radius` e varredura `sweep` precisa.
fn segment_count(radius: f64, sweep: f64, chord_error: f64, minimum: usize) -> usize {
    // A flecha de uma corda que varre `φ` é `r·(1 − cos(φ/2))`; a maior
    // varredura que respeita o erro é `2·acos(1 − e/r)`, até a meia-volta.
    let ratio = chord_error / radius.abs();
    let widest = if ratio >= 1.0 {
        PI
    } else {
        2.0 * (1.0 - ratio).acos()
    };
    let needed = (sweep / widest).ceil();

    if needed.is_finite() && needed > 0.0 {
        (needed as usize).clamp(minimum, MAX_SEGMENTS)
    } else {
        minimum
    }
}

/// Acrescenta os pontos de um arco, de `start` ao fim, sem repetir `start`.
///
/// O fim vem pronto para que a junção com o trecho vizinho seja exata.
fn push_arc(points: &mut Vec<Point2>, arc: &Arc, reversed: bool, end: Point2, chord_error: f64) {
    let radius = arc.radius.abs();
    let sweep = arc.sweep();
    let count = segment_count(radius, sweep, chord_error, 1);

    for step in 1..count {
        let fraction = step as f64 / count as f64;
        let fraction = if reversed { 1.0 - fraction } else { fraction };
        points.push(point_on_circle(
            arc.center,
            radius,
            arc.start_angle + fraction * sweep,
        ));
    }
    points.push(end);
}

/// Divide a curva em uma sequência de pontos ligados por segmentos de reta.
///
/// O primeiro ponto é o início da curva e o último, o fim; na circunferência os
/// dois coincidem. Nenhuma corda se afasta do arco mais que `chord_error`.
///
/// Devolve `None` quando `chord_error` não é finito e positivo.
#[must_use]
pub fn tessellate(curve: &AnyCurve, chord_error: f64) -> Option<Vec<Point2>> {
    if !(chord_error.is_finite() && chord_error > 0.0) {
        return None;
    }

    let points = match curve {
        AnyCurve::Line(line) => vec![line.start, line.end],
        AnyCurve::Arc(arc) => {
            let start = point_on_circle(arc.center, arc.radius.abs(), arc.start_angle);
            let end = point_on_circle(arc.center, arc.radius.abs(), arc.start_angle + arc.sweep());
            let mut points = vec![start];
            push_arc(&mut points, arc, false, end, chord_error);
            points
        }
        AnyCurve::Circle(circle) => {
            let radius = circle.radius.abs();
            let count = segment_count(radius, TAU, chord_error, 3);
            let mut points: Vec<Point2> = (0..count)
                .map(|step| {
                    point_on_circle(circle.center, radius, TAU * step as f64 / count as f64)
                })
                .collect();
            points.push(points[0]);
            points
        }
        AnyCurve::Polyline(polyline) => {
            let mut points: Vec<Point2> = polyline
                .vertices
                .first()
                .map(|v| v.point)
                .into_iter()
                .collect();
            for index in 0..polyline.segment_count() {
                let Some(ends) = polyline.segment_ends(index) else {
                    continue;
                };
                match polyline.segment(index) {
                    Some(PolylineSegment::Curved { arc, reversed }) => {
                        push_arc(&mut points, &arc, reversed, ends.1, chord_error);
                    }
                    _ => points.push(ends.1),
                }
            }
            points
        }
    };

    Some(points)
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};

    use super::*;
    use crate::{Arc, Circle, Line, Polyline, PolylineVertex, Tolerance};

    /// Distância do arco à corda entre `a` e `b`, medida no meio da corda.
    fn flecha(centro: Point2, raio: f64, a: Point2, b: Point2) -> f64 {
        let meio = Point2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
        raio - centro.distance_to(meio)
    }

    /// Confere que todos os pontos estão no círculo e toda corda respeita o erro.
    fn confere_arco(pontos: &[Point2], centro: Point2, raio: f64, erro: f64) {
        let tol = Tolerance::default();
        let folga = tol.effective_linear(centro.x.abs().max(centro.y.abs()) + raio);

        assert!(pontos.len() >= 2, "arco com menos de dois pontos");
        for p in pontos {
            assert!(
                (centro.distance_to(*p) - raio).abs() <= folga,
                "ponto {p:?} fora do círculo"
            );
        }
        for par in pontos.windows(2) {
            let f = flecha(centro, raio, par[0], par[1]);
            assert!(f <= erro + folga, "flecha {f} acima do erro {erro}");
        }
    }

    #[test]
    fn a_reta_vira_os_dois_extremos() {
        let reta = Line {
            start: Point2::new(1.0, 2.0),
            end: Point2::new(3.0, 5.0),
        };

        let pontos = tessellate(&AnyCurve::Line(reta), 0.01).expect("erro válido");

        assert_eq!(pontos, vec![reta.start, reta.end]);
    }

    #[test]
    fn o_arco_respeita_o_erro_de_corda_e_comeca_e_termina_nas_pontas() {
        let arco = Arc {
            center: Point2::new(10.0, -4.0),
            radius: 25.0,
            start_angle: 0.3,
            end_angle: 0.3 + 2.0,
        };

        let pontos = tessellate(&AnyCurve::Arc(arco), 0.01).expect("erro válido");

        confere_arco(&pontos, arco.center, arco.radius, 0.01);
        let tol = Tolerance::default();
        let inicio = Point2::new(10.0 + 25.0 * 0.3_f64.cos(), -4.0 + 25.0 * 0.3_f64.sin());
        let fim = Point2::new(10.0 + 25.0 * 2.3_f64.cos(), -4.0 + 25.0 * 2.3_f64.sin());
        assert!(
            tol.points_equal(pontos[0], inicio),
            "início {:?}",
            pontos[0]
        );
        assert!(tol.points_equal(*pontos.last().expect("não vazio"), fim));
    }

    #[test]
    fn o_arco_que_cruza_o_angulo_zero_segue_no_sentido_anti_horario() {
        let arco = Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle: 3.0 * FRAC_PI_2,
            end_angle: FRAC_PI_2,
        };

        let pontos = tessellate(&AnyCurve::Arc(arco), 1e-3).expect("erro válido");

        confere_arco(&pontos, arco.center, arco.radius, 1e-3);
        // Passa por (1, 0), e não por (-1, 0).
        assert!(pontos.iter().all(|p| p.x >= -1e-9), "foi pelo lado errado");
    }

    #[test]
    fn a_circunferencia_fecha_e_respeita_o_erro() {
        let circulo = Circle {
            center: Point2::new(-3.0, 7.0),
            radius: 2.0,
        };

        let pontos = tessellate(&AnyCurve::Circle(circulo), 1e-3).expect("erro válido");

        confere_arco(&pontos, circulo.center, circulo.radius, 1e-3);
        assert!(Tolerance::default().points_equal(pontos[0], *pontos.last().expect("não vazio")));
    }

    #[test]
    fn a_divisao_nao_desperdica_segmentos() {
        // Raio 1 e erro 1e-3: cada corda cobre no máximo 2·acos(0,999) rad, o
        // que dá 71 cordas na volta inteira. Mais que isso é desperdício.
        let circulo = Circle {
            center: Point2::ORIGIN,
            radius: 1.0,
        };

        let pontos = tessellate(&AnyCurve::Circle(circulo), 1e-3).expect("erro válido");

        assert_eq!(pontos.len() - 1, 71);
    }

    #[test]
    fn longe_da_origem_o_erro_continua_valendo() {
        // Coordenada UTM: a divisão é calculada em torno do centro, não na
        // coordenada absoluta.
        let circulo = Circle {
            center: Point2::new(5_000_000.0, 7_500_000.0),
            radius: 3.0,
        };

        let pontos = tessellate(&AnyCurve::Circle(circulo), 1e-3).expect("erro válido");

        confere_arco(&pontos, circulo.center, circulo.radius, 1e-3);
    }

    #[test]
    fn erro_maior_que_o_raio_ainda_desenha_um_poligono() {
        let circulo = Circle {
            center: Point2::ORIGIN,
            radius: 1.0,
        };

        let pontos = tessellate(&AnyCurve::Circle(circulo), 10.0).expect("erro válido");

        assert!(pontos.len() > 3, "círculo com {} cordas", pontos.len() - 1);
    }

    #[test]
    fn a_polilinha_passa_por_todos_os_vertices_na_ordem_e_respeita_o_erro() {
        // Meia-volta anti-horária de (0,0) a (2,0), reta até (2,-3) e meia-volta
        // horária (abaulamento negativo) até (0,-3), fechando em (0,0).
        let polilinha = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::new(0.0, 0.0), 1.0),
                PolylineVertex::straight(Point2::new(2.0, 0.0)),
                PolylineVertex::new(Point2::new(2.0, -3.0), -1.0),
                PolylineVertex::straight(Point2::new(0.0, -3.0)),
            ],
            closed: true,
        };

        let pontos = tessellate(&AnyCurve::Polyline(polilinha.clone()), 1e-3).expect("erro válido");

        let tol = Tolerance::default();
        let mut proximo = 0;
        for p in &pontos {
            if proximo < polilinha.vertices.len()
                && tol.points_equal(*p, polilinha.vertices[proximo].point)
            {
                proximo += 1;
            }
        }
        assert_eq!(
            proximo,
            polilinha.vertices.len(),
            "vértice pulado ou fora de ordem"
        );
        assert!(
            tol.points_equal(pontos[0], *pontos.last().expect("não vazio")),
            "não fechou"
        );
        for par in pontos.windows(2) {
            assert!(
                !tol.points_equal(par[0], par[1]),
                "ponto repetido em {:?}",
                par[0]
            );
        }

        // O primeiro trecho é a meia-volta de centro (1,0): vai por baixo do eixo?
        // Abaulamento positivo de (0,0) a (2,0) é anti-horário, logo por baixo.
        let meia_volta: Vec<Point2> = pontos
            .iter()
            .copied()
            .take_while(|p| !tol.points_equal(*p, Point2::new(2.0, 0.0)))
            .chain(std::iter::once(Point2::new(2.0, 0.0)))
            .collect();
        confere_arco(&meia_volta, Point2::new(1.0, 0.0), 1.0, 1e-3);
        assert!(
            meia_volta.iter().all(|p| p.y <= 1e-9),
            "meia-volta pelo lado errado"
        );
    }

    #[test]
    fn o_arco_da_polilinha_com_abaulamento_negativo_vai_do_vertice_ao_seguinte() {
        let polilinha = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::new(0.0, 0.0), -1.0),
                PolylineVertex::straight(Point2::new(2.0, 0.0)),
            ],
            closed: false,
        };

        let pontos = tessellate(&AnyCurve::Polyline(polilinha), 1e-3).expect("erro válido");

        let tol = Tolerance::default();
        assert!(tol.points_equal(pontos[0], Point2::ORIGIN));
        assert!(tol.points_equal(*pontos.last().expect("não vazio"), Point2::new(2.0, 0.0)));
        confere_arco(&pontos, Point2::new(1.0, 0.0), 1.0, 1e-3);
        // Horário de (0,0) a (2,0): passa por cima.
        assert!(pontos.iter().all(|p| p.y >= -1e-9));
    }

    #[test]
    fn erro_de_corda_invalido_e_recusado() {
        let circulo = AnyCurve::Circle(Circle {
            center: Point2::ORIGIN,
            radius: 1.0,
        });

        for erro in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(tessellate(&circulo, erro), None, "erro {erro} aceito");
        }
    }

    #[test]
    fn arco_de_raio_zero_reduz_se_ao_centro() {
        let arco = Arc {
            center: Point2::new(4.0, 4.0),
            radius: 0.0,
            start_angle: 0.0,
            end_angle: PI,
        };

        let pontos = tessellate(&AnyCurve::Arc(arco), 1e-3).expect("erro válido");

        assert_eq!(pontos, vec![arco.center, arco.center]);
    }
}
