// Caminho relativo: kernel/neocad-geometry/src/predicate.rs
//! \file kernel/neocad-geometry/src/predicate.rs
//! \brief Predicado de orientação com sinal exato, conforme o ADR 0006.
//! \author Iago Leal
//! \date 2026-09-01
//!
//! Tolerância responde "estas duas coisas se tocam?". Este módulo responde a
//! outra pergunta, que tolerância nenhuma resolve: **de que lado?**
//!
//! Comparar um determinante contra epsilon produz respostas mutuamente
//! contraditórias — três pontos podem sair "A à esquerda de BC", "B à esquerda
//! de CA" e "C à esquerda de AB" ao mesmo tempo, o que é impossível. Algoritmos
//! que confiam nessa consistência entram em laço ou emitem geometria inválida.
//!
//! A saída aqui é sempre o sinal verdadeiro do determinante. O caminho rápido é
//! o cálculo em `f64` com limite de erro conhecido; quando o resultado cai
//! dentro desse limite, o sinal é recalculado em aritmética exata.

use crate::Point2;

/// Metade do espaçamento entre `f64` representáveis perto de `1.0`.
///
/// É o `ε` da análise de erro clássica: o erro relativo máximo de uma operação
/// aritmética arredondada. `f64::EPSILON` é o espaçamento inteiro, daí a metade.
const HALF_ULP: f64 = f64::EPSILON / 2.0;

/// Limite de erro do determinante calculado em `f64`.
///
/// Enquanto o módulo do determinante superar este limite vezes a soma dos
/// módulos das duas parcelas, o sinal calculado em ponto flutuante é o
/// verdadeiro e não há por que recorrer ao caminho exato.
const ORIENTATION_ERROR_BOUND: f64 = (3.0 + 16.0 * HALF_ULP) * HALF_ULP;

/// Posição de um ponto em relação a uma reta orientada.
///
/// A convenção é a do sentido anti-horário com o eixo Y para cima, a mesma dos
/// ângulos do kernel e do DXF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// O terceiro ponto está à esquerda da reta orientada pelos dois primeiros.
    CounterClockwise,
    /// O terceiro ponto está à direita.
    Clockwise,
    /// Os três pontos são colineares — e **exatamente** colineares, não
    /// "colineares dentro de uma tolerância".
    Collinear,
}

/// Diz de que lado da reta orientada `a → b` está o ponto `c`.
///
/// O resultado é o sinal verdadeiro do determinante, sem tolerância envolvida.
/// Um ponto a um único passo representável da reta é declarado fora dela, ainda
/// que essa distância esteja muito abaixo da tolerância linear do kernel — e é
/// exatamente esse o ponto: coincidência e lado são perguntas diferentes.
///
/// Coordenada `NaN` produz [`Orientation::Collinear`], por não haver resposta
/// melhor a dar sobre um ponto que não existe.
///
/// # Exemplo
///
/// ```
/// use neocad_geometry::{orientation, Orientation, Point2};
///
/// let a = Point2::ORIGIN;
/// let b = Point2::new(1.0, 0.0);
///
/// assert_eq!(
///     orientation(a, b, Point2::new(0.5, 1.0)),
///     Orientation::CounterClockwise
/// );
/// assert_eq!(
///     orientation(a, b, Point2::new(0.5, 0.0)),
///     Orientation::Collinear
/// );
/// ```
#[must_use]
pub fn orientation(a: Point2, b: Point2, c: Point2) -> Orientation {
    let left = (a.x - c.x) * (b.y - c.y);
    let right = (a.y - c.y) * (b.x - c.x);
    let determinant = left - right;

    // Quando as duas parcelas têm sinais opostos ou uma delas é nula, não há
    // cancelamento catastrófico e o sinal da subtração já é o verdadeiro.
    let sum = if left > 0.0 {
        if right <= 0.0 {
            return sign_of(determinant);
        }
        left + right
    } else if left < 0.0 {
        if right >= 0.0 {
            return sign_of(determinant);
        }
        -left - right
    } else {
        return sign_of(determinant);
    };

    if determinant.abs() >= ORIENTATION_ERROR_BOUND * sum {
        return sign_of(determinant);
    }

    exact_orientation(a, b, c)
}

/// Converte o sinal de um determinante em orientação.
fn sign_of(determinant: f64) -> Orientation {
    if determinant > 0.0 {
        Orientation::CounterClockwise
    } else if determinant < 0.0 {
        Orientation::Clockwise
    } else {
        Orientation::Collinear
    }
}

/// Recalcula a orientação em aritmética exata.
///
/// A forma com diferenças não serve aqui, porque as próprias diferenças
/// arredondam. Expandindo o determinante e cancelando os termos em `c`:
///
/// ```text
/// (aₓ−cₓ)(b_y−c_y) − (a_y−c_y)(bₓ−cₓ)
///   = (aₓb_y − a_ybₓ) + (a_ycₓ − aₓc_y) + (bₓc_y − b_ycₓ)
/// ```
///
/// São seis produtos das coordenadas originais. Cada um é decomposto na soma
/// exata de dois `f64`, e os doze valores resultantes são somados sem perda.
pub(crate) fn exact_orientation(a: Point2, b: Point2, c: Point2) -> Orientation {
    let terms = [
        two_product(a.x, b.y),
        negated(two_product(a.y, b.x)),
        two_product(a.y, c.x),
        negated(two_product(a.x, c.y)),
        two_product(b.x, c.y),
        negated(two_product(b.y, c.x)),
    ];

    let mut expansion = Expansion::new();

    for (product, error) in terms {
        if !product.is_finite() {
            return Orientation::Collinear;
        }

        expansion.add(error);
        expansion.add(product);
    }

    sign_of(expansion.leading())
}

/// Troca o sinal das duas metades de um produto exato.
///
/// Negar um `f64` é exato, então a soma das duas metades continua valendo o
/// produto trocado de sinal.
const fn negated((product, error): (f64, f64)) -> (f64, f64) {
    (-product, -error)
}

/// Produto de dois `f64` como soma exata de dois `f64`.
///
/// O primeiro valor é o produto arredondado; o segundo é exatamente o que o
/// arredondamento descartou. A multiplicação-soma fundida calcula o resto com um
/// único arredondamento, que é o que torna a decomposição exata.
fn two_product(a: f64, b: f64) -> (f64, f64) {
    let product = a * b;

    (product, a.mul_add(b, -product))
}

/// Soma de dois `f64` como soma exata de dois `f64`.
///
/// Não exige que as parcelas estejam ordenadas por magnitude, ao contrário da
/// variante rápida — e essa generalidade é necessária, já que os termos do
/// determinante chegam em qualquer ordem.
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let sum = a + b;
    let b_virtual = sum - a;
    let a_virtual = sum - b_virtual;

    (sum, (a - a_virtual) + (b - b_virtual))
}

/// Capacidade da expansão.
///
/// São doze valores somados, e cada soma acrescenta no máximo uma componente,
/// de modo que doze bastam. A folga existe para que um engano de contagem vire
/// pânico em teste e não corrupção silenciosa.
const EXPANSION_CAPACITY: usize = 16;

/// Soma exata representada como conjunto de `f64` que não se sobrepõem.
///
/// O valor representado é a soma das componentes, guardadas em ordem crescente
/// de módulo. Como não se sobrepõem, o sinal da soma é o sinal da componente de
/// maior módulo — que é tudo o que um predicado precisa.
struct Expansion {
    components: [f64; EXPANSION_CAPACITY],
    len: usize,
}

impl Expansion {
    /// Expansão vazia, que representa zero.
    const fn new() -> Self {
        Self {
            components: [0.0; EXPANSION_CAPACITY],
            len: 0,
        }
    }

    /// Acrescenta uma parcela, mantendo a soma exata.
    ///
    /// As componentes nulas são descartadas, o que mantém a expansão curta sem
    /// alterar o valor representado.
    fn add(&mut self, value: f64) {
        let mut carry = value;
        let mut written = 0;

        for index in 0..self.len {
            let (sum, error) = two_sum(carry, self.components[index]);
            carry = sum;

            if error != 0.0 {
                // `written` nunca ultrapassa `index`, então a leitura acima já
                // ocorreu quando esta escrita acontece.
                self.components[written] = error;
                written += 1;
            }
        }

        if carry != 0.0 {
            self.components[written] = carry;
            written += 1;
        }

        self.len = written;
    }

    /// Componente de maior módulo, cujo sinal é o sinal da soma.
    fn leading(&self) -> f64 {
        if self.len == 0 {
            0.0
        } else {
            self.components[self.len - 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todas as seis permutações de uma trinca, com o sinal que cada uma deve
    /// ter em relação à ordem original.
    fn permutations(a: Point2, b: Point2, c: Point2) -> [(Point2, Point2, Point2, bool); 6] {
        [
            (a, b, c, true),
            (b, c, a, true),
            (c, a, b, true),
            (a, c, b, false),
            (c, b, a, false),
            (b, a, c, false),
        ]
    }

    fn reversed(orientation: Orientation) -> Orientation {
        match orientation {
            Orientation::CounterClockwise => Orientation::Clockwise,
            Orientation::Clockwise => Orientation::CounterClockwise,
            Orientation::Collinear => Orientation::Collinear,
        }
    }

    fn assert_consistent(a: Point2, b: Point2, c: Point2) {
        let reference = orientation(a, b, c);

        for (first, second, third, same_sign) in permutations(a, b, c) {
            let expected = if same_sign {
                reference
            } else {
                reversed(reference)
            };

            assert_eq!(
                orientation(first, second, third),
                expected,
                "permutação inconsistente para {a:?} {b:?} {c:?}"
            );
        }
    }

    #[test]
    fn lados_seguem_a_convencao_anti_horaria() {
        let a = Point2::ORIGIN;
        let b = Point2::new(1.0, 0.0);

        assert_eq!(
            orientation(a, b, Point2::new(0.5, 1.0)),
            Orientation::CounterClockwise
        );
        assert_eq!(
            orientation(a, b, Point2::new(0.5, -1.0)),
            Orientation::Clockwise
        );
        assert_eq!(
            orientation(a, b, Point2::new(0.5, 0.0)),
            Orientation::Collinear
        );
    }

    #[test]
    fn as_seis_permutacoes_concordam_perto_da_origem() {
        assert_consistent(
            Point2::ORIGIN,
            Point2::new(1.0, 1.0),
            Point2::new(0.5, 0.5 + 1e-17),
        );
        assert_consistent(
            Point2::ORIGIN,
            Point2::new(1.0, 1.0),
            Point2::new(0.5, 0.5 - 1e-17),
        );
        assert_consistent(Point2::ORIGIN, Point2::new(1.0, 1.0), Point2::new(0.5, 0.5));
    }

    #[test]
    fn as_seis_permutacoes_concordam_em_coordenada_utm() {
        let base = 7.0e6;
        let a = Point2::new(base, base);
        let b = Point2::new(base + 8.0, base + 8.0);
        let meio = base + 4.0;

        for passo in [-2, -1, 0, 1, 2] {
            assert_consistent(a, b, Point2::new(meio, nudge(meio, passo)));
        }
    }

    /// Desloca um valor positivo em `steps` posições representáveis.
    ///
    /// Para `f64` positivo os bits crescem junto com o valor, então somar ao
    /// padrão de bits caminha exatamente de um representável ao seguinte.
    fn nudge(value: f64, steps: i64) -> f64 {
        debug_assert!(value > 0.0, "o passo por bits só vale para positivos");

        #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
        let bits = (value.to_bits() as i64 + steps) as u64;

        f64::from_bits(bits)
    }

    #[test]
    fn um_passo_representavel_ja_define_o_lado_em_coordenada_utm() {
        let base = 7.0e6;
        let a = Point2::new(base, base);
        let b = Point2::new(base + 8.0, base + 8.0);
        let meio = base + 4.0;

        // O deslocamento é de um único passo representável — cerca de `9,3e-10`
        // nesta magnitude, portanto **abaixo** da tolerância linear do kernel.
        // Um predicado por epsilon chamaria os três de colineares; o sinal não
        // se decide por tolerância.
        let acima = Point2::new(meio, nudge(meio, 1));
        let abaixo = Point2::new(meio, nudge(meio, -1));

        assert!(nudge(meio, 1) - meio < crate::BASE_LINEAR);
        assert_eq!(orientation(a, b, acima), Orientation::CounterClockwise);
        assert_eq!(orientation(a, b, abaixo), Orientation::Clockwise);
        assert_eq!(
            orientation(a, b, Point2::new(meio, meio)),
            Orientation::Collinear
        );
    }

    #[test]
    fn colinearidade_e_exata_e_nao_aproximada() {
        // Base com mantissa cheia, para que os produtos do determinante
        // arredondem de verdade. Somar um inteiro pequeno a uma coordenada desta
        // magnitude é exato — o passo representável aqui é da ordem de `1e-10` —,
        // de modo que os três pontos ficam exatamente sobre a direção `(1, 7)`.
        let a = Point2::new(7_000_000.123_456_789, 7_000_000.987_654_321);
        let b = Point2::new(a.x + 1.0, a.y + 7.0);
        let c = Point2::new(a.x + 3.0, a.y + 21.0);

        assert_eq!(orientation(a, b, c), Orientation::Collinear);
        assert_eq!(exact_orientation(a, b, c), Orientation::Collinear);
    }

    #[test]
    fn as_permutacoes_concordam_em_trincas_quase_degeneradas() {
        // Gerador determinístico: o teste precisa ser reproduzível, e o que se
        // quer dele é volume de trincas quase colineares com mantissa cheia —
        // que é onde o filtro de erro não basta e o caminho exato entra.
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            #[allow(clippy::cast_precision_loss)]
            let unit = (state >> 11) as f64 / 9_007_199_254_740_992.0;

            unit
        };

        for magnitude in [1.0_f64, 7.0e6] {
            for passo in 0..2_000_i64 {
                let a = Point2::new(magnitude * next(), magnitude * next());
                let (dx, dy) = (1.0 + next(), 1.0 + next());
                let b = Point2::new(a.x + dx, a.y + dy);

                // Ponto sobre a reta, desviado de poucos passos representáveis.
                let t = next();
                let sobre = Point2::new(a.x + dx * t, a.y + dy * t);
                let c = Point2::new(sobre.x, nudge(sobre.y, passo % 5 - 2));

                assert_consistent(a, b, c);
            }
        }
    }

    #[test]
    fn o_caminho_exato_concorda_com_o_rapido_quando_o_filtro_basta() {
        let casos = [
            (Point2::ORIGIN, Point2::new(1.0, 0.0), Point2::new(0.5, 1.0)),
            (
                Point2::new(7.0e6, 7.0e6),
                Point2::new(7.0e6 + 8.0, 7.0e6 + 8.0),
                Point2::new(7.0e6 + 4.0, 7.0e6 + 4.5),
            ),
            (
                Point2::new(-3.5, 2.25),
                Point2::new(11.0, -8.75),
                Point2::new(0.0, 0.0),
            ),
        ];

        for (a, b, c) in casos {
            assert_eq!(
                exact_orientation(a, b, c),
                orientation(a, b, c),
                "os dois caminhos divergiram para {a:?} {b:?} {c:?}"
            );
        }
    }

    #[test]
    fn coordenada_nao_numerica_nao_decide_lado() {
        let invalido = Point2::new(f64::NAN, 0.0);

        assert_eq!(
            orientation(Point2::ORIGIN, Point2::new(1.0, 0.0), invalido),
            Orientation::Collinear
        );
    }

    #[test]
    fn a_decomposicao_do_produto_e_exata() {
        let (product, error) = two_product(0.1, 0.3);

        // Refazendo a soma em precisão maior não há nada a acrescentar: o par
        // representa o produto verdadeiro.
        assert_eq!(product + error, product);
        assert!(error != 0.0, "0,1 × 0,3 arredonda, logo o resto não é nulo");
    }

    #[test]
    fn a_decomposicao_da_soma_e_exata() {
        let (sum, error) = two_sum(1.0, 1e-20);

        assert_eq!(sum, 1.0);
        assert_eq!(error, 1e-20);
    }
}
