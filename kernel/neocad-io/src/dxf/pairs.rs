// Caminho relativo: kernel/neocad-io/src/dxf/pairs.rs
//! \file kernel/neocad-io/src/dxf/pairs.rs
//! \brief Leitura do fluxo de pares código/valor de um arquivo DXF.
//! \author Iago Leal
//! \date 2026-08-11

use core::fmt;

/// Como o valor de um par deve ser interpretado.
///
/// O DXF não declara o tipo em lugar nenhum: ele decorre da **faixa** do código
/// de grupo. É por isso que a tipagem precisa acontecer aqui, na leitura, e não
/// espalhada por quem consome — cada consumidor teria de reimplementar a tabela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueKind {
    Text,
    Integer,
    Real,
    Boolean,
    Binary,
}

/// Resolve o tipo do valor a partir do código de grupo.
///
/// A tabela cobre as faixas que aparecem em desenho real. Código fora delas cai
/// em [`ValueKind::Text`], que é o tratamento conservador: preserva o conteúdo
/// como veio, em vez de recusar o arquivo por causa de uma extensão que não
/// conhecemos.
fn value_kind(code: u16) -> ValueKind {
    match code {
        290..=299 => ValueKind::Boolean,
        310..=319 => ValueKind::Binary,
        60..=79 | 170..=179 | 270..=289 | 370..=389 | 400..=409 | 1060..=1070 => ValueKind::Integer,
        90..=99 | 160..=169 | 420..=429 | 440..=459 | 1071 => ValueKind::Integer,
        10..=59 | 110..=149 | 210..=239 | 460..=469 | 1010..=1059 => ValueKind::Real,
        _ => ValueKind::Text,
    }
}

/// Valor de um par, já tipado conforme a faixa do código.
#[derive(Debug, Clone, PartialEq)]
pub enum DxfValue {
    /// Texto — nomes, marcadores de seção, identificadores.
    Text(String),
    /// Inteiro — sinalizadores, contagens, enumerações.
    Integer(i64),
    /// Real — coordenadas, distâncias, ângulos.
    Real(f64),
    /// Booleano.
    Boolean(bool),
    /// Bloco binário, transportado em hexadecimal no arquivo.
    Binary(Vec<u8>),
}

impl DxfValue {
    /// Devolve o texto, se o valor for textual.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Devolve o inteiro, se o valor for inteiro.
    #[must_use]
    pub const fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    /// Devolve o real, se o valor for real.
    #[must_use]
    pub const fn as_real(&self) -> Option<f64> {
        match self {
            Self::Real(value) => Some(*value),
            _ => None,
        }
    }

    /// Devolve o booleano, se o valor for booleano.
    #[must_use]
    pub const fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }
}

/// Par código/valor, a unidade elementar do DXF.
#[derive(Debug, Clone, PartialEq)]
pub struct DxfPair {
    /// Código de grupo.
    pub code: u16,
    /// Valor, tipado conforme a faixa do código.
    pub value: DxfValue,
}

impl DxfPair {
    /// Indica se o par é um comentário (`999`).
    ///
    /// Comentários são entregues em vez de descartados: quem percorre as seções
    /// decide ignorá-los, e assim uma ferramenta que queira preservá-los pode.
    #[must_use]
    pub const fn is_comment(&self) -> bool {
        self.code == 999
    }
}

/// Falha ao ler o fluxo de pares.
///
/// Todas carregam o número da linha, porque um DXF de desenho real tem centenas
/// de milhares delas e "arquivo inválido" não ajuda ninguém a achar o problema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DxfPairError {
    /// O arquivo terminou depois de um código, sem o valor correspondente.
    TruncatedPair {
        /// Linha do código órfão, começando em 1.
        line: usize,
        /// Código lido.
        code: u16,
    },
    /// A linha do código não é um número de grupo válido.
    InvalidCode {
        /// Linha, começando em 1.
        line: usize,
        /// Conteúdo encontrado.
        found: String,
    },
    /// O valor não corresponde ao tipo que a faixa do código exige.
    InvalidValue {
        /// Linha do valor, começando em 1.
        line: usize,
        /// Código do par.
        code: u16,
        /// Conteúdo encontrado.
        found: String,
    },
}

impl fmt::Display for DxfPairError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedPair { line, code } => write!(
                formatter,
                "linha {line}: o arquivo termina após o código {code}, sem valor"
            ),
            Self::InvalidCode { line, found } => write!(
                formatter,
                "linha {line}: {found:?} não é um código de grupo válido"
            ),
            Self::InvalidValue { line, code, found } => write!(
                formatter,
                "linha {line}: {found:?} não é um valor válido para o código {code}"
            ),
        }
    }
}

impl core::error::Error for DxfPairError {}

/// Decodifica uma linha de texto do arquivo.
///
/// DXF de origem real raramente é UTF-8: as ferramentas gravam na página de
/// código do sistema, e no Ocidente isso costuma ser Windows-1252. Decodificar
/// como UTF-8 e cair para Windows-1252 quando falha preserva nomes acentuados de
/// camada — `Fiação`, `Cotas Elétricas` — que de outro modo virariam lixo.
///
/// A página de código declarada em `$DWGCODEPAGE` ainda não é consultada; quando
/// for, esta função é o único lugar a mudar.
fn decode_line(bytes: &[u8]) -> String {
    match core::str::from_utf8(bytes) {
        Ok(text) => text.to_owned(),
        Err(_) => bytes.iter().map(|&b| windows_1252_char(b)).collect(),
    }
}

/// Traduz um byte Windows-1252 para o caractere Unicode correspondente.
///
/// De `0xA0` para cima, Windows-1252 coincide com Latin-1, que por sua vez
/// coincide com os pontos de código Unicode — daí a conversão direta. A faixa
/// `0x80..=0x9F` é o que a distingue, e vai na tabela.
fn windows_1252_char(byte: u8) -> char {
    const ALTOS: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž',
        '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}',
        'ž', 'Ÿ',
    ];

    match byte {
        0x80..=0x9F => ALTOS[(byte - 0x80) as usize],
        _ => char::from(byte),
    }
}

/// Percorre um arquivo DXF entregando um par por vez.
///
/// # Erro encerra a leitura
///
/// Depois do primeiro erro o iterador se esgota. Um fluxo de pares que perdeu o
/// sincronismo não tem como ser retomado com confiança: o leitor não sabe se a
/// próxima linha é código ou valor, e continuar produziria pares inventados —
/// pior do que parar.
#[derive(Debug)]
pub struct DxfPairs<'a> {
    linhas: Linhas<'a>,
    encerrado: bool,
}

/// Linhas numeradas do arquivo, com espiada na seguinte.
///
/// A espiada é o que distingue o valor vazio legítimo do resto que sobra depois
/// da quebra final; ver [`DxfPairs::linha_de_valor`].
type Linhas<'a> = core::iter::Peekable<core::iter::Enumerate<core::slice::Split<'a, u8, EhQuebra>>>;

/// Predicado de quebra de linha, como ponteiro de função para que o tipo do
/// iterador possa ser nomeado.
type EhQuebra = fn(&u8) -> bool;

impl<'a> DxfPairs<'a> {
    fn new(input: &'a [u8]) -> Self {
        fn eh_quebra(byte: &u8) -> bool {
            *byte == b'\n'
        }

        Self {
            linhas: input.split(eh_quebra as EhQuebra).enumerate().peekable(),
            encerrado: false,
        }
    }

    /// Próxima linha não vazia, já decodificada e sem o `\r` do CRLF.
    ///
    /// Linhas em branco no fim do arquivo são ignoradas em vez de virarem par
    /// truncado: quase todo gravador de DXF deixa uma.
    fn proxima_linha(&mut self) -> Option<(usize, String)> {
        for (indice, bytes) in self.linhas.by_ref() {
            let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);
            let texto = decode_line(bytes);

            if !texto.trim().is_empty() {
                return Some((indice + 1, texto));
            }
        }

        None
    }

    /// Linha do valor, preservada na íntegra.
    ///
    /// O único cuidado é distinguir o valor vazio legítimo — código `1` com texto
    /// em branco existe — do resto vazio que sobra depois da quebra final do
    /// arquivo. Como quase todo gravador termina o arquivo com `\n`, a divisão
    /// deixa um pedaço vazio no fim; se o candidato a valor é justamente esse
    /// pedaço, não há valor, e o par está truncado.
    fn linha_de_valor(&mut self) -> Option<(usize, String)> {
        let (indice, bytes) = self.linhas.next()?;
        let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);

        if bytes.is_empty() && self.linhas.peek().is_none() {
            return None;
        }

        Some((indice + 1, decode_line(bytes)))
    }
}

impl Iterator for DxfPairs<'_> {
    type Item = Result<DxfPair, DxfPairError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.encerrado {
            return None;
        }

        let (linha_codigo, bruto_codigo) = self.proxima_linha()?;

        let Ok(code) = bruto_codigo.trim().parse::<u16>() else {
            self.encerrado = true;
            return Some(Err(DxfPairError::InvalidCode {
                line: linha_codigo,
                found: bruto_codigo.trim().to_owned(),
            }));
        };

        // O valor é a linha seguinte **na íntegra**: texto de DXF pode conter
        // espaços significativos, então aqui não se apara nada.
        let Some((linha_valor, bruto_valor)) = self.linha_de_valor() else {
            self.encerrado = true;
            return Some(Err(DxfPairError::TruncatedPair {
                line: linha_codigo,
                code,
            }));
        };

        match interpretar(code, &bruto_valor) {
            Some(value) => Some(Ok(DxfPair { code, value })),
            None => {
                self.encerrado = true;
                Some(Err(DxfPairError::InvalidValue {
                    line: linha_valor,
                    code,
                    found: bruto_valor,
                }))
            }
        }
    }
}

/// Converte o texto cru no valor que a faixa do código exige.
fn interpretar(code: u16, bruto: &str) -> Option<DxfValue> {
    let aparado = bruto.trim();

    match value_kind(code) {
        ValueKind::Text => Some(DxfValue::Text(bruto.to_owned())),
        ValueKind::Integer => aparado.parse::<i64>().ok().map(DxfValue::Integer),
        ValueKind::Real => aparado.parse::<f64>().ok().map(DxfValue::Real),
        ValueKind::Boolean => match aparado {
            "0" => Some(DxfValue::Boolean(false)),
            "1" => Some(DxfValue::Boolean(true)),
            _ => None,
        },
        ValueKind::Binary => decodificar_hex(aparado).map(DxfValue::Binary),
    }
}

/// Decodifica um bloco binário, que o DXF transporta em hexadecimal.
fn decodificar_hex(texto: &str) -> Option<Vec<u8>> {
    if texto.len() % 2 != 0 {
        return None;
    }

    let bytes = texto.as_bytes();

    (0..bytes.len() / 2)
        .map(|indice| {
            let par = core::str::from_utf8(&bytes[indice * 2..indice * 2 + 2]).ok()?;
            u8::from_str_radix(par, 16).ok()
        })
        .collect()
}

/// Percorre os pares código/valor de um arquivo DXF.
///
/// # Exemplo
///
/// ```
/// use neocad_io::{pairs, DxfValue};
///
/// let arquivo = b"  0\nSECTION\n  2\nHEADER\n";
/// let lidos: Result<Vec<_>, _> = pairs(arquivo).collect();
/// let lidos = lidos?;
///
/// assert_eq!(lidos.len(), 2);
/// assert_eq!(lidos[0].code, 0);
/// assert_eq!(lidos[0].value, DxfValue::Text(String::from("SECTION")));
/// # Ok::<(), neocad_io::DxfPairError>(())
/// ```
#[must_use]
pub fn pairs(input: &[u8]) -> DxfPairs<'_> {
    DxfPairs::new(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ler(entrada: &[u8]) -> Result<Vec<DxfPair>, DxfPairError> {
        pairs(entrada).collect()
    }

    fn texto(valor: &str) -> DxfValue {
        DxfValue::Text(String::from(valor))
    }

    #[test]
    fn le_par_de_texto() {
        let lidos = ler(b"  0\nSECTION\n").expect("entrada válida");

        assert_eq!(
            lidos,
            vec![DxfPair {
                code: 0,
                value: texto("SECTION")
            }]
        );
    }

    #[test]
    fn tipa_o_valor_pela_faixa_do_codigo() {
        let lidos =
            ler(b"  8\nParede\n 10\n12.5\n 70\n-3\n 90\n70000\n290\n1\n").expect("entrada válida");

        assert_eq!(lidos[0].value, texto("Parede"));
        assert_eq!(lidos[1].value, DxfValue::Real(12.5));
        assert_eq!(lidos[2].value, DxfValue::Integer(-3));
        assert_eq!(
            lidos[3].value,
            DxfValue::Integer(70_000),
            "código 90 é inteiro de 32 bits e não cabe em 16"
        );
        assert_eq!(lidos[4].value, DxfValue::Boolean(true));
    }

    #[test]
    fn le_bloco_binario_em_hexadecimal() {
        let lidos = ler(b"310\n48656C6C6F\n").expect("entrada válida");

        assert_eq!(lidos[0].value, DxfValue::Binary(b"Hello".to_vec()));
    }

    #[test]
    fn hexadecimal_impar_ou_invalido_e_recusado() {
        assert!(matches!(
            ler(b"310\nABC\n"),
            Err(DxfPairError::InvalidValue { code: 310, .. })
        ));
        assert!(matches!(
            ler(b"310\nZZ\n"),
            Err(DxfPairError::InvalidValue { .. })
        ));
    }

    #[test]
    fn aceita_crlf_e_lf_indistintamente() {
        let com_lf = ler(b"  0\nSECTION\n  2\nHEADER\n").expect("LF");
        let com_crlf = ler(b"  0\r\nSECTION\r\n  2\r\nHEADER\r\n").expect("CRLF");

        assert_eq!(com_lf, com_crlf);
    }

    #[test]
    fn tolera_espacos_a_esquerda_do_codigo() {
        let alinhado = ler(b"      0\nSECTION\n").expect("com espaços");
        let cru = ler(b"0\nSECTION\n").expect("sem espaços");

        assert_eq!(alinhado, cru);
    }

    #[test]
    fn comentario_999_e_entregue_e_reconhecivel() {
        let lidos = ler(b"999\nGerado por alguma ferramenta\n  0\nSECTION\n").expect("válida");

        assert!(lidos[0].is_comment());
        assert_eq!(lidos[0].value, texto("Gerado por alguma ferramenta"));
        assert!(!lidos[1].is_comment());
    }

    #[test]
    fn valor_textual_preserva_espacos_internos_e_de_borda() {
        // Nome de camada com espaço à direita é legal em DXF, e aparar mudaria o
        // nome — que é a identidade da camada no arquivo.
        let lidos = ler(b"  8\n Parede Externa \n").expect("válida");

        assert_eq!(lidos[0].value, texto(" Parede Externa "));
    }

    #[test]
    fn arquivo_truncado_vira_erro_nomeado_e_nao_panico() {
        let erro = ler(b"  0\nSECTION\n  2\n").expect_err("falta o valor do código 2");

        assert_eq!(erro, DxfPairError::TruncatedPair { line: 3, code: 2 });
        assert!(erro.to_string().contains("linha 3"));
    }

    #[test]
    fn codigo_nao_numerico_e_recusado_com_a_linha() {
        let erro = ler(b"  0\nSECTION\nabc\nqualquer\n").expect_err("código inválido");

        assert_eq!(
            erro,
            DxfPairError::InvalidCode {
                line: 3,
                found: String::from("abc")
            }
        );
    }

    #[test]
    fn valor_incompativel_com_a_faixa_e_recusado() {
        let erro = ler(" 10\nnão é número\n".as_bytes()).expect_err("código 10 é real");

        assert!(matches!(
            erro,
            DxfPairError::InvalidValue {
                line: 2,
                code: 10,
                ..
            }
        ));
    }

    #[test]
    fn a_leitura_encerra_no_primeiro_erro() {
        // Depois de perder o sincronismo, o leitor não sabe se a linha seguinte é
        // código ou valor; continuar produziria pares inventados.
        let lidos: Vec<_> = pairs(b"abc\nx\n  0\nSECTION\n").collect();

        assert_eq!(lidos.len(), 1);
        assert!(lidos[0].is_err());
    }

    #[test]
    fn linhas_em_branco_ao_final_nao_viram_par_truncado() {
        let lidos = ler(b"  0\nSECTION\n\n\n").expect("quase todo gravador deixa uma");

        assert_eq!(lidos.len(), 1);
    }

    #[test]
    fn valor_de_texto_vazio_e_valor_e_nao_truncamento() {
        // Texto em branco existe em arquivo real — um `TEXT` sem conteúdo, por
        // exemplo. O que o distingue do par truncado é haver quebra depois dele.
        let lidos = ler(b"  1\n\n").expect("valor vazio é valor");

        assert_eq!(lidos.len(), 1);
        assert_eq!(lidos[0].value, texto(""));
    }

    #[test]
    fn texto_windows_1252_preserva_acentuacao() {
        // "Fiação" em Windows-1252: 0xE7 é ç e 0xE3 é ã. Não é UTF-8 válido, e
        // decodificar de forma tolerante evita transformar o nome em lixo.
        let entrada = b"  8\nFia\xE7\xE3o\n";
        let lidos = ler(entrada).expect("válida");

        assert_eq!(lidos[0].value, texto("Fiação"));
    }

    #[test]
    fn texto_utf8_continua_sendo_lido_como_utf8() {
        let lidos = ler("  8\nFiação\n".as_bytes()).expect("válida");

        assert_eq!(lidos[0].value, texto("Fiação"));
    }

    #[test]
    fn codigo_fora_das_faixas_conhecidas_vira_texto() {
        // Tratamento conservador: preserva o conteúdo em vez de recusar o arquivo
        // por causa de uma extensão que não conhecemos.
        let lidos = ler(b"1234\nqualquer coisa\n").expect("válida");

        assert_eq!(lidos[0].value, texto("qualquer coisa"));
    }

    #[test]
    fn acessores_devolvem_none_para_o_tipo_errado() {
        let real = DxfValue::Real(1.0);

        assert_eq!(real.as_real(), Some(1.0));
        assert_eq!(real.as_integer(), None);
        assert_eq!(real.as_text(), None);
        assert_eq!(real.as_boolean(), None);
    }

    #[test]
    fn entrada_vazia_produz_nenhum_par() {
        assert_eq!(ler(b"").expect("vazio é válido"), vec![]);
    }
}
