// Caminho relativo: kernel/neocad-model/src/entity.rs
//! \file kernel/neocad-model/src/entity.rs
//! \brief Entidades de desenho 2D e suas caixas envolventes.
//! \author Iago Leal
//! \date 2026-08-07

use neocad_geometry::{
    Aabb, Arc, Circle, Curve, Line, Point2, Polyline, PolylineVertex, Transform,
};

use crate::layer::{Color, LayerId};
use crate::viewport::Viewport;

/// Razão entre a largura de avanço de um caractere e a altura do texto.
///
/// As fontes CAD tradicionais ficam entre 0,6 e 0,7. O valor é deliberadamente
/// mais alto para **superestimar** a caixa: uma caixa maior que a real faz o
/// ajuste de vista sobrar, enquanto uma menor cortaria o texto da tela.
const TEXT_ADVANCE_RATIO: f64 = 0.8;

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
    /// Janela de espaço-papel que mostra uma vista do espaço-modelo.
    ///
    /// Diferente das demais, não é uma forma que se desenha: é uma moldura que
    /// mostra outra coisa. Está aqui porque o formato a trata como entidade do
    /// espaço-papel, e porque a folha precisa saber onde ela fica.
    Viewport(Viewport),
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

    /// A mesma entidade, com a forma levada pela transformação.
    ///
    /// Camada e cor não são geometria e atravessam intactas.
    #[must_use]
    pub fn transformed(&self, transform: &Transform) -> Self {
        Self {
            layer: self.layer,
            color: self.color,
            geometry: self.geometry.transformed(transform),
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
    /// A mesma forma, levada pela transformação.
    ///
    /// A semelhança preserva o tipo — circunferência continua circunferência,
    /// arco continua arco —, que é a razão de o kernel só admitir semelhanças
    /// (ver `Transform`).
    ///
    /// Duas consequências da reflexão merecem atenção, porque errá-las produz um
    /// desenho parecido com o certo e diferente dele. **O arco troca de pontas:**
    /// ele é sempre anti-horário, e o que era percorrido num sentido passa a ser
    /// percorrido no outro, de modo que o ângulo final vira inicial. **O
    /// abaulamento troca de sinal**, pela mesma razão.
    ///
    /// O texto é levado de posição e de altura, mas as letras **não** são
    /// espelhadas: é a convenção do AutoCAD com `MIRRTEXT` desligado, e existe
    /// porque carimbo espelhado é ilegível, não é desenho.
    #[must_use]
    pub fn transformed(&self, transform: &Transform) -> Self {
        match self {
            Self::Line(line) => Self::Line(Line {
                start: transform.apply(line.start),
                end: transform.apply(line.end),
            }),
            Self::Circle(circle) => Self::Circle(Circle {
                center: transform.apply(circle.center),
                radius: circle.radius * transform.scale(),
            }),
            Self::Arc(arc) => {
                let (start, end) = if transform.is_reflected() {
                    (arc.end_angle, arc.start_angle)
                } else {
                    (arc.start_angle, arc.end_angle)
                };

                Self::Arc(Arc {
                    center: transform.apply(arc.center),
                    radius: arc.radius * transform.scale(),
                    start_angle: transform.apply_angle(start),
                    end_angle: transform.apply_angle(end),
                })
            }
            Self::Polyline(polyline) => Self::Polyline(Polyline {
                vertices: polyline
                    .vertices
                    .iter()
                    .map(|vertex| PolylineVertex {
                        point: transform.apply(vertex.point),
                        bulge: if transform.is_reflected() {
                            -vertex.bulge
                        } else {
                            vertex.bulge
                        },
                    })
                    .collect(),
                closed: polyline.closed,
            }),
            Self::Text(text) => Self::Text(Text {
                position: transform.apply(text.position),
                content: text.content.clone(),
                height: text.height * transform.scale(),
                rotation: transform.apply_angle(text.rotation),
            }),
            Self::Viewport(viewport) => Self::Viewport(viewport.transformed(transform)),
        }
    }

    /// Caixa envolvente da forma.
    #[must_use]
    pub fn bounding_box(&self) -> Aabb {
        match self {
            Self::Line(line) => line.bounding_box(),
            Self::Circle(circle) => circle.bounding_box(),
            Self::Arc(arc) => arc.bounding_box(),
            Self::Polyline(polyline) => polyline.bounding_box(),
            Self::Text(text) => text_bounding_box(text),
            Self::Viewport(viewport) => viewport.bounding_box(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::LayerTable;
    use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};
    use neocad_geometry::{PolylineVertex, Tolerance, Transform};

    fn layer() -> LayerId {
        LayerTable::new().default_layer()
    }

    fn assert_bounds(actual: Aabb, min: Point2, max: Point2) {
        let tolerancia = Tolerance::default();

        assert!(
            tolerancia.points_equal(actual.min(), min)
                && tolerancia.points_equal(actual.max(), max),
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
    fn a_reflexao_troca_as_pontas_do_arco() {
        // Arco do primeiro quadrante, espelhado pelo eixo X. O resultado ocupa o
        // quarto quadrante, e o que era ângulo final vira inicial — porque o
        // arco é sempre anti-horário e o percurso inverteu.
        let arco = Geometry::Arc(Arc {
            center: Point2::ORIGIN,
            radius: 2.0,
            start_angle: 0.0,
            end_angle: FRAC_PI_2,
        });
        let espelho = Transform::mirror(Point2::ORIGIN, 0.0);

        let Geometry::Arc(refletido) = arco.transformed(&espelho) else {
            panic!("continua sendo um arco");
        };
        let tolerancia = Tolerance::default();

        assert!(tolerancia.angles_equal(refletido.start_angle, -FRAC_PI_2));
        assert!(tolerancia.angles_equal(refletido.end_angle, 0.0));

        // E o desenho é o mesmo conjunto de pontos: espelhar duas vezes volta.
        let de_volta = arco.transformed(&espelho).transformed(&espelho);

        assert_eq!(de_volta, arco);
    }

    #[test]
    fn a_reflexao_troca_o_sinal_do_abaulamento() {
        let polilinha = Geometry::Polyline(Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 0.6),
                PolylineVertex::straight(Point2::new(4.0, 0.0)),
            ],
            closed: false,
        });
        let espelho = Transform::mirror(Point2::ORIGIN, 0.0);

        let Geometry::Polyline(refletida) = polilinha.transformed(&espelho) else {
            panic!("continua sendo uma polilinha");
        };

        assert!(Tolerance::default().lengths_equal(refletida.vertices[0].bulge, -0.6));
        assert_eq!(
            polilinha.transformed(&espelho).transformed(&espelho),
            polilinha
        );
    }

    #[test]
    fn o_giro_nao_mexe_no_sinal_do_abaulamento() {
        let polilinha = Geometry::Polyline(Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 0.6),
                PolylineVertex::straight(Point2::new(4.0, 0.0)),
            ],
            closed: false,
        });
        let giro = Transform::rotation_around(Point2::new(2.0, 2.0), FRAC_PI_4);

        let Geometry::Polyline(girada) = polilinha.transformed(&giro) else {
            panic!("continua sendo uma polilinha");
        };

        assert!(Tolerance::default().lengths_equal(girada.vertices[0].bulge, 0.6));
    }

    #[test]
    fn a_escala_multiplica_o_raio_e_a_altura_do_texto() {
        let tolerancia = Tolerance::default();
        let escala = Transform::scaling_around(Point2::ORIGIN, 3.0);

        let Geometry::Circle(circulo) = Geometry::Circle(Circle {
            center: Point2::new(1.0, 0.0),
            radius: 2.0,
        })
        .transformed(&escala) else {
            panic!("continua sendo uma circunferência");
        };

        assert!(tolerancia.lengths_equal(circulo.radius, 6.0));
        assert!(tolerancia.points_equal(circulo.center, Point2::new(3.0, 0.0)));

        let Geometry::Text(texto) = Geometry::Text(Text {
            position: Point2::ORIGIN,
            content: "COTA".to_owned(),
            height: 2.5,
            rotation: 0.0,
        })
        .transformed(&escala) else {
            panic!("continua sendo um texto");
        };

        assert!(tolerancia.lengths_equal(texto.height, 7.5));
    }

    #[test]
    fn o_texto_espelhado_continua_legivel() {
        // Convenção do AutoCAD com `MIRRTEXT` desligado: a posição é espelhada,
        // as letras não. Carimbo espelhado é ilegível, não é desenho.
        let texto = Geometry::Text(Text {
            position: Point2::new(3.0, 4.0),
            content: "PLANTA".to_owned(),
            height: 2.0,
            rotation: 0.0,
        });
        let espelho = Transform::mirror(Point2::ORIGIN, 0.0);

        let Geometry::Text(refletido) = texto.transformed(&espelho) else {
            panic!("continua sendo um texto");
        };
        let tolerancia = Tolerance::default();

        assert!(tolerancia.points_equal(refletido.position, Point2::new(3.0, -4.0)));
        assert!(tolerancia.angles_equal(refletido.rotation, 0.0));
        assert_eq!(refletido.content, "PLANTA");
    }

    #[test]
    fn camada_e_cor_atravessam_a_transformacao() {
        let camada = layer();
        let mut entidade = Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::ORIGIN,
                end: Point2::new(1.0, 1.0),
            }),
        );
        entidade.color = EntityColor::ByBlock;

        let movida = entidade.transformed(&Transform::translation(5.0, 5.0));

        assert_eq!(movida.layer, camada);
        assert_eq!(movida.color, EntityColor::ByBlock);
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
            Tolerance::default().lengths_equal(caixa.max().x, 1.0),
            "o cruzamento de 0° tem de entrar na caixa"
        );
    }

    #[test]
    fn caixa_do_arco_que_cruza_o_eixo_y_positivo() {
        // De 45° a 135°: atinge y = 1 ao cruzar 90°.
        let caixa = Geometry::Arc(arc(FRAC_PI_4, 3.0 * FRAC_PI_4)).bounding_box();

        assert!(Tolerance::default().lengths_equal(caixa.max().y, 1.0));
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

        assert!(Tolerance::default().lengths_equal(caixa.min().x, -1.0));
    }

    #[test]
    fn caixa_do_arco_que_cruza_o_eixo_y_negativo() {
        // De 225° a 315°: atinge y = -1 ao cruzar 270°.
        let caixa = Geometry::Arc(arc(5.0 * FRAC_PI_4, 7.0 * FRAC_PI_4)).bounding_box();

        assert!(Tolerance::default().lengths_equal(caixa.min().y, -1.0));
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
        let geometria = Geometry::Polyline(Polyline::from_points(
            [
                Point2::new(0.0, 0.0),
                Point2::new(5.0, 2.0),
                Point2::new(-1.0, 8.0),
            ],
            false,
        ));

        assert_bounds(
            geometria.bounding_box(),
            Point2::new(-1.0, 0.0),
            Point2::new(5.0, 8.0),
        );
    }

    #[test]
    fn a_caixa_da_polilinha_acompanha_o_trecho_em_arco() {
        // Meia-volta de (0,0) a (2,0), que desce até y = −1, e depois um trecho
        // reto para cima. Uma caixa tirada só dos vértices pararia em y = 0 e
        // cortaria o arco no ajuste de vista.
        let geometria = Geometry::Polyline(Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 1.0),
                PolylineVertex::straight(Point2::new(2.0, 0.0)),
                PolylineVertex::straight(Point2::new(2.0, 5.0)),
            ],
            closed: false,
        });

        assert_bounds(
            geometria.bounding_box(),
            Point2::new(0.0, -1.0),
            Point2::new(2.0, 5.0),
        );
    }

    #[test]
    fn fechar_a_polilinha_nao_muda_a_caixa() {
        let vertices = [Point2::ORIGIN, Point2::new(4.0, 0.0), Point2::new(4.0, 3.0)];
        let aberta = Geometry::Polyline(Polyline::from_points(vertices, false));
        let fechada = Geometry::Polyline(Polyline::from_points(vertices, true));

        assert_eq!(aberta.bounding_box(), fechada.bounding_box());
    }

    #[test]
    fn polilinha_sem_vertices_produz_caixa_degenerada_na_origem() {
        let geometria = Geometry::Polyline(Polyline::from_points([], false));

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
