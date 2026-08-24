// Caminho relativo: kernel/neocad-model/src/viewport.rs
//! \file kernel/neocad-model/src/viewport.rs
//! \brief Janela de espaço-papel que mostra uma vista do espaço-modelo.
//! \author Iago Leal
//! \date 2026-08-21

use neocad_geometry::{Aabb, Point2};

use crate::id::EntityId;

/// O que delimita o que a janela mostra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportClip {
    /// A própria janela retangular delimita a vista. É o caso comum.
    Window,
    /// Uma entidade do espaço-papel delimita a vista.
    ///
    /// Resolver a geometria dessa entidade é trabalho de quem desenha, não do
    /// modelo: aqui fica o vínculo, que é o que o arquivo guarda.
    Boundary(EntityId),
}

/// Janela de espaço-papel que mostra uma vista do espaço-modelo.
///
/// É o que transforma desenho em prancha: sem viewport, a folha tem carimbo e
/// não tem desenho.
///
/// # A escala é derivada, não guardada
///
/// A escala de uma viewport é a razão entre a altura da janela no papel e a
/// altura da vista no modelo. Guardá-la num campo próprio criaria uma segunda
/// fonte de verdade que pode divergir das outras duas — e uma prancha com escala
/// declarada diferente da desenhada é pior do que uma sem escala, porque o erro
/// só aparece quando alguém mede no papel impresso.
///
/// # Convenção de giro
///
/// `twist` é o ângulo, em radianos e no sentido anti-horário, com que o conteúdo
/// do modelo **aparece girado na folha**. O DXF grava esse ângulo no código
/// `51`, e mapear um no outro é trabalho da leitura (MT-KL-11), onde há teste
/// para fixar o sinal — sinal trocado aqui gira a prancha para o lado errado, e
/// o erro passa despercebido até alguém plotar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Centro da janela, em coordenadas do espaço-papel.
    pub center: Point2,
    /// Largura da janela no papel.
    pub width: f64,
    /// Altura da janela no papel.
    pub height: f64,
    /// Ponto do espaço-modelo que aparece no centro da janela.
    pub view_center: Point2,
    /// Altura da vista, em unidades do espaço-modelo.
    pub view_height: f64,
    /// Giro do conteúdo na folha, em radianos, anti-horário.
    pub twist: f64,
    /// O que delimita a vista.
    pub clip: ViewportClip,
    /// Janela desligada não mostra nada, mas continua existindo na folha.
    pub is_on: bool,
}

impl Viewport {
    /// Escala da janela: unidades de papel por unidade de modelo.
    ///
    /// `None` quando a altura da vista é zero ou não é finita — arquivo
    /// defeituoso, e devolver infinito espalharia o defeito para dentro do
    /// desenho em vez de deixá-lo aparecer aqui.
    #[must_use]
    pub fn scale(&self) -> Option<f64> {
        (self.view_height.is_finite() && self.view_height != 0.0)
            .then(|| self.height / self.view_height)
    }

    /// Caixa envolvente da janela, no espaço-papel.
    ///
    /// É a janela que ocupa lugar na folha; o que ela mostra vive no
    /// espaço-modelo e não entra nesta conta.
    #[must_use]
    pub fn bounding_box(&self) -> Aabb {
        let meia_largura = self.width.abs() / 2.0;
        let meia_altura = self.height.abs() / 2.0;

        Aabb::new(
            Point2::new(self.center.x - meia_largura, self.center.y - meia_altura),
            Point2::new(self.center.x + meia_largura, self.center.y + meia_altura),
        )
    }

    /// Leva um ponto do espaço-modelo para o espaço-papel.
    ///
    /// `None` quando a escala não existe; ver [`Viewport::scale`].
    #[must_use]
    pub fn model_to_paper(&self, point: Point2) -> Option<Point2> {
        let escala = self.scale()?;
        let (seno, cosseno) = self.twist.sin_cos();

        let x = (point.x - self.view_center.x) * escala;
        let y = (point.y - self.view_center.y) * escala;

        Some(Point2::new(
            self.center.x + x * cosseno - y * seno,
            self.center.y + x * seno + y * cosseno,
        ))
    }

    /// Leva um ponto do espaço-papel para o espaço-modelo.
    ///
    /// É a inversa exata de [`Viewport::model_to_paper`]: giro contrário e
    /// escala recíproca. `None` quando a escala não existe.
    #[must_use]
    pub fn paper_to_model(&self, point: Point2) -> Option<Point2> {
        let escala = self.scale()?;
        let (seno, cosseno) = (-self.twist).sin_cos();

        let x = point.x - self.center.x;
        let y = point.y - self.center.y;

        Some(Point2::new(
            self.view_center.x + (x * cosseno - y * seno) / escala,
            self.view_center.y + (x * seno + y * cosseno) / escala,
        ))
    }

    /// Região do espaço-modelo que a janela mostra, sem giro.
    ///
    /// Com giro diferente de zero a região vista é um retângulo inclinado, e
    /// esta caixa é a menor alinhada aos eixos que o contém — o que é o
    /// suficiente para descarte grosseiro, e não serve como recorte exato.
    #[must_use]
    pub fn visible_model_area(&self) -> Option<Aabb> {
        let cantos = [
            Point2::new(
                self.center.x - self.width.abs() / 2.0,
                self.center.y - self.height.abs() / 2.0,
            ),
            Point2::new(
                self.center.x + self.width.abs() / 2.0,
                self.center.y - self.height.abs() / 2.0,
            ),
            Point2::new(
                self.center.x + self.width.abs() / 2.0,
                self.center.y + self.height.abs() / 2.0,
            ),
            Point2::new(
                self.center.x - self.width.abs() / 2.0,
                self.center.y + self.height.abs() / 2.0,
            ),
        ];

        let mut no_modelo = Vec::with_capacity(cantos.len());

        for canto in cantos {
            no_modelo.push(self.paper_to_model(canto)?);
        }

        Aabb::from_points(no_modelo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Janela de 100×50 no papel, centrada em (200, 150), mostrando o modelo em
    /// torno de (10, 20) com 25 unidades de altura — escala 2.
    fn janela() -> Viewport {
        Viewport {
            center: Point2::new(200.0, 150.0),
            width: 100.0,
            height: 50.0,
            view_center: Point2::new(10.0, 20.0),
            view_height: 25.0,
            twist: 0.0,
            clip: ViewportClip::Window,
            is_on: true,
        }
    }

    fn perto(um: Point2, outro: Point2) -> bool {
        (um.x - outro.x).abs() < 1e-9 && (um.y - outro.y).abs() < 1e-9
    }

    #[test]
    fn a_escala_e_derivada_da_altura() {
        assert_eq!(janela().scale(), Some(2.0));
    }

    #[test]
    fn altura_de_vista_invalida_nao_vira_infinito() {
        // Arquivo defeituoso não pode espalhar o defeito para dentro do desenho.
        for altura in [0.0, f64::NAN, f64::INFINITY] {
            let torta = Viewport {
                view_height: altura,
                ..janela()
            };

            assert_eq!(torta.scale(), None);
            assert_eq!(torta.model_to_paper(Point2::ORIGIN), None);
            assert_eq!(torta.paper_to_model(Point2::ORIGIN), None);
        }
    }

    #[test]
    fn a_caixa_envolvente_e_a_janela_no_papel() {
        let caixa = janela().bounding_box();

        assert_eq!(caixa.min(), Point2::new(150.0, 125.0));
        assert_eq!(caixa.max(), Point2::new(250.0, 175.0));
    }

    #[test]
    fn a_caixa_envolvente_tolera_dimensao_negativa() {
        // Arquivo real traz largura negativa quando a janela foi espelhada; a
        // caixa continua sendo um retângulo, e não um vazio.
        let espelhada = Viewport {
            width: -100.0,
            height: -50.0,
            ..janela()
        };

        assert_eq!(espelhada.bounding_box(), janela().bounding_box());
    }

    #[test]
    fn o_centro_da_vista_cai_no_centro_da_janela() {
        let janela = janela();

        assert_eq!(
            janela.model_to_paper(janela.view_center),
            Some(janela.center)
        );
    }

    #[test]
    fn a_escala_amplia_a_distancia_ao_centro() {
        let janela = janela();

        // Um ponto 5 unidades à direita do centro da vista aparece 10 unidades à
        // direita do centro da janela, porque a escala é 2.
        assert_eq!(
            janela.model_to_paper(Point2::new(15.0, 20.0)),
            Some(Point2::new(210.0, 150.0))
        );
    }

    #[test]
    fn a_ida_e_volta_sem_giro_devolve_o_mesmo_ponto() {
        let janela = janela();

        for ponto in [
            Point2::new(10.0, 20.0),
            Point2::new(-30.0, 45.5),
            Point2::new(1234.5, -678.25),
        ] {
            let ida = janela.model_to_paper(ponto).expect("escala válida");
            let volta = janela.paper_to_model(ida).expect("escala válida");

            assert!(perto(volta, ponto), "{ponto:?} voltou como {volta:?}");
        }
    }

    #[test]
    fn a_ida_e_volta_com_giro_devolve_o_mesmo_ponto() {
        // O critério de aceite do MT-KL-07: com giro diferente de zero, os dois
        // sentidos precisam se desfazer exatamente.
        for giro in [
            core::f64::consts::FRAC_PI_6,
            core::f64::consts::FRAC_PI_2,
            -core::f64::consts::FRAC_PI_3,
            2.345,
        ] {
            let girada = Viewport {
                twist: giro,
                ..janela()
            };

            for ponto in [
                Point2::new(10.0, 20.0),
                Point2::new(-30.0, 45.5),
                Point2::new(1234.5, -678.25),
            ] {
                let ida = girada.model_to_paper(ponto).expect("escala válida");
                let volta = girada.paper_to_model(ida).expect("escala válida");

                assert!(
                    perto(volta, ponto),
                    "giro {giro}: {ponto:?} voltou como {volta:?}"
                );
            }
        }
    }

    #[test]
    fn o_giro_gira_no_sentido_anti_horario() {
        // Fixar o sinal aqui é o que impede a prancha de sair girada para o lado
        // errado sem ninguém notar até plotar.
        let girada = Viewport {
            twist: core::f64::consts::FRAC_PI_2,
            ..janela()
        };

        // Um ponto à direita do centro da vista aparece **acima** do centro da
        // janela quando o conteúdo gira um quarto de volta anti-horário.
        let no_papel = girada
            .model_to_paper(Point2::new(15.0, 20.0))
            .expect("escala válida");

        assert!(perto(no_papel, Point2::new(200.0, 160.0)), "{no_papel:?}");
    }

    #[test]
    fn o_centro_nao_se_move_com_o_giro() {
        let girada = Viewport {
            twist: 1.234,
            ..janela()
        };

        assert!(perto(
            girada
                .model_to_paper(girada.view_center)
                .expect("escala válida"),
            girada.center
        ));
    }

    #[test]
    fn a_area_vista_sem_giro_e_a_janela_dividida_pela_escala() {
        let area = janela().visible_model_area().expect("escala válida");

        // 100×50 no papel, escala 2, dá 50×25 no modelo, em torno de (10, 20).
        assert!(perto(area.min(), Point2::new(-15.0, 7.5)));
        assert!(perto(area.max(), Point2::new(35.0, 32.5)));
    }

    #[test]
    fn a_area_vista_com_giro_cresce_porque_a_caixa_e_alinhada_aos_eixos() {
        // Com giro, a região vista é um retângulo inclinado; esta caixa é a menor
        // alinhada aos eixos que o contém, e serve para descarte grosseiro — não
        // como recorte exato.
        let girada = Viewport {
            twist: core::f64::consts::FRAC_PI_4,
            ..janela()
        };

        let sem_giro = janela().visible_model_area().expect("escala válida");
        let com_giro = girada.visible_model_area().expect("escala válida");

        assert!(com_giro.width() > sem_giro.width());
        assert!(com_giro.height() > sem_giro.height());
    }

    #[test]
    fn area_vista_de_janela_torta_nao_existe() {
        let torta = Viewport {
            view_height: 0.0,
            ..janela()
        };

        assert_eq!(torta.visible_model_area(), None);
    }

    #[test]
    fn o_recorte_por_entidade_e_guardado_como_vinculo() {
        let recortada = Viewport {
            clip: ViewportClip::Boundary(EntityId::from_bits(1).expect("identificador válido")),
            ..janela()
        };

        assert!(matches!(recortada.clip, ViewportClip::Boundary(_)));
        assert_ne!(recortada.clip, ViewportClip::Window);
    }
}
