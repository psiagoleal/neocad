// Caminho relativo: kernel/neocad-io/src/dxf/writer/mod.rs
//! \file kernel/neocad-io/src/dxf/writer/mod.rs
//! \brief Escrita de arquivos DXF a partir do modelo.
//! \author Iago Leal
//! \date 2026-08-15
//!
//! A escrita é **determinística**: o mesmo conteúdo produz os mesmos bytes, em
//! qualquer máquina e em qualquer execução (ADR 0004). Sem isso não há diff
//! legível entre versões de um desenho, e o controle de versão de um projeto de
//! engenharia vira ruído.
//!
//! O que garante o determinismo, em três frentes:
//!
//! - **Ordem.** Nada é percorrido por estrutura de ordem indefinida. A tabela de
//!   camadas já itera em ordem alfabética normalizada, por decisão do MT-K1-04.
//! - **Números.** Reais são formatados por [`formatar_real`], que não depende de
//!   locale nem de notação científica.
//! - **Identificadores.** Os handles saem de um contador monotônico, e não de
//!   endereço de memória ou de ordem de alocação.

mod entities;
mod tables;

use neocad_geometry::Point2;
use neocad_model::LayerTable;

use super::blocks::BlockDefinition;
use super::entities::ReadEntity;
use entities::{model_space_extents, write_blocks, write_entities};
use tables::write_tables;

/// O conteúdo de um desenho, na forma que a escrita consome.
///
/// Reúne o que a leitura produz — camadas, entidades com o seu espaço e
/// definições de bloco — sem exigir um [`neocad_model::Document`], que ainda não
/// pode ser montado enquanto os blocos de espaço-papel não existirem (fase KL).
/// É também o que torna a ida e volta do MT-K2-09 exprimível.
#[derive(Debug, Clone, Copy)]
pub struct DxfContents<'a> {
    /// Tabela de camadas do desenho.
    pub layers: &'a LayerTable,
    /// Entidades dos espaços, na ordem de desenho.
    pub entities: &'a [ReadEntity],
    /// Definições de bloco.
    pub blocks: &'a [BlockDefinition],
}

impl<'a> DxfContents<'a> {
    /// Conteúdo com apenas as camadas — um desenho sem traço nenhum.
    #[must_use]
    pub const fn from_layers(layers: &'a LayerTable) -> Self {
        Self {
            layers,
            entities: &[],
            blocks: &[],
        }
    }
}

/// Versão do formato que a escrita produz.
///
/// # Por que 2007, e não 2000
///
/// O piso funcional seria `AC1015` (AutoCAD 2000): é de lá que vêm polilinha
/// leve, cor verdadeira e **layouts**, sem os quais o ADR 0005 não se cumpre.
///
/// Quem decide é a **codificação**. Até o `AC1018` o arquivo não é Unicode: o
/// texto vale segundo a página de código declarada em `$DWGCODEPAGE`, e um nome
/// como `Fiação` gravado em UTF-8 seria lido errado por um leitor que segue a
/// especificação. A partir do `AC1021` o DXF **é** UTF-8, e o problema
/// desaparece em vez de ser contornado.
///
/// Em desenho de engenharia brasileiro nome acentuado é regra, não exceção —
/// `Fiação`, `Cotas Elétricas`. Escolher a versão em que ele simplesmente
/// funciona custa compatibilidade com leitores anteriores a 2007, e é troca boa.
pub const ACAD_VERSION: &str = "AC1021";

/// Terminador de linha do arquivo gerado.
///
/// O DXF do AutoCAD usa `CRLF`, e produzi-lo evita diferença gratuita ao abrir e
/// regravar um arquivo de origem real. A leitura aceita os dois.
const FIM_DE_LINHA: &str = "\r\n";

/// Distribui handles em sequência.
///
/// A partir do AC1015 todo objeto tem handle, e ele precisa ser único no
/// arquivo. Um contador monotônico dá unicidade **e** determinismo — derivar o
/// handle de endereço ou de ordem de alocação daria arquivos diferentes a cada
/// execução, que é justamente o que o ADR 0004 proíbe.
#[derive(Debug)]
pub(super) struct Handles {
    proximo: u64,
}

impl Handles {
    /// Começa em `0x10`, faixa acima da que o próprio formato reserva.
    pub(super) const fn new() -> Self {
        Self { proximo: 0x10 }
    }

    /// Devolve o próximo handle, em hexadecimal maiúsculo como o formato exige.
    pub(super) fn proximo(&mut self) -> String {
        let handle = self.proximo;
        self.proximo += 1;

        format!("{handle:X}")
    }

    /// Handle seguinte ao último entregue, para o `$HANDSEED` do cabeçalho.
    pub(super) fn semente(&self) -> String {
        format!("{:X}", self.proximo)
    }
}

/// Acumula os pares do arquivo em construção.
#[derive(Debug, Default)]
pub(super) struct Saida {
    texto: String,
}

impl Saida {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Escreve um par código/valor.
    ///
    /// O código sai alinhado à direita em três colunas, como o AutoCAD grava.
    /// A leitura apara espaços, mas um arquivo que *parece* com os outros é mais
    /// fácil de comparar com um gravado por outra ferramenta.
    pub(super) fn par(&mut self, code: u16, value: &str) {
        self.texto
            .push_str(&format!("{code:>3}{FIM_DE_LINHA}{value}{FIM_DE_LINHA}"));
    }

    /// Escreve um par cujo valor é inteiro.
    pub(super) fn inteiro(&mut self, code: u16, value: i64) {
        self.par(code, &value.to_string());
    }

    /// Escreve um par cujo valor é real.
    pub(super) fn real(&mut self, code: u16, value: f64) {
        self.par(code, &formatar_real(value));
    }

    /// Texto acumulado.
    pub(super) fn texto(&self) -> &str {
        &self.texto
    }

    /// Consome a saída, devolvendo os bytes.
    pub(super) fn into_bytes(self) -> Vec<u8> {
        self.texto.into_bytes()
    }
}

/// Formata um real de modo determinístico e sem perda.
///
/// Três exigências ao mesmo tempo, e nenhuma dispensável:
///
/// 1. **Sem perda** — o valor relido é bit a bit o mesmo. Coordenada arredondada
///    na gravação é desenho alterado sem ninguém pedir.
/// 2. **Sem notação científica** — `1e-9` é aceito por leitores tolerantes e
///    recusado por outros; escrever a forma decimal evita a aposta.
/// 3. **Sem locale** — a formatação do Rust não usa vírgula decimal, então isso
///    já vem de graça, mas é requisito e não acaso.
///
/// A forma curta que o Rust produz com `{:?}` já round-trips; ela só é
/// substituída quando traz expoente, e aí a busca pela menor precisão decimal
/// que ainda round-trips assume.
#[must_use]
pub fn formatar_real(value: f64) -> String {
    // Valores não finitos não têm representação no formato. Zero é a escolha
    // que não propaga o problema para dentro do arquivo.
    if !value.is_finite() {
        return String::from("0.0");
    }

    // `-0.0` e `0.0` são iguais na comparação e diferentes na formatação;
    // normalizar evita dois arquivos diferentes para o mesmo desenho.
    let value = if value == 0.0 { 0.0 } else { value };
    let curta = format!("{value:?}");

    if !curta.contains(['e', 'E']) {
        return curta;
    }

    // O limite alcança o menor subnormal: `f64` precisa de até 1074 casas
    // decimais para ser escrito exatamente. Parece exagero, e é — mas parar em
    // 17 fazia `f64::MIN_POSITIVE` virar `0.0`, que é a perda silenciosa que
    // este projeto passou uma semana caçando no conversor alheio. Valor
    // patológico paga o laço; coordenada de desenho sai na primeira iteração.
    for precisao in 1..=1074 {
        let candidata = format!("{value:.precisao$}");

        if candidata.parse::<f64>() == Ok(value) {
            return candidata;
        }
    }

    // Inalcançável na prática. Se acontecer, a forma com expoente é exata, e um
    // valor correto que algum leitor recusa é melhor que um valor errado que
    // todos aceitam.
    curta
}

/// Escreve um DXF completo: cabeçalho, tabelas, blocos e entidades.
///
/// # Exemplo
///
/// ```
/// use neocad_io::{read_dxf, write_dxf, DxfContents};
/// use neocad_model::LayerTable;
///
/// let mut camadas = LayerTable::new();
/// camadas.create("Eixos")?;
///
/// let conteudo = DxfContents::from_layers(&camadas);
/// let bytes = write_dxf(&conteudo);
///
/// // O que escrevemos, lemos de volta.
/// let relido = read_dxf(&bytes);
/// assert!(relido.layers.get_by_name("Eixos").is_some());
///
/// // E duas execuções produzem os mesmos bytes.
/// assert_eq!(bytes, write_dxf(&conteudo));
/// # Ok::<(), neocad_model::LayerError>(())
/// ```
#[must_use]
pub fn write_dxf(contents: &DxfContents<'_>) -> Vec<u8> {
    let mut handles = Handles::new();

    // O corpo vem antes no tempo, ainda que depois no arquivo: só depois de
    // distribuir todos os handles é que se sabe qual `$HANDSEED` declarar, e a
    // extensão do desenho depende das entidades já conhecidas.
    let mut corpo = Saida::new();
    write_tables(&mut corpo, contents, &mut handles);
    write_blocks(&mut corpo, contents, &mut handles);
    write_entities(&mut corpo, contents, &mut handles);

    let mut saida = Saida::new();
    write_header(&mut saida, contents, &handles);
    saida.texto.push_str(corpo.texto());
    saida.par(0, "EOF");

    saida.into_bytes()
}

/// Escreve o cabeçalho mínimo.
///
/// "Mínimo" aqui é o que basta para um leitor identificar a versão e posicionar
/// o desenho. As extensões (`$EXTMIN`/`$EXTMAX`) dependem das entidades e entram
/// no MT-K2-08, junto com quem as produz.
fn write_header(saida: &mut Saida, contents: &DxfContents<'_>, handles: &Handles) {
    saida.par(0, "SECTION");
    saida.par(2, "HEADER");

    saida.par(9, "$ACADVER");
    saida.par(1, ACAD_VERSION);

    saida.par(9, "$HANDSEED");
    saida.par(5, &handles.semente());

    saida.par(9, "$INSBASE");
    escrever_ponto(saida, Point2::ORIGIN);

    // Extensão só existe se houver desenho. Declarar a de um arquivo vazio
    // seria inventar um retângulo que ninguém desenhou.
    if let Some(extensao) = model_space_extents(contents) {
        saida.par(9, "$EXTMIN");
        escrever_ponto(saida, extensao.min());
        saida.par(9, "$EXTMAX");
        escrever_ponto(saida, extensao.max());
    }

    saida.par(0, "ENDSEC");
}

/// Escreve um ponto nos códigos `10`/`20`/`30`, como o cabeçalho os espera.
fn escrever_ponto(saida: &mut Saida, ponto: Point2) {
    saida.real(10, ponto.x);
    saida.real(20, ponto.y);
    saida.real(30, 0.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{read_dxf, SectionKind};

    #[test]
    fn saida_e_identica_entre_execucoes() {
        // O critério de aceite do MT-K2-07.
        let mut camadas = LayerTable::new();
        camadas.create("Eixos").expect("nome válido");
        camadas.create("Cotas").expect("nome válido");

        let conteudo = DxfContents::from_layers(&camadas);

        assert_eq!(write_dxf(&conteudo), write_dxf(&conteudo));
    }

    #[test]
    fn saida_nao_depende_da_ordem_de_criacao() {
        // É o risco real do determinismo: duas tabelas com o mesmo conteúdo,
        // montadas em ordens diferentes, precisam gravar igual. Só vale porque
        // a iteração da tabela é alfabética por decisão do MT-K1-04.
        let mut uma = LayerTable::new();
        uma.create("Eixos").expect("nome válido");
        uma.create("Cotas").expect("nome válido");

        let mut outra = LayerTable::new();
        outra.create("Cotas").expect("nome válido");
        outra.create("Eixos").expect("nome válido");

        assert_eq!(
            write_dxf(&DxfContents::from_layers(&uma)),
            write_dxf(&DxfContents::from_layers(&outra))
        );
    }

    #[test]
    fn o_que_escrevemos_conseguimos_ler() {
        let mut camadas = LayerTable::new();
        camadas.create("Fiação").expect("nome válido");

        let leitura = read_dxf(&write_dxf(&DxfContents::from_layers(&camadas)));

        assert!(leitura.layers.get_by_name("Fiação").is_some());
        assert!(leitura.report.is_clean());
        assert!(leitura.report.section_errors.is_empty());
    }

    #[test]
    fn o_arquivo_tem_as_secoes_esperadas() {
        let camadas = LayerTable::new();
        let bytes = write_dxf(&DxfContents::from_layers(&camadas));
        let secoes: Vec<SectionKind> = crate::sections(&bytes)
            .map(|s| s.expect("bem formada").kind)
            .collect();

        assert_eq!(
            secoes,
            [
                SectionKind::Header,
                SectionKind::Tables,
                SectionKind::Blocks,
                SectionKind::Entities
            ]
        );
    }

    #[test]
    fn as_linhas_terminam_em_crlf() {
        let camadas = LayerTable::new();
        let bytes = write_dxf(&DxfContents::from_layers(&camadas));
        let texto = String::from_utf8(bytes).expect("saída é UTF-8");

        assert!(texto.starts_with("  0\r\nSECTION\r\n"));
        assert!(texto.ends_with("  0\r\nEOF\r\n"));
        // Nenhum `\n` solto: todos são precedidos de `\r`.
        assert_eq!(texto.matches('\n').count(), texto.matches("\r\n").count());
    }

    #[test]
    fn a_versao_declarada_suporta_layout() {
        let camadas = LayerTable::new();
        let bytes = write_dxf(&DxfContents::from_layers(&camadas));
        let texto = String::from_utf8(bytes).expect("saída é UTF-8");

        assert!(texto.contains("$ACADVER"));
        assert!(texto.contains(ACAD_VERSION));
    }

    #[test]
    fn nome_acentuado_sai_em_utf8_coerente_com_a_versao() {
        // A versão declarada é a que define a codificação. Gravar UTF-8 sob uma
        // versão pré-2007 faria `Fiação` chegar torto a um leitor correto.
        let mut camadas = LayerTable::new();
        camadas.create("Fiação").expect("nome válido");

        let bytes = write_dxf(&DxfContents::from_layers(&camadas));

        assert!(ACAD_VERSION >= "AC1021", "versão anterior não é Unicode");
        assert!(String::from_utf8(bytes.clone())
            .expect("saída é UTF-8")
            .contains("Fiação"));
        // E os bytes são mesmo os do UTF-8 (`ç` = C3 A7), não os de uma página
        // de código, onde `ç` seria o byte único E7.
        let ce_cedilha = "ç".as_bytes();
        assert_eq!(ce_cedilha, [0xC3, 0xA7]);
        assert!(bytes.windows(2).any(|par| par == ce_cedilha));
    }

    #[test]
    fn handles_sao_unicos_e_a_semente_os_supera() {
        let mut handles = Handles::new();
        let primeiro = handles.proximo();
        let segundo = handles.proximo();

        assert_ne!(primeiro, segundo);
        assert_eq!(primeiro, "10");
        assert_eq!(segundo, "11");
        assert_eq!(handles.semente(), "12");
    }

    #[test]
    fn real_inteiro_sai_com_casa_decimal() {
        // `1` seria lido como inteiro por quem se guia pelo texto; o formato
        // espera decimal nos códigos de coordenada.
        assert_eq!(formatar_real(1.0), "1.0");
        assert_eq!(formatar_real(-2.0), "-2.0");
    }

    #[test]
    fn real_sobrevive_a_ida_e_volta() {
        let valores = [
            0.1,
            1.0 / 3.0,
            123_456.789,
            f64::MIN_POSITIVE,
            f64::MAX,
            -0.000_000_001,
            core::f64::consts::PI,
        ];

        for valor in valores {
            let texto = formatar_real(valor);

            assert_eq!(
                texto.parse::<f64>(),
                Ok(valor),
                "{valor} não sobreviveu como {texto:?}"
            );
        }
    }

    #[test]
    fn real_nunca_sai_em_notacao_cientifica() {
        // Leitor tolerante aceita `1e-9`; outros recusam. Não vale a aposta.
        for valor in [1e-9, 1e21, f64::MIN_POSITIVE, -1e-30] {
            let texto = formatar_real(valor);

            assert!(
                !texto.contains(['e', 'E']),
                "{valor} saiu como {texto:?}, com expoente"
            );
        }
    }

    #[test]
    fn zero_negativo_e_normalizado() {
        // Sem isso, dois desenhos iguais gravariam bytes diferentes.
        assert_eq!(formatar_real(-0.0), formatar_real(0.0));
        assert_eq!(formatar_real(-0.0), "0.0");
    }

    #[test]
    fn valor_nao_finito_nao_contamina_o_arquivo() {
        assert_eq!(formatar_real(f64::NAN), "0.0");
        assert_eq!(formatar_real(f64::INFINITY), "0.0");
        assert_eq!(formatar_real(f64::NEG_INFINITY), "0.0");
    }
}
