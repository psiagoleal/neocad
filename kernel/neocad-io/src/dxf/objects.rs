// Caminho relativo: kernel/neocad-io/src/dxf/objects.rs
//! \file kernel/neocad-io/src/dxf/objects.rs
//! \brief Leitura dos objetos não gráficos — hoje, os layouts.
//! \author Iago Leal
//! \date 2026-08-29

use std::collections::BTreeMap;

use neocad_model::{PageSetup, PlotMargins, PlotRotation, PlotUnits};

use super::pairs::DxfPair;
use super::sections::{Section, SectionKind};

/// Marcador de subclasse da configuração de página.
const SUBCLASSE_PLOT: &str = "AcDbPlotSettings";

/// Marcador de subclasse do layout propriamente dito.
const SUBCLASSE_LAYOUT: &str = "AcDbLayout";

/// Um layout declarado pelo arquivo.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutDefinition {
    /// Nome da aba.
    pub name: String,
    /// Posição da aba na barra.
    pub tab_order: u16,
    /// Nome do registro de bloco associado, quando o vínculo resolve.
    ///
    /// `None` quando o código `330` aponta para um handle que a tabela
    /// `BLOCK_RECORD` não declara — arquivo inconsistente, e o layout é
    /// **relatado** em vez de descartado.
    pub block_name: Option<String>,
    /// Configuração de página.
    pub page_setup: PageSetup,
}

/// Resultado da leitura dos layouts.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutsReading {
    /// Layouts declarados, na ordem do arquivo.
    pub layouts: Vec<LayoutDefinition>,
    /// Nomes de aba cujo vínculo com o bloco não resolveu.
    ///
    /// O layout continua na lista acima, com `block_name` vazio: perder a aba
    /// porque o ponteiro está torto seria descartar a prancha inteira por causa
    /// de um handle.
    pub unresolved_blocks: Vec<String>,
}

/// Lê os layouts de uma seção `OBJECTS`.
///
/// # As duas subclasses compartilham códigos
///
/// Um objeto `LAYOUT` traz duas partes: a configuração de página
/// (`AcDbPlotSettings`) e o layout (`AcDbLayout`). Elas **repetem** códigos de
/// grupo — o `1` é nome da configuração numa e nome da aba na outra; o `70` é
/// sinalizador de plotagem numa e de controle na outra. Ler o registro como um
/// mapa achatado faria o nome da aba virar o nome da configuração, que costuma
/// ser vazio.
///
/// Por isso o percurso acompanha o marcador `100` e decide a cada par a que
/// parte ele pertence. Foi exatamente aqui que uma fixture escrita à mão
/// tropeçou durante o MT-KL-01, e é o tipo de detalhe que só aparece ao
/// escrever o leitor.
///
/// `block_names` mapeia handle para nome de registro de bloco, e sai da tabela
/// `BLOCK_RECORD`; sem ele o código `330` não teria como virar um bloco.
#[must_use]
pub fn read_layouts(section: &Section, block_names: &BTreeMap<String, String>) -> LayoutsReading {
    let mut leitura = LayoutsReading {
        layouts: Vec::new(),
        unresolved_blocks: Vec::new(),
    };

    if section.kind != SectionKind::Objects {
        return leitura;
    }

    let mut atual: Option<Vec<DxfPair>> = None;

    for par in &section.pairs {
        if par.code == 0 {
            if let Some(pares) = atual.take() {
                concluir(&pares, block_names, &mut leitura);
            }

            if marcador(par) == Some("LAYOUT") {
                atual = Some(Vec::new());
            }

            continue;
        }

        if let Some(pares) = atual.as_mut() {
            pares.push(par.clone());
        }
    }

    if let Some(pares) = atual.take() {
        concluir(&pares, block_names, &mut leitura);
    }

    leitura
}

/// Traduz um registro `LAYOUT` acumulado.
fn concluir(
    pares: &[DxfPair],
    block_names: &BTreeMap<String, String>,
    leitura: &mut LayoutsReading,
) {
    let plot = fatia(pares, SUBCLASSE_PLOT);
    let layout = fatia(pares, SUBCLASSE_LAYOUT);

    let name = texto(&layout, 1).unwrap_or_default().trim().to_owned();

    if name.is_empty() {
        // Layout sem nome não tem aba, e uma aba sem nome não é apresentável.
        return;
    }

    let block_name = texto(&layout, 330)
        .and_then(|handle| block_names.get(handle.trim()))
        .cloned();

    if block_name.is_none() {
        leitura.unresolved_blocks.push(name.clone());
    }

    leitura.layouts.push(LayoutDefinition {
        name,
        tab_order: inteiro(&layout, 71)
            .and_then(|valor| u16::try_from(valor).ok())
            .unwrap_or(0),
        block_name,
        page_setup: configuracao_de_pagina(&plot),
    });
}

/// Recorta os pares que pertencem a uma subclasse.
///
/// A fatia vai do marcador `100` da subclasse até o próximo marcador `100`, que
/// é o que separa uma parte da outra dentro do mesmo objeto.
fn fatia<'a>(pares: &'a [DxfPair], subclasse: &str) -> Vec<&'a DxfPair> {
    let mut dentro = false;
    let mut fatia = Vec::new();

    for par in pares {
        if par.code == 100 {
            dentro = marcador(par) == Some(subclasse);
            continue;
        }

        if dentro {
            fatia.push(par);
        }
    }

    fatia
}

/// Monta a configuração de página a partir da fatia de `AcDbPlotSettings`.
fn configuracao_de_pagina(plot: &[&DxfPair]) -> PageSetup {
    let padrao = PageSetup::default();

    PageSetup {
        paper_width: real(plot, 44).unwrap_or(padrao.paper_width),
        paper_height: real(plot, 45).unwrap_or(padrao.paper_height),
        units: match inteiro(plot, 72) {
            Some(0) => PlotUnits::Inches,
            _ => PlotUnits::Millimeters,
        },
        margins: PlotMargins {
            left: real(plot, 40).unwrap_or_default(),
            bottom: real(plot, 41).unwrap_or_default(),
            right: real(plot, 42).unwrap_or_default(),
            top: real(plot, 43).unwrap_or_default(),
        },
        // A escala é razão, e é assim que o arquivo a guarda: numerador e
        // denominador separados. Dividir aqui perderia a distinção entre `1:3` e
        // o decimal que o aproxima.
        scale_numerator: real(plot, 142).unwrap_or(padrao.scale_numerator),
        scale_denominator: real(plot, 143).unwrap_or(padrao.scale_denominator),
        rotation: match inteiro(plot, 73) {
            Some(1) => PlotRotation::Quarter,
            Some(2) => PlotRotation::Half,
            Some(3) => PlotRotation::ThreeQuarters,
            _ => PlotRotation::None,
        },
    }
}

/// Texto de um par, aparado, quando o par é textual.
fn marcador(par: &DxfPair) -> Option<&str> {
    par.value.as_text().map(str::trim)
}

/// Primeiro valor textual de um código, dentro da fatia.
fn texto<'a>(pares: &[&'a DxfPair], code: u16) -> Option<&'a str> {
    pares
        .iter()
        .find(|par| par.code == code)
        .and_then(|par| par.value.as_text())
}

/// Primeiro valor inteiro de um código, dentro da fatia.
fn inteiro(pares: &[&DxfPair], code: u16) -> Option<i64> {
    pares
        .iter()
        .find(|par| par.code == code)
        .and_then(|par| par.value.as_integer())
}

/// Primeiro valor real de um código, dentro da fatia.
fn real(pares: &[&DxfPair], code: u16) -> Option<f64> {
    pares
        .iter()
        .find(|par| par.code == code)
        .and_then(|par| par.value.as_real())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{read_block_record_names, read_dxf, sections};

    fn secao_objects(pares: &[(u16, &str)]) -> Section {
        let mut texto = String::from("  0\nSECTION\n  2\nOBJECTS\n");

        for (codigo, valor) in pares {
            texto.push_str(&format!("{codigo:>3}\n{valor}\n"));
        }

        texto.push_str("  0\nENDSEC\n  0\nEOF\n");

        sections(texto.as_bytes())
            .next()
            .expect("há seção")
            .expect("bem formada")
    }

    fn blocos(pares: &[(&str, &str)]) -> BTreeMap<String, String> {
        pares
            .iter()
            .map(|(handle, nome)| ((*handle).to_owned(), (*nome).to_owned()))
            .collect()
    }

    /// Um `LAYOUT` com as duas subclasses, como o formato o grava.
    fn layout<'a>(nome: &'a str, ordem: &'a str, bloco: &'a str) -> Vec<(u16, &'a str)> {
        vec![
            (0, "LAYOUT"),
            (5, "1A"),
            (100, "AcDbPlotSettings"),
            (1, ""),
            (44, "420.0"),
            (45, "297.0"),
            (72, "1"),
            (73, "0"),
            (142, "1.0"),
            (143, "100.0"),
            (100, "AcDbLayout"),
            (1, nome),
            (71, ordem),
            (330, bloco),
        ]
    }

    #[test]
    fn o_nome_da_aba_vem_da_subclasse_certa() {
        // O código `1` existe nas duas subclasses: na de plotagem é o nome da
        // configuração, quase sempre vazio. Ler o registro achatado faria a aba
        // ficar sem nome — foi o que derrubou uma fixture no MT-KL-01.
        let leitura = read_layouts(
            &secao_objects(&layout("Prancha A1", "1", "21")),
            &blocos(&[("21", "*Paper_Space")]),
        );

        assert_eq!(leitura.layouts.len(), 1);
        assert_eq!(leitura.layouts[0].name, "Prancha A1");
        assert_eq!(leitura.layouts[0].tab_order, 1);
    }

    #[test]
    fn o_vinculo_com_o_bloco_sai_do_handle() {
        let leitura = read_layouts(
            &secao_objects(&layout("Prancha", "1", "21")),
            &blocos(&[("21", "*Paper_Space"), ("22", "*Paper_Space0")]),
        );

        assert_eq!(
            leitura.layouts[0].block_name.as_deref(),
            Some("*Paper_Space")
        );
        assert!(leitura.unresolved_blocks.is_empty());
    }

    #[test]
    fn layout_sem_bloco_correspondente_e_relatado_e_nao_descartado() {
        // O critério de aceite. Perder a prancha porque o ponteiro está torto
        // seria descartar o conteúdo por causa de um handle.
        let leitura = read_layouts(
            &secao_objects(&layout("Órfã", "1", "handle-que-nao-existe")),
            &blocos(&[("21", "*Paper_Space")]),
        );

        assert_eq!(leitura.layouts.len(), 1);
        assert_eq!(leitura.layouts[0].name, "Órfã");
        assert_eq!(leitura.layouts[0].block_name, None);
        assert_eq!(leitura.unresolved_blocks, ["Órfã"]);
    }

    #[test]
    fn a_configuracao_de_pagina_e_lida() {
        let leitura = read_layouts(
            &secao_objects(&layout("Prancha", "1", "21")),
            &blocos(&[("21", "*Paper_Space")]),
        );
        let pagina = leitura.layouts[0].page_setup;

        assert_eq!(pagina.paper_width, 420.0);
        assert_eq!(pagina.paper_height, 297.0);
        assert_eq!(pagina.units, PlotUnits::Millimeters);
        // A escala é razão, e chega como razão: 1:100, e não 0,01.
        assert_eq!(pagina.scale_numerator, 1.0);
        assert_eq!(pagina.scale_denominator, 100.0);
        assert_eq!(pagina.scale(), Some(0.01));
    }

    #[test]
    fn a_rotacao_da_folha_cobre_os_quatro_quartos() {
        for (codigo, esperada) in [
            ("0", PlotRotation::None),
            ("1", PlotRotation::Quarter),
            ("2", PlotRotation::Half),
            ("3", PlotRotation::ThreeQuarters),
        ] {
            let mut pares = layout("Prancha", "1", "21");
            let posicao = pares.iter().position(|(c, _)| *c == 73).expect("há 73");
            pares[posicao] = (73, codigo);

            let leitura = read_layouts(&secao_objects(&pares), &blocos(&[("21", "*Paper_Space")]));

            assert_eq!(leitura.layouts[0].page_setup.rotation, esperada);
        }
    }

    #[test]
    fn a_unidade_da_folha_distingue_polegada_de_milimetro() {
        let mut pares = layout("Prancha", "1", "21");
        let posicao = pares.iter().position(|(c, _)| *c == 72).expect("há 72");
        pares[posicao] = (72, "0");

        let leitura = read_layouts(&secao_objects(&pares), &blocos(&[("21", "*Paper_Space")]));

        assert_eq!(leitura.layouts[0].page_setup.units, PlotUnits::Inches);
    }

    #[test]
    fn layout_sem_nome_nao_vira_aba() {
        // Aba sem nome não é apresentável, e inventar um esconderia o defeito.
        let pares = vec![
            (0, "LAYOUT"),
            (100, "AcDbPlotSettings"),
            (1, ""),
            (100, "AcDbLayout"),
            (1, "   "),
        ];

        assert!(read_layouts(&secao_objects(&pares), &BTreeMap::new())
            .layouts
            .is_empty());
    }

    #[test]
    fn secao_que_nao_e_objects_nao_produz_layout() {
        let texto = "  0\nSECTION\n  2\nENTITIES\n  0\nLAYOUT\n  1\nX\n  0\nENDSEC\n  0\nEOF\n";
        let secao = sections(texto.as_bytes())
            .next()
            .expect("há seção")
            .expect("bem formada");

        assert!(read_layouts(&secao, &BTreeMap::new()).layouts.is_empty());
    }

    #[test]
    fn a_fixture_de_dois_layouts_traz_as_tres_abas() {
        // O critério de aceite: nome, ordem, tamanho de papel e bloco.
        let caminho = format!(
            "{}/../../e2e/fixtures/two-layouts.dxf",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(&caminho).expect("fixture existe");
        let leitura = read_dxf(&bytes);

        let nomes: Vec<&str> = leitura
            .layouts
            .iter()
            .map(|declarada| declarada.name.as_str())
            .collect();
        assert_eq!(nomes, ["Model", "Prancha A1", "Prancha A2"]);

        let prancha = &leitura.layouts[1];
        assert_eq!(prancha.tab_order, 1);
        assert_eq!(prancha.page_setup.paper_width, 420.0);
        assert_eq!(prancha.page_setup.paper_height, 297.0);
        assert_eq!(prancha.block_name.as_deref(), Some("*Paper_Space"));
        assert!(leitura.report.unresolved_layouts.is_empty());
    }

    #[test]
    fn os_handles_dos_registros_de_bloco_sao_lidos() {
        let caminho = format!(
            "{}/../../e2e/fixtures/two-layouts.dxf",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(&caminho).expect("fixture existe");
        let tabelas = sections(&bytes)
            .filter_map(Result::ok)
            .find(|secao| secao.kind == SectionKind::Tables)
            .expect("a fixture tem TABLES");

        let nomes = read_block_record_names(&tabelas);

        assert_eq!(nomes.get("20").map(String::as_str), Some("*Model_Space"));
        assert_eq!(nomes.get("21").map(String::as_str), Some("*Paper_Space"));
        assert_eq!(nomes.get("22").map(String::as_str), Some("*Paper_Space0"));
    }
}
