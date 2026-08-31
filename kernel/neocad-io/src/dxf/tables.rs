// Caminho relativo: kernel/neocad-io/src/dxf/tables.rs
//! \file kernel/neocad-io/src/dxf/tables.rs
//! \brief Leitura das tabelas de símbolos de um arquivo DXF.
//! \author Iago Leal
//! \date 2026-08-12

use std::collections::BTreeMap;

use neocad_model::{Color, LayerError, LayerTable, LineWeight};

use super::pairs::{DxfPair, DxfValue};
use super::sections::{Section, SectionKind};

/// Nome da tabela de camadas dentro da seção `TABLES`.
const TABELA_DE_CAMADAS: &str = "LAYER";

/// Nome da tabela de registros de bloco dentro da seção `TABLES`.
const TABELA_DE_BLOCOS: &str = "BLOCK_RECORD";

/// Códigos que estruturam o arquivo e não descrevem a camada.
///
/// Ficam de fora da contagem de [`LayerTableReading::unread_codes`] para que ela
/// signifique "atributo de camada que ainda não lemos", e não "tudo o que passou
/// por aqui". Contar handle e marcador de subclasse encheria o relatório de ruído
/// e esconderia o que importa.
const CODIGOS_ESTRUTURAIS: [u16; 6] = [
    0,   // marcador de registro
    2,   // nome, já consumido
    5,   // handle
    100, // marcador de subclasse
    102, // início/fim de grupo definido por aplicação
    330, // ponteiro para o dono
];

/// Camada que a tabela do modelo recusou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedLayer {
    /// Nome como veio do arquivo.
    pub name: String,
    /// Motivo da recusa.
    pub reason: LayerError,
}

/// Resultado da leitura da tabela de camadas.
///
/// Traz a tabela **e** o que não coube nela. Camada recusada e código não lido
/// são devolvidos em vez de descartados: foi exatamente o descarte silencioso —
/// no conversor do upstream, que ignora o que não sabe converter — que fez uma
/// medição de cobertura mentir por meses.
// `LayerTable` não implementa `PartialEq` — a igualdade de tabela de símbolos é
// semântica e mora no modelo, não aqui.
#[derive(Debug)]
pub struct LayerTableReading {
    /// Tabela montada, sempre contendo a camada `0`.
    pub table: LayerTable,
    /// Camadas do arquivo que a tabela recusou, na ordem em que apareceram.
    pub rejected: Vec<RejectedLayer>,
    /// Códigos de grupo vistos num registro de camada e ainda não interpretados,
    /// com quantas vezes apareceram.
    pub unread_codes: BTreeMap<u16, usize>,
}

/// Atributos de uma camada, colhidos antes de virarem registro.
///
/// Existe porque a cor depende de dois códigos que podem vir em qualquer ordem —
/// o índice ACI (`62`) e a cor verdadeira (`420`) — e porque o sinal do `62`
/// carrega o estado de desligada. Decidir a cada par exigiria voltar atrás.
#[derive(Debug, Default)]
struct CamadaCrua {
    name: String,
    aci: Option<i64>,
    true_color: Option<i64>,
    linetype: Option<String>,
    line_weight: Option<i64>,
    flags: i64,
}

impl CamadaCrua {
    /// Cor resultante, dando precedência à cor verdadeira.
    ///
    /// Quando os dois códigos estão presentes o `420` vence, como no AutoCAD: o
    /// `62` costuma trazer o índice mais próximo, gravado para quem só entende a
    /// paleta antiga. Preferir o índice descartaria precisão que o arquivo tem.
    fn color(&self) -> Option<Color> {
        if let Some(bruto) = self.true_color {
            return Some(cor_verdadeira(bruto));
        }

        // O sinal do 62 não faz parte da cor: negativo significa camada
        // desligada, e o índice é o valor absoluto. Ignorar isso faria toda
        // camada desligada de um desenho real virar cor inválida.
        self.aci
            .map(i64::unsigned_abs)
            .and_then(|indice| u16::try_from(indice).ok())
            .and_then(Color::from_aci)
    }

    /// Camada desligada — o `62` negativo é a convenção do formato.
    const fn is_off(&self) -> bool {
        matches!(self.aci, Some(valor) if valor < 0)
    }

    /// Camada congelada — bit `1` do código `70`.
    const fn is_frozen(&self) -> bool {
        self.flags & 1 != 0
    }

    /// Camada bloqueada — bit `4` do código `70`.
    const fn is_locked(&self) -> bool {
        self.flags & 4 != 0
    }

    /// Espessura de linha.
    ///
    /// O DXF usa negativos para os herdados — `-3` padrão, `-2` do bloco, `-1`
    /// da camada — e o modelo só distingue padrão de explícita. Os três viram
    /// [`LineWeight::Default`]; a distinção entre eles é lacuna do modelo, não da
    /// leitura, e só faz sentido resolver quando houver quem a use.
    fn line_weight(&self) -> LineWeight {
        match self.line_weight {
            Some(centesimos) if centesimos >= 0 => {
                u16::try_from(centesimos).map_or(LineWeight::Default, LineWeight::Hundredths)
            }
            _ => LineWeight::Default,
        }
    }
}

/// Decompõe uma cor verdadeira de 24 bits, como o código `420` a transporta.
fn cor_verdadeira(bruto: i64) -> Color {
    let empacotada = bruto as u32;

    Color::Rgb {
        red: ((empacotada >> 16) & 0xFF) as u8,
        green: ((empacotada >> 8) & 0xFF) as u8,
        blue: (empacotada & 0xFF) as u8,
    }
}

/// Lê a tabela de camadas de uma seção `TABLES`.
///
/// Uma seção de outro tipo devolve apenas a tabela inicial — não é erro, é
/// ausência: só a `TABLES` carrega tabela de camadas.
///
/// # A camada `0` é atualizada, não recriada
///
/// Todo DXF define a camada `0`, e toda [`LayerTable`] já nasce com ela. Criar de
/// novo seria recusa por nome duplicado em **todo arquivo real**. As propriedades
/// vindas do arquivo são aplicadas sobre a que já existe.
///
/// # Exemplo
///
/// ```
/// use neocad_io::{read_layer_table, sections, SectionKind};
/// use neocad_model::Color;
///
/// let arquivo = b"  0\nSECTION\n  2\nTABLES\n  0\nTABLE\n  2\nLAYER\n\
///                 0\nLAYER\n  2\nFia\xc3\xa7\xc3\xa3o\n 62\n1\n  0\nENDTAB\n\
///                 0\nENDSEC\n  0\nEOF\n";
/// let secao = sections(arquivo).next().expect("há seção")?;
/// let leitura = read_layer_table(&secao);
///
/// assert_eq!(secao.kind, SectionKind::Tables);
/// let camada = leitura.table.get_by_name("Fiação").expect("lida");
/// assert_eq!(camada.color(), Color::Index(1));
/// # Ok::<(), neocad_io::DxfSectionError>(())
/// ```
#[must_use]
pub fn read_layer_table(section: &Section) -> LayerTableReading {
    let mut leitura = LayerTableReading {
        table: LayerTable::new(),
        rejected: Vec::new(),
        unread_codes: BTreeMap::new(),
    };

    if section.kind != SectionKind::Tables {
        return leitura;
    }

    let mut atual: Option<CamadaCrua> = None;

    for par in &section.pairs {
        if par.code == 0 {
            // Um novo registro fecha o anterior — inclusive o `ENDTAB`, que é o
            // que fecha o último de uma tabela.
            if let Some(camada) = atual.take() {
                aplicar(&mut leitura, camada);
            }

            if marcador(par) == Some(TABELA_DE_CAMADAS) {
                atual = Some(CamadaCrua::default());
            }

            continue;
        }

        let Some(camada) = atual.as_mut() else {
            continue;
        };

        acumular(&mut leitura.unread_codes, camada, par);
    }

    if let Some(camada) = atual.take() {
        aplicar(&mut leitura, camada);
    }

    leitura
}

/// Texto de um par, aparado, quando o par é textual.
fn marcador(par: &DxfPair) -> Option<&str> {
    par.value.as_text().map(str::trim)
}

/// Absorve um par no registro de camada em construção.
fn acumular(nao_lidos: &mut BTreeMap<u16, usize>, camada: &mut CamadaCrua, par: &DxfPair) {
    match (par.code, &par.value) {
        (2, DxfValue::Text(nome)) => camada.name = nome.trim().to_owned(),
        (6, DxfValue::Text(tipo)) => camada.linetype = Some(tipo.trim().to_owned()),
        (62, DxfValue::Integer(valor)) => camada.aci = Some(*valor),
        (70, DxfValue::Integer(valor)) => camada.flags = *valor,
        (370, DxfValue::Integer(valor)) => camada.line_weight = Some(*valor),
        (420, DxfValue::Integer(valor)) => camada.true_color = Some(*valor),
        _ => {
            if !CODIGOS_ESTRUTURAIS.contains(&par.code) {
                *nao_lidos.entry(par.code).or_insert(0) += 1;
            }
        }
    }
}

/// Cria ou atualiza a camada correspondente ao registro lido.
fn aplicar(leitura: &mut LayerTableReading, camada: CamadaCrua) {
    let id = match leitura.table.id_of(&camada.name) {
        Some(id) => id,
        None => match leitura.table.create(camada.name.clone()) {
            Ok(id) => id,
            Err(reason) => {
                leitura.rejected.push(RejectedLayer {
                    name: camada.name,
                    reason,
                });
                return;
            }
        },
    };

    let Some(registro) = leitura.table.get_mut(id) else {
        return;
    };

    if let Some(color) = camada.color() {
        registro.set_color(color);
    }

    if let Some(linetype) = &camada.linetype {
        registro.set_linetype(linetype.clone());
    }

    registro.set_line_weight(camada.line_weight());
    registro.set_off(camada.is_off());
    registro.set_frozen(camada.is_frozen());
    registro.set_locked(camada.is_locked());
}

/// Mapeia handle para nome de registro de bloco.
///
/// # Para que serve
///
/// Um objeto `LAYOUT` aponta para o seu bloco pelo **handle**, no código `330`.
/// Sem este mapa o ponteiro não vira nada, e o vínculo entre a aba e o lugar
/// onde as entidades dela moram se perde — a prancha existiria sem conteúdo.
///
/// Só o handle e o nome são lidos: o resto do registro de bloco já vem pela
/// seção `BLOCKS`, e duplicar aqui abriria espaço para as duas leituras
/// divergirem.
#[must_use]
pub fn read_block_record_names(section: &Section) -> BTreeMap<String, String> {
    let mut nomes = BTreeMap::new();

    if section.kind != SectionKind::Tables {
        return nomes;
    }

    let mut na_tabela = false;
    let mut handle: Option<String> = None;
    let mut nome: Option<String> = None;

    for par in &section.pairs {
        if par.code == 0 {
            guardar(&mut nomes, handle.take(), nome.take());

            match marcador(par) {
                Some("TABLE") => na_tabela = false,
                Some(TABELA_DE_BLOCOS) => na_tabela = true,
                Some("ENDTAB") => na_tabela = false,
                _ => {}
            }

            continue;
        }

        if !na_tabela {
            // O `2` que nomeia a tabela vem depois do `0/TABLE`, e é ele que
            // liga o percurso.
            if par.code == 2 && marcador(par) == Some(TABELA_DE_BLOCOS) {
                na_tabela = true;
            }

            continue;
        }

        match par.code {
            5 => handle = marcador(par).map(str::to_owned),
            2 => nome = marcador(par).map(str::to_owned),
            _ => {}
        }
    }

    guardar(&mut nomes, handle, nome);

    nomes
}

/// Guarda o par handle/nome quando os dois existem.
fn guardar(nomes: &mut BTreeMap<String, String>, handle: Option<String>, nome: Option<String>) {
    if let (Some(handle), Some(nome)) = (handle, nome) {
        if !handle.is_empty() && !nome.is_empty() {
            nomes.insert(handle, nome);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections;

    /// Monta uma seção `TABLES` contendo apenas a tabela de camadas.
    fn tabela_de_camadas(pares: &[(u16, &str)]) -> Section {
        let mut texto = String::from("  0\nSECTION\n  2\nTABLES\n  0\nTABLE\n  2\nLAYER\n");

        for (codigo, valor) in pares {
            texto.push_str(&format!("{codigo:>3}\n{valor}\n"));
        }

        texto.push_str("  0\nENDTAB\n  0\nENDSEC\n  0\nEOF\n");

        sections(texto.as_bytes())
            .next()
            .expect("há seção")
            .expect("bem formada")
    }

    fn ler(pares: &[(u16, &str)]) -> LayerTableReading {
        read_layer_table(&tabela_de_camadas(pares))
    }

    #[test]
    fn le_nome_tipo_de_linha_e_espessura() {
        let leitura = ler(&[
            (0, "LAYER"),
            (2, "Cotas"),
            (6, "DASHED"),
            (370, "35"),
            (62, "3"),
        ]);

        let camada = leitura.table.get_by_name("Cotas").expect("lida");
        assert_eq!(camada.name(), "Cotas");
        assert_eq!(camada.linetype(), "DASHED");
        assert_eq!(camada.line_weight(), LineWeight::Hundredths(35));
        assert!(leitura.rejected.is_empty());
    }

    #[test]
    fn cor_por_indice_aci() {
        let leitura = ler(&[(0, "LAYER"), (2, "Vermelha"), (62, "1")]);

        assert_eq!(
            leitura.table.get_by_name("Vermelha").expect("lida").color(),
            Color::Index(1)
        );
    }

    #[test]
    fn cor_verdadeira_chega_decomposta() {
        // 0x336699 = 3368601.
        let leitura = ler(&[(0, "LAYER"), (2, "Azulada"), (420, "3368601")]);

        assert_eq!(
            leitura.table.get_by_name("Azulada").expect("lida").color(),
            Color::Rgb {
                red: 0x33,
                green: 0x66,
                blue: 0x99
            }
        );
    }

    #[test]
    fn cor_verdadeira_vence_o_indice() {
        // O gravador põe os dois: o 62 é aproximação para quem só lê a paleta.
        let leitura = ler(&[(0, "LAYER"), (2, "Ambas"), (62, "1"), (420, "255")]);

        assert_eq!(
            leitura.table.get_by_name("Ambas").expect("lida").color(),
            Color::Rgb {
                red: 0,
                green: 0,
                blue: 255
            }
        );
    }

    #[test]
    fn extremos_da_paleta_nao_viram_cor() {
        // O defeito que dado real revelou: 0 e 256 não cabem em `u8` nem são
        // cores — são herança de bloco e de camada.
        let leitura = ler(&[
            (0, "LAYER"),
            (2, "DoBloco"),
            (62, "0"),
            (0, "LAYER"),
            (2, "DaCamada"),
            (62, "256"),
        ]);

        assert_eq!(
            leitura.table.get_by_name("DoBloco").expect("lida").color(),
            Color::ByBlock
        );
        assert_eq!(
            leitura.table.get_by_name("DaCamada").expect("lida").color(),
            Color::ByLayer
        );
        assert!(leitura.rejected.is_empty());
    }

    #[test]
    fn indice_negativo_significa_camada_desligada() {
        // A convenção do formato: o sinal carrega o estado, não a cor.
        let leitura = ler(&[(0, "LAYER"), (2, "Apagada"), (62, "-5")]);

        let camada = leitura.table.get_by_name("Apagada").expect("lida");
        assert_eq!(camada.color(), Color::Index(5));
        assert!(camada.is_off());
        assert!(!camada.is_visible());
    }

    #[test]
    fn indice_positivo_deixa_a_camada_ligada() {
        let leitura = ler(&[(0, "LAYER"), (2, "Acesa"), (62, "5")]);

        assert!(!leitura.table.get_by_name("Acesa").expect("lida").is_off());
    }

    #[test]
    fn extremo_256_negativo_continua_sendo_bylayer() {
        let leitura = ler(&[(0, "LAYER"), (2, "Herdada"), (62, "-256")]);

        let camada = leitura.table.get_by_name("Herdada").expect("lida");
        assert_eq!(camada.color(), Color::ByLayer);
        assert!(camada.is_off());
    }

    #[test]
    fn bits_do_codigo_70_viram_congelada_e_bloqueada() {
        let leitura = ler(&[
            (0, "LAYER"),
            (2, "Congelada"),
            (70, "1"),
            (0, "LAYER"),
            (2, "Bloqueada"),
            (70, "4"),
            (0, "LAYER"),
            (2, "Ambas"),
            (70, "5"),
            (0, "LAYER"),
            (2, "Livre"),
            (70, "0"),
        ]);

        let congelada = leitura.table.get_by_name("Congelada").expect("lida");
        assert!(congelada.is_frozen() && !congelada.is_locked());

        let bloqueada = leitura.table.get_by_name("Bloqueada").expect("lida");
        assert!(bloqueada.is_locked() && !bloqueada.is_frozen());

        let ambas = leitura.table.get_by_name("Ambas").expect("lida");
        assert!(ambas.is_frozen() && ambas.is_locked());

        let livre = leitura.table.get_by_name("Livre").expect("lida");
        assert!(!livre.is_frozen() && !livre.is_locked());
    }

    #[test]
    fn espessura_herdada_vira_padrao() {
        // -3 padrão, -2 do bloco, -1 da camada: o modelo não distingue os três.
        for herdada in ["-1", "-2", "-3"] {
            let leitura = ler(&[(0, "LAYER"), (2, "Herdada"), (370, herdada)]);

            assert_eq!(
                leitura
                    .table
                    .get_by_name("Herdada")
                    .expect("lida")
                    .line_weight(),
                LineWeight::Default
            );
        }
    }

    #[test]
    fn camada_zero_e_atualizada_e_nao_recusada() {
        // Todo DXF real define a camada 0, e toda tabela já nasce com ela.
        // Recriar seria recusa por nome duplicado em todo arquivo do mundo.
        let leitura = ler(&[(0, "LAYER"), (2, "0"), (62, "2"), (70, "4")]);

        assert!(leitura.rejected.is_empty());
        let zero = leitura.table.get_by_name("0").expect("sempre existe");
        assert_eq!(zero.color(), Color::Index(2));
        assert!(zero.is_locked());
        assert_eq!(leitura.table.iter().count(), 1);
    }

    #[test]
    fn nome_recusado_pelo_modelo_e_relatado_e_nao_derruba_a_leitura() {
        let leitura = ler(&[
            (0, "LAYER"),
            (2, "Proi/bida"),
            (0, "LAYER"),
            (2, "Boa"),
            (62, "4"),
        ]);

        assert_eq!(
            leitura.rejected,
            [RejectedLayer {
                name: String::from("Proi/bida"),
                reason: LayerError::ForbiddenCharacter('/')
            }]
        );
        assert!(leitura.table.get_by_name("Boa").is_some());
    }

    #[test]
    fn camada_repetida_atualiza_em_vez_de_recusar() {
        let leitura = ler(&[
            (0, "LAYER"),
            (2, "Dupla"),
            (62, "1"),
            (0, "LAYER"),
            (2, "DUPLA"),
            (62, "5"),
        ]);

        assert!(leitura.rejected.is_empty());
        assert_eq!(leitura.table.iter().count(), 2); // a camada 0 mais esta
        assert_eq!(
            leitura.table.get_by_name("Dupla").expect("lida").color(),
            Color::Index(5)
        );
    }

    #[test]
    fn codigo_nao_interpretado_e_contado() {
        // 290 é o sinalizador de plotagem, que o modelo ainda não representa.
        let leitura = ler(&[
            (0, "LAYER"),
            (2, "Uma"),
            (290, "1"),
            (0, "LAYER"),
            (2, "Outra"),
            (290, "0"),
            (390, "8A"),
        ]);

        assert_eq!(leitura.unread_codes.get(&290), Some(&2));
        assert_eq!(leitura.unread_codes.get(&390), Some(&1));
    }

    #[test]
    fn codigo_estrutural_nao_polui_o_relatorio() {
        let leitura = ler(&[
            (0, "LAYER"),
            (5, "2F"),
            (330, "2"),
            (100, "AcDbSymbolTableRecord"),
            (100, "AcDbLayerTableRecord"),
            (2, "Limpa"),
        ]);

        assert!(leitura.unread_codes.is_empty());
        assert!(leitura.table.get_by_name("Limpa").is_some());
    }

    #[test]
    fn outras_tabelas_da_secao_sao_ignoradas() {
        // A seção TABLES traz VPORT, LTYPE, STYLE e outras. Nenhuma delas pode
        // virar camada por descuido.
        let secao = {
            let texto = concat!(
                "  0\nSECTION\n  2\nTABLES\n",
                "  0\nTABLE\n  2\nLTYPE\n  0\nLTYPE\n  2\nDASHED\n  0\nENDTAB\n",
                "  0\nTABLE\n  2\nLAYER\n  0\nLAYER\n  2\nÚnica\n 62\n7\n  0\nENDTAB\n",
                "  0\nTABLE\n  2\nSTYLE\n  0\nSTYLE\n  2\nStandard\n  0\nENDTAB\n",
                "  0\nENDSEC\n  0\nEOF\n"
            );

            sections(texto.as_bytes())
                .next()
                .expect("há seção")
                .expect("bem formada")
        };

        let leitura = read_layer_table(&secao);

        assert_eq!(leitura.table.iter().count(), 2); // a camada 0 mais Única
        assert!(leitura.table.get_by_name("Única").is_some());
        assert!(leitura.table.get_by_name("DASHED").is_none());
        assert!(leitura.table.get_by_name("Standard").is_none());
    }

    #[test]
    fn secao_que_nao_e_tables_devolve_a_tabela_inicial() {
        let texto =
            "  0\nSECTION\n  2\nENTITIES\n  0\nLAYER\n  2\nEnganosa\n  0\nENDSEC\n  0\nEOF\n";
        let secao = sections(texto.as_bytes())
            .next()
            .expect("há seção")
            .expect("bem formada");

        let leitura = read_layer_table(&secao);

        assert_eq!(leitura.table.iter().count(), 1);
        assert!(leitura.table.get_by_name("Enganosa").is_none());
    }

    #[test]
    fn ultima_camada_antes_do_endtab_nao_se_perde() {
        // O registro só fecha no próximo `0`, e o último `0` é o ENDTAB.
        let leitura = ler(&[
            (0, "LAYER"),
            (2, "Primeira"),
            (0, "LAYER"),
            (2, "Última"),
            (62, "6"),
        ]);

        assert_eq!(
            leitura.table.get_by_name("Última").expect("lida").color(),
            Color::Index(6)
        );
    }

    #[test]
    fn nome_acentuado_sobrevive() {
        let leitura = ler(&[(0, "LAYER"), (2, "Cotas Elétricas")]);

        assert!(leitura.table.get_by_name("Cotas Elétricas").is_some());
    }

    #[test]
    fn tabela_de_camadas_vazia_produz_apenas_a_camada_zero() {
        let leitura = ler(&[]);

        assert_eq!(leitura.table.iter().count(), 1);
        assert!(leitura.rejected.is_empty());
        assert!(leitura.unread_codes.is_empty());
    }
}
