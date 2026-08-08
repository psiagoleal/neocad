// Caminho relativo: kernel/neocad-geometry/src/aabb.rs
//! \file kernel/neocad-geometry/src/aabb.rs
//! \brief Caixa envolvente alinhada aos eixos.
//! \author Iago Leal
//! \date 2026-08-07

use crate::point::Point2;

/// Caixa envolvente alinhada aos eixos, no plano.
///
/// É a aproximação grosseira da extensão de uma entidade, usada por ajuste de
/// vista, seleção por janela e, mais adiante, pelo índice espacial de K3.
///
/// # Invariante
///
/// `min.x <= max.x` e `min.y <= max.y`, sempre. Os construtores normalizam, de
/// modo que não existe caixa invertida.
///
/// # Ausência de caixa vazia
///
/// Não há representação de caixa vazia: um único ponto produz uma caixa
/// degenerada, de largura e altura zero, que continua sendo uma caixa válida. A
/// ausência de extensão é representada por `Option<Aabb>` em quem agrega —
/// alternativa preferida a um estado interno inválido que todo consumidor teria
/// de checar.
///
/// # Exemplo
///
/// ```
/// use neocad_geometry::{Aabb, Point2};
///
/// let caixa = Aabb::new(Point2::new(3.0, 1.0), Point2::new(-1.0, 4.0));
///
/// assert_eq!(caixa.min(), Point2::new(-1.0, 1.0));
/// assert_eq!(caixa.max(), Point2::new(3.0, 4.0));
/// assert_eq!(caixa.width(), 4.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    min: Point2,
    max: Point2,
}

impl Aabb {
    /// Cria a caixa que contém os dois pontos, normalizando os limites.
    #[must_use]
    pub fn new(a: Point2, b: Point2) -> Self {
        Self {
            min: Point2::new(a.x.min(b.x), a.y.min(b.y)),
            max: Point2::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    /// Cria a caixa degenerada de um único ponto.
    #[must_use]
    pub const fn from_point(point: Point2) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    /// Cria a caixa que contém todos os pontos, ou `None` se a sequência for
    /// vazia.
    #[must_use]
    pub fn from_points(points: impl IntoIterator<Item = Point2>) -> Option<Self> {
        let mut points = points.into_iter();
        let first = points.next()?;

        Some(points.fold(Self::from_point(first), Self::expanded_to_include))
    }

    /// Canto inferior esquerdo.
    #[must_use]
    pub const fn min(self) -> Point2 {
        self.min
    }

    /// Canto superior direito.
    #[must_use]
    pub const fn max(self) -> Point2 {
        self.max
    }

    /// Extensão no eixo X.
    #[must_use]
    pub fn width(self) -> f64 {
        self.max.x - self.min.x
    }

    /// Extensão no eixo Y.
    #[must_use]
    pub fn height(self) -> f64 {
        self.max.y - self.min.y
    }

    /// Ponto central.
    #[must_use]
    pub fn center(self) -> Point2 {
        // `f64::midpoint` só é estável a partir do Rust 1.85 e a MSRV declarada
        // no workspace é 1.82.
        Point2::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }

    /// Indica se a caixa não tem extensão em nenhum dos eixos.
    #[must_use]
    pub fn is_degenerate(self) -> bool {
        self.width() == 0.0 && self.height() == 0.0
    }

    /// Indica se o ponto está dentro da caixa, incluindo a borda.
    #[must_use]
    pub fn contains(self, point: Point2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Menor caixa que contém esta e a outra.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self {
            min: Point2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: Point2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }

    /// Menor caixa que contém esta e o ponto.
    #[must_use]
    pub fn expanded_to_include(self, point: Point2) -> Self {
        Self {
            min: Point2::new(self.min.x.min(point.x), self.min.y.min(point.y)),
            max: Point2::new(self.max.x.max(point.x), self.max.y.max(point.y)),
        }
    }

    /// Menor caixa que contém todas as caixas, ou `None` se a sequência for
    /// vazia.
    #[must_use]
    pub fn union_all(boxes: impl IntoIterator<Item = Self>) -> Option<Self> {
        boxes.into_iter().reduce(Self::union)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construtor_normaliza_limites_invertidos() {
        let caixa = Aabb::new(Point2::new(3.0, 4.0), Point2::new(-1.0, 1.0));

        assert_eq!(caixa.min(), Point2::new(-1.0, 1.0));
        assert_eq!(caixa.max(), Point2::new(3.0, 4.0));
    }

    #[test]
    fn caixa_de_ponto_unico_e_degenerada() {
        let caixa = Aabb::from_point(Point2::new(2.0, 2.0));

        assert!(caixa.is_degenerate());
        assert_eq!(caixa.width(), 0.0);
        assert_eq!(caixa.height(), 0.0);
        assert_eq!(caixa.center(), Point2::new(2.0, 2.0));
    }

    #[test]
    fn dimensoes_e_centro() {
        let caixa = Aabb::new(Point2::new(0.0, 0.0), Point2::new(4.0, 2.0));

        assert_eq!(caixa.width(), 4.0);
        assert_eq!(caixa.height(), 2.0);
        assert_eq!(caixa.center(), Point2::new(2.0, 1.0));
        assert!(!caixa.is_degenerate());
    }

    #[test]
    fn from_points_vazio_devolve_none() {
        assert_eq!(Aabb::from_points(core::iter::empty()), None);
    }

    #[test]
    fn from_points_envolve_todos_os_pontos() {
        let caixa = Aabb::from_points([
            Point2::new(1.0, 1.0),
            Point2::new(-2.0, 5.0),
            Point2::new(3.0, -1.0),
        ])
        .expect("sequência não vazia");

        assert_eq!(caixa.min(), Point2::new(-2.0, -1.0));
        assert_eq!(caixa.max(), Point2::new(3.0, 5.0));
    }

    #[test]
    fn contains_inclui_a_borda() {
        let caixa = Aabb::new(Point2::ORIGIN, Point2::new(2.0, 2.0));

        assert!(caixa.contains(Point2::new(1.0, 1.0)));
        assert!(caixa.contains(Point2::ORIGIN));
        assert!(caixa.contains(Point2::new(2.0, 2.0)));
        assert!(!caixa.contains(Point2::new(2.5, 1.0)));
        assert!(!caixa.contains(Point2::new(1.0, -0.1)));
    }

    #[test]
    fn uniao_de_caixas_disjuntas() {
        let esquerda = Aabb::new(Point2::ORIGIN, Point2::new(1.0, 1.0));
        let direita = Aabb::new(Point2::new(5.0, 5.0), Point2::new(6.0, 7.0));

        let uniao = esquerda.union(direita);

        assert_eq!(uniao.min(), Point2::ORIGIN);
        assert_eq!(uniao.max(), Point2::new(6.0, 7.0));
    }

    #[test]
    fn uniao_com_caixa_contida_nao_muda_nada() {
        let externa = Aabb::new(Point2::ORIGIN, Point2::new(10.0, 10.0));
        let interna = Aabb::new(Point2::new(2.0, 2.0), Point2::new(3.0, 3.0));

        assert_eq!(externa.union(interna), externa);
    }

    #[test]
    fn expansao_para_incluir_ponto_externo() {
        let caixa = Aabb::new(Point2::ORIGIN, Point2::new(1.0, 1.0));

        let expandida = caixa.expanded_to_include(Point2::new(-3.0, 0.5));

        assert_eq!(expandida.min(), Point2::new(-3.0, 0.0));
        assert_eq!(expandida.max(), Point2::new(1.0, 1.0));
    }

    #[test]
    fn union_all_vazio_devolve_none() {
        assert_eq!(Aabb::union_all(core::iter::empty()), None);
    }

    #[test]
    fn union_all_combina_todas() {
        let caixa = Aabb::union_all([
            Aabb::from_point(Point2::new(1.0, 1.0)),
            Aabb::from_point(Point2::new(-1.0, 4.0)),
        ])
        .expect("sequência não vazia");

        assert_eq!(caixa.min(), Point2::new(-1.0, 1.0));
        assert_eq!(caixa.max(), Point2::new(1.0, 4.0));
    }
}
