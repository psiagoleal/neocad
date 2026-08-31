// Caminho relativo: kernel/neocad-io/src/dxf/writer/tables.rs
//! \file kernel/neocad-io/src/dxf/writer/tables.rs
//! \brief Escrita das tabelas de símbolos de um arquivo DXF.
//! \author Iago Leal
//! \date 2026-08-15

use neocad_model::{Color, LayerRecord, LayerTable, LineWeight};

use super::entities::block_record_names;
use super::{DxfContents, Handles, Saida};

/// Tipo de linha padrão, que toda camada referencia enquanto não há tabela de
/// tipos de linha no modelo.
const CONTINUOUS: &str = "Continuous";

/// Estilo de texto padrão, protegido no modelo desde o MT-K1-06.
const STANDARD: &str = "Standard";

/// Espessura herdada do padrão do documento, na convenção do formato.
const LINE_WEIGHT_DEFAULT: i64 = -3;

/// Escreve a seção `TABLES`.
///
/// # Por que sai mais do que a tabela de camadas
///
/// Toda camada referencia um tipo de linha pelo nome, e todo texto referencia um
/// estilo. Gravar só a `LAYER` deixaria essas referências **penduradas**, e o
/// arquivo abriria com aviso ou com substituição silenciosa. As tabelas `LTYPE` e
/// `STYLE` saem com uma entrada cada — `Continuous` e `Standard` —, que é o
/// mínimo para o arquivo se sustentar sozinho.
///
/// Quando o modelo ganhar tabela própria de tipos de linha, esta função passa a
/// escrevê-la por inteiro; a `LTYPE` mínima daqui é o piso, não o teto.
pub(super) fn write_tables(saida: &mut Saida, contents: &DxfContents<'_>, handles: &mut Handles) {
    saida.par(0, "SECTION");
    saida.par(2, "TABLES");

    write_linetype_table(saida, handles);
    write_layer_table(saida, contents.layers, handles);
    write_text_style_table(saida, handles);
    write_block_record_table(saida, contents, handles);

    saida.par(0, "ENDSEC");
}

/// Escreve a tabela de registros de bloco.
///
/// # Por que ela precisa existir
///
/// Toda entidade pertence a um registro de bloco, e é essa tabela que declara
/// quais existem — inclusive `*Model_Space` e `*Paper_Space`, que o formato exige
/// mesmo num desenho sem bloco nenhum. Omiti-la deixa as entidades sem dono
/// declarado, e é o tipo de ausência que um leitor estrito recusa.
///
/// É também a estrutura sobre a qual o ADR 0005 modela layout: cada aba é um
/// registro daqui.
fn write_block_record_table(saida: &mut Saida, contents: &DxfContents<'_>, handles: &mut Handles) {
    let nomes = block_record_names(contents);

    abrir_tabela(saida, "BLOCK_RECORD", nomes.len(), handles);

    for nome in nomes {
        let handle = abrir_registro(saida, "BLOCK_RECORD", "AcDbBlockTableRecord", handles);
        handles.registrar_bloco(nome, &handle);
        saida.par(2, nome);
        saida.inteiro(70, 0);
    }

    saida.par(0, "ENDTAB");
}

/// Abre uma tabela, com seu handle e a contagem de entradas.
fn abrir_tabela(saida: &mut Saida, nome: &str, entradas: usize, handles: &mut Handles) {
    saida.par(0, "TABLE");
    saida.par(2, nome);
    saida.par(5, &handles.proximo());
    saida.par(100, "AcDbSymbolTable");
    // A contagem é declarada como referência, não como contrato: o formato
    // admite que ela divirja, e leitor nenhum deve confiar nela.
    saida.inteiro(70, i64::try_from(entradas).unwrap_or(i64::MAX));
}

/// Abre um registro de tabela, com handle e os marcadores de subclasse.
fn abrir_registro(saida: &mut Saida, tipo: &str, subclasse: &str, handles: &mut Handles) -> String {
    let handle = handles.proximo();

    saida.par(0, tipo);
    saida.par(5, &handle);
    saida.par(100, "AcDbSymbolTableRecord");
    saida.par(100, subclasse);

    handle
}

/// Escreve a tabela de tipos de linha, com a entrada `Continuous`.
fn write_linetype_table(saida: &mut Saida, handles: &mut Handles) {
    abrir_tabela(saida, "LTYPE", 1, handles);

    abrir_registro(saida, "LTYPE", "AcDbLinetypeTableRecord", handles);
    saida.par(2, CONTINUOUS);
    saida.inteiro(70, 0);
    saida.par(3, "Solid line");
    saida.inteiro(72, 65);
    saida.inteiro(73, 0);
    saida.real(40, 0.0);

    saida.par(0, "ENDTAB");
}

/// Escreve a tabela de camadas.
pub(super) fn write_layer_table(saida: &mut Saida, layers: &LayerTable, handles: &mut Handles) {
    abrir_tabela(saida, "LAYER", layers.iter().count(), handles);

    // A ordem vem da tabela, que itera alfabeticamente pelo nome normalizado
    // (MT-K1-04). É daí que sai o determinismo desta seção.
    for (_, camada) in layers.iter() {
        write_layer(saida, camada, handles);
    }

    saida.par(0, "ENDTAB");
}

/// Escreve um registro de camada.
fn write_layer(saida: &mut Saida, camada: &LayerRecord, handles: &mut Handles) {
    let handle = abrir_registro(saida, "LAYER", "AcDbLayerTableRecord", handles);
    handles.registrar_camada(camada.name(), &handle);
    saida.par(2, camada.name());
    saida.inteiro(70, flags(camada));

    escrever_cor(saida, camada);

    saida.par(6, linetype(camada));
    saida.inteiro(370, line_weight(camada));
}

/// Sinalizadores do código `70`.
fn flags(camada: &LayerRecord) -> i64 {
    let mut flags = 0;

    if camada.is_frozen() {
        flags |= 1;
    }

    if camada.is_locked() {
        flags |= 4;
    }

    flags
}

/// Escreve a cor da camada.
///
/// # O sinal do `62` carrega o estado, não a cor
///
/// Camada desligada é gravada com o índice **negativo**, que é a convenção do
/// formato — a mesma que a leitura interpreta. Escrever o índice positivo e
/// perder o estado faria uma camada apagada reaparecer ao reabrir o arquivo.
///
/// # A cor verdadeira sai com um índice de companhia
///
/// Quem lê o `420` obtém a cor exata. Quem não lê ficaria sem cor nenhuma se o
/// `62` fosse omitido, então ele sai com o valor padrão. O índice **mais
/// próximo** da cor verdadeira seria melhor, mas exige a tabela da paleta ACI,
/// que o modelo ainda não tem; até lá, o valor exato está no arquivo e a
/// aproximação é que falta.
fn escrever_cor(saida: &mut Saida, camada: &LayerRecord) {
    const PADRAO: i64 = 7;

    let (indice, verdadeira) = match camada.color() {
        Color::ByBlock => (0, None),
        Color::ByLayer => (256, None),
        Color::Index(indice) => (i64::from(indice), None),
        Color::Rgb { red, green, blue } => (
            PADRAO,
            Some((i64::from(red) << 16) | (i64::from(green) << 8) | i64::from(blue)),
        ),
    };

    saida.inteiro(62, if camada.is_off() { -indice } else { indice });

    if let Some(empacotada) = verdadeira {
        saida.inteiro(420, empacotada);
    }
}

/// Nome do tipo de linha, com o padrão quando a camada não tem um.
fn linetype(camada: &LayerRecord) -> &str {
    let nome = camada.linetype().trim();

    if nome.is_empty() {
        CONTINUOUS
    } else {
        nome
    }
}

/// Espessura na convenção do código `370`.
fn line_weight(camada: &LayerRecord) -> i64 {
    match camada.line_weight() {
        LineWeight::Default => LINE_WEIGHT_DEFAULT,
        LineWeight::Hundredths(centesimos) => i64::from(centesimos),
    }
}

/// Escreve a tabela de estilos de texto, com a entrada `Standard`.
fn write_text_style_table(saida: &mut Saida, handles: &mut Handles) {
    abrir_tabela(saida, "STYLE", 1, handles);

    abrir_registro(saida, "STYLE", "AcDbTextStyleTableRecord", handles);
    saida.par(2, STANDARD);
    saida.inteiro(70, 0);
    saida.real(40, 0.0);
    saida.real(41, 1.0);
    saida.real(50, 0.0);
    saida.inteiro(71, 0);
    saida.real(42, 2.5);
    saida.par(3, "txt");
    saida.par(4, "");

    saida.par(0, "ENDTAB");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{read_dxf, write_dxf, DxfContents};

    /// Grava uma tabela de camadas e a lê de volta.
    fn ida_e_volta(layers: &LayerTable) -> LayerTable {
        read_dxf(&write_dxf(&DxfContents::from_layers(layers))).layers
    }

    fn com_camada(nome: &str, ajuste: impl FnOnce(&mut LayerRecord)) -> LayerTable {
        let mut camadas = LayerTable::new();
        let id = camadas.create(nome).expect("nome válido");
        ajuste(camadas.get_mut(id).expect("recém-criada"));

        camadas
    }

    #[test]
    fn nome_e_cor_por_indice_sobrevivem() {
        let camadas = com_camada("Eixos", |camada| camada.set_color(Color::Index(5)));

        let relidas = ida_e_volta(&camadas);

        assert_eq!(
            relidas.get_by_name("Eixos").expect("relida").color(),
            Color::Index(5)
        );
    }

    #[test]
    fn cor_verdadeira_sobrevive() {
        let camadas = com_camada("Azulada", |camada| {
            camada.set_color(Color::Rgb {
                red: 0x33,
                green: 0x66,
                blue: 0x99,
            });
        });

        assert_eq!(
            relida(&camadas, "Azulada").color(),
            Color::Rgb {
                red: 0x33,
                green: 0x66,
                blue: 0x99
            }
        );
    }

    fn relida(camadas: &LayerTable, nome: &str) -> LayerRecord {
        ida_e_volta(camadas)
            .get_by_name(nome)
            .expect("relida")
            .clone()
    }

    #[test]
    fn extremos_da_paleta_sobrevivem() {
        for cor in [Color::ByBlock, Color::ByLayer] {
            let camadas = com_camada("Herdada", |camada| camada.set_color(cor));

            assert_eq!(relida(&camadas, "Herdada").color(), cor);
        }
    }

    #[test]
    fn camada_desligada_continua_desligada() {
        // O sinal do 62 carrega o estado. Gravar o índice positivo faria a
        // camada apagada reaparecer ao reabrir.
        let camadas = com_camada("Apagada", |camada| {
            camada.set_color(Color::Index(5));
            camada.set_off(true);
        });

        let camada = relida(&camadas, "Apagada");
        assert!(camada.is_off());
        assert_eq!(camada.color(), Color::Index(5));
    }

    #[test]
    fn congelada_e_bloqueada_sobrevivem() {
        let camadas = com_camada("Travada", |camada| {
            camada.set_frozen(true);
            camada.set_locked(true);
        });

        let camada = relida(&camadas, "Travada");
        assert!(camada.is_frozen());
        assert!(camada.is_locked());
    }

    #[test]
    fn tipo_de_linha_e_espessura_sobrevivem() {
        let camadas = com_camada("Tracejada", |camada| {
            camada.set_linetype("DASHED");
            camada.set_line_weight(LineWeight::Hundredths(35));
        });

        let camada = relida(&camadas, "Tracejada");
        assert_eq!(camada.linetype(), "DASHED");
        assert_eq!(camada.line_weight(), LineWeight::Hundredths(35));
    }

    #[test]
    fn espessura_padrao_sobrevive_como_padrao() {
        let camadas = com_camada("Comum", |_| {});

        assert_eq!(relida(&camadas, "Comum").line_weight(), LineWeight::Default);
    }

    #[test]
    fn a_camada_zero_atravessa_sem_duplicar() {
        let camadas = com_camada("Outra", |_| {});

        let relidas = ida_e_volta(&camadas);

        assert_eq!(relidas.iter().count(), 2);
        assert!(relidas.get_by_name("0").is_some());
    }

    #[test]
    fn nome_acentuado_atravessa() {
        let camadas = com_camada("Cotas Elétricas", |_| {});

        assert!(ida_e_volta(&camadas)
            .get_by_name("Cotas Elétricas")
            .is_some());
    }

    #[test]
    fn as_tabelas_de_apoio_saem_no_arquivo() {
        // Sem elas, o `Continuous` que toda camada referencia fica pendurado.
        let camadas = LayerTable::new();
        let texto = String::from_utf8(write_dxf(&DxfContents::from_layers(&camadas)))
            .expect("saída é UTF-8");

        assert!(texto.contains("LTYPE"));
        assert!(texto.contains(CONTINUOUS));
        assert!(texto.contains("STYLE"));
        assert!(texto.contains(STANDARD));
    }

    #[test]
    fn nenhum_codigo_de_camada_fica_por_interpretar() {
        // Se a escrita emitir um código que a leitura não conhece, o relatório
        // acusa — e é assim que os dois lados não se separam em silêncio.
        let camadas = com_camada("Completa", |camada| {
            camada.set_color(Color::Index(3));
            camada.set_linetype("DASHED");
            camada.set_line_weight(LineWeight::Hundredths(50));
            camada.set_frozen(true);
        });

        let leitura = read_dxf(&write_dxf(&DxfContents::from_layers(&camadas)));

        assert!(
            leitura.report.unread_layer_codes.is_empty(),
            "códigos não lidos: {:?}",
            leitura.report.unread_layer_codes
        );
    }
}
