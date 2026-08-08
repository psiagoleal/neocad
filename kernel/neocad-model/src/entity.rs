// Caminho relativo: kernel/neocad-model/src/entity.rs
//! \file kernel/neocad-model/src/entity.rs
//! \brief Entidades de desenho 2D e suas caixas envolventes.
//! \author Iago Leal
//! \date 2026-08-07

use core::f64::consts::{FRAC_PI_2, TAU};

use neocad_geometry::{point_on_circle, Aabb, Point2};

use crate::layer::{Color, LayerId};

/// Razão entre a largura de avanço de um caractere e a altura do texto.
///
/// As fontes CAD tradicionais ficam entre 0,6 e 0,7. O valor é deliberadamente
/// mais alto para **superestimar** a caixa: uma caixa maior que a real faz o
/// ajuste de vista sobrar, enquanto uma menor cortaria o texto da tela.
const TEXT_ADVANCE_RATIO: f64 = 0.8;

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
/// trechos em arco dentro de uma polilinha. Entra em K3, junto com as operações
/// que dependem dele.
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    /// Vértices, em ordem de percurso.
    pub vertices: Vec<Point2>,
    /// Se verdadeiro, há um segmento ligando o último vértice ao primeiro.
    pub closed: bool,
}

/// Texto de uma linha.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    /// Ponto de inserção, na linha de base do primeiro caractere.
    pub position: Point2,
    /// Conteúdo.
    pub content: String,
    /// Altura dos caracteres, em unidades do desenho.
    pub height: f64,
    /// Rotação, em radianos, no sentido anti-horário.
    pub rotation: f64,
}

/// Forma geométrica de uma entidade.
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    /// Segmento de reta.
    Line(Line),
    /// Circunferência.
    Circle(Circle),
    /// Arco de circunferência.
    Arc(Arc),
    /// Polilinha.
    Polyline(Polyline),
    /// Texto.
    Text(Text),
}

/// Cor de uma entidade.
///
/// Reproduz o modelo dos formatos CAD, em que a cor da entidade normalmente é
/// herdada e só ocasionalmente explícita.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityColor {
    /// Herda a cor da camada. É o padrão.
    #[default]
    ByLayer,
    /// Herda a cor da referência de bloco que contém a entidade.
    ByBlock,
    /// Cor própria, ignorando camada e bloco.
    Explicit(Color),
}

/// Entidade de desenho: uma forma geométrica com seus atributos de exibição.
///
/// # Exemplo
///
/// ```
/// use neocad_model::{Entity, Geometry, LayerTable, Line};
/// use neocad_geometry::Point2;
///
/// let layers = LayerTable::new();
/// let entidade = Entity::new(
///     layers.default_layer(),
///     Geometry::Line(Line {
///         start: Point2::ORIGIN,
///         end: Point2::new(10.0, 5.0),
///     }),
/// );
///
/// assert_eq!(entidade.bounding_box().max(), Point2::new(10.0, 5.0));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    /// Camada à qual a entidade pertence.
    pub layer: LayerId,
    /// Cor.
    pub color: EntityColor,
    /// Forma geométrica.
    pub geometry: Geometry,
}

impl Entity {
    /// Cria uma entidade na camada informada, com cor herdada da camada.
    #[must_use]
    pub const fn new(layer: LayerId, geometry: Geometry) -> Self {
        Self {
            layer,
            color: EntityColor::ByLayer,
            geometry,
        }
    }

    /// Caixa envolvente da entidade.
    ///
    /// Uma polilinha sem vértices produz caixa degenerada na origem, por não
    /// haver posição melhor a informar.
    #[must_use]
    pub fn bounding_box(&self) -> Aabb {
        self.geometry.bounding_box()
    }
}

impl Geometry {
    /// Caixa envolvente da forma.
    #[must_use]
    pub fn bounding_box(&self) -> Aabb {
        match self {
            Self::Line(line) => Aabb::new(line.start, line.end),
            Self::Circle(circle) => circle_bounding_box(circle.center, circle.radius),
            Self::Arc(arc) => arc_bounding_box(arc),
            Self::Polyline(polyline) => Aabb::from_points(polyline.vertices.iter().copied())
                .unwrap_or_else(|| Aabb::from_point(Point2::ORIGIN)),
            Self::Text(text) => text_bounding_box(text),
        }
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

    let sweep = normalize_angle(arc.end_angle - arc.start_angle);
    let mut bounds = Aabb::new(start, end);

    for quarter in 0..4 {
        let angle = FRAC_PI_2 * f64::from(quarter);

        if normalize_angle(angle - arc.start_angle) <= sweep {
            bounds = bounds.expanded_to_include(point_on_circle(arc.center, radius, angle));
        }
    }

    bounds
}

/// Caixa envolvente aproximada de um texto.
///
/// Uma caixa fiel exige métricas da fonte, que o kernel ainda não tem — elas
/// chegam com a renderização própria de K5. Até lá, a largura é estimada por
/// contagem de caracteres, deliberadamente por cima, e a caixa é girada junto
/// com o texto.
fn text_bounding_box(text: &Text) -> Aabb {
    let height = text.height.abs();
    let count = text.content.chars().count();
    // `as f64` é exato para as contagens envolvidas; um texto de uma linha nunca
    // se aproxima do limite de precisão de f64.
    let width = count as f64 * height * TEXT_ADVANCE_RATIO;

    let corners = [
        text.position,
        text.position.translated(width, 0.0),
        text.position.translated(width, height),
        text.position.translated(0.0, height),
    ];

    let rotated = corners
        .into_iter()
        .map(|corner| corner.rotated_around(text.position, text.rotation));

    Aabb::from_points(rotated).unwrap_or_else(|| Aabb::from_point(text.position))
}

/// Normaliza um ângulo para o intervalo `[0, 2π)`.
fn normalize_angle(angle: f64) -> f64 {
    let remainder = angle % TAU;

    if remainder < 0.0 {
        remainder + TAU
    } else {
        remainder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::LayerTable;
    use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    const TOLERANCE: f64 = 1e-9;

    fn layer() -> LayerId {
        LayerTable::new().default_layer()
    }

    fn assert_bounds(actual: Aabb, min: Point2, max: Point2) {
        assert!(
            (actual.min().x - min.x).abs() < TOLERANCE
                && (actual.min().y - min.y).abs() < TOLERANCE
                && (actual.max().x - max.x).abs() < TOLERANCE
                && (actual.max().y - max.y).abs() < TOLERANCE,
            "esperado min {min:?} max {max:?}, obtido min {:?} max {:?}",
            actual.min(),
            actual.max()
        );
    }

    fn arc(start_angle: f64, end_angle: f64) -> Arc {
        Arc {
            center: Point2::ORIGIN,
            radius: 1.0,
            start_angle,
            end_angle,
        }
    }

    #[test]
    fn entidade_nova_herda_a_cor_da_camada() {
        let entidade = Entity::new(
            layer(),
            Geometry::Line(Line {
                start: Point2::ORIGIN,
                end: Point2::new(1.0, 1.0),
            }),
        );

        assert_eq!(entidade.color, EntityColor::ByLayer);
        assert_eq!(EntityColor::default(), EntityColor::ByLayer);
    }

    #[test]
    fn cor_explicita_pode_ser_atribuida() {
        let mut entidade = Entity::new(
            layer(),
            Geometry::Circle(Circle {
                center: Point2::ORIGIN,
                radius: 1.0,
            }),
        );
        entidade.color = EntityColor::Explicit(Color::Index(3));

        assert_eq!(entidade.color, EntityColor::Explicit(Color::Index(3)));
    }

    #[test]
    fn caixa_da_linha_normaliza_os_extremos() {
        let geometria = Geometry::Line(Line {
            start: Point2::new(4.0, 1.0),
            end: Point2::new(-2.0, 6.0),
        });

        assert_bounds(
            geometria.bounding_box(),
            Point2::new(-2.0, 1.0),
            Point2::new(4.0, 6.0),
        );
    }

    #[test]
    fn caixa_da_linha_degenerada_e_um_ponto() {
        let ponto = Point2::new(2.0, 3.0);
        let geometria = Geometry::Line(Line {
            start: ponto,
            end: ponto,
        });

        assert!(geometria.bounding_box().is_degenerate());
    }

    #[test]
    fn caixa_do_circulo_envolve_todo_o_raio() {
        let geometria = Geometry::Circle(Circle {
            center: Point2::new(2.0, -1.0),
            radius: 3.0,
        });

        assert_bounds(
            geometria.bounding_box(),
            Point2::new(-1.0, -4.0),
            Point2::new(5.0, 2.0),
        );
    }

    #[test]
    fn caixa_do_circulo_tolera_raio_negativo() {
        let geometria = Geometry::Circle(Circle {
            center: Point2::ORIGIN,
            radius: -2.0,
        });

        assert_bounds(
            geometria.bounding_box(),
            Point2::new(-2.0, -2.0),
            Point2::new(2.0, 2.0),
        );
    }

    #[test]
    fn caixa_do_arco_no_primeiro_quadrante() {
        // De 0° a 90°: os dois extremos já são os pontos extremos.
        assert_bounds(
            Geometry::Arc(arc(0.0, FRAC_PI_2)).bounding_box(),
            Point2::ORIGIN,
            Point2::new(1.0, 1.0),
        );
    }

    #[test]
    fn caixa_do_arco_que_cruza_o_eixo_x_positivo() {
        // De 315° a 45°. Os extremos ficam em x ≈ 0,707, mas o arco atinge x = 1
        // ao cruzar 0°. Sem incluir o cardeal, a caixa sairia estreita demais.
        let caixa = Geometry::Arc(arc(-FRAC_PI_4, FRAC_PI_4)).bounding_box();

        assert_bounds(
            caixa,
            Point2::new(FRAC_PI_4.cos(), -FRAC_PI_4.sin()),
            Point2::new(1.0, FRAC_PI_4.sin()),
        );
        assert!(
            (caixa.max().x - 1.0).abs() < TOLERANCE,
            "o cruzamento de 0° tem de entrar na caixa"
        );
    }

    #[test]
    fn caixa_do_arco_que_cruza_o_eixo_y_positivo() {
        // De 45° a 135°: atinge y = 1 ao cruzar 90°.
        let caixa = Geometry::Arc(arc(FRAC_PI_4, 3.0 * FRAC_PI_4)).bounding_box();

        assert!((caixa.max().y - 1.0).abs() < TOLERANCE);
        assert_bounds(
            caixa,
            Point2::new(-FRAC_PI_4.cos(), FRAC_PI_4.sin()),
            Point2::new(FRAC_PI_4.cos(), 1.0),
        );
    }

    #[test]
    fn caixa_do_arco_que_cruza_o_eixo_x_negativo() {
        // De 135° a 225°: atinge x = -1 ao cruzar 180°.
        let caixa = Geometry::Arc(arc(3.0 * FRAC_PI_4, 5.0 * FRAC_PI_4)).bounding_box();

        assert!((caixa.min().x + 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn caixa_do_arco_que_cruza_o_eixo_y_negativo() {
        // De 225° a 315°: atinge y = -1 ao cruzar 270°.
        let caixa = Geometry::Arc(arc(5.0 * FRAC_PI_4, 7.0 * FRAC_PI_4)).bounding_box();

        assert!((caixa.min().y + 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn caixa_do_arco_quase_completo_iguala_a_do_circulo() {
        // Varredura de quase 360°: todos os quatro cardeais entram.
        let caixa = Geometry::Arc(arc(0.1, 0.05)).bounding_box();

        assert_bounds(caixa, Point2::new(-1.0, -1.0), Point2::new(1.0, 1.0));
    }

    #[test]
    fn caixa_do_arco_aceita_angulos_fora_do_intervalo_canonico() {
        // Mesmo arco de 315° a 45°, escrito com ângulos além de 2π.
        let deslocado = Arc {
            start_angle: -FRAC_PI_4 + TAU * 3.0,
            end_angle: FRAC_PI_4 + TAU * 3.0,
            ..arc(0.0, 0.0)
        };

        assert_bounds(
            Geometry::Arc(deslocado).bounding_box(),
            Point2::new(FRAC_PI_4.cos(), -FRAC_PI_4.sin()),
            Point2::new(1.0, FRAC_PI_4.sin()),
        );
    }

    #[test]
    fn arco_degenerado_reduz_se_a_um_ponto() {
        let caixa = Geometry::Arc(arc(FRAC_PI_2, FRAC_PI_2)).bounding_box();

        assert!(caixa.is_degenerate());
        assert_bounds(caixa, Point2::new(0.0, 1.0), Point2::new(0.0, 1.0));
    }

    #[test]
    fn arco_de_meia_volta_cobre_o_semiplano_superior() {
        let caixa = Geometry::Arc(arc(0.0, PI)).bounding_box();

        assert_bounds(caixa, Point2::new(-1.0, 0.0), Point2::new(1.0, 1.0));
    }

    #[test]
    fn caixa_da_polilinha_envolve_os_vertices() {
        let geometria = Geometry::Polyline(Polyline {
            vertices: vec![
                Point2::new(0.0, 0.0),
                Point2::new(5.0, 2.0),
                Point2::new(-1.0, 8.0),
            ],
            closed: false,
        });

        assert_bounds(
            geometria.bounding_box(),
            Point2::new(-1.0, 0.0),
            Point2::new(5.0, 8.0),
        );
    }

    #[test]
    fn fechar_a_polilinha_nao_muda_a_caixa() {
        let vertices = vec![Point2::ORIGIN, Point2::new(4.0, 0.0), Point2::new(4.0, 3.0)];
        let aberta = Geometry::Polyline(Polyline {
            vertices: vertices.clone(),
            closed: false,
        });
        let fechada = Geometry::Polyline(Polyline {
            vertices,
            closed: true,
        });

        assert_eq!(aberta.bounding_box(), fechada.bounding_box());
    }

    #[test]
    fn polilinha_sem_vertices_produz_caixa_degenerada_na_origem() {
        let geometria = Geometry::Polyline(Polyline {
            vertices: Vec::new(),
            closed: false,
        });

        assert!(geometria.bounding_box().is_degenerate());
        assert_eq!(geometria.bounding_box().min(), Point2::ORIGIN);
    }

    #[test]
    fn caixa_do_texto_cresce_com_conteudo_e_altura() {
        let geometria = Geometry::Text(Text {
            position: Point2::ORIGIN,
            content: String::from("ABC"),
            height: 2.0,
            rotation: 0.0,
        });

        let caixa = geometria.bounding_box();

        assert_bounds(
            caixa,
            Point2::ORIGIN,
            Point2::new(3.0 * 2.0 * TEXT_ADVANCE_RATIO, 2.0),
        );
        assert_eq!(caixa.height(), 2.0);
    }

    #[test]
    fn caixa_do_texto_girado_envolve_o_retangulo_girado() {
        let geometria = Geometry::Text(Text {
            position: Point2::ORIGIN,
            content: String::from("AB"),
            height: 1.0,
            rotation: FRAC_PI_2,
        });

        let caixa = geometria.bounding_box();
        let largura = 2.0 * TEXT_ADVANCE_RATIO;

        // Girado 90°, a largura do texto passa a ocupar o eixo Y.
        assert_bounds(caixa, Point2::new(-1.0, 0.0), Point2::new(0.0, largura));
    }

    #[test]
    fn texto_vazio_produz_caixa_sem_largura() {
        let geometria = Geometry::Text(Text {
            position: Point2::new(3.0, 3.0),
            content: String::new(),
            height: 2.0,
            rotation: 0.0,
        });

        let caixa = geometria.bounding_box();

        assert_eq!(caixa.width(), 0.0);
        assert_eq!(caixa.height(), 2.0);
    }

    #[test]
    fn caixa_do_texto_conta_caracteres_e_nao_bytes() {
        let ascii = Geometry::Text(Text {
            position: Point2::ORIGIN,
            content: String::from("aaa"),
            height: 1.0,
            rotation: 0.0,
        });
        let acentuado = Geometry::Text(Text {
            position: Point2::ORIGIN,
            content: String::from("ãçõ"),
            height: 1.0,
            rotation: 0.0,
        });

        assert_eq!(
            ascii.bounding_box().width(),
            acentuado.bounding_box().width(),
            "três caracteres devem medir o mesmo, ainda que ocupem mais bytes"
        );
    }

    #[test]
    fn entidade_delega_a_caixa_a_sua_geometria() {
        let entidade = Entity::new(
            layer(),
            Geometry::Circle(Circle {
                center: Point2::ORIGIN,
                radius: 1.0,
            }),
        );

        assert_eq!(
            entidade.bounding_box(),
            entidade.geometry.bounding_box(),
            "a caixa da entidade é a da sua geometria"
        );
    }
}
