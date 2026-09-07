// Caminho relativo: kernel/neocad-geometry/src/curve.rs
//! \file kernel/neocad-geometry/src/curve.rs
//! \brief Curvas 2D do kernel: reta, circunferência, arco e polilinha.
//! \author Iago Leal
//! \date 2026-09-01
//!
//! São formas puras, sem camada, cor ou identidade — o que faz delas uma
//! entidade de desenho é a `Geometry` de `neocad-model`, que as embrulha.
//!
//! Nasceram em `neocad-model` porque K1 precisou de entidade antes de precisar
//! de geometria; o MT-K3-01 as trouxe para o lugar que o ADR 0003 lhes destina.

use crate::Point2;

/// Segmento de reta entre dois pontos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// Ponto inicial.
    pub start: Point2,
    /// Ponto final.
    pub end: Point2,
}

/// Circunferência completa.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    /// Centro.
    pub center: Point2,
    /// Raio.
    pub radius: f64,
}

/// Arco de circunferência.
///
/// Os ângulos são medidos em radianos, no sentido anti-horário a partir do eixo
/// X positivo, e o arco vai de `start_angle` até `end_angle` **nesse sentido** —
/// a convenção do DXF. Um arco com `start_angle == end_angle` é degenerado e
/// reduz-se a um ponto; circunferências completas são representadas por
/// [`Circle`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc {
    /// Centro.
    pub center: Point2,
    /// Raio.
    pub radius: f64,
    /// Ângulo inicial, em radianos.
    pub start_angle: f64,
    /// Ângulo final, em radianos.
    pub end_angle: f64,
}

/// Vértice de polilinha, com o abaulamento do segmento que começa nele.
///
/// O abaulamento é a `tan(θ/4)` do ângulo `θ` varrido pelo arco, convenção do
/// DXF: zero é trecho reto, positivo varre no sentido anti-horário, e `1` é a
/// semicircunferência. Guardá-lo no vértice, e não numa lista paralela, impede
/// que as duas listas fiquem com comprimentos diferentes — que é a única forma
/// de uma polilinha se tornar incoerente consigo mesma.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PolylineVertex {
    /// Posição do vértice.
    pub point: Point2,
    /// Abaulamento do segmento que **começa** neste vértice.
    pub bulge: f64,
}

impl PolylineVertex {
    /// Vértice com abaulamento.
    #[must_use]
    pub const fn new(point: Point2, bulge: f64) -> Self {
        Self { point, bulge }
    }

    /// Vértice de trecho reto.
    #[must_use]
    pub const fn straight(point: Point2) -> Self {
        Self { point, bulge: 0.0 }
    }
}

impl From<Point2> for PolylineVertex {
    fn from(point: Point2) -> Self {
        Self::straight(point)
    }
}

/// Sequência de vértices ligados por segmentos de reta ou de arco.
///
/// # Exemplo
///
/// ```
/// use neocad_geometry::{Point2, Polyline, PolylineVertex};
///
/// // Meia-volta de (0,0) a (2,0), depois um trecho reto de volta.
/// let polilinha = Polyline {
///     vertices: vec![
///         PolylineVertex::new(Point2::ORIGIN, 1.0),
///         PolylineVertex::straight(Point2::new(2.0, 0.0)),
///     ],
///     closed: true,
/// };
///
/// let arco = polilinha.segment_arc(0).expect("o primeiro trecho é curvo");
///
/// assert_eq!(arco.center, Point2::new(1.0, 0.0));
/// assert_eq!(arco.radius, 1.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    /// Vértices, em ordem de percurso.
    pub vertices: Vec<PolylineVertex>,
    /// Se verdadeiro, há um segmento ligando o último vértice ao primeiro.
    pub closed: bool,
}

impl Polyline {
    /// Polilinha toda reta, a partir de uma sequência de pontos.
    #[must_use]
    pub fn from_points(points: impl IntoIterator<Item = Point2>, closed: bool) -> Self {
        Self {
            vertices: points.into_iter().map(PolylineVertex::straight).collect(),
            closed,
        }
    }

    /// Posições dos vértices, na ordem de percurso.
    pub fn points(&self) -> impl Iterator<Item = Point2> + '_ {
        self.vertices.iter().map(|vertex| vertex.point)
    }

    /// Quantidade de segmentos, contando o de fechamento quando houver.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        match self.vertices.len() {
            0 | 1 => 0,
            count if self.closed => count,
            count => count - 1,
        }
    }

    /// Extremos do segmento que começa no vértice `index`.
    #[must_use]
    pub fn segment_ends(&self, index: usize) -> Option<(Point2, Point2)> {
        if index >= self.segment_count() {
            return None;
        }

        let start = self.vertices[index].point;
        let end = self.vertices[(index + 1) % self.vertices.len()].point;

        Some((start, end))
    }

    /// Arco do segmento que começa no vértice `index`, quando ele for curvo.
    ///
    /// Devolve `None` para trecho reto, para segmento de comprimento nulo e para
    /// índice fora da faixa. O arco devolvido é sempre anti-horário, como manda
    /// [`Arc`]; num abaulamento negativo o percurso da polilinha vai do fim para
    /// o começo dele.
    #[must_use]
    pub fn segment_arc(&self, index: usize) -> Option<Arc> {
        let (start, end) = self.segment_ends(index)?;
        let bulge = self.vertices[index].bulge;

        if bulge == 0.0 || !bulge.is_finite() {
            return None;
        }

        let chord = (end.x - start.x, end.y - start.y);
        let length = start.distance_to(end);

        if length == 0.0 || !length.is_finite() {
            return None;
        }

        // Meia-corda sobre o seno de meio ângulo dá o raio; escrito em função do
        // abaulamento, poupa uma trigonometria e evita o caso em que o seno
        // calculado escapa de `[-1, 1]` por arredondamento.
        let radius = length * (1.0 + bulge * bulge) / (4.0 * bulge.abs());

        // O centro fica sobre a perpendicular que passa pelo meio da corda. Ele
        // cai à esquerda do percurso quando o abaulamento é positivo, que é o
        // que faz o arco pender para a direita.
        let offset = length * (1.0 - bulge * bulge) / (4.0 * bulge);
        let normal = (-chord.1 / length, chord.0 / length);
        let center = Point2::new(
            (start.x + end.x) / 2.0 + normal.0 * offset,
            (start.y + end.y) / 2.0 + normal.1 * offset,
        );

        let angle_of = |point: Point2| (point.y - center.y).atan2(point.x - center.x);
        let (first, second) = if bulge > 0.0 {
            (start, end)
        } else {
            (end, start)
        };

        Some(Arc {
            center,
            radius,
            start_angle: angle_of(first),
            end_angle: angle_of(second),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tolerance;
    use core::f64::consts::{FRAC_PI_2, FRAC_PI_8, PI};

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    #[test]
    fn abaulamento_um_e_meia_volta() {
        let polilinha = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 1.0),
                PolylineVertex::straight(Point2::new(2.0, 0.0)),
            ],
            closed: false,
        };
        let arco = polilinha.segment_arc(0).expect("o trecho é curvo");

        assert!(tol().points_equal(arco.center, Point2::new(1.0, 0.0)));
        assert!(tol().lengths_equal(arco.radius, 1.0));
        // Varre de π a 0 no sentido anti-horário, ou seja, por baixo da corda.
        assert!(tol().angles_equal(arco.start_angle, PI));
        assert!(tol().angles_equal(arco.end_angle, 0.0));
    }

    #[test]
    fn o_sinal_do_abaulamento_troca_o_lado_do_arco() {
        let arco_de = |bulge: f64| {
            Polyline {
                vertices: vec![
                    PolylineVertex::new(Point2::ORIGIN, bulge),
                    PolylineVertex::straight(Point2::new(2.0, 0.0)),
                ],
                closed: false,
            }
            .segment_arc(0)
            .expect("o trecho é curvo")
        };

        let positivo = arco_de(1.0);
        let negativo = arco_de(-1.0);

        // Mesmo centro e mesmo raio; o que muda é o sentido da varredura, e com
        // ele o lado da corda por onde o arco passa.
        assert!(tol().points_equal(positivo.center, negativo.center));
        assert!(tol().lengths_equal(positivo.radius, negativo.radius));
        assert!(tol().angles_equal(negativo.start_angle, 0.0));
        assert!(tol().angles_equal(negativo.end_angle, PI));
    }

    #[test]
    fn um_quarto_de_volta_tem_o_raio_da_corda_sobre_raiz_de_dois() {
        let polilinha = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, FRAC_PI_8.tan()),
                PolylineVertex::straight(Point2::new(1.0, 0.0)),
            ],
            closed: false,
        };
        let arco = polilinha.segment_arc(0).expect("o trecho é curvo");

        assert!(tol().lengths_equal(arco.radius, 1.0 / 2.0_f64.sqrt()));
        assert!(tol().points_equal(arco.center, Point2::new(0.5, 0.5)));

        let varredura = arco.end_angle - arco.start_angle;

        assert!(tol().angles_equal(varredura.rem_euclid(2.0 * PI), FRAC_PI_2));
    }

    #[test]
    fn trecho_reto_nao_tem_arco() {
        let polilinha = Polyline::from_points([Point2::ORIGIN, Point2::new(1.0, 0.0)], false);

        assert!(polilinha.segment_arc(0).is_none());
    }

    #[test]
    fn segmento_de_comprimento_nulo_nao_tem_arco() {
        let polilinha = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 1.0),
                PolylineVertex::straight(Point2::ORIGIN),
            ],
            closed: false,
        };

        assert!(polilinha.segment_arc(0).is_none());
    }

    #[test]
    fn a_contagem_de_segmentos_conta_o_fechamento() {
        let pontos = [Point2::ORIGIN, Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)];

        assert_eq!(Polyline::from_points(pontos, false).segment_count(), 2);
        assert_eq!(Polyline::from_points(pontos, true).segment_count(), 3);
        assert_eq!(
            Polyline::from_points([Point2::ORIGIN], true).segment_count(),
            0
        );
        assert_eq!(Polyline::from_points([], false).segment_count(), 0);
    }

    #[test]
    fn o_ultimo_segmento_de_uma_polilinha_fechada_volta_ao_comeco() {
        let pontos = [Point2::ORIGIN, Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)];
        let fechada = Polyline::from_points(pontos, true);

        assert_eq!(
            fechada.segment_ends(2),
            Some((Point2::new(1.0, 1.0), Point2::ORIGIN))
        );
        assert_eq!(Polyline::from_points(pontos, false).segment_ends(2), None);
    }

    #[test]
    fn indice_fora_da_faixa_nao_tem_arco() {
        let polilinha = Polyline::from_points([Point2::ORIGIN, Point2::new(1.0, 0.0)], false);

        assert!(polilinha.segment_arc(1).is_none());
        assert!(polilinha.segment_arc(99).is_none());
    }

    #[test]
    fn o_arco_atravessa_a_conversao_em_coordenada_utm() {
        let base = 7.0e6;
        let polilinha = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::new(base, base), 1.0),
                PolylineVertex::straight(Point2::new(base + 2.0, base)),
            ],
            closed: false,
        };
        let arco = polilinha.segment_arc(0).expect("o trecho é curvo");

        assert!(tol().points_equal(arco.center, Point2::new(base + 1.0, base)));
        assert!(tol().lengths_equal(arco.radius, 1.0));
    }
}
