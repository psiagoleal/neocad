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

/// Sequência de vértices ligados por segmentos de reta.
///
/// Não modela abaulamento (`bulge`) de segmento, que o DXF usa para representar
/// trechos em arco dentro de uma polilinha. Entra no MT-K3-03, junto com as
/// operações que dependem dele.
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    /// Vértices, em ordem de percurso.
    pub vertices: Vec<Point2>,
    /// Se verdadeiro, há um segmento ligando o último vértice ao primeiro.
    pub closed: bool,
}
