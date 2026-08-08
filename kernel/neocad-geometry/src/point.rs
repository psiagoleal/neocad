// Caminho relativo: kernel/neocad-geometry/src/point.rs
//! \file kernel/neocad-geometry/src/point.rs
//! \brief Ponto no plano.
//! \author Iago Leal
//! \date 2026-08-07

/// Ponto no plano cartesiano, em unidades do desenho.
///
/// As coordenadas são `f64` porque desenhos CAD combinam, no mesmo arquivo,
/// dimensões de milímetros e coordenadas geográficas de milhões de unidades;
/// `f32` perde precisão nessa faixa.
///
/// # Igualdade
///
/// A comparação é exata, herdada de `f64`. Comparação com tolerância pertence às
/// operações geométricas de K3, onde a tolerância do documento estará definida.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point2 {
    /// Coordenada no eixo X.
    pub x: f64,
    /// Coordenada no eixo Y.
    pub y: f64,
}

impl Point2 {
    /// Origem do sistema de coordenadas.
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };

    /// Cria um ponto a partir de suas coordenadas.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Indica se ambas as coordenadas são finitas.
    ///
    /// Útil como guarda ao consumir dados de arquivo, onde `NaN` e infinito
    /// aparecem em arquivos corrompidos.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Distância euclidiana até outro ponto.
    ///
    /// Usa `hypot`, que evita estouro intermediário quando as coordenadas são
    /// grandes — situação corriqueira em desenhos georreferenciados.
    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }

    /// Devolve o ponto deslocado por `dx` e `dy`.
    #[must_use]
    pub const fn translated(self, dx: f64, dy: f64) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }

    /// Devolve o ponto girado em torno de `pivot` por `angle`, em radianos,
    /// no sentido anti-horário.
    #[must_use]
    pub fn rotated_around(self, pivot: Self, angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        let dx = self.x - pivot.x;
        let dy = self.y - pivot.y;

        Self::new(pivot.x + dx * cos - dy * sin, pivot.y + dx * sin + dy * cos)
    }
}

/// Ponto sobre uma circunferência.
///
/// O ângulo é medido em radianos, no sentido anti-horário a partir do eixo X
/// positivo — a convenção do DXF.
#[must_use]
pub fn point_on_circle(center: Point2, radius: f64, angle: f64) -> Point2 {
    let (sin, cos) = angle.sin_cos();
    Point2::new(center.x + radius * cos, center.y + radius * sin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64::consts::{FRAC_PI_2, PI};

    const TOLERANCE: f64 = 1e-12;

    fn assert_close(actual: Point2, expected: Point2) {
        assert!(
            (actual.x - expected.x).abs() < TOLERANCE && (actual.y - expected.y).abs() < TOLERANCE,
            "esperado {expected:?}, obtido {actual:?}"
        );
    }

    #[test]
    fn origem_tem_coordenadas_zero() {
        assert_eq!(Point2::ORIGIN, Point2::new(0.0, 0.0));
        assert_eq!(Point2::default(), Point2::ORIGIN);
    }

    #[test]
    fn distancia_entre_pontos() {
        assert!((Point2::new(3.0, 4.0).distance_to(Point2::ORIGIN) - 5.0).abs() < TOLERANCE);
    }

    #[test]
    fn deteccao_de_coordenada_nao_finita() {
        assert!(Point2::new(1.0, 2.0).is_finite());
        assert!(!Point2::new(f64::NAN, 0.0).is_finite());
        assert!(!Point2::new(0.0, f64::INFINITY).is_finite());
    }

    #[test]
    fn translacao_desloca_as_duas_coordenadas() {
        assert_eq!(
            Point2::new(1.0, 2.0).translated(3.0, -1.0),
            Point2::new(4.0, 1.0)
        );
    }

    #[test]
    fn rotacao_de_noventa_graus_em_torno_da_origem() {
        assert_close(
            Point2::new(1.0, 0.0).rotated_around(Point2::ORIGIN, FRAC_PI_2),
            Point2::new(0.0, 1.0),
        );
    }

    #[test]
    fn rotacao_em_torno_de_pivo_deslocado() {
        let pivot = Point2::new(2.0, 2.0);

        assert_close(
            Point2::new(3.0, 2.0).rotated_around(pivot, PI),
            Point2::new(1.0, 2.0),
        );
    }

    #[test]
    fn ponto_na_circunferencia_segue_convencao_anti_horaria() {
        let center = Point2::new(1.0, 1.0);

        assert_close(point_on_circle(center, 2.0, 0.0), Point2::new(3.0, 1.0));
        assert_close(
            point_on_circle(center, 2.0, FRAC_PI_2),
            Point2::new(1.0, 3.0),
        );
        assert_close(point_on_circle(center, 2.0, PI), Point2::new(-1.0, 1.0));
    }
}
