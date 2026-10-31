// Caminho relativo: kernel/neocad-geometry/src/transform.rs
//! \file kernel/neocad-geometry/src/transform.rs
//! \brief Transformações de semelhança do plano.
//! \author Iago Leal
//! \date 2026-09-08
//!
//! Mover, girar, espelhar e escalar respondem pela maior parte do tempo de quem
//! edita um desenho, e as quatro são a mesma coisa vista de ângulos diferentes:
//! uma **semelhança** do plano.
//!
//! # Por que só semelhanças
//!
//! Uma transformação afim qualquer leva circunferência em elipse, e o modelo não
//! representa elipse. Escalar em `x` mais que em `y` produziria um objeto que o
//! kernel não sabe guardar, e o resultado seria descartado em silêncio ou
//! aproximado sem aviso — as duas coisas que este projeto recusa.
//!
//! Restringindo a semelhança — escala uniforme, rotação, reflexão e translação —
//! circunferência continua circunferência, arco continua arco, e o que sai da
//! operação é sempre representável. Escala não uniforme volta quando houver
//! elipse no modelo.

use crate::Point2;

/// Semelhança do plano: escala uniforme, reflexão, rotação e translação.
///
/// A ordem em que as partes se aplicam é fixa e é a que está escrita aqui:
/// escala, depois reflexão pelo eixo X, depois rotação, depois translação.
/// Qualquer semelhança se escreve assim.
///
/// # Exemplo
///
/// ```
/// use core::f64::consts::FRAC_PI_2;
/// use neocad_geometry::{Point2, Tolerance, Transform};
///
/// // Um quarto de volta em torno da origem leva o eixo X ao eixo Y.
/// let giro = Transform::rotation_around(Point2::ORIGIN, FRAC_PI_2);
///
/// assert!(Tolerance::default().points_equal(
///     giro.apply(Point2::new(1.0, 0.0)),
///     Point2::new(0.0, 1.0),
/// ));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    scale: f64,
    rotation: f64,
    reflected: bool,
    offset: (f64, f64),
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// A transformação que não move nada.
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            scale: 1.0,
            rotation: 0.0,
            reflected: false,
            offset: (0.0, 0.0),
        }
    }

    /// Translação por um deslocamento.
    #[must_use]
    pub const fn translation(dx: f64, dy: f64) -> Self {
        Self {
            scale: 1.0,
            rotation: 0.0,
            reflected: false,
            offset: (dx, dy),
        }
    }

    /// Rotação em torno de um ponto, em radianos e no sentido anti-horário.
    #[must_use]
    pub fn rotation_around(pivot: Point2, angle: f64) -> Self {
        Self::around(pivot, 1.0, angle, false)
    }

    /// Escala uniforme em torno de um ponto.
    ///
    /// Fator negativo é escala pelo módulo mais meia-volta, que é o que ele
    /// significa no plano — e não uma reflexão, já que o determinante continua
    /// positivo.
    #[must_use]
    pub fn scaling_around(pivot: Point2, factor: f64) -> Self {
        let rotation = if factor < 0.0 {
            core::f64::consts::PI
        } else {
            0.0
        };

        Self::around(pivot, factor.abs(), rotation, false)
    }

    /// Reflexão por uma reta que passa por `through` com a direção `angle`.
    #[must_use]
    pub fn mirror(through: Point2, angle: f64) -> Self {
        // Refletir pelo eixo X e depois girar `2·φ` é o mesmo que refletir por
        // uma reta a `φ` — é assim que a reflexão cabe na forma normal.
        Self::around(through, 1.0, 2.0 * angle, true)
    }

    /// Monta a transformação que fixa um ponto.
    fn around(pivot: Point2, scale: f64, rotation: f64, reflected: bool) -> Self {
        let linear = Self {
            scale,
            rotation,
            reflected,
            offset: (0.0, 0.0),
        };
        let moved = linear.apply(pivot);

        Self {
            offset: (pivot.x - moved.x, pivot.y - moved.y),
            ..linear
        }
    }

    /// Fator de escala, sempre positivo.
    #[must_use]
    pub const fn scale(self) -> f64 {
        self.scale
    }

    /// Rotação da parte linear, em radianos.
    #[must_use]
    pub const fn rotation(self) -> f64 {
        self.rotation
    }

    /// Se a transformação inverte a orientação do plano.
    ///
    /// É o que obriga arco e abaulamento a trocarem de sentido: o que era
    /// anti-horário passa a ser horário.
    #[must_use]
    pub const fn is_reflected(self) -> bool {
        self.reflected
    }

    /// Leva um ponto ao seu transformado.
    #[must_use]
    pub fn apply(self, point: Point2) -> Point2 {
        let (x, y) = (point.x * self.scale, point.y * self.scale);
        let y = if self.reflected { -y } else { y };
        let (cos, sin) = (self.rotation.cos(), self.rotation.sin());

        Point2::new(
            cos * x - sin * y + self.offset.0,
            sin * x + cos * y + self.offset.1,
        )
    }

    /// Leva uma direção ao seu transformado, em radianos.
    ///
    /// Direção não tem posição, então a translação não a afeta; a reflexão troca
    /// o sinal antes de a rotação somar.
    #[must_use]
    pub fn apply_angle(self, angle: f64) -> f64 {
        if self.reflected {
            self.rotation - angle
        } else {
            self.rotation + angle
        }
    }

    /// Aplica esta transformação e depois a outra.
    #[must_use]
    pub fn then(self, other: Self) -> Self {
        // Refletir e depois girar equivale a girar ao contrário e depois
        // refletir, e é daí que sai o sinal da rotação acumulada.
        let rotation = if other.reflected {
            other.rotation - self.rotation
        } else {
            other.rotation + self.rotation
        };
        let carried = other.apply(Point2::new(self.offset.0, self.offset.1));

        Self {
            scale: self.scale * other.scale,
            rotation,
            reflected: self.reflected != other.reflected,
            offset: (carried.x, carried.y),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tolerance;
    use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    fn igual(a: Point2, b: Point2) -> bool {
        tol().points_equal(a, b)
    }

    #[test]
    fn a_identidade_nao_move_nada() {
        let ponto = Point2::new(3.0, -7.0);

        assert!(igual(Transform::identity().apply(ponto), ponto));
        assert!(igual(Transform::default().apply(ponto), ponto));
    }

    #[test]
    fn a_translacao_desloca_sem_girar() {
        let movida = Transform::translation(2.0, -3.0);

        assert!(igual(movida.apply(Point2::ORIGIN), Point2::new(2.0, -3.0)));
        assert!(tol().angles_equal(movida.apply_angle(FRAC_PI_4), FRAC_PI_4));
        assert!(tol().lengths_equal(movida.scale(), 1.0));
    }

    #[test]
    fn a_rotacao_fixa_o_pivo() {
        let pivo = Point2::new(5.0, 5.0);
        let giro = Transform::rotation_around(pivo, FRAC_PI_2);

        assert!(igual(giro.apply(pivo), pivo));
        assert!(igual(
            giro.apply(Point2::new(6.0, 5.0)),
            Point2::new(5.0, 6.0)
        ));
        assert!(!giro.is_reflected());
    }

    #[test]
    fn a_escala_fixa_o_pivo_e_multiplica_a_distancia() {
        let pivo = Point2::new(1.0, 1.0);
        let dobro = Transform::scaling_around(pivo, 2.0);

        assert!(igual(dobro.apply(pivo), pivo));
        assert!(igual(
            dobro.apply(Point2::new(2.0, 1.0)),
            Point2::new(3.0, 1.0)
        ));
        assert!(tol().lengths_equal(dobro.scale(), 2.0));
    }

    #[test]
    fn fator_negativo_e_meia_volta_e_nao_reflexao() {
        let invertida = Transform::scaling_around(Point2::ORIGIN, -2.0);

        // Meia-volta leva (1,0) a (−2,0) e (0,1) a (0,−2): os dois eixos viram,
        // que é o que distingue isto de uma reflexão.
        assert!(igual(
            invertida.apply(Point2::new(1.0, 0.0)),
            Point2::new(-2.0, 0.0)
        ));
        assert!(igual(
            invertida.apply(Point2::new(0.0, 1.0)),
            Point2::new(0.0, -2.0)
        ));
        assert!(
            !invertida.is_reflected(),
            "escala negativa não inverte a orientação"
        );
    }

    #[test]
    fn a_reflexao_fixa_a_reta_e_troca_o_lado() {
        // Espelho no eixo X.
        let espelho = Transform::mirror(Point2::ORIGIN, 0.0);

        assert!(igual(
            espelho.apply(Point2::new(3.0, 0.0)),
            Point2::new(3.0, 0.0)
        ));
        assert!(igual(
            espelho.apply(Point2::new(3.0, 2.0)),
            Point2::new(3.0, -2.0)
        ));
        assert!(espelho.is_reflected());
    }

    #[test]
    fn a_reflexao_por_reta_inclinada_fixa_essa_reta() {
        let espelho = Transform::mirror(Point2::ORIGIN, FRAC_PI_4);

        // A reta a 45° passa por (2,2) e por (−1,−1); ambos ficam parados.
        assert!(igual(
            espelho.apply(Point2::new(2.0, 2.0)),
            Point2::new(2.0, 2.0)
        ));
        assert!(igual(
            espelho.apply(Point2::new(-1.0, -1.0)),
            Point2::new(-1.0, -1.0)
        ));
        // E (1,0) vai para (0,1), que é o seu reflexo por essa reta.
        assert!(igual(
            espelho.apply(Point2::new(1.0, 0.0)),
            Point2::new(0.0, 1.0)
        ));
    }

    #[test]
    fn a_reflexao_troca_o_sinal_da_direcao() {
        let espelho = Transform::mirror(Point2::ORIGIN, 0.0);

        assert!(tol().angles_equal(espelho.apply_angle(FRAC_PI_4), -FRAC_PI_4));
        assert!(tol().angles_equal(espelho.apply_angle(0.0), 0.0));
    }

    #[test]
    fn refletir_duas_vezes_pela_mesma_reta_volta_ao_lugar() {
        let espelho = Transform::mirror(Point2::new(1.0, 2.0), FRAC_PI_4 / 2.0);
        let ida_e_volta = espelho.then(espelho);
        let ponto = Point2::new(-4.0, 7.0);

        assert!(igual(ida_e_volta.apply(ponto), ponto));
        assert!(!ida_e_volta.is_reflected());
    }

    #[test]
    fn compor_e_o_mesmo_que_aplicar_em_sequencia() {
        let primeira = Transform::rotation_around(Point2::new(2.0, 0.0), FRAC_PI_2);
        let segunda = Transform::scaling_around(Point2::new(-1.0, 3.0), 1.5);
        let composta = primeira.then(segunda);

        for ponto in [
            Point2::ORIGIN,
            Point2::new(5.0, -2.0),
            Point2::new(-3.0, 8.0),
            Point2::new(7.0e6, 7.0e6),
        ] {
            assert!(
                igual(composta.apply(ponto), segunda.apply(primeira.apply(ponto))),
                "compor divergiu de aplicar em sequência em {ponto:?}"
            );
        }
    }

    #[test]
    fn compor_com_reflexao_tambem_confere() {
        let espelho = Transform::mirror(Point2::new(1.0, 1.0), FRAC_PI_4);
        let giro = Transform::rotation_around(Point2::ORIGIN, PI / 3.0);

        for (primeira, segunda) in [(espelho, giro), (giro, espelho), (espelho, espelho)] {
            let composta = primeira.then(segunda);

            for ponto in [Point2::new(4.0, -1.0), Point2::new(-2.0, 6.0)] {
                assert!(
                    igual(composta.apply(ponto), segunda.apply(primeira.apply(ponto))),
                    "compor divergiu em {ponto:?}"
                );
            }
        }
    }

    #[test]
    fn a_transformacao_preserva_a_forma_em_coordenada_utm() {
        let base = 7.0e6;
        let pivo = Point2::new(base, base);
        let giro = Transform::rotation_around(pivo, FRAC_PI_2);
        let a = Point2::new(base + 3.0, base);
        let b = Point2::new(base + 3.0, base + 4.0);

        // Semelhança de escala 1 preserva distância.
        assert!(tol().lengths_equal(giro.apply(a).distance_to(giro.apply(b)), a.distance_to(b)));
    }
}
