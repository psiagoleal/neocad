// Caminho relativo: kernel/neocad-io/src/dxf/writer/objects.rs
//! \file kernel/neocad-io/src/dxf/writer/objects.rs
//! \brief Escrita da seção `OBJECTS` — hoje, os layouts.
//! \author Iago Leal
//! \date 2026-08-31

use neocad_model::{PageSetup, PlotRotation, PlotUnits, MODEL_LAYOUT_NAME};

use super::super::entities::DEFAULT_PAPER_SPACE;
use super::super::objects::LayoutDefinition;
use super::{DxfContents, Handles, Saida};

/// Nome do registro de bloco do espaço-modelo.
const MODEL_SPACE_BLOCK: &str = "*Model_Space";

/// Escreve a seção `OBJECTS`.
///
/// # A aba do espaço-modelo sai sempre
///
/// Todo arquivo tem a aba `Model`, e um leitor que não a encontra precisa
/// inventá-la. Gravá-la aqui poupa o outro lado de adivinhar, e é o que o
/// AutoCAD faz.
///
/// # As duas subclasses saem separadas
///
/// Um objeto `LAYOUT` tem `AcDbPlotSettings` e `AcDbLayout`, e as duas repetem
/// códigos — o `1` é nome da configuração numa e nome da aba na outra. Gravar
/// sem os marcadores `100` faria um leitor correto ler o nome errado, que foi o
/// defeito que a leitura do MT-KL-10 teve de contornar.
pub(super) fn write_objects(saida: &mut Saida, contents: &DxfContents<'_>, handles: &mut Handles) {
    saida.par(0, "SECTION");
    saida.par(2, "OBJECTS");

    write_layout(
        saida,
        MODEL_LAYOUT_NAME,
        0,
        MODEL_SPACE_BLOCK,
        PageSetup::default(),
        handles,
    );

    // A aba do espaço-modelo é gravada acima, sempre. Uma que venha junto do
    // conteúdo — porque a leitura a trouxe do arquivo — é descartada aqui, e não
    // no chamador: o contrato diz "sem a aba do modelo", e fazer a escrita
    // garanti-lo é o que impede o arquivo de sair com duas abas `Model`. Foi o
    // teste de ponto fixo que cobrou isso.
    for layout in contents
        .layouts
        .iter()
        .filter(|layout| !eh_aba_do_modelo(layout))
    {
        write_layout(
            saida,
            &layout.name,
            layout.tab_order,
            layout.block_name.as_deref().unwrap_or(DEFAULT_PAPER_SPACE),
            layout.page_setup,
            handles,
        );
    }

    saida.par(0, "ENDSEC");
}

/// Indica se o layout é a aba do espaço-modelo.
///
/// Reconhecida pelo bloco quando ele existe, e pelo nome quando não — o bloco é
/// a identidade, o nome é o que sobra quando o vínculo não resolveu.
fn eh_aba_do_modelo(layout: &LayoutDefinition) -> bool {
    match layout.block_name.as_deref() {
        Some(bloco) => bloco.eq_ignore_ascii_case(MODEL_SPACE_BLOCK),
        None => layout.name.eq_ignore_ascii_case(MODEL_LAYOUT_NAME),
    }
}

/// Escreve um objeto `LAYOUT`.
fn write_layout(
    saida: &mut Saida,
    nome: &str,
    ordem: u16,
    bloco: &str,
    pagina: PageSetup,
    handles: &mut Handles,
) {
    saida.par(0, "LAYOUT");
    saida.par(5, &handles.proximo());

    saida.par(100, "AcDbPlotSettings");
    // O nome da configuração de página é outro campo, e sai vazio: o modelo não
    // o representa, e inventar um faria o arquivo afirmar o que não sabe.
    saida.par(1, "");
    saida.par(2, "");
    saida.par(4, "");
    saida.par(6, "");
    saida.real(40, pagina.margins.left);
    saida.real(41, pagina.margins.bottom);
    saida.real(42, pagina.margins.right);
    saida.real(43, pagina.margins.top);
    saida.real(44, pagina.paper_width);
    saida.real(45, pagina.paper_height);
    saida.real(46, 0.0);
    saida.real(47, 0.0);
    saida.real(48, 0.0);
    saida.real(49, 0.0);
    saida.real(140, 0.0);
    saida.real(141, 0.0);
    // A escala sai como razão, que é como o arquivo a guarda e como o carimbo a
    // mostra. Gravar o quociente perderia a distinção entre `1:3` e o decimal.
    saida.real(142, pagina.scale_numerator);
    saida.real(143, pagina.scale_denominator);
    saida.inteiro(70, 688);
    saida.inteiro(
        72,
        match pagina.units {
            PlotUnits::Inches => 0,
            PlotUnits::Millimeters => 1,
        },
    );
    saida.inteiro(
        73,
        match pagina.rotation {
            PlotRotation::None => 0,
            PlotRotation::Quarter => 1,
            PlotRotation::Half => 2,
            PlotRotation::ThreeQuarters => 3,
        },
    );
    saida.inteiro(74, 5);
    saida.par(7, "");
    saida.inteiro(75, 16);
    saida.real(147, 1.0);

    saida.par(100, "AcDbLayout");
    saida.par(1, nome);
    saida.inteiro(70, 1);
    saida.inteiro(71, i64::from(ordem));
    saida.real(10, 0.0);
    saida.real(20, 0.0);
    saida.real(11, pagina.paper_width);
    saida.real(21, pagina.paper_height);
    saida.real(12, 0.0);
    saida.real(22, 0.0);
    saida.real(32, 0.0);
    saida.real(14, 0.0);
    saida.real(24, 0.0);
    saida.real(34, 0.0);
    saida.real(15, pagina.paper_width);
    saida.real(25, pagina.paper_height);
    saida.real(35, 0.0);
    saida.real(146, 0.0);
    saida.real(13, 0.0);
    saida.real(23, 0.0);
    saida.real(33, 0.0);
    saida.real(16, 1.0);
    saida.real(26, 0.0);
    saida.real(36, 0.0);
    saida.real(17, 0.0);
    saida.real(27, 1.0);
    saida.real(37, 0.0);
    saida.inteiro(76, 0);

    // O `330` é o que liga a aba ao bloco onde as entidades dela moram. Sem o
    // handle o vínculo se perde, e a prancha reabre vazia.
    if let Some(handle) = handles.bloco(bloco).map(str::to_owned) {
        saida.par(330, &handle);
    }
}
