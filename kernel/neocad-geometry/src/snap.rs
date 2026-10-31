// Caminho relativo: kernel/neocad-geometry/src/snap.rs
//! \file kernel/neocad-geometry/src/snap.rs
//! \brief Teste de acerto e prendimento do cursor a pontos notáveis.
//! \author Iago Leal
//! \date 2026-09-07
//!
//! É o gesto mais frequente de qualquer CAD, e o que separa desenhar de
//! rabiscar: sem prendimento não há precisão, e sem precisão não há desenho de
//! engenharia.
//!
//! # Por que a prioridade não é a distância
//!
//! Dentro do raio de captura costuma haver mais de um ponto notável, e a
//! tentação é entregar o mais próximo. Não funciona: o modo **próximo** devolve,
//! por definição, o ponto da curva mais perto do cursor, de modo que a sua
//! distância é sempre menor ou igual à de qualquer outro modo. Ordenar por
//! distância tornaria todos os demais inalcançáveis.
//!
//! A ordem é, então, por **modo** primeiro, e só entre modos iguais é que a
//! distância desempata. Restam empates de distância, e para eles há a ordem das
//! curvas e a das coordenadas — porque a resposta não pode depender da ordem em
//! que os candidatos foram gerados.

use core::f64::consts::{FRAC_PI_2, TAU};

use crate::{intersect, AnyCurve, Arc, Circle, Curve, Line, Point2, PolylineSegment, Tolerance};

/// Ponto notável a que o cursor pode prender.
///
/// A ordem de declaração **é** a ordem de prioridade, do mais específico ao mais
/// genérico, e mudá-la muda o comportamento do prendimento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnapMode {
    /// Extremidade de uma curva aberta ou vértice de polilinha.
    Endpoint,
    /// Meio da curva, pelo parâmetro.
    Midpoint,
    /// Centro de circunferência ou de arco.
    Center,
    /// Quadrante: os pontos a 0°, 90°, 180° e 270°.
    Quadrant,
    /// Cruzamento entre duas curvas.
    Intersection,
    /// Pé da perpendicular baixada do cursor.
    Perpendicular,
    /// Ponto em que a tangente passa pelo cursor.
    Tangent,
    /// Ponto da curva mais próximo do cursor. É o último recurso, e o único que
    /// nunca falha quando a curva está ao alcance.
    Nearest,
}

impl SnapMode {
    /// Todos os modos, na ordem de prioridade.
    pub const ALL: [Self; 8] = [
        Self::Endpoint,
        Self::Midpoint,
        Self::Center,
        Self::Quadrant,
        Self::Intersection,
        Self::Perpendicular,
        Self::Tangent,
        Self::Nearest,
    ];
}

/// Um ponto notável ao alcance do cursor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapCandidate {
    /// O modo que o produziu.
    pub mode: SnapMode,
    /// O ponto ao qual prender.
    pub point: Point2,
    /// Distância do cursor até ele.
    pub distance: f64,
    /// Índice da curva que o produziu.
    ///
    /// Um cruzamento nasce de duas curvas; fica registrado o **menor** dos dois
    /// índices, para que o candidato tenha identidade estável.
    pub curve: usize,
}

/// Curvas cuja geometria passa a menos de `radius` do cursor.
///
/// Diferente do índice espacial, que responde por caixas envolventes, aqui a
/// distância é até o traço. É o refino que o MT-K3-06 deixou para este ticket:
/// primeiro o índice reduz o desenho a meia dúzia de candidatos, depois isto
/// decide quais estão de fato sob o cursor.
///
/// A ordem devolvida é a de entrada, e não a de proximidade: quem quiser a mais
/// próxima que ordene, mas quem quiser todas não deve pagar por uma ordenação
/// que não pediu.
#[must_use]
pub fn hit_test(curves: &[AnyCurve], cursor: Point2, radius: f64) -> Vec<usize> {
    let radius = sane_radius(radius);

    curves
        .iter()
        .enumerate()
        .filter(|(_, curve)| curve.closest_point(cursor).distance <= radius)
        .map(|(index, _)| index)
        .collect()
}

/// O ponto notável a que o cursor deve prender, se houver algum.
///
/// # Exemplo
///
/// ```
/// use neocad_geometry::{snap, AnyCurve, Line, Point2, SnapMode, Tolerance};
///
/// let reta = Line {
///     start: Point2::ORIGIN,
///     end: Point2::new(10.0, 0.0),
/// };
/// let curvas = [AnyCurve::from(reta)];
///
/// // Perto da ponta, e com o meio da reta longe: prende na extremidade.
/// let preso = snap(
///     &curvas,
///     Point2::new(0.2, 0.2),
///     1.0,
///     &SnapMode::ALL,
///     Tolerance::default(),
/// )
/// .expect("há algo ao alcance");
///
/// assert_eq!(preso.mode, SnapMode::Endpoint);
/// assert_eq!(preso.point, Point2::ORIGIN);
/// ```
#[must_use]
pub fn snap(
    curves: &[AnyCurve],
    cursor: Point2,
    radius: f64,
    modes: &[SnapMode],
    tolerance: Tolerance,
) -> Option<SnapCandidate> {
    snap_candidates(curves, cursor, radius, modes, tolerance)
        .into_iter()
        .next()
}

/// Todos os pontos notáveis ao alcance, já na ordem de prioridade.
///
/// Serve à interface que quer mostrar as alternativas em vez de escolher por
/// conta própria — e é sobre esta lista que a ordem é testada.
#[must_use]
pub fn snap_candidates(
    curves: &[AnyCurve],
    cursor: Point2,
    radius: f64,
    modes: &[SnapMode],
    tolerance: Tolerance,
) -> Vec<SnapCandidate> {
    let radius = sane_radius(radius);
    let mut candidates = Vec::new();

    for (index, curve) in curves.iter().enumerate() {
        collect_from_curve(curve, index, cursor, modes, tolerance, &mut candidates);
    }

    if modes.contains(&SnapMode::Intersection) {
        collect_intersections(curves, cursor, radius, tolerance, &mut candidates);
    }

    candidates.retain(|candidate| candidate.distance <= radius);
    sort_candidates(&mut candidates);
    dedup_candidates(&mut candidates, tolerance);

    candidates
}

/// Raio de captura utilizável.
///
/// Raio negativo ou não numérico vira zero: prender a nada é melhor que prender
/// ao desenho inteiro por causa de um valor inválido.
fn sane_radius(radius: f64) -> f64 {
    if radius.is_finite() && radius > 0.0 {
        radius
    } else {
        0.0
    }
}

/// Ordena por prioridade de modo, depois distância, depois identidade.
///
/// As duas últimas chaves existem só para que empates não deixem a resposta ao
/// sabor da ordem de geração.
fn sort_candidates(candidates: &mut [SnapCandidate]) {
    candidates.sort_by(|first, second| {
        first
            .mode
            .cmp(&second.mode)
            .then(first.distance.total_cmp(&second.distance))
            .then(first.curve.cmp(&second.curve))
            .then(first.point.x.total_cmp(&second.point.x))
            .then(first.point.y.total_cmp(&second.point.y))
    });
}

/// Remove candidatos que são o mesmo ponto no mesmo modo.
fn dedup_candidates(candidates: &mut Vec<SnapCandidate>, tolerance: Tolerance) {
    let mut kept: Vec<SnapCandidate> = Vec::with_capacity(candidates.len());

    for candidate in candidates.drain(..) {
        let repeated = kept.iter().any(|other| {
            other.mode == candidate.mode && tolerance.points_equal(other.point, candidate.point)
        });

        if !repeated {
            kept.push(candidate);
        }
    }

    *candidates = kept;
}

/// Junta os candidatos de uma curva.
fn collect_from_curve(
    curve: &AnyCurve,
    index: usize,
    cursor: Point2,
    modes: &[SnapMode],
    tolerance: Tolerance,
    out: &mut Vec<SnapCandidate>,
) {
    let mut push = |mode: SnapMode, point: Point2| {
        if modes.contains(&mode) {
            out.push(SnapCandidate {
                mode,
                point,
                distance: cursor.distance_to(point),
                curve: index,
            });
        }
    };

    if modes.contains(&SnapMode::Nearest) {
        push(SnapMode::Nearest, curve.closest_point(cursor).point);
    }

    match curve {
        AnyCurve::Line(line) => line_points(line, cursor, &mut push),
        AnyCurve::Circle(circle) => circle_points(circle, cursor, &mut push),
        AnyCurve::Arc(arc) => arc_points(arc, cursor, tolerance, &mut push),
        AnyCurve::Polyline(polyline) => {
            // A polilinha aberta tem duas extremidades; a fechada não tem
            // nenhuma, e os seus vértices continuam sendo pontos notáveis.
            for vertex in polyline.points() {
                push(SnapMode::Endpoint, vertex);
            }

            for segment in polyline.segments() {
                match segment {
                    PolylineSegment::Straight(line) => line_points(&line, cursor, &mut push),
                    PolylineSegment::Curved { arc, .. } => {
                        arc_points(&arc, cursor, tolerance, &mut push);
                    }
                }
            }
        }
    }
}

/// Pontos notáveis de um segmento de reta.
fn line_points(line: &Line, cursor: Point2, push: &mut impl FnMut(SnapMode, Point2)) {
    push(SnapMode::Endpoint, line.start);
    push(SnapMode::Endpoint, line.end);
    push(SnapMode::Midpoint, line.point_at(0.5));

    // A perpendicular só existe se o pé cair dentro do segmento; prolongar o
    // segmento para alcançá-la é outra operação, e tem outro nome.
    let foot = line.closest_point(cursor);

    if foot.parameter > 0.0 && foot.parameter < 1.0 {
        push(SnapMode::Perpendicular, foot.point);
    }
}

/// Pontos notáveis de uma circunferência.
fn circle_points(circle: &Circle, cursor: Point2, push: &mut impl FnMut(SnapMode, Point2)) {
    push(SnapMode::Center, circle.center);

    for quarter in 0..4 {
        push(
            SnapMode::Quadrant,
            circle.point_at(f64::from(quarter) / 4.0),
        );
    }

    // Numa circunferência a perpendicular baixada do cursor é a projeção radial,
    // que coincide com o ponto mais próximo — o mesmo ponto em dois modos, e a
    // prioridade decide qual prevalece.
    push(SnapMode::Perpendicular, circle.closest_point(cursor).point);

    for point in tangent_points(circle.center, circle.radius.abs(), cursor) {
        push(SnapMode::Tangent, point);
    }
}

/// Pontos notáveis de um arco.
fn arc_points(
    arc: &Arc,
    cursor: Point2,
    tolerance: Tolerance,
    push: &mut impl FnMut(SnapMode, Point2),
) {
    push(SnapMode::Endpoint, arc.point_at(0.0));
    push(SnapMode::Endpoint, arc.point_at(1.0));
    push(SnapMode::Midpoint, arc.point_at(0.5));
    push(SnapMode::Center, arc.center);

    let sweep = arc.sweep();
    let radius = arc.radius.abs();

    for quarter in 0..4 {
        let angle = FRAC_PI_2 * f64::from(quarter);

        if (angle - arc.start_angle).rem_euclid(TAU) <= sweep {
            push(
                SnapMode::Quadrant,
                crate::point_on_circle(arc.center, radius, angle),
            );
        }
    }

    let radial = arc.closest_point(cursor);

    // Fora da varredura, o mais próximo é uma ponta, e ponta não é pé de
    // perpendicular — seria o mesmo ponto anunciado com outro nome.
    if radial.parameter > 0.0 && radial.parameter < 1.0 {
        push(SnapMode::Perpendicular, radial.point);
    }

    // A tangente da circunferência de suporte só vale se cair dentro da
    // varredura — e "cair dentro" é pergunta de coincidência, logo de tolerância
    // (ADR 0006), nunca de um epsilon escrito aqui.
    for point in tangent_points(arc.center, radius, cursor) {
        let magnitude = point.x.abs().max(point.y.abs()).max(radius);

        if arc.closest_point(point).distance <= tolerance.effective_linear(magnitude) {
            push(SnapMode::Tangent, point);
        }
    }
}

/// Pontos de uma circunferência em que a tangente passa pelo cursor.
///
/// De um ponto interno não sai tangente alguma, e do próprio traço sai uma só —
/// o próprio ponto. De fora saem duas.
fn tangent_points(center: Point2, radius: f64, cursor: Point2) -> Vec<Point2> {
    let distance = center.distance_to(cursor);

    if radius <= 0.0 || distance < radius {
        return Vec::new();
    }

    let direction = (cursor.y - center.y).atan2(cursor.x - center.x);
    let opening = (radius / distance).clamp(-1.0, 1.0).acos();

    if opening == 0.0 {
        return vec![crate::point_on_circle(center, radius, direction)];
    }

    vec![
        crate::point_on_circle(center, radius, direction + opening),
        crate::point_on_circle(center, radius, direction - opening),
    ]
}

/// Cruzamentos entre as curvas que estão ao alcance do cursor.
///
/// Só os pares cujas duas curvas o cursor alcança entram na conta: cruzamento a
/// dez telas de distância não é candidato a prendimento, e cruzar todas as
/// curvas do desenho contra todas custaria o quadrado do desenho.
fn collect_intersections(
    curves: &[AnyCurve],
    cursor: Point2,
    radius: f64,
    tolerance: Tolerance,
    out: &mut Vec<SnapCandidate>,
) {
    let reachable = hit_test(curves, cursor, radius);

    for (position, first) in reachable.iter().enumerate() {
        for second in &reachable[position + 1..] {
            for point in intersect(&curves[*first], &curves[*second], tolerance).points {
                out.push(SnapCandidate {
                    mode: SnapMode::Intersection,
                    point,
                    distance: cursor.distance_to(point),
                    curve: *first,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Polyline, PolylineVertex};

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    fn reta(x1: f64, y1: f64, x2: f64, y2: f64) -> AnyCurve {
        AnyCurve::from(Line {
            start: Point2::new(x1, y1),
            end: Point2::new(x2, y2),
        })
    }

    fn preso(curvas: &[AnyCurve], cursor: Point2, raio: f64) -> SnapCandidate {
        snap(curvas, cursor, raio, &SnapMode::ALL, tol()).expect("há algo ao alcance")
    }

    #[test]
    fn o_teste_de_acerto_mede_ate_o_traco_e_nao_ate_a_caixa() {
        // O cursor está no canto da caixa envolvente da circunferência, longe do
        // traço. O índice espacial o daria como candidato; aqui ele não passa.
        let circunferencia = AnyCurve::from(Circle {
            center: Point2::ORIGIN,
            radius: 10.0,
        });
        let curvas = [circunferencia];

        assert!(hit_test(&curvas, Point2::new(10.0, 10.0), 1.0).is_empty());
        assert_eq!(hit_test(&curvas, Point2::new(10.5, 0.0), 1.0), [0]);
    }

    #[test]
    fn a_extremidade_vence_o_ponto_mais_proximo() {
        // Os dois modos estão ao alcance, e o "próximo" está mais perto — é
        // sempre. Se a distância mandasse, extremidade nunca prenderia.
        let curvas = [reta(0.0, 0.0, 10.0, 0.0)];
        let achado = preso(&curvas, Point2::new(0.3, 0.4), 2.0);

        assert_eq!(achado.mode, SnapMode::Endpoint);
        assert!(tol().points_equal(achado.point, Point2::ORIGIN));

        let proximo = snap_candidates(&curvas, Point2::new(0.3, 0.4), 2.0, &SnapMode::ALL, tol())
            .into_iter()
            .find(|c| c.mode == SnapMode::Nearest)
            .expect("o modo próximo também está ao alcance");

        assert!(
            proximo.distance < achado.distance,
            "o mais próximo tinha de estar mais perto, e ainda assim perder"
        );
    }

    #[test]
    fn desligar_um_modo_entrega_o_seguinte() {
        let curvas = [reta(0.0, 0.0, 10.0, 0.0)];
        let modos = [SnapMode::Midpoint, SnapMode::Nearest];
        let achado =
            snap(&curvas, Point2::new(0.3, 0.4), 2.0, &modos, tol()).expect("há algo ao alcance");

        // Sem extremidade habilitada, sobra o próximo — o meio da reta está a
        // cinco unidades, fora do raio.
        assert_eq!(achado.mode, SnapMode::Nearest);
        assert!(tol().points_equal(achado.point, Point2::new(0.3, 0.0)));
    }

    #[test]
    fn a_ordem_dos_modos_e_a_da_declaracao() {
        // A ordem de prioridade é comportamento, não detalhe: mudá-la muda o
        // prendimento, e por isso fica presa aqui.
        assert_eq!(
            SnapMode::ALL,
            [
                SnapMode::Endpoint,
                SnapMode::Midpoint,
                SnapMode::Center,
                SnapMode::Quadrant,
                SnapMode::Intersection,
                SnapMode::Perpendicular,
                SnapMode::Tangent,
                SnapMode::Nearest,
            ]
        );
        assert!(SnapMode::Endpoint < SnapMode::Nearest);
    }

    #[test]
    fn empate_de_distancia_nao_depende_da_ordem_de_entrada() {
        // Duas retas simétricas em relação ao cursor: as extremidades ficam
        // exatamente à mesma distância.
        let esquerda = reta(-1.0, 0.0, -5.0, 0.0);
        let direita = reta(1.0, 0.0, 5.0, 0.0);

        let uma = preso(&[esquerda.clone(), direita.clone()], Point2::ORIGIN, 2.0);
        let outra = preso(&[direita, esquerda], Point2::ORIGIN, 2.0);

        // A curva de índice menor vence, e como a ordem de entrada mudou, o
        // ponto muda junto — o que **não** pode mudar é o critério.
        assert_eq!(uma.curve, 0);
        assert_eq!(outra.curve, 0);
        assert!(tol().points_equal(uma.point, Point2::new(-1.0, 0.0)));
        assert!(tol().points_equal(outra.point, Point2::new(1.0, 0.0)));
    }

    #[test]
    fn empate_na_mesma_curva_decide_pela_coordenada() {
        // Cursor no meio de uma reta horizontal: as duas extremidades estão à
        // mesma distância e pertencem à mesma curva.
        let curvas = [reta(-3.0, 0.0, 3.0, 0.0)];
        let candidatos =
            snap_candidates(&curvas, Point2::ORIGIN, 10.0, &[SnapMode::Endpoint], tol());

        assert_eq!(candidatos.len(), 2);
        // Menor x primeiro, sempre.
        assert!(tol().points_equal(candidatos[0].point, Point2::new(-3.0, 0.0)));
        assert!(tol().points_equal(candidatos[1].point, Point2::new(3.0, 0.0)));
    }

    #[test]
    fn o_cruzamento_prende_onde_as_duas_se_encontram() {
        let curvas = [reta(-5.0, 0.0, 5.0, 0.0), reta(0.0, -5.0, 0.0, 5.0)];
        let achado = snap(
            &curvas,
            Point2::new(0.2, 0.2),
            1.0,
            &[SnapMode::Intersection, SnapMode::Nearest],
            tol(),
        )
        .expect("há algo ao alcance");

        assert_eq!(achado.mode, SnapMode::Intersection);
        assert!(tol().points_equal(achado.point, Point2::ORIGIN));
    }

    #[test]
    fn cruzamento_fora_do_alcance_nao_e_candidato() {
        // As retas se cruzam na origem, mas o cursor está longe dali.
        let curvas = [reta(-50.0, 0.0, 50.0, 0.0), reta(0.0, -50.0, 0.0, 50.0)];
        let candidatos =
            snap_candidates(&curvas, Point2::new(30.0, 0.2), 1.0, &SnapMode::ALL, tol());

        assert!(candidatos.iter().all(|c| c.mode != SnapMode::Intersection));
    }

    #[test]
    fn os_quadrantes_da_circunferencia_sao_os_quatro_pontos_cardeais() {
        let curvas = [AnyCurve::from(Circle {
            center: Point2::ORIGIN,
            radius: 4.0,
        })];
        let candidatos = snap_candidates(
            &curvas,
            Point2::new(4.1, 0.0),
            1.0,
            &[SnapMode::Quadrant],
            tol(),
        );

        assert_eq!(candidatos.len(), 1, "só um quadrante está ao alcance");
        assert!(tol().points_equal(candidatos[0].point, Point2::new(4.0, 0.0)));
    }

    #[test]
    fn o_centro_prende_mesmo_com_o_traco_longe() {
        let curvas = [AnyCurve::from(Circle {
            center: Point2::new(5.0, 5.0),
            radius: 100.0,
        })];
        let achado = preso(&curvas, Point2::new(5.2, 5.1), 1.0);

        assert_eq!(achado.mode, SnapMode::Center);
        assert!(tol().points_equal(achado.point, Point2::new(5.0, 5.0)));
    }

    #[test]
    fn a_perpendicular_so_existe_dentro_do_segmento() {
        let curvas = [reta(0.0, 0.0, 10.0, 0.0)];

        let dentro = snap_candidates(
            &curvas,
            Point2::new(5.0, 0.5),
            1.0,
            &[SnapMode::Perpendicular],
            tol(),
        );

        assert_eq!(dentro.len(), 1);
        assert!(tol().points_equal(dentro[0].point, Point2::new(5.0, 0.0)));

        // Além da ponta, o pé cairia fora: prolongar o segmento para alcançá-lo
        // é outra operação.
        let fora = snap_candidates(
            &curvas,
            Point2::new(10.5, 0.2),
            1.0,
            &[SnapMode::Perpendicular],
            tol(),
        );

        assert!(fora.is_empty(), "{fora:?}");
    }

    #[test]
    fn a_tangente_sai_de_fora_e_nao_de_dentro() {
        let circunferencia = Circle {
            center: Point2::ORIGIN,
            radius: 1.0,
        };
        let curvas = [AnyCurve::from(circunferencia)];

        // De um ponto interno não há tangente alguma.
        assert!(snap_candidates(
            &curvas,
            Point2::new(0.2, 0.0),
            5.0,
            &[SnapMode::Tangent],
            tol()
        )
        .is_empty());

        // De fora saem duas, e ambas estão sobre a circunferência.
        let de_fora = snap_candidates(
            &curvas,
            Point2::new(2.0, 0.0),
            5.0,
            &[SnapMode::Tangent],
            tol(),
        );

        assert_eq!(de_fora.len(), 2, "{de_fora:?}");
        assert!(de_fora
            .iter()
            .all(|c| tol().lengths_equal(c.point.distance_to(Point2::ORIGIN), 1.0)));
    }

    #[test]
    fn os_vertices_da_polilinha_sao_extremidades() {
        let polilinha = Polyline::from_points(
            [Point2::ORIGIN, Point2::new(5.0, 0.0), Point2::new(5.0, 5.0)],
            false,
        );
        let curvas = [AnyCurve::from(polilinha)];
        let achado = preso(&curvas, Point2::new(5.1, 0.1), 1.0);

        assert_eq!(achado.mode, SnapMode::Endpoint);
        assert!(tol().points_equal(achado.point, Point2::new(5.0, 0.0)));
    }

    #[test]
    fn o_trecho_curvo_da_polilinha_oferece_centro_e_meio() {
        let polilinha = Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 1.0),
                PolylineVertex::straight(Point2::new(2.0, 0.0)),
            ],
            closed: false,
        };
        let curvas = [AnyCurve::from(polilinha)];
        let candidatos =
            snap_candidates(&curvas, Point2::new(1.0, -0.1), 1.5, &SnapMode::ALL, tol());

        assert!(
            candidatos.iter().any(|c| c.mode == SnapMode::Center
                && tol().points_equal(c.point, Point2::new(1.0, 0.0))),
            "o centro do arco tinha de estar entre os candidatos: {candidatos:?}"
        );
        assert!(
            candidatos.iter().any(|c| c.mode == SnapMode::Midpoint
                && tol().points_equal(c.point, Point2::new(1.0, -1.0))),
            "o meio do arco tinha de estar entre os candidatos: {candidatos:?}"
        );
    }

    #[test]
    fn fora_do_raio_nao_prende_em_nada() {
        let curvas = [reta(0.0, 0.0, 10.0, 0.0)];

        assert!(snap(&curvas, Point2::new(5.0, 50.0), 1.0, &SnapMode::ALL, tol()).is_none());
        assert!(hit_test(&curvas, Point2::new(5.0, 50.0), 1.0).is_empty());
    }

    #[test]
    fn raio_invalido_nao_prende_ao_desenho_inteiro() {
        let curvas = [reta(0.0, 0.0, 10.0, 0.0)];

        assert!(snap(
            &curvas,
            Point2::new(5.0, 1.0),
            f64::NAN,
            &SnapMode::ALL,
            tol()
        )
        .is_none());
        assert!(snap(&curvas, Point2::new(5.0, 1.0), -5.0, &SnapMode::ALL, tol()).is_none());
    }

    #[test]
    fn o_prendimento_funciona_em_coordenada_utm() {
        let base = 7.0e6;
        let curvas = [reta(base, base, base + 10.0, base)];
        let achado = preso(&curvas, Point2::new(base + 0.3, base + 0.4), 2.0);

        assert_eq!(achado.mode, SnapMode::Endpoint);
        assert!(tol().points_equal(achado.point, Point2::new(base, base)));
    }
}
