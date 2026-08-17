// Caminho relativo: kernel/neocad-io/tests/round_trip.rs
//! \file kernel/neocad-io/tests/round_trip.rs
//! \brief Ida e volta da leitura e da escrita DXF, com as perdas declaradas.
//! \author Iago Leal
//! \date 2026-08-16
//!
//! Ler, escrever, reler e comparar. O valor deste arquivo não está no que ele
//! prova sobreviver — está no que ele **declara não sobreviver**. Perda que
//! aparece num teste é lacuna conhecida; perda que só aparece no desenho do
//! usuário é trabalho alheio destruído em silêncio, que é o defeito que este
//! projeto vem perseguindo desde a validação contra desenhos reais.

use neocad_io::{read_dxf, write_dxf, DxfContents, DxfReading, EntitySpace};
use neocad_model::{Color, EntityColor, Geometry, LineWeight};

/// Fixtures sintéticas que a suíte E2E também usa.
const FIXTURES: [&str; 6] = [
    "minimal.dxf",
    "with-unsupported.dxf",
    "legacy-polyline.dxf",
    "block-reference.dxf",
    "block-with-entities.dxf",
    "two-layouts.dxf",
];

/// Retrato comparável de uma leitura.
///
/// # Por que não se compara `Document`, e por que nem `Entity`
///
/// O critério de aceite do MT-K2-09 falava em comparar `Document` pela sua
/// `PartialEq` semântica. Não é possível ainda: montar um `Document` exige pôr
/// cada entidade num registro de bloco, e as de espaço-papel precisam dos blocos
/// `*Paper_Space*`, que a `BlockTable` recusa criar. Abrir essa via é o MT-KL-04.
///
/// Comparar `Entity` direto também enganaria: ela guarda `LayerId`, que é
/// posição na arena. Duas leituras do mesmo desenho podem criar as camadas em
/// ordens diferentes — a primeira na ordem do arquivo, a segunda na ordem
/// alfabética em que a escrita as grava — e aí identificadores iguais
/// significariam camadas diferentes. O retrato resolve a camada **pelo nome**,
/// que é o que o desenho de fato diz.
#[derive(Debug, PartialEq)]
struct Retrato {
    camadas: Vec<Camada>,
    entidades: Vec<Entidade>,
    blocos: Vec<Bloco>,
}

#[derive(Debug, PartialEq)]
struct Camada {
    nome: String,
    cor: Color,
    tipo_de_linha: String,
    espessura: LineWeight,
    desligada: bool,
    congelada: bool,
    bloqueada: bool,
}

#[derive(Debug, PartialEq)]
struct Entidade {
    espaco: EntitySpace,
    camada: String,
    cor: EntityColor,
    geometria: Geometry,
}

#[derive(Debug, PartialEq)]
struct Bloco {
    nome: String,
    entidades: Vec<Entidade>,
    xref: Option<String>,
}

impl Retrato {
    fn de(leitura: &DxfReading) -> Self {
        let camadas = leitura
            .layers
            .iter()
            .map(|(_, registro)| Camada {
                nome: registro.name().to_owned(),
                cor: registro.color(),
                tipo_de_linha: registro.linetype().to_owned(),
                espessura: registro.line_weight(),
                desligada: registro.is_off(),
                congelada: registro.is_frozen(),
                bloqueada: registro.is_locked(),
            })
            .collect();

        let nome_da_camada = |entidade: &neocad_model::Entity| {
            leitura
                .layers
                .get(entidade.layer)
                .map_or_else(|| String::from("?"), |c| c.name().to_owned())
        };

        let entidades = leitura
            .entities
            .iter()
            .map(|lida| Entidade {
                espaco: lida.space.clone(),
                camada: nome_da_camada(&lida.entity),
                cor: lida.entity.color,
                geometria: lida.entity.geometry.clone(),
            })
            .collect();

        let blocos = leitura
            .blocks
            .iter()
            .map(|bloco| Bloco {
                nome: bloco.name.clone(),
                entidades: bloco
                    .entities
                    .iter()
                    .map(|entidade| Entidade {
                        espaco: EntitySpace::Model,
                        camada: nome_da_camada(entidade),
                        cor: entidade.color,
                        geometria: entidade.geometry.clone(),
                    })
                    .collect(),
                xref: bloco.xref_path.clone(),
            })
            .collect();

        Self {
            camadas,
            entidades,
            blocos,
        }
    }
}

/// Carrega uma fixture sintética.
fn fixture(nome: &str) -> Vec<u8> {
    let caminho = format!("{}/../../e2e/fixtures/{nome}", env!("CARGO_MANIFEST_DIR"));

    std::fs::read(&caminho).unwrap_or_else(|erro| panic!("{caminho} não pôde ser lida: {erro}"))
}

/// Grava a leitura de volta em bytes.
fn regravar(leitura: &DxfReading) -> Vec<u8> {
    write_dxf(&DxfContents {
        layers: &leitura.layers,
        entities: &leitura.entities,
        blocks: &leitura.blocks,
    })
}

/// A escrita normaliza o arquivo — ordena camadas, converte polilinha antiga em
/// leve, acrescenta os blocos de espaço. Por isso a comparação é entre a
/// **primeira** e a **segunda** gravação, e não contra o arquivo de origem: o
/// que se exige é que a leitura seja fiel ao que ela mesma escreveu.
fn primeira_e_segunda_geracao(bytes: &[u8]) -> (DxfReading, DxfReading) {
    let original = read_dxf(bytes);
    let primeira = regravar(&original);
    let relido = read_dxf(&primeira);

    (read_dxf(&primeira), read_dxf(&regravar(&relido)))
}

#[test]
fn cada_fixture_atravessa_a_ida_e_volta_sem_mudar() {
    for nome in FIXTURES {
        let (uma, outra) = primeira_e_segunda_geracao(&fixture(nome));

        assert_eq!(
            Retrato::de(&uma),
            Retrato::de(&outra),
            "{nome} mudou entre gerações"
        );
    }
}

#[test]
fn a_segunda_gravacao_e_byte_a_byte_igual_a_primeira() {
    // Ponto fixo: depois da primeira normalização, nada mais muda. É o que
    // permite versionar um desenho sem que cada abertura produza um diff.
    for nome in FIXTURES {
        let original = read_dxf(&fixture(nome));
        let primeira = regravar(&original);
        let segunda = regravar(&read_dxf(&primeira));

        assert_eq!(primeira, segunda, "{nome} não é ponto fixo");
    }
}

#[test]
fn o_conteudo_do_modelo_atravessa_a_gravacao() {
    for nome in FIXTURES {
        let original = read_dxf(&fixture(nome));
        let relido = read_dxf(&regravar(&original));

        assert_eq!(
            relido.model_space_count(),
            original.model_space_count(),
            "{nome} perdeu entidade do espaço-modelo"
        );
        assert_eq!(
            relido
                .blocks
                .iter()
                .map(|b| b.entities.len())
                .sum::<usize>(),
            original
                .blocks
                .iter()
                .map(|b| b.entities.len())
                .sum::<usize>(),
            "{nome} perdeu entidade de bloco"
        );
    }
}

// -- Perdas declaradas -------------------------------------------------------
//
// O que segue não é defeito a corrigir aqui: é o custo conhecido de gravar a
// partir de um modelo que ainda não representa tudo. Está escrito para que
// ninguém o descubra num desenho de cliente.

#[test]
fn perda_declarada_entidade_nao_modelada_nao_sobrevive() {
    // `with-unsupported.dxf` traz um `SOLID`, que o modelo não representa. Ele é
    // contado na primeira leitura e simplesmente não existe na segunda.
    let original = read_dxf(&fixture("with-unsupported.dxf"));
    assert_eq!(original.report.unsupported.get("SOLID"), Some(&1));

    let relido = read_dxf(&regravar(&original));

    assert!(
        relido.report.unsupported.is_empty(),
        "a entidade não modelada não é gravada, então some da contagem também"
    );
    assert_eq!(relido.model_space_count(), original.model_space_count());
}

#[test]
fn perda_declarada_referencia_de_bloco_nao_sobrevive() {
    // `INSERT` é referência de bloco, que exige transformação de instância — é
    // fase K3. Até lá o desenho perde a inserção ao ser regravado.
    let original = read_dxf(&fixture("block-reference.dxf"));
    assert_eq!(original.report.unsupported.get("INSERT"), Some(&1));

    let relido = read_dxf(&regravar(&original));

    assert!(relido.report.unsupported.is_empty());
}

#[test]
fn perda_declarada_polilinha_antiga_vira_polilinha_leve() {
    // Não é perda de geometria: os vértices e o fechamento atravessam. O que
    // muda é a **representação** — a `POLYLINE`/`VERTEX`/`SEQEND` do R12 é
    // gravada como `LWPOLYLINE`. Um diff contra o arquivo de origem acusa, e
    // isso é esperado.
    let original = read_dxf(&fixture("legacy-polyline.dxf"));
    let regravado = String::from_utf8(regravar(&original)).expect("saída é UTF-8");

    assert!(regravado.contains("LWPOLYLINE"));
    assert!(!regravado.contains("\nVERTEX\r\n") && !regravado.contains("\nSEQEND\r\n"));

    let relido = read_dxf(regravado.as_bytes());
    let polilinhas = |leitura: &DxfReading| {
        leitura
            .entities
            .iter()
            .filter(|e| matches!(e.entity.geometry, Geometry::Polyline(_)))
            .map(|e| e.entity.geometry.clone())
            .collect::<Vec<_>>()
    };

    assert_eq!(polilinhas(&relido), polilinhas(&original));
}

#[test]
fn perda_declarada_secoes_nao_consumidas_nao_sao_gravadas() {
    // O que a leitura não interpreta, a escrita não reproduz. Hoje isso inclui a
    // `HEADER` do arquivo de origem — as variáveis do desenho — e a `OBJECTS`,
    // onde moram os `LAYOUT`. A segunda é a lacuna que a fase KL fecha.
    let original = read_dxf(&fixture("minimal.dxf"));
    assert!(
        original.report.skipped_sections.contains_key("HEADER"),
        "a fixture tem HEADER, e ele não é consumido"
    );

    // O cabeçalho gravado é o **nosso**, não o do arquivo de origem: as
    // variáveis que estavam lá não voltam. A fixture declara `AC1009`; o que
    // sai declara a versão que a escrita produz.
    let regravado = String::from_utf8(regravar(&original)).expect("saída é UTF-8");

    assert!(String::from_utf8(fixture("minimal.dxf"))
        .expect("fixture é UTF-8")
        .contains("AC1009"));
    assert!(!regravado.contains("AC1009"));
    assert!(regravado.contains(neocad_io::ACAD_VERSION));
}

#[test]
fn perda_declarada_angulo_de_arco_e_de_ultimo_bit() {
    // O modelo guarda radianos; o formato grava graus. A conversão de ida e
    // volta é exata para os ângulos cardeais e aproximada para os demais.
    let arquivo = b"  0\nSECTION\n  2\nENTITIES\n\
                    0\nARC\n  8\n0\n 10\n0.0\n 20\n0.0\n 40\n1.0\n 50\n0.0\n 51\n37.13\n\
                    0\nENDSEC\n  0\nEOF\n";
    let original = read_dxf(arquivo);
    let relido = read_dxf(&regravar(&original));

    let angulo = |leitura: &DxfReading| match &leitura.entities[0].entity.geometry {
        Geometry::Arc(arco) => arco.end_angle,
        outra => panic!("esperava arco, veio {outra:?}"),
    };

    let diferenca = (angulo(&relido) - angulo(&original)).abs();
    assert!(diferenca < 1e-12, "diferença grande demais: {diferenca}");
}

#[test]
fn camada_criada_por_citacao_deixa_de_ser_criada_na_segunda_leitura() {
    // Não é perda: é o arquivo ficando mais correto do que era. A camada que o
    // original citava sem definir passa a estar na tabela do arquivo gravado.
    let arquivo = b"  0\nSECTION\n  2\nENTITIES\n\
                    0\nLINE\n  8\nSemDefinicao\n 10\n0.0\n 20\n0.0\n 11\n1.0\n 21\n1.0\n\
                    0\nENDSEC\n  0\nEOF\n";
    let original = read_dxf(arquivo);
    assert_eq!(original.report.created_layers, ["SemDefinicao"]);

    let relido = read_dxf(&regravar(&original));

    assert!(relido.report.created_layers.is_empty());
    assert!(relido.layers.get_by_name("SemDefinicao").is_some());
}

#[test]
fn desenho_montado_no_papel_atravessa_com_a_aba() {
    // O caso dos 8% do acervo. O espaço de cada entidade precisa sobreviver à
    // gravação, senão a prancha vira desenho solto no espaço-modelo.
    let arquivo = b"  0\nSECTION\n  2\nENTITIES\n\
                    0\nLINE\n  8\n0\n 67\n1\n410\nPrancha A1\n\
                    10\n0.0\n 20\n0.0\n 11\n1.0\n 21\n1.0\n\
                    0\nENDSEC\n  0\nEOF\n";
    let original = read_dxf(arquivo);
    let relido = read_dxf(&regravar(&original));

    assert_eq!(relido.model_space_count(), 0);
    assert_eq!(relido.paper_space_layouts(), ["Prancha A1"]);
    assert_eq!(
        relido.entities[0].space,
        EntitySpace::Paper(String::from("Prancha A1"))
    );
}

#[test]
fn arquivo_gravado_por_nos_e_lido_sem_nenhuma_queixa() {
    // O relatório limpo é a prova de que os dois lados falam a mesma língua: se
    // a escrita emitisse algo que a leitura não entende, apareceria aqui.
    for nome in FIXTURES {
        let leitura = read_dxf(&regravar(&read_dxf(&fixture(nome))));

        assert!(
            leitura.report.is_clean(),
            "{nome} regravado produziu relatório sujo: {:?}",
            leitura.report
        );
    }
}

#[test]
fn a_fixture_de_layouts_atravessa_com_as_duas_pranchas() {
    // Escrita à mão para exercitar o caminho de layout. O parser do upstream
    // abre o arquivo mas joga as cinco entidades no espaço-modelo, ignorando os
    // códigos `67` e `410`; a leitura própria separa corretamente.
    let original = read_dxf(&fixture("two-layouts.dxf"));

    assert_eq!(original.model_space_count(), 2);
    assert_eq!(original.paper_space_layouts(), ["Prancha A1", "Prancha A2"]);

    let relido = read_dxf(&regravar(&original));

    assert_eq!(relido.model_space_count(), 2);
    assert_eq!(relido.paper_space_layouts(), ["Prancha A1", "Prancha A2"]);
}
