// Caminho relativo: kernel/neocad-geometry/src/spatial.rs
//! \file kernel/neocad-geometry/src/spatial.rs
//! \brief Índice espacial para consulta por região.
//! \author Iago Leal
//! \date 2026-09-07
//!
//! Varredura linear serve para mil entidades e não serve para as centenas de
//! milhares de um desenho real. Cada movimento de cursor da interface é uma
//! consulta destas.
//!
//! # O que este índice responde, e o que ele não responde
//!
//! Ele trabalha sobre **caixas envolventes**, não sobre a geometria. O que sai
//! daqui é a lista de candidatos cuja caixa toca a região perguntada — nunca a
//! lista do que de fato a toca. Um círculo grande tem caixa grande, e o canto
//! dessa caixa está longe do traço. Refinar é trabalho do teste de acerto
//! (MT-K3-07), que só precisa olhar os poucos candidatos em vez do desenho
//! inteiro.
//!
//! # Por que hierarquia de caixas, e não grade
//!
//! Grade uniforme pressupõe entidades de tamanho parecido. Um desenho técnico
//! tem a moldura da prancha e o texto de uma cota no mesmo arquivo, com quatro
//! ordens de grandeza entre eles — a célula que serve a um é inútil para o
//! outro. A hierarquia se adapta ao que existe, porque é construída a partir dos
//! próprios dados.

use crate::{Aabb, Point2};

/// Quantos itens uma folha guarda antes de valer a pena dividir.
///
/// Abaixo disso, percorrer a folha é mais barato que descer mais um nível.
const LEAF_CAPACITY: usize = 8;

/// Nó da hierarquia.
#[derive(Debug)]
enum Node {
    /// Faixa de itens, já ordenada pela construção.
    Leaf {
        bounds: Aabb,
        from: usize,
        to: usize,
    },
    /// Dois filhos, com a caixa que envolve ambos.
    Branch {
        bounds: Aabb,
        left: Box<Node>,
        right: Box<Node>,
    },
}

impl Node {
    fn bounds(&self) -> Aabb {
        match self {
            Self::Leaf { bounds, .. } | Self::Branch { bounds, .. } => *bounds,
        }
    }
}

/// Índice espacial estático sobre caixas envolventes.
///
/// É construído de uma vez e não é atualizado: uma edição reconstrói. Manter a
/// hierarquia equilibrada sob inserção e remoção custa complexidade que só se
/// paga com medição, e medição ainda não há.
#[derive(Debug)]
pub struct SpatialIndex<T> {
    items: Vec<(Aabb, T)>,
    /// Posição de entrada de cada item, já que a construção os reordena.
    origin: Vec<usize>,
    root: Option<Node>,
}

impl<T: Copy> SpatialIndex<T> {
    /// Constrói o índice a partir das caixas e dos seus donos.
    ///
    /// A ordem de entrada é preservada nas respostas, o que torna o resultado
    /// independente da forma que a hierarquia tomou.
    #[must_use]
    pub fn build(items: impl IntoIterator<Item = (Aabb, T)>) -> Self {
        let items: Vec<(Aabb, T)> = items.into_iter().collect();
        let mut order: Vec<usize> = (0..items.len()).collect();
        let root = build_node(&items, &mut order, 0);

        // A construção reordena os índices; guardar os itens na ordem em que a
        // hierarquia os agrupou é o que permite a folha ser uma faixa contígua.
        // A posição de entrada segue junto, para que a resposta não dependa da
        // forma que a hierarquia tomou.
        let origin = order.clone();
        let items = order.iter().map(|index| items[*index]).collect();

        Self {
            items,
            origin,
            root,
        }
    }

    /// Quantidade de itens indexados.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Se o índice está vazio.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Candidatos cuja caixa toca a região.
    #[must_use]
    pub fn query_box(&self, area: Aabb) -> Vec<T> {
        let mut found = Vec::new();

        if let Some(root) = &self.root {
            self.visit(root, area, &mut found);
        }

        found.sort_unstable_by_key(|index| self.origin[*index]);
        found.into_iter().map(|index| self.items[index].1).collect()
    }

    /// Candidatos cuja caixa alcança o ponto dentro do raio.
    ///
    /// Raio negativo ou não numérico vira zero, que é a consulta mais restrita
    /// possível — devolver o desenho inteiro por causa de um valor inválido
    /// seria pior que devolver nada.
    #[must_use]
    pub fn query_point(&self, point: Point2, radius: f64) -> Vec<T> {
        let radius = if radius.is_finite() && radius > 0.0 {
            radius
        } else {
            0.0
        };

        self.query_box(Aabb::new(
            Point2::new(point.x - radius, point.y - radius),
            Point2::new(point.x + radius, point.y + radius),
        ))
    }

    /// Percorre a hierarquia, recolhendo as posições que interessam.
    fn visit(&self, node: &Node, area: Aabb, found: &mut Vec<usize>) {
        if !node.bounds().intersects(area) {
            return;
        }

        match node {
            Node::Leaf { from, to, .. } => {
                for index in *from..*to {
                    if self.items[index].0.intersects(area) {
                        found.push(index);
                    }
                }
            }
            Node::Branch { left, right, .. } => {
                self.visit(left, area, found);
                self.visit(right, area, found);
            }
        }
    }
}

/// Constrói um nó sobre a faixa de `order` que começa em `offset`.
fn build_node<T: Copy>(items: &[(Aabb, T)], order: &mut [usize], offset: usize) -> Option<Node> {
    if order.is_empty() {
        return None;
    }

    let bounds = Aabb::union_all(order.iter().map(|index| items[*index].0))?;

    if order.len() <= LEAF_CAPACITY {
        return Some(Node::Leaf {
            bounds,
            from: offset,
            to: offset + order.len(),
        });
    }

    // Divide pelo eixo mais longo da caixa: é o que separa melhor, porque é onde
    // os itens estão mais espalhados.
    let horizontal = bounds.width() >= bounds.height();

    order.sort_by(|left, right| {
        center(items[*left].0, horizontal).total_cmp(&center(items[*right].0, horizontal))
    });

    let middle = order.len() / 2;
    let (first, second) = order.split_at_mut(middle);

    match (
        build_node(items, first, offset),
        build_node(items, second, offset + middle),
    ) {
        (Some(left), Some(right)) => Some(Node::Branch {
            bounds,
            left: Box::new(left),
            right: Box::new(right),
        }),
        (Some(only), None) | (None, Some(only)) => Some(only),
        (None, None) => None,
    }
}

/// Centro da caixa no eixo escolhido.
fn center(bounds: Aabb, horizontal: bool) -> f64 {
    if horizontal {
        bounds.center().x
    } else {
        bounds.center().y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Gerador determinístico: o teste precisa ser reproduzível.
    fn generator(seed: u64) -> impl FnMut() -> f64 {
        let mut state = seed;

        move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            #[allow(clippy::cast_precision_loss)]
            let unit = (state >> 11) as f64 / 9_007_199_254_740_992.0;

            unit
        }
    }

    fn caixas(quantidade: usize, magnitude: f64, seed: u64) -> Vec<(Aabb, usize)> {
        let mut next = generator(seed);

        (0..quantidade)
            .map(|indice| {
                let x = magnitude * (next() - 0.5) * 2.0;
                let y = magnitude * (next() - 0.5) * 2.0;
                // Tamanhos com quatro ordens de grandeza entre si, que é o que
                // um desenho técnico de fato tem.
                let largura = magnitude * next() * next() * next() / 100.0;
                let altura = magnitude * next() * next() * next() / 100.0;

                (
                    Aabb::new(Point2::new(x, y), Point2::new(x + largura, y + altura)),
                    indice,
                )
            })
            .collect()
    }

    /// Resposta da varredura linear, que é o gabarito.
    fn varredura(itens: &[(Aabb, usize)], area: Aabb) -> Vec<usize> {
        itens
            .iter()
            .filter(|(caixa, _)| caixa.intersects(area))
            .map(|(_, dono)| *dono)
            .collect()
    }

    #[test]
    fn o_indice_devolve_exatamente_o_que_a_varredura_devolveria() {
        for magnitude in [1.0, 7.0e6] {
            let itens = caixas(500, magnitude, 0x9e37_79b9_7f4a_7c15);
            let indice = SpatialIndex::build(itens.iter().copied());
            let mut next = generator(0x2545_f491_4f6c_dd1d);

            for _ in 0..200 {
                let x = magnitude * (next() - 0.5) * 2.0;
                let y = magnitude * (next() - 0.5) * 2.0;
                let lado = magnitude * next() / 10.0;
                let area = Aabb::new(Point2::new(x, y), Point2::new(x + lado, y + lado));

                let mut do_indice = indice.query_box(area);
                let mut da_varredura = varredura(&itens, area);

                do_indice.sort_unstable();
                da_varredura.sort_unstable();

                assert_eq!(
                    do_indice, da_varredura,
                    "magnitude {magnitude}: o índice pode ser mais rápido, nunca diferente"
                );
            }
        }
    }

    #[test]
    fn a_ordem_de_entrada_sobrevive_a_consulta() {
        let itens = caixas(200, 1.0, 0x1234_5678_9abc_def0);
        let indice = SpatialIndex::build(itens.iter().copied());
        let tudo = Aabb::new(Point2::new(-1e9, -1e9), Point2::new(1e9, 1e9));

        let achados = indice.query_box(tudo);
        let mut ordenados = achados.clone();
        ordenados.sort_unstable();

        assert_eq!(
            achados, ordenados,
            "a resposta não pode depender da forma que a hierarquia tomou"
        );
        assert_eq!(achados.len(), itens.len());
    }

    #[test]
    fn indice_vazio_nao_responde_nada() {
        let indice: SpatialIndex<usize> = SpatialIndex::build([]);

        assert!(indice.is_empty());
        assert_eq!(indice.len(), 0);
        assert!(indice
            .query_box(Aabb::new(Point2::ORIGIN, Point2::new(1.0, 1.0)))
            .is_empty());
    }

    #[test]
    fn a_consulta_por_ponto_alcanca_o_que_esta_dentro_do_raio() {
        let itens = vec![
            (Aabb::new(Point2::ORIGIN, Point2::new(1.0, 1.0)), 0),
            (
                Aabb::new(Point2::new(10.0, 10.0), Point2::new(11.0, 11.0)),
                1,
            ),
        ];
        let indice = SpatialIndex::build(itens);

        assert_eq!(indice.query_point(Point2::new(0.5, 0.5), 0.0), [0]);
        assert!(indice.query_point(Point2::new(5.0, 5.0), 1.0).is_empty());
        assert_eq!(indice.query_point(Point2::new(5.0, 5.0), 6.0), [0, 1]);
    }

    #[test]
    fn raio_invalido_vira_consulta_pontual() {
        let indice =
            SpatialIndex::build([(Aabb::new(Point2::ORIGIN, Point2::new(1.0, 1.0)), 0_usize)]);

        // Devolver o desenho inteiro por causa de um valor inválido seria pior
        // que devolver nada.
        assert!(indice
            .query_point(Point2::new(5.0, 5.0), f64::NAN)
            .is_empty());
        assert!(indice.query_point(Point2::new(5.0, 5.0), -3.0).is_empty());
        assert_eq!(indice.query_point(Point2::new(0.5, 0.5), f64::NAN), [0]);
    }

    #[test]
    fn caixas_que_apenas_se_encostam_contam_como_tocando() {
        let indice =
            SpatialIndex::build([(Aabb::new(Point2::ORIGIN, Point2::new(1.0, 1.0)), 0_usize)]);
        let vizinha = Aabb::new(Point2::new(1.0, 0.0), Point2::new(2.0, 1.0));

        assert_eq!(indice.query_box(vizinha), [0]);
    }
}
