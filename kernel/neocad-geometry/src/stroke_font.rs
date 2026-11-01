// Caminho relativo: kernel/neocad-geometry/src/stroke_font.rs
//! \file kernel/neocad-geometry/src/stroke_font.rs
//! \brief Texto em traços: a fonte Hershey Roman Simplex (ADR 0010).
//! \author Iago Leal
//! \date 2026-11-01
//!
//! Cada caractere vira segmentos de reta, como na `txt.shx` do AutoCAD, para a
//! tela desenhar o texto pelo mesmo caminho das linhas. A altura pedida é a das
//! maiúsculas; a posição é o início da linha de base; a rotação é em torno dela.

use std::sync::OnceLock;

use crate::Point2;

/// Os glifos de 32 a 127; ver `data/LEIA-ME.md` para o formato e a licença.
const DATA: &str = include_str!("../data/rowmans.jhf");

/// Linha de base e altura das maiúsculas, em unidades da fonte (y cresce para baixo).
const BASELINE: i32 = 9;
const CAP_HEIGHT: f64 = 21.0;

/// Onde fica a base do acento: acima da altura de x nas minúsculas, acima do
/// topo nas maiúsculas.
const ACCENT_LOWER: i32 = -8;
const ACCENT_UPPER: i32 = -14;

type Stroke = Vec<(i32, i32)>;

/// Um glifo: margens esquerda e direita e os traços, em unidades da fonte.
#[derive(Debug, Clone)]
struct Glyph {
    left: i32,
    right: i32,
    strokes: Vec<Stroke>,
}

fn parse(line: &str) -> Glyph {
    let coord = |b: u8| i32::from(b) - i32::from(b'R');
    let pairs: Vec<&[u8]> = line
        .as_bytes()
        .get(8..)
        .unwrap_or_default()
        .chunks(2)
        .collect();
    let (left, right) = pairs
        .first()
        .filter(|pair| pair.len() == 2)
        .map_or((0, 0), |pair| (coord(pair[0]), coord(pair[1])));

    let mut strokes: Vec<Stroke> = vec![Vec::new()];
    for pair in pairs.iter().skip(1).filter(|pair| pair.len() == 2) {
        if *pair == b" R" {
            strokes.push(Vec::new());
        } else if let Some(stroke) = strokes.last_mut() {
            stroke.push((coord(pair[0]), coord(pair[1])));
        }
    }
    strokes.retain(|stroke| stroke.len() > 1);

    Glyph {
        left,
        right,
        strokes,
    }
}

/// A fonte, lida na primeira chamada.
fn font() -> &'static [Glyph] {
    static FONT: OnceLock<Vec<Glyph>> = OnceLock::new();
    FONT.get_or_init(|| DATA.lines().filter(|l| l.len() > 8).map(parse).collect())
}

fn ascii(c: char) -> Glyph {
    let index = (c as usize).wrapping_sub(32);
    font()
        .get(index)
        .or_else(|| font().get(usize::from(b'?' - 32)))
        .cloned()
        .unwrap_or(Glyph {
            left: 0,
            right: 0,
            strokes: Vec::new(),
        })
}

/// Pingo de um ponto: o losango que a própria fonte usa no `i`.
fn dot(x: i32, y: i32) -> Stroke {
    vec![(x - 1, y), (x, y + 1), (x + 1, y), (x, y - 1), (x - 1, y)]
}

#[derive(Clone, Copy)]
enum Accent {
    Acute,
    Grave,
    Circumflex,
    Tilde,
    Diaeresis,
    Cedilla,
}

fn accent_strokes(accent: Accent, upper: bool) -> Vec<Stroke> {
    let y = if upper { ACCENT_UPPER } else { ACCENT_LOWER };
    match accent {
        Accent::Acute => vec![vec![(-1, y), (2, y - 3)]],
        Accent::Grave => vec![vec![(1, y), (-2, y - 3)]],
        Accent::Circumflex => vec![vec![(-3, y), (0, y - 3), (3, y)]],
        Accent::Tilde => vec![vec![
            (-4, y),
            (-3, y - 2),
            (-1, y - 2),
            (1, y - 1),
            (3, y - 1),
            (4, y - 3),
        ]],
        Accent::Diaeresis => vec![dot(-3, y - 1), dot(3, y - 1)],
        Accent::Cedilla => vec![vec![
            (1, BASELINE),
            (1, 11),
            (3, 12),
            (3, 13),
            (2, 14),
            (-1, 14),
        ]],
    }
}

/// `i` sem o pingo: só os traços que descem até a altura de x.
fn dotless_i() -> Glyph {
    let mut glyph = ascii('i');
    glyph
        .strokes
        .retain(|stroke| stroke.iter().any(|&(_, y)| y > ACCENT_LOWER));
    glyph
}

/// Letra latina acentuada: a base e o acento.
fn decompose(c: char) -> Option<(char, Accent)> {
    use Accent::{Acute, Cedilla, Circumflex, Diaeresis, Grave, Tilde};
    let pair = match c {
        'á' => ('a', Acute),
        'à' => ('a', Grave),
        'â' => ('a', Circumflex),
        'ã' => ('a', Tilde),
        'ä' => ('a', Diaeresis),
        'é' => ('e', Acute),
        'è' => ('e', Grave),
        'ê' => ('e', Circumflex),
        'ë' => ('e', Diaeresis),
        'í' => ('ı', Acute),
        'ì' => ('ı', Grave),
        'î' => ('ı', Circumflex),
        'ï' => ('ı', Diaeresis),
        'ó' => ('o', Acute),
        'ò' => ('o', Grave),
        'ô' => ('o', Circumflex),
        'õ' => ('o', Tilde),
        'ö' => ('o', Diaeresis),
        'ú' => ('u', Acute),
        'ù' => ('u', Grave),
        'û' => ('u', Circumflex),
        'ü' => ('u', Diaeresis),
        'ñ' => ('n', Tilde),
        'ç' => ('c', Cedilla),
        'Á' => ('A', Acute),
        'À' => ('A', Grave),
        'Â' => ('A', Circumflex),
        'Ã' => ('A', Tilde),
        'Ä' => ('A', Diaeresis),
        'É' => ('E', Acute),
        'È' => ('E', Grave),
        'Ê' => ('E', Circumflex),
        'Ë' => ('E', Diaeresis),
        'Í' => ('I', Acute),
        'Ì' => ('I', Grave),
        'Î' => ('I', Circumflex),
        'Ï' => ('I', Diaeresis),
        'Ó' => ('O', Acute),
        'Ò' => ('O', Grave),
        'Ô' => ('O', Circumflex),
        'Õ' => ('O', Tilde),
        'Ö' => ('O', Diaeresis),
        'Ú' => ('U', Acute),
        'Ù' => ('U', Grave),
        'Û' => ('U', Circumflex),
        'Ü' => ('U', Diaeresis),
        'Ñ' => ('N', Tilde),
        'Ç' => ('C', Cedilla),
        _ => return None,
    };
    Some(pair)
}

/// O glifo de qualquer caractere; o que a fonte não tem vira `?`.
fn glyph(c: char) -> Glyph {
    match c {
        ' '..='~' => ascii(c),
        '°' => font().get(95).cloned().unwrap_or_else(|| ascii('?')),
        'ı' => dotless_i(),
        '±' => Glyph {
            left: -13,
            right: 13,
            strokes: vec![
                vec![(0, -9), (0, 3)],
                vec![(-9, -3), (9, -3)],
                vec![(-9, 9), (9, 9)],
            ],
        },
        '⌀' | 'Ø' => slashed(ascii('O'), vec![(-9, 11), (9, -14)]),
        'ø' => slashed(ascii('o'), vec![(-7, 10), (7, -6)]),
        _ => match decompose(c) {
            Some((base, accent)) => {
                let mut glyph = glyph(base);
                glyph
                    .strokes
                    .extend(accent_strokes(accent, base.is_uppercase()));
                glyph
            }
            None => ascii('?'),
        },
    }
}

fn slashed(mut glyph: Glyph, slash: Stroke) -> Glyph {
    glyph.strokes.push(slash);
    glyph
}

/// Troca os códigos `%%` do AutoCAD pelo caractere que representam.
///
/// `%%u` e `%%o` (sublinhado e sobrelinhado) ainda não são desenhados: somem.
fn decode(content: &str) -> Vec<char> {
    let chars: Vec<char> = content.chars().collect();
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && chars.get(i + 1) == Some(&'%') {
            if let Some(&code) = chars.get(i + 2) {
                let digits: String = chars[i + 2..].iter().take(3).collect();
                let replacement = match code.to_ascii_lowercase() {
                    'd' => Some(Some('°')),
                    'p' => Some(Some('±')),
                    'c' => Some(Some('⌀')),
                    '%' => Some(Some('%')),
                    'u' | 'o' => Some(None),
                    _ if digits.len() == 3 && digits.chars().all(|d| d.is_ascii_digit()) => {
                        digits.parse::<u32>().ok().map(char::from_u32)
                    }
                    _ => None,
                };
                if let Some(replacement) = replacement {
                    out.extend(replacement);
                    i += if code.is_ascii_digit() { 5 } else { 3 };
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Segmentos que desenham `content` com a altura e a rotação dadas.
///
/// A altura é a das maiúsculas; `position` é o início da linha de base.
/// Altura não finita ou não positiva não desenha nada.
#[must_use]
pub fn text_strokes(
    content: &str,
    position: Point2,
    height: f64,
    rotation: f64,
) -> Vec<(Point2, Point2)> {
    if !(height.is_finite() && height > 0.0 && rotation.is_finite()) {
        return Vec::new();
    }

    let scale = height / CAP_HEIGHT;
    let (sin, cos) = rotation.sin_cos();
    let place = |x: f64, y: i32| {
        let local_x = x * scale;
        let local_y = f64::from(BASELINE - y) * scale;
        Point2::new(
            position.x + local_x * cos - local_y * sin,
            position.y + local_x * sin + local_y * cos,
        )
    };

    let mut out = Vec::new();
    let mut cursor = 0.0_f64;
    for c in decode(content) {
        let glyph = glyph(c);
        let offset = cursor - f64::from(glyph.left);
        for stroke in &glyph.strokes {
            for pair in stroke.windows(2) {
                out.push((
                    place(offset + f64::from(pair[0].0), pair[0].1),
                    place(offset + f64::from(pair[1].0), pair[1].1),
                ));
            }
        }
        cursor += f64::from(glyph.right - glyph.left);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perto(a: Point2, b: Point2) -> bool {
        (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9
    }

    /// Caixa `[min_x, min_y, max_x, max_y]` dos segmentos.
    fn caixa(segmentos: &[(Point2, Point2)]) -> [f64; 4] {
        segmentos
            .iter()
            .flat_map(|(a, b)| [*a, *b])
            .fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |c, p| {
                [c[0].min(p.x), c[1].min(p.y), c[2].max(p.x), c[3].max(p.y)]
            })
    }

    #[test]
    fn o_i_maiusculo_e_um_traco_da_base_ao_topo() {
        // Na fonte, `I` tem margens -4 e 4 e um traço vertical em x = 0, do topo
        // das maiúsculas (-12) à base (9): 21 unidades, a altura pedida aqui.
        let s = text_strokes("I", Point2::new(10.0, 20.0), 21.0, 0.0);

        assert_eq!(s.len(), 1);
        assert!(perto(s[0].0, Point2::new(14.0, 41.0)));
        assert!(perto(s[0].1, Point2::new(14.0, 20.0)));
    }

    #[test]
    fn a_altura_escala_e_o_caractere_seguinte_avanca() {
        let s = text_strokes("II", Point2::ORIGIN, 2.1, 0.0);

        assert_eq!(s.len(), 2);
        // Escala 0,1; avanço de 8 unidades da fonte.
        assert!(perto(s[1].0, Point2::new(1.2, 2.1)));
        assert!(perto(s[1].1, Point2::new(1.2, 0.0)));
    }

    #[test]
    fn a_rotacao_gira_em_torno_da_posicao() {
        let s = text_strokes(
            "I",
            Point2::new(1.0, 1.0),
            21.0,
            std::f64::consts::FRAC_PI_2,
        );

        assert_eq!(s.len(), 1);
        assert!(perto(s[0].0, Point2::new(1.0 - 21.0, 1.0 + 4.0)));
        assert!(perto(s[0].1, Point2::new(1.0, 1.0 + 4.0)));
    }

    #[test]
    fn espaco_nao_desenha_mas_avanca() {
        assert!(text_strokes(" ", Point2::ORIGIN, 1.0, 0.0).is_empty());
        // Espaço tem 16 unidades; o `I` depois dele começa em 16 + 4.
        let s = text_strokes(" I", Point2::ORIGIN, 21.0, 0.0);
        assert_eq!(s.len(), 1);
        assert!(perto(s[0].1, Point2::new(20.0, 0.0)));
    }

    #[test]
    fn altura_invalida_nao_desenha() {
        assert!(text_strokes("I", Point2::ORIGIN, 0.0, 0.0).is_empty());
        assert!(text_strokes("I", Point2::ORIGIN, f64::NAN, 0.0).is_empty());
        assert!(text_strokes("", Point2::ORIGIN, 1.0, 0.0).is_empty());
    }

    #[test]
    fn letras_acentuadas_ganham_o_acento_acima_e_a_cedilha_abaixo() {
        let e = caixa(&text_strokes("e", Point2::ORIGIN, 21.0, 0.0));
        let agudo = caixa(&text_strokes("é", Point2::ORIGIN, 21.0, 0.0));
        assert!(agudo[3] > e[3] + 1.0, "o agudo fica acima do e: {agudo:?}");

        let c = caixa(&text_strokes("c", Point2::ORIGIN, 21.0, 0.0));
        let cedilha = caixa(&text_strokes("ç", Point2::ORIGIN, 21.0, 0.0));
        assert!(
            cedilha[1] < c[1] - 1.0,
            "a cedilha fica abaixo do c: {cedilha:?}"
        );

        for letra in "áàâãäéêíóôõúüçÁÀÂÃÉÊÍÓÔÕÚÇñÑ".chars() {
            let base = text_strokes(&letra.to_string(), Point2::ORIGIN, 21.0, 0.0);
            assert!(!base.is_empty(), "{letra} sem traço");
        }
    }

    #[test]
    fn o_i_acentuado_perde_o_pingo() {
        let i = text_strokes("i", Point2::ORIGIN, 21.0, 0.0);
        let agudo = text_strokes("í", Point2::ORIGIN, 21.0, 0.0);
        let sem_pingo = text_strokes("ı", Point2::ORIGIN, 21.0, 0.0);
        assert!(!sem_pingo.is_empty());
        assert!(sem_pingo.len() < i.len());
        assert_eq!(agudo.len(), sem_pingo.len() + 1);
    }

    #[test]
    fn codigos_do_autocad_e_caractere_desconhecido() {
        let grau = text_strokes("°", Point2::ORIGIN, 1.0, 0.0);
        assert!(!grau.is_empty());
        assert_eq!(text_strokes("%%d", Point2::ORIGIN, 1.0, 0.0), grau);
        assert_eq!(text_strokes("%%D", Point2::ORIGIN, 1.0, 0.0), grau);
        assert_eq!(
            text_strokes("%%%", Point2::ORIGIN, 1.0, 0.0),
            text_strokes("%", Point2::ORIGIN, 1.0, 0.0)
        );
        assert_eq!(
            text_strokes("%%p", Point2::ORIGIN, 1.0, 0.0),
            text_strokes("±", Point2::ORIGIN, 1.0, 0.0)
        );
        assert_eq!(
            text_strokes("%%c", Point2::ORIGIN, 1.0, 0.0),
            text_strokes("⌀", Point2::ORIGIN, 1.0, 0.0)
        );
        assert!(!text_strokes("±", Point2::ORIGIN, 1.0, 0.0).is_empty());
        // Sublinhado e sobrelinhado ainda não são desenhados: o código some.
        assert_eq!(
            text_strokes("%%uI", Point2::ORIGIN, 1.0, 0.0),
            text_strokes("I", Point2::ORIGIN, 1.0, 0.0)
        );
        assert_eq!(
            text_strokes("中", Point2::ORIGIN, 1.0, 0.0),
            text_strokes("?", Point2::ORIGIN, 1.0, 0.0)
        );
    }
}
