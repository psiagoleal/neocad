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

use core::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::{point_on_circle, Aabb, Point2};

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

/// Ponto de uma curva, achado por proximidade.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClosestPoint {
    /// Parâmetro em `[0, 1]` onde o ponto foi encontrado.
    pub parameter: f64,
    /// O ponto sobre a curva.
    pub point: Point2,
    /// Distância entre o alvo e esse ponto.
    pub distance: f64,
}

/// O que toda curva 2D do kernel sabe responder.
///
/// Existe para que aparo, paralela e concordância não reescrevam, cada uma, o
/// mesmo `match` sobre as quatro primitivas — cada cópia sendo uma chance de as
/// quatro divergirem.
///
/// # O parâmetro
///
/// Vai de `0` a `1`, do começo ao fim da curva, e é **grampeado** nessa faixa:
/// pedir `1,5` devolve o fim, não a extensão da curva além dele. Estender é uma
/// operação própria, com nome próprio, e não um efeito colateral de avaliar.
///
/// O parâmetro **não** é comprimento de arco: numa polilinha ele se reparte
/// igualmente entre os segmentos, de modo que `0,5` cai no meio da contagem de
/// segmentos e não no meio do percurso. Parametrizar por comprimento exige
/// integração numérica e não tem consumidor nesta fase.
pub trait Curve {
    /// Ponto da curva no parâmetro dado.
    fn point_at(&self, parameter: f64) -> Point2;

    /// Direção da curva no parâmetro dado, em radianos.
    ///
    /// É um ângulo, e não um vetor, porque é assim que o kernel já fala de
    /// direção — e porque um ângulo não deixa dúvida sobre estar normalizado.
    /// Devolve `None` onde a curva não tem direção definida: segmento de
    /// comprimento nulo, raio nulo, arco degenerado.
    fn tangent_at(&self, parameter: f64) -> Option<f64>;

    /// Comprimento total da curva.
    fn length(&self) -> f64;

    /// Ponto da curva mais próximo do alvo.
    fn closest_point(&self, target: Point2) -> ClosestPoint;

    /// Caixa envolvente da curva.
    fn bounding_box(&self) -> Aabb;
}

/// Grampeia o parâmetro na faixa válida.
///
/// Parâmetro não numérico vira `0`: não há resposta melhor, e devolver `NaN`
/// espalharia o defeito para dentro da geometria.
fn clamped(parameter: f64) -> f64 {
    if parameter.is_nan() {
        0.0
    } else {
        parameter.clamp(0.0, 1.0)
    }
}

/// Normaliza um ângulo para `[0, 2π)`.
fn normalized_angle(angle: f64) -> f64 {
    let remainder = angle % TAU;

    if remainder < 0.0 {
        remainder + TAU
    } else {
        remainder
    }
}

impl Curve for Line {
    fn point_at(&self, parameter: f64) -> Point2 {
        let t = clamped(parameter);

        Point2::new(
            self.start.x + (self.end.x - self.start.x) * t,
            self.start.y + (self.end.y - self.start.y) * t,
        )
    }

    fn tangent_at(&self, _parameter: f64) -> Option<f64> {
        let (dx, dy) = (self.end.x - self.start.x, self.end.y - self.start.y);

        if dx == 0.0 && dy == 0.0 {
            return None;
        }

        Some(dy.atan2(dx))
    }

    fn length(&self) -> f64 {
        self.start.distance_to(self.end)
    }

    fn closest_point(&self, target: Point2) -> ClosestPoint {
        let (dx, dy) = (self.end.x - self.start.x, self.end.y - self.start.y);
        let squared = dx * dx + dy * dy;

        // Segmento degenerado é um ponto, e o ponto mais próximo de um ponto é
        // ele mesmo, qualquer que seja o alvo.
        let parameter = if squared == 0.0 {
            0.0
        } else {
            let projection =
                ((target.x - self.start.x) * dx + (target.y - self.start.y) * dy) / squared;

            clamped(projection)
        };

        let point = self.point_at(parameter);

        ClosestPoint {
            parameter,
            point,
            distance: target.distance_to(point),
        }
    }

    fn bounding_box(&self) -> Aabb {
        Aabb::new(self.start, self.end)
    }
}

impl Curve for Circle {
    fn point_at(&self, parameter: f64) -> Point2 {
        point_on_circle(self.center, self.radius.abs(), clamped(parameter) * TAU)
    }

    fn tangent_at(&self, parameter: f64) -> Option<f64> {
        if self.radius == 0.0 {
            return None;
        }

        // A circunferência é percorrida no sentido anti-horário, então a
        // tangente está um quarto de volta adiante do raio.
        Some(clamped(parameter) * TAU + FRAC_PI_2)
    }

    fn length(&self) -> f64 {
        TAU * self.radius.abs()
    }

    fn closest_point(&self, target: Point2) -> ClosestPoint {
        let radius = self.radius.abs();
        let (dx, dy) = (target.x - self.center.x, target.y - self.center.y);

        // Do centro, todos os pontos da circunferência são igualmente próximos.
        // Escolher o de ângulo zero é arbitrário, mas é estável — e resposta
        // estável é o que impede a interface de piscar entre duas soluções.
        let angle = if dx == 0.0 && dy == 0.0 {
            0.0
        } else {
            normalized_angle(dy.atan2(dx))
        };

        let parameter = angle / TAU;
        let point = point_on_circle(self.center, radius, angle);

        ClosestPoint {
            parameter,
            point,
            distance: (self.center.distance_to(target) - radius).abs(),
        }
    }

    fn bounding_box(&self) -> Aabb {
        circle_bounding_box(self.center, self.radius)
    }
}

impl Arc {
    /// Ângulo varrido, sempre positivo, no sentido anti-horário.
    #[must_use]
    pub fn sweep(&self) -> f64 {
        normalized_angle(self.end_angle - self.start_angle)
    }
}

impl Curve for Arc {
    fn point_at(&self, parameter: f64) -> Point2 {
        point_on_circle(
            self.center,
            self.radius.abs(),
            self.start_angle + clamped(parameter) * self.sweep(),
        )
    }

    fn tangent_at(&self, parameter: f64) -> Option<f64> {
        if self.radius == 0.0 || self.sweep() == 0.0 {
            return None;
        }

        Some(self.start_angle + clamped(parameter) * self.sweep() + FRAC_PI_2)
    }

    fn length(&self) -> f64 {
        self.radius.abs() * self.sweep()
    }

    fn closest_point(&self, target: Point2) -> ClosestPoint {
        let radius = self.radius.abs();
        let sweep = self.sweep();
        let (dx, dy) = (target.x - self.center.x, target.y - self.center.y);

        if sweep != 0.0 && !(dx == 0.0 && dy == 0.0) {
            let offset = normalized_angle(dy.atan2(dx) - self.start_angle);

            // Dentro da varredura, o mais próximo é a projeção radial. Fora
            // dela, é uma das pontas — o arco não existe naquele ângulo.
            if offset <= sweep {
                let parameter = offset / sweep;
                let point = point_on_circle(self.center, radius, self.start_angle + offset);

                return ClosestPoint {
                    parameter,
                    point,
                    distance: (self.center.distance_to(target) - radius).abs(),
                };
            }
        }

        let start = self.point_at(0.0);
        let end = self.point_at(1.0);

        if target.distance_to(start) <= target.distance_to(end) {
            ClosestPoint {
                parameter: 0.0,
                point: start,
                distance: target.distance_to(start),
            }
        } else {
            ClosestPoint {
                parameter: 1.0,
                point: end,
                distance: target.distance_to(end),
            }
        }
    }

    fn bounding_box(&self) -> Aabb {
        arc_bounding_box(self)
    }
}

/// Um trecho de polilinha, reto ou curvo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolylineSegment {
    /// Trecho reto.
    Straight(Line),
    /// Trecho em arco.
    ///
    /// O arco é sempre anti-horário, como manda [`Arc`]. Quando `reversed` é
    /// verdadeiro, a polilinha o percorre do fim para o começo — que é o caso do
    /// abaulamento negativo.
    Curved {
        /// O arco, na sua orientação anti-horária.
        arc: Arc,
        /// Se a polilinha o percorre ao contrário.
        reversed: bool,
    },
}

impl PolylineSegment {
    /// Converte o parâmetro do trecho para o parâmetro do arco subjacente.
    fn arc_parameter(reversed: bool, parameter: f64) -> f64 {
        if reversed {
            1.0 - parameter
        } else {
            parameter
        }
    }
}

impl Curve for PolylineSegment {
    fn point_at(&self, parameter: f64) -> Point2 {
        match self {
            Self::Straight(line) => line.point_at(parameter),
            Self::Curved { arc, reversed } => {
                arc.point_at(Self::arc_parameter(*reversed, clamped(parameter)))
            }
        }
    }

    fn tangent_at(&self, parameter: f64) -> Option<f64> {
        match self {
            Self::Straight(line) => line.tangent_at(parameter),
            Self::Curved { arc, reversed } => {
                let tangent = arc.tangent_at(Self::arc_parameter(*reversed, clamped(parameter)))?;

                // Percorrido ao contrário, o trecho aponta para o lado oposto.
                Some(if *reversed { tangent + PI } else { tangent })
            }
        }
    }

    fn length(&self) -> f64 {
        match self {
            Self::Straight(line) => line.length(),
            Self::Curved { arc, .. } => arc.length(),
        }
    }

    fn closest_point(&self, target: Point2) -> ClosestPoint {
        match self {
            Self::Straight(line) => line.closest_point(target),
            Self::Curved { arc, reversed } => {
                let found = arc.closest_point(target);

                ClosestPoint {
                    parameter: Self::arc_parameter(*reversed, found.parameter),
                    ..found
                }
            }
        }
    }

    fn bounding_box(&self) -> Aabb {
        match self {
            Self::Straight(line) => line.bounding_box(),
            Self::Curved { arc, .. } => arc.bounding_box(),
        }
    }
}

impl Polyline {
    /// O trecho que começa no vértice `index`.
    #[must_use]
    pub fn segment(&self, index: usize) -> Option<PolylineSegment> {
        let (start, end) = self.segment_ends(index)?;

        match self.segment_arc(index) {
            Some(arc) => Some(PolylineSegment::Curved {
                arc,
                reversed: self.vertices[index].bulge < 0.0,
            }),
            None => Some(PolylineSegment::Straight(Line { start, end })),
        }
    }

    /// Todos os trechos, na ordem de percurso.
    pub fn segments(&self) -> impl Iterator<Item = PolylineSegment> + '_ {
        (0..self.segment_count()).filter_map(|index| self.segment(index))
    }

    /// Reparte o parâmetro global entre o trecho e o parâmetro dentro dele.
    fn locate(&self, parameter: f64) -> Option<(usize, f64)> {
        let count = self.segment_count();

        if count == 0 {
            return None;
        }

        // `as f64` é exato: uma polilinha com mais de 2^53 vértices não cabe na
        // memória muito antes de a conversão perder precisão.
        #[allow(clippy::cast_precision_loss)]
        let scaled = clamped(parameter) * count as f64;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let index = (scaled as usize).min(count - 1);
        #[allow(clippy::cast_precision_loss)]
        let local = scaled - index as f64;

        Some((index, local))
    }
}

impl Curve for Polyline {
    fn point_at(&self, parameter: f64) -> Point2 {
        match self
            .locate(parameter)
            .and_then(|(index, local)| self.segment(index).map(|segment| segment.point_at(local)))
        {
            Some(point) => point,
            // Sem trechos, a polilinha é um vértice solto ou nada.
            None => self.vertices.first().map_or(Point2::ORIGIN, |v| v.point),
        }
    }

    fn tangent_at(&self, parameter: f64) -> Option<f64> {
        let (index, local) = self.locate(parameter)?;

        self.segment(index)?.tangent_at(local)
    }

    fn length(&self) -> f64 {
        self.segments().map(|segment| segment.length()).sum()
    }

    fn closest_point(&self, target: Point2) -> ClosestPoint {
        let count = self.segment_count();

        if count == 0 {
            let point = self.vertices.first().map_or(Point2::ORIGIN, |v| v.point);

            return ClosestPoint {
                parameter: 0.0,
                point,
                distance: target.distance_to(point),
            };
        }

        let mut best: Option<ClosestPoint> = None;

        for index in 0..count {
            let Some(segment) = self.segment(index) else {
                continue;
            };
            let found = segment.closest_point(target);

            // O empate fica com o trecho de índice menor, e não com o último a
            // ser visto: percorrer a polilinha ao contrário não pode mudar a
            // resposta.
            if best
                .as_ref()
                .is_none_or(|atual| found.distance < atual.distance)
            {
                #[allow(clippy::cast_precision_loss)]
                let parameter = (index as f64 + found.parameter) / count as f64;

                best = Some(ClosestPoint { parameter, ..found });
            }
        }

        best.unwrap_or(ClosestPoint {
            parameter: 0.0,
            point: Point2::ORIGIN,
            distance: f64::INFINITY,
        })
    }

    fn bounding_box(&self) -> Aabb {
        let mut bounds =
            Aabb::from_points(self.points()).unwrap_or_else(|| Aabb::from_point(Point2::ORIGIN));

        // Os vértices não bastam: um trecho em arco sai da corda que liga os
        // dois, e a caixa tirada só deles cortaria o desenho no ajuste de vista.
        for segment in self.segments() {
            if let PolylineSegment::Curved { arc, .. } = segment {
                bounds = bounds.union(arc.bounding_box());
            }
        }

        bounds
    }
}

/// Caixa envolvente de uma circunferência completa.
fn circle_bounding_box(center: Point2, radius: f64) -> Aabb {
    let radius = radius.abs();

    Aabb::new(
        Point2::new(center.x - radius, center.y - radius),
        Point2::new(center.x + radius, center.y + radius),
    )
}

/// Caixa envolvente de um arco.
///
/// A caixa dos dois extremos **não basta**: um arco que cruza um dos eixos
/// cardeais atinge, nesse cruzamento, um valor extremo que nenhum dos extremos
/// alcança. Um arco de 315° a 45°, por exemplo, tem os dois extremos em
/// `x = r·cos(45°) ≈ 0,707·r`, mas passa por `x = r` ao cruzar 0°. Por isso cada
/// ângulo cardeal contido na varredura entra na caixa.
fn arc_bounding_box(arc: &Arc) -> Aabb {
    let radius = arc.radius.abs();
    let start = point_on_circle(arc.center, radius, arc.start_angle);
    let end = point_on_circle(arc.center, radius, arc.end_angle);

    let sweep = arc.sweep();
    let mut bounds = Aabb::new(start, end);

    for quarter in 0..4 {
        let angle = FRAC_PI_2 * f64::from(quarter);

        if normalized_angle(angle - arc.start_angle) <= sweep {
            bounds = bounds.expanded_to_include(point_on_circle(arc.center, radius, angle));
        }
    }

    bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tolerance;
    use core::f64::consts::{FRAC_PI_2, FRAC_PI_8, PI, TAU};

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    /// Uma amostra de cada primitiva, na magnitude pedida.
    fn primitivas(magnitude: f64) -> Vec<(&'static str, Box<dyn Curve>)> {
        let o = Point2::new(magnitude, magnitude);

        vec![
            (
                "reta",
                Box::new(Line {
                    start: o,
                    end: Point2::new(o.x + 4.0, o.y + 3.0),
                }) as Box<dyn Curve>,
            ),
            (
                "circunferência",
                Box::new(Circle {
                    center: o,
                    radius: 2.5,
                }),
            ),
            (
                "arco",
                Box::new(Arc {
                    center: o,
                    radius: 2.5,
                    start_angle: FRAC_PI_8,
                    end_angle: PI,
                }),
            ),
            (
                "polilinha com trecho curvo",
                Box::new(Polyline {
                    vertices: vec![
                        PolylineVertex::new(o, 1.0),
                        PolylineVertex::new(Point2::new(o.x + 2.0, o.y), -0.5),
                        PolylineVertex::straight(Point2::new(o.x + 2.0, o.y + 5.0)),
                    ],
                    closed: false,
                }),
            ),
        ]
    }

    #[test]
    fn o_mais_proximo_de_um_ponto_da_curva_e_ele_mesmo() {
        for magnitude in [1.0, 1.0e6] {
            for (nome, curva) in primitivas(magnitude) {
                for passo in 0..=10 {
                    let t = f64::from(passo) / 10.0;
                    let sobre = curva.point_at(t);
                    let achado = curva.closest_point(sobre);

                    assert!(
                        tol().points_equal(achado.point, sobre),
                        "{nome} em magnitude {magnitude}, t={t}: \
                         esperado {sobre:?}, obtido {:?}",
                        achado.point
                    );
                    assert!(
                        tol().is_zero_length(achado.distance),
                        "{nome}: distância {} para um ponto da própria curva",
                        achado.distance
                    );
                }
            }
        }
    }

    #[test]
    fn o_parametro_devolvido_reencontra_o_mesmo_ponto() {
        for (nome, curva) in primitivas(1.0) {
            let alvo = Point2::new(3.0, -2.0);
            let achado = curva.closest_point(alvo);

            assert!(
                tol().points_equal(curva.point_at(achado.parameter), achado.point),
                "{nome}: o parâmetro devolvido não reencontra o ponto"
            );
        }
    }

    #[test]
    fn o_parametro_e_grampeado_e_nao_estende_a_curva() {
        let reta = Line {
            start: Point2::ORIGIN,
            end: Point2::new(10.0, 0.0),
        };

        assert!(tol().points_equal(reta.point_at(-5.0), reta.start));
        assert!(tol().points_equal(reta.point_at(1.5), reta.end));
        assert!(tol().points_equal(reta.point_at(f64::NAN), reta.start));
    }

    #[test]
    fn os_comprimentos_seguem_a_geometria() {
        let reta = Line {
            start: Point2::ORIGIN,
            end: Point2::new(3.0, 4.0),
        };
        let circunferencia = Circle {
            center: Point2::ORIGIN,
            radius: 2.0,
        };
        let meio_arco = Arc {
            center: Point2::ORIGIN,
            radius: 2.0,
            start_angle: 0.0,
            end_angle: PI,
        };

        assert!(tol().lengths_equal(reta.length(), 5.0));
        assert!(tol().lengths_equal(circunferencia.length(), 2.0 * TAU));
        assert!(tol().lengths_equal(meio_arco.length(), 2.0 * PI));

        // Meia-volta de raio 1 seguida de um trecho reto de 5.
        let polilinha = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 1.0),
                PolylineVertex::straight(Point2::new(2.0, 0.0)),
                PolylineVertex::straight(Point2::new(2.0, 5.0)),
            ],
            closed: false,
        };

        assert!(tol().lengths_equal(polilinha.length(), PI + 5.0));
    }

    #[test]
    fn a_tangente_da_circunferencia_e_perpendicular_ao_raio() {
        let circunferencia = Circle {
            center: Point2::ORIGIN,
            radius: 3.0,
        };

        // Em t = 0 o ponto está em (3, 0) e a curva sobe.
        assert!(tol().angles_equal(circunferencia.tangent_at(0.0).unwrap(), FRAC_PI_2));
        assert!(tol().angles_equal(circunferencia.tangent_at(0.25).unwrap(), PI));
    }

    #[test]
    fn o_trecho_percorrido_ao_contrario_aponta_para_o_outro_lado() {
        // Dois trechos com o mesmo arco geométrico e abaulamentos opostos: a
        // polilinha os percorre em sentidos contrários.
        let direto = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 1.0),
                PolylineVertex::straight(Point2::new(2.0, 0.0)),
            ],
            closed: false,
        };
        let inverso = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, -1.0),
                PolylineVertex::straight(Point2::new(2.0, 0.0)),
            ],
            closed: false,
        };

        // Ambos saem da origem e chegam a (2, 0), qualquer que seja o lado.
        assert!(tol().points_equal(direto.point_at(0.0), Point2::ORIGIN));
        assert!(tol().points_equal(inverso.point_at(0.0), Point2::ORIGIN));
        assert!(tol().points_equal(direto.point_at(1.0), Point2::new(2.0, 0.0)));
        assert!(tol().points_equal(inverso.point_at(1.0), Point2::new(2.0, 0.0)));

        // No meio, um passa por baixo e o outro por cima.
        assert!(direto.point_at(0.5).y < 0.0);
        assert!(inverso.point_at(0.5).y > 0.0);
    }

    #[test]
    fn fora_da_varredura_o_arco_responde_com_a_ponta_mais_proxima() {
        let arco = Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle: 0.0,
            end_angle: FRAC_PI_2,
        };

        // Alvo no terceiro quadrante: o ângulo não está na varredura, e a ponta
        // mais próxima é a de 0°.
        let achado = arco.closest_point(Point2::new(2.0, -2.0));

        assert!(tol().points_equal(achado.point, Point2::new(1.0, 0.0)));
        assert!(tol().lengths_equal(achado.parameter, 0.0));
    }

    #[test]
    fn do_centro_a_circunferencia_responde_de_forma_estavel() {
        let circunferencia = Circle {
            center: Point2::new(5.0, 5.0),
            radius: 2.0,
        };
        let achado = circunferencia.closest_point(circunferencia.center);

        assert!(tol().points_equal(achado.point, Point2::new(7.0, 5.0)));
        assert!(tol().lengths_equal(achado.distance, 2.0));
        assert_eq!(achado, circunferencia.closest_point(circunferencia.center));
    }

    #[test]
    fn geometria_degenerada_nao_tem_direcao() {
        let ponto = Line {
            start: Point2::ORIGIN,
            end: Point2::ORIGIN,
        };
        let sem_raio = Circle {
            center: Point2::ORIGIN,
            radius: 0.0,
        };
        let arco_nulo = Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle: FRAC_PI_2,
            end_angle: FRAC_PI_2,
        };

        assert!(ponto.tangent_at(0.5).is_none());
        assert!(sem_raio.tangent_at(0.5).is_none());
        assert!(arco_nulo.tangent_at(0.5).is_none());
        assert!(Polyline::from_points([], false).tangent_at(0.5).is_none());
    }

    #[test]
    fn polilinha_sem_trecho_responde_pelo_vertice_que_tiver() {
        let solto = Polyline::from_points([Point2::new(3.0, 4.0)], false);

        assert!(tol().points_equal(solto.point_at(0.7), Point2::new(3.0, 4.0)));
        assert!(tol().lengths_equal(solto.length(), 0.0));
        assert!(tol().lengths_equal(solto.closest_point(Point2::ORIGIN).distance, 5.0));

        let vazia = Polyline::from_points([], false);

        assert!(tol().points_equal(vazia.point_at(0.5), Point2::ORIGIN));
        assert!(vazia.bounding_box().is_degenerate());
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
