// Caminho relativo: kernel/neocad-geometry/src/tolerance.rs
//! \file kernel/neocad-geometry/src/tolerance.rs
//! \brief Tolerância geométrica do kernel, conforme o ADR 0006.
//! \author Iago Leal
//! \date 2026-09-01
//!
//! Fonte única da verdade para a pergunta que toda operação de K3 faz: **estas
//! duas coisas se tocam?** Nenhuma outra crate define constante de tolerância.

use core::f64::consts::TAU;

use crate::Point2;

/// Tolerância linear base, em unidades de desenho.
///
/// Menor que qualquer distância que um desenho pretenda distinguir e maior que
/// o ruído de algumas operações encadeadas em magnitude ordinária.
pub const BASE_LINEAR: f64 = 1e-9;

/// Tolerância angular, em radianos.
///
/// Não é derivada da linear: um ângulo só vira distância quando há um raio, e
/// dividir uma pela outra exigiria arbitrar esse raio (ADR 0006).
pub const BASE_ANGULAR: f64 = 1e-9;

/// Quantos passos de arredondamento a tolerância absorve longe da origem.
///
/// Multiplica o espaçamento entre `f64` representáveis na magnitude em jogo. Em
/// coordenada da ordem de `1e6` isso dá folga da ordem do décimo de micrômetro —
/// invisível para o desenho e suficiente para algumas centenas de operações
/// encadeadas.
const MAGNITUDE_FACTOR: f64 = 1024.0;

/// Tolerância com que se decide coincidência geométrica.
///
/// O valor efetivo cresce com a magnitude das coordenadas envolvidas, porque o
/// espaçamento entre `f64` representáveis cresce com ela: perto de `1.0` esse
/// espaçamento é `2,2e-16`, e perto de `7 000 000` já é `9,3e-10` — praticamente
/// a tolerância base inteira. Sem o termo adaptativo, "coincidente dentro da
/// tolerância" degeneraria em igualdade exata em desenho georreferenciado.
///
/// # Exemplo
///
/// ```
/// use neocad_geometry::{Point2, Tolerance};
///
/// let tolerancia = Tolerance::default();
///
/// // Perto da origem, `1e-8` é uma distância que o desenho distingue.
/// assert!(!tolerancia.points_equal(Point2::ORIGIN, Point2::new(1e-8, 0.0)));
///
/// // Na magnitude de uma coordenada UTM, a mesma distância está abaixo do que
/// // a aritmética consegue afirmar.
/// let longe = Point2::new(7.0e6, 7.0e6);
/// assert!(tolerancia.points_equal(longe, longe.translated(1e-8, 0.0)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance {
    linear: f64,
    angular: f64,
}

impl Default for Tolerance {
    fn default() -> Self {
        Self {
            linear: BASE_LINEAR,
            angular: BASE_ANGULAR,
        }
    }
}

impl Tolerance {
    /// Cria uma tolerância com valores próprios.
    ///
    /// Existe para o teste que precisa apertar ou afrouxar um caso específico.
    /// O kernel usa [`Tolerance::default`]; enquanto não houver tolerância por
    /// documento, é ela que vale (ADR 0006).
    #[must_use]
    pub const fn new(linear: f64, angular: f64) -> Self {
        Self { linear, angular }
    }

    /// Tolerância linear base, antes do ajuste por magnitude.
    #[must_use]
    pub const fn linear(self) -> f64 {
        self.linear
    }

    /// Tolerância angular, em radianos.
    #[must_use]
    pub const fn angular(self) -> f64 {
        self.angular
    }

    /// Tolerância linear efetiva na magnitude informada.
    ///
    /// É o maior entre a base e o piso adaptativo. Magnitude negativa ou `NaN`
    /// não encolhe a tolerância: o que vale é o módulo, e `NaN` cai na base.
    #[must_use]
    pub fn effective_linear(self, magnitude: f64) -> f64 {
        let adaptive = MAGNITUDE_FACTOR * magnitude.abs() * f64::EPSILON;

        if adaptive > self.linear {
            adaptive
        } else {
            self.linear
        }
    }

    /// Decide se dois comprimentos são o mesmo.
    ///
    /// A magnitude considerada é a do maior dos dois em módulo.
    #[must_use]
    pub fn lengths_equal(self, a: f64, b: f64) -> bool {
        (a - b).abs() <= self.effective_linear(a.abs().max(b.abs()))
    }

    /// Decide se um comprimento é nulo.
    #[must_use]
    pub fn is_zero_length(self, value: f64) -> bool {
        value.abs() <= self.effective_linear(value)
    }

    /// Decide se dois pontos são o mesmo ponto.
    ///
    /// A comparação é pela distância, e não coordenada a coordenada, para que a
    /// vizinhança de um ponto seja um disco e não um quadrado — um quadrado
    /// tornaria a resposta dependente da orientação dos eixos, e girar o desenho
    /// passaria a mudar o resultado.
    ///
    /// Ponto com coordenada `NaN` nunca é igual a nada, inclusive a si mesmo,
    /// que é a resposta segura: o cálculo que o produziu já estava errado.
    #[must_use]
    pub fn points_equal(self, a: Point2, b: Point2) -> bool {
        let magnitude = a.x.abs().max(a.y.abs()).max(b.x.abs()).max(b.y.abs());

        a.distance_to(b) <= self.effective_linear(magnitude)
    }

    /// Decide se duas direções são a mesma, em radianos.
    ///
    /// A volta completa é levada em conta: `0` e `2π` são a mesma direção. Não
    /// há ajuste por magnitude, porque ângulo não cresce — ele dá a volta.
    #[must_use]
    pub fn angles_equal(self, a: f64, b: f64) -> bool {
        let difference = (a - b) % TAU;
        let shortest = if difference.abs() > TAU / 2.0 {
            TAU - difference.abs()
        } else {
            difference.abs()
        };

        shortest <= self.angular
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perto_da_origem_vale_a_tolerancia_base() {
        let tolerancia = Tolerance::default();

        assert_eq!(tolerancia.effective_linear(0.0), BASE_LINEAR);
        assert_eq!(tolerancia.effective_linear(1.0), BASE_LINEAR);
    }

    #[test]
    fn em_coordenada_utm_o_piso_adaptativo_governa() {
        let tolerancia = Tolerance::default();
        let efetiva = tolerancia.effective_linear(7.0e6);

        assert!(
            efetiva > BASE_LINEAR,
            "em magnitude 7e6 o piso deveria superar a base, obtido {efetiva}"
        );
        assert_eq!(efetiva, MAGNITUDE_FACTOR * 7.0e6 * f64::EPSILON);
    }

    #[test]
    fn o_sinal_da_magnitude_nao_encolhe_a_tolerancia() {
        let tolerancia = Tolerance::default();

        assert_eq!(
            tolerancia.effective_linear(-7.0e6),
            tolerancia.effective_linear(7.0e6)
        );
    }

    #[test]
    fn magnitude_nao_numerica_cai_na_base() {
        assert_eq!(Tolerance::default().effective_linear(f64::NAN), BASE_LINEAR);
    }

    #[test]
    fn a_mesma_separacao_decide_diferente_perto_e_longe_da_origem() {
        let tolerancia = Tolerance::default();
        let separacao = 1e-8;

        // Perto da origem, `1e-8` é dez vezes a tolerância: são dois pontos.
        assert!(!tolerancia.points_equal(Point2::ORIGIN, Point2::new(separacao, 0.0)));

        // Em magnitude UTM, `1e-8` está abaixo do que a aritmética afirma.
        let longe = Point2::new(7.0e6, 7.0e6);
        assert!(tolerancia.points_equal(longe, longe.translated(separacao, 0.0)));
    }

    #[test]
    fn pontos_iguais_usam_disco_e_nao_quadrado() {
        let tolerancia = Tolerance::default();
        // Deslocamento diagonal de `0,8·tol` em cada eixo: dentro do quadrado,
        // fora do disco, porque a distância resultante é `1,13·tol`.
        let passo = 0.8 * BASE_LINEAR;

        assert!(!tolerancia.points_equal(Point2::ORIGIN, Point2::new(passo, passo)));
    }

    #[test]
    fn ponto_com_nan_nao_e_igual_a_nada() {
        let tolerancia = Tolerance::default();
        let invalido = Point2::new(f64::NAN, 0.0);

        assert!(!tolerancia.points_equal(invalido, invalido));
        assert!(!tolerancia.points_equal(invalido, Point2::ORIGIN));
    }

    #[test]
    fn comprimentos_comparam_pela_maior_magnitude() {
        let tolerancia = Tolerance::default();

        assert!(!tolerancia.lengths_equal(0.0, 1e-8));
        assert!(tolerancia.lengths_equal(7.0e6, 7.0e6 + 1e-8));
    }

    #[test]
    fn comprimento_nulo_e_o_que_esta_abaixo_da_tolerancia() {
        let tolerancia = Tolerance::default();

        assert!(tolerancia.is_zero_length(1e-10));
        assert!(!tolerancia.is_zero_length(1e-8));
    }

    #[test]
    fn a_volta_completa_e_a_mesma_direcao() {
        let tolerancia = Tolerance::default();

        assert!(tolerancia.angles_equal(0.0, TAU));
        assert!(tolerancia.angles_equal(0.0, -TAU));
        assert!(tolerancia.angles_equal(TAU / 4.0, TAU / 4.0 + TAU));
    }

    #[test]
    fn direcoes_opostas_nao_sao_a_mesma() {
        let tolerancia = Tolerance::default();

        assert!(!tolerancia.angles_equal(0.0, TAU / 2.0));
        assert!(!tolerancia.angles_equal(0.0, 1e-8));
    }
}
