// Caminho relativo: kernel/neocad-io/src/dxf/entities.rs
//! \file kernel/neocad-io/src/dxf/entities.rs
//! \brief Leitura das entidades de desenho de um arquivo DXF.
//! \author Iago Leal
//! \date 2026-08-12

use std::collections::{BTreeMap, BTreeSet};

use neocad_geometry::Point2;
use neocad_model::{
    Arc, Circle, Color, Entity, EntityColor, Geometry, LayerError, LayerTable, Line, Polyline,
    Text, Viewport, ViewportClip,
};

use super::pairs::DxfPair;
use super::sections::{Section, SectionKind};

/// Nome dado ao espaço-papel de um arquivo que não nomeia a aba.
///
/// O DXF de versão antiga tem um único espaço-papel e não grava o código `410`.
/// O nome do registro de bloco correspondente é este, e usá-lo mantém a leitura
/// alinhada ao que o próprio formato chama a coisa.
pub const DEFAULT_PAPER_SPACE: &str = "*Paper_Space";

/// Onde uma entidade mora.
///
/// A distinção existe desde a leitura porque **descartá-la é proibido** pelo
/// ADR 0005: 70% dos desenhos do acervo têm conteúdo em espaço-papel, e é lá que
/// está a prancha que o usuário emite. O modelo próprio de layouts é da fase KL;
/// até ela chegar, esta é a forma de a informação atravessar K2 sem se perder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntitySpace {
    /// Espaço-modelo.
    Model,
    /// Um layout de espaço-papel, nomeado pela aba.
    Paper(String),
}

/// Entidade lida, com o espaço a que pertence.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadEntity {
    /// Espaço de origem.
    pub space: EntitySpace,
    /// Entidade já traduzida para o modelo.
    pub entity: Entity,
}

/// Entidade que não pôde ser criada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedEntity {
    /// Tipo como veio do arquivo.
    pub entity_type: String,
    /// Camada citada.
    pub layer_name: String,
    /// Motivo — hoje sempre a recusa do nome de camada pela tabela.
    pub reason: LayerError,
}

/// Resultado da leitura da seção `ENTITIES`.
#[derive(Debug, Clone, PartialEq)]
pub struct EntitiesReading {
    /// Entidades lidas, **na ordem do arquivo**, que é a ordem de desenho.
    pub entities: Vec<ReadEntity>,
    /// Tipos que o modelo ainda não representa, com quantas vezes apareceram.
    pub unsupported: BTreeMap<String, usize>,
    /// Camadas citadas por entidade e ausentes da tabela, criadas na leitura.
    pub created_layers: Vec<String>,
    /// Entidades que não puderam ser criadas.
    pub rejected: Vec<RejectedEntity>,
    /// Janelas cujo recorte por entidade não pôde ser resolvido.
    ///
    /// O código `340` aponta para a entidade que delimita a vista, por handle, e
    /// a leitura não mantém mapa de handle para entidade. A janela é lida com
    /// recorte retangular, e a contagem existe para a diferença **aparecer** —
    /// uma prancha com recorte perdido mostra mais do que deveria.
    pub clipped_viewports: usize,
    /// Camadas congeladas cujo handle não resolveu para camada nenhuma.
    pub unresolved_frozen_layers: usize,
}

impl EntitiesReading {
    /// Leitura vazia, pronta para receber registros.
    #[must_use]
    pub(super) fn vazia() -> Self {
        Self {
            entities: Vec::new(),
            unsupported: BTreeMap::new(),
            created_layers: Vec::new(),
            rejected: Vec::new(),
            clipped_viewports: 0,
            unresolved_frozen_layers: 0,
        }
    }

    /// Entidades de um espaço, na ordem de desenho.
    pub fn in_space<'a>(&'a self, space: &'a EntitySpace) -> impl Iterator<Item = &'a Entity> {
        self.entities
            .iter()
            .filter(move |lida| &lida.space == space)
            .map(|lida| &lida.entity)
    }

    /// Quantidade de entidades do espaço-modelo.
    #[must_use]
    pub fn model_space_count(&self) -> usize {
        self.in_space(&EntitySpace::Model).count()
    }

    /// Total de entidades não modeladas, somando todos os tipos.
    #[must_use]
    pub fn unsupported_count(&self) -> usize {
        self.unsupported.values().sum()
    }
}

/// Lê as entidades de uma seção `ENTITIES`.
///
/// A tabela de camadas é recebida por referência mutável porque **camada citada
/// e ausente é criada**, como o AutoCAD faz ao abrir um arquivo assim. A
/// alternativa — descartar a entidade — perderia desenho por causa de uma
/// inconsistência que o próprio AutoCAD conserta em silêncio; aqui ela é
/// consertada e **registrada** em [`EntitiesReading::created_layers`].
///
/// Uma seção de outro tipo devolve leitura vazia: só a `ENTITIES` traz entidades
/// de desenho, e a `BLOCKS` é assunto do MT-K2-05.
///
/// # Exemplo
///
/// ```
/// use neocad_io::{read_entities, sections, EntitySpace};
/// use neocad_model::LayerTable;
///
/// let arquivo = b"  0\nSECTION\n  2\nENTITIES\n\
///                 0\nCIRCLE\n  8\n0\n 10\n5.0\n 20\n5.0\n 40\n2.5\n\
///                 0\nENDSEC\n  0\nEOF\n";
/// let secao = sections(arquivo).next().expect("há seção")?;
/// let mut camadas = LayerTable::new();
/// let leitura = read_entities(&secao, &mut camadas, &Default::default());
///
/// assert_eq!(leitura.model_space_count(), 1);
/// assert_eq!(leitura.entities[0].space, EntitySpace::Model);
/// # Ok::<(), neocad_io::DxfSectionError>(())
/// ```
pub fn read_entities(
    section: &Section,
    layers: &mut LayerTable,
    layer_handles: &BTreeMap<String, String>,
) -> EntitiesReading {
    let mut leitura = EntitiesReading::vazia();

    if section.kind != SectionKind::Entities {
        return leitura;
    }

    ler_registros(&section.pairs, layers, layer_handles, &mut leitura);
    leitura
}

/// Percorre os registros `0/<TIPO>` de uma sequência de pares.
///
/// Extraído de [`read_entities`] porque a seção `BLOCKS` vai reaproveitá-lo:
/// dentro de uma definição de bloco as entidades têm exatamente esta forma.
pub(super) fn ler_registros(
    pares: &[DxfPair],
    layers: &mut LayerTable,
    layer_handles: &BTreeMap<String, String>,
    leitura: &mut EntitiesReading,
) {
    let mut tipo = String::new();
    let mut atual: Vec<DxfPair> = Vec::new();
    let mut polilinha: Option<PolilinhaEmCurso> = None;

    for par in pares {
        if par.code != 0 {
            atual.push(par.clone());
            continue;
        }

        fechar(
            &tipo,
            &atual,
            &mut polilinha,
            layers,
            layer_handles,
            leitura,
        );
        tipo = par
            .value
            .as_text()
            .map(str::trim)
            .unwrap_or_default()
            .to_owned();
        atual.clear();
    }

    fechar(
        &tipo,
        &atual,
        &mut polilinha,
        layers,
        layer_handles,
        leitura,
    );

    // Polilinha antiga sem `SEQEND` não é motivo para perder os vértices já
    // lidos: o arquivo está torto, o desenho não precisa sumir junto.
    if let Some(em_curso) = polilinha.take() {
        concluir_polilinha(em_curso, layers, leitura);
    }
}

/// Polilinha de estilo antigo, montada ao longo de vários registros.
///
/// A `POLYLINE` do DXF R12 declara-se num registro e entrega os vértices em
/// registros `VERTEX` seguintes, até um `SEQEND`. É a única entidade cujo
/// conteúdo não cabe no próprio registro, e por isso precisa de estado.
#[derive(Debug)]
struct PolilinhaEmCurso {
    layer_name: String,
    color: EntityColor,
    space: EntitySpace,
    closed: bool,
    vertices: Vec<Point2>,
}

/// Fecha o registro corrente, traduzindo-o ou contando-o.
fn fechar(
    tipo: &str,
    pares: &[DxfPair],
    polilinha: &mut Option<PolilinhaEmCurso>,
    layers: &mut LayerTable,
    layer_handles: &BTreeMap<String, String>,
    leitura: &mut EntitiesReading,
) {
    if tipo.is_empty() {
        return;
    }

    match tipo {
        "VERTEX" => {
            if let Some(em_curso) = polilinha.as_mut() {
                em_curso.vertices.extend(pontos(pares));
                return;
            }
        }
        "SEQEND" => {
            if let Some(em_curso) = polilinha.take() {
                concluir_polilinha(em_curso, layers, leitura);
            }
            return;
        }
        "POLYLINE" => {
            // Uma polilinha nova fecha a anterior: `SEQEND` faltando é arquivo
            // torto, não motivo para juntar duas polilinhas numa só.
            if let Some(anterior) = polilinha.take() {
                concluir_polilinha(anterior, layers, leitura);
            }

            *polilinha = Some(PolilinhaEmCurso {
                layer_name: nome_da_camada(pares),
                color: cor(pares),
                space: espaco(pares),
                closed: inteiro(pares, 70).is_some_and(|flags| flags & 1 != 0),
                vertices: Vec::new(),
            });
            return;
        }
        _ => {
            // Qualquer outro tipo encerra uma polilinha aberta antes de si.
            if let Some(anterior) = polilinha.take() {
                concluir_polilinha(anterior, layers, leitura);
            }
        }
    }

    if tipo == "VIEWPORT" {
        // O viewport de identificador `1` é a **folha**, e não uma janela: todo
        // espaço-papel tem um, e transformá-lo em entidade criaria uma moldura
        // fantasma do tamanho da prancha em cima de tudo. É compreendido e
        // deliberadamente não modelado — diferente de "não suportado".
        if inteiro(pares, 69) == Some(1) {
            return;
        }

        let Some(janela) = janela(pares, layers, layer_handles, leitura) else {
            *leitura.unsupported.entry(tipo.to_owned()).or_insert(0) += 1;
            return;
        };

        registrar(
            &nome_da_camada(pares),
            cor(pares),
            espaco(pares),
            Geometry::Viewport(janela),
            tipo,
            layers,
            leitura,
        );

        return;
    }

    let Some(geometry) = geometria(tipo, pares) else {
        *leitura.unsupported.entry(tipo.to_owned()).or_insert(0) += 1;
        return;
    };

    registrar(
        &nome_da_camada(pares),
        cor(pares),
        espaco(pares),
        geometry,
        tipo,
        layers,
        leitura,
    );
}

/// Transforma a polilinha acumulada em entidade.
fn concluir_polilinha(
    em_curso: PolilinhaEmCurso,
    layers: &mut LayerTable,
    leitura: &mut EntitiesReading,
) {
    let geometry = Geometry::Polyline(Polyline {
        vertices: em_curso.vertices,
        closed: em_curso.closed,
    });

    registrar(
        &em_curso.layer_name,
        em_curso.color,
        em_curso.space,
        geometry,
        "POLYLINE",
        layers,
        leitura,
    );
}

/// Resolve a camada e guarda a entidade.
fn registrar(
    layer_name: &str,
    color: EntityColor,
    space: EntitySpace,
    geometry: Geometry,
    entity_type: &str,
    layers: &mut LayerTable,
    leitura: &mut EntitiesReading,
) {
    let layer = match layers.id_of(layer_name) {
        Some(id) => id,
        None => match layers.create(layer_name.to_owned()) {
            Ok(id) => {
                leitura.created_layers.push(layer_name.to_owned());
                id
            }
            Err(reason) => {
                leitura.rejected.push(RejectedEntity {
                    entity_type: entity_type.to_owned(),
                    layer_name: layer_name.to_owned(),
                    reason,
                });
                return;
            }
        },
    };

    leitura.entities.push(ReadEntity {
        space,
        entity: Entity {
            layer,
            color,
            geometry,
        },
    });
}

/// Monta a janela de um registro `VIEWPORT`.
///
/// `None` quando falta o que define a janela — sem centro, largura, altura ou
/// altura de vista não há como situá-la no papel, e inventar um retângulo seria
/// desenhar o que o arquivo não diz.
///
/// # O giro
///
/// O código `51` traz o ângulo em **graus**, e o modelo guarda radianos na
/// convenção fixada no MT-KL-07: anti-horário, medido sobre o conteúdo como ele
/// aparece na folha. É a mesma orientação, então a conversão é só de unidade —
/// e é o teste de sinal do modelo que impede isso de virar suposição.
fn janela(
    pares: &[DxfPair],
    layers: &LayerTable,
    layer_handles: &BTreeMap<String, String>,
    leitura: &mut EntitiesReading,
) -> Option<Viewport> {
    let center = ponto(pares, 10, 20)?;
    let width = real(pares, 40)?;
    let height = real(pares, 41)?;
    let view_center = ponto(pares, 12, 22)?;
    let view_height = real(pares, 45)?;

    if real(pares, 340).is_some() || texto(pares, 340).is_some() {
        leitura.clipped_viewports += 1;
    }

    let mut frozen_layers = BTreeSet::new();

    for handle in todos_os_textos(pares, 331) {
        match layer_handles
            .get(handle.trim())
            .and_then(|nome| layers.id_of(nome))
        {
            Some(id) => {
                frozen_layers.insert(id);
            }
            None => leitura.unresolved_frozen_layers += 1,
        }
    }

    Some(Viewport {
        center,
        width,
        height,
        view_center,
        view_height,
        twist: real(pares, 51).unwrap_or_default().to_radians(),
        // O recorte por entidade não resolve sem mapa de handle para entidade;
        // a contagem acima é o que impede a diferença de passar em silêncio.
        clip: ViewportClip::Window,
        // Código `68` zero é janela desligada. Ausente, a janela é tratada como
        // ligada: mostrar algo que o usuário pode desligar é melhor do que
        // esconder algo que ele não encontra.
        is_on: inteiro(pares, 68) != Some(0),
        frozen_layers,
    })
}

/// Todos os valores textuais de um código, na ordem em que aparecem.
fn todos_os_textos(pares: &[DxfPair], code: u16) -> Vec<&str> {
    pares
        .iter()
        .filter(|par| par.code == code)
        .filter_map(|par| par.value.as_text())
        .collect()
}

/// Traduz um registro para a geometria correspondente do modelo.
///
/// `None` significa "tipo que o modelo não representa" — hachura, cota, spline —,
/// e não erro: o arquivo continua sendo lido e o tipo entra na contagem.
fn geometria(tipo: &str, pares: &[DxfPair]) -> Option<Geometry> {
    match tipo {
        "LINE" => Some(Geometry::Line(Line {
            start: ponto(pares, 10, 20)?,
            end: ponto(pares, 11, 21)?,
        })),
        "CIRCLE" => Some(Geometry::Circle(Circle {
            center: ponto(pares, 10, 20)?,
            radius: real(pares, 40)?,
        })),
        "ARC" => Some(Geometry::Arc(Arc {
            center: ponto(pares, 10, 20)?,
            radius: real(pares, 40)?,
            // O DXF grava ângulo em graus; o modelo trabalha em radianos.
            start_angle: real(pares, 50)?.to_radians(),
            end_angle: real(pares, 51)?.to_radians(),
        })),
        "LWPOLYLINE" => Some(Geometry::Polyline(Polyline {
            vertices: pontos(pares),
            closed: inteiro(pares, 70).is_some_and(|flags| flags & 1 != 0),
        })),
        "TEXT" => Some(Geometry::Text(Text {
            position: ponto(pares, 10, 20)?,
            content: texto(pares, 1).unwrap_or_default().to_owned(),
            height: real(pares, 40)?,
            rotation: real(pares, 50).unwrap_or(0.0).to_radians(),
        })),
        _ => None,
    }
}

/// Nome da camada do registro, com a camada `0` como padrão do formato.
fn nome_da_camada(pares: &[DxfPair]) -> String {
    texto(pares, 8)
        .filter(|nome| !nome.is_empty())
        .unwrap_or("0")
        .to_owned()
}

/// Cor da entidade.
///
/// Os extremos da paleta continuam significando herança, como na camada; a cor
/// verdadeira (`420`) tem precedência sobre o índice, pelo mesmo motivo.
fn cor(pares: &[DxfPair]) -> EntityColor {
    if let Some(bruto) = inteiro(pares, 420) {
        let empacotada = bruto as u32;

        return EntityColor::Explicit(Color::Rgb {
            red: ((empacotada >> 16) & 0xFF) as u8,
            green: ((empacotada >> 8) & 0xFF) as u8,
            blue: (empacotada & 0xFF) as u8,
        });
    }

    match inteiro(pares, 62).map(i64::unsigned_abs) {
        Some(0) => EntityColor::ByBlock,
        Some(indice) => u16::try_from(indice).ok().and_then(Color::from_aci).map_or(
            EntityColor::ByLayer,
            |color| match color {
                Color::ByLayer => EntityColor::ByLayer,
                Color::ByBlock => EntityColor::ByBlock,
                explicita => EntityColor::Explicit(explicita),
            },
        ),
        None => EntityColor::ByLayer,
    }
}

/// Espaço a que o registro pertence.
///
/// O `410` nomeia a aba e tem precedência; o `67` é o sinalizador antigo, sem
/// nome de layout, e só diz "é papel".
fn espaco(pares: &[DxfPair]) -> EntitySpace {
    if let Some(aba) = texto(pares, 410).filter(|aba| !aba.is_empty()) {
        return if aba.eq_ignore_ascii_case("Model") {
            EntitySpace::Model
        } else {
            EntitySpace::Paper(aba.to_owned())
        };
    }

    if inteiro(pares, 67) == Some(1) {
        return EntitySpace::Paper(String::from(DEFAULT_PAPER_SPACE));
    }

    EntitySpace::Model
}

/// Primeiro valor real de um código.
fn real(pares: &[DxfPair], code: u16) -> Option<f64> {
    pares
        .iter()
        .find(|par| par.code == code)
        .and_then(|par| par.value.as_real())
}

/// Primeiro valor inteiro de um código.
fn inteiro(pares: &[DxfPair], code: u16) -> Option<i64> {
    pares
        .iter()
        .find(|par| par.code == code)
        .and_then(|par| par.value.as_integer())
}

/// Primeiro valor textual de um código, aparado.
fn texto(pares: &[DxfPair], code: u16) -> Option<&str> {
    pares
        .iter()
        .find(|par| par.code == code)
        .and_then(|par| par.value.as_text())
        .map(str::trim)
}

/// Ponto formado por dois códigos.
fn ponto(pares: &[DxfPair], x: u16, y: u16) -> Option<Point2> {
    Some(Point2::new(real(pares, x)?, real(pares, y)?))
}

/// Todos os pontos `10`/`20` do registro, na ordem em que aparecem.
///
/// É como a `LWPOLYLINE` entrega seus vértices — repetindo o par de códigos —, e
/// também serve ao `VERTEX`, que traz um ponto por registro.
fn pontos(pares: &[DxfPair]) -> Vec<Point2> {
    let mut pontos = Vec::new();
    let mut x = None;

    for par in pares {
        match par.code {
            10 => x = par.value.as_real(),
            20 => {
                if let (Some(px), Some(py)) = (x.take(), par.value.as_real()) {
                    pontos.push(Point2::new(px, py));
                }
            }
            _ => {}
        }
    }

    pontos
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um registro `VIEWPORT` com os códigos que a janela usa.
    fn viewport<'a>(extras: &[(u16, &'a str)]) -> Vec<(u16, &'a str)> {
        let mut pares = vec![
            (0, "VIEWPORT"),
            (8, "0"),
            (67, "1"),
            (410, "Prancha"),
            (10, "200.0"),
            (20, "150.0"),
            (40, "100.0"),
            (41, "50.0"),
            (68, "1"),
            (69, "2"),
            (12, "10.0"),
            (22, "20.0"),
            (45, "25.0"),
        ];
        pares.extend_from_slice(extras);

        pares
    }

    fn janela_lida(leitura: &EntitiesReading) -> &neocad_model::Viewport {
        match &leitura.entities[0].entity.geometry {
            Geometry::Viewport(janela) => janela,
            outra => panic!("esperava janela, veio {outra:?}"),
        }
    }

    #[test]
    fn le_a_janela_com_medidas_e_escala() {
        let (leitura, _) = ler(&viewport(&[]));
        let janela = janela_lida(&leitura);

        assert_eq!(janela.center, Point2::new(200.0, 150.0));
        assert_eq!(janela.width, 100.0);
        assert_eq!(janela.height, 50.0);
        assert_eq!(janela.view_center, Point2::new(10.0, 20.0));
        assert_eq!(janela.view_height, 25.0);
        // A escala é derivada: 50 de janela sobre 25 de vista.
        assert_eq!(janela.scale(), Some(2.0));
        assert!(janela.is_on);
    }

    #[test]
    fn a_janela_fica_no_espaco_papel_da_aba() {
        let (leitura, _) = ler(&viewport(&[]));

        assert_eq!(
            leitura.entities[0].space,
            EntitySpace::Paper(String::from("Prancha"))
        );
    }

    #[test]
    fn o_giro_vem_em_graus_e_e_guardado_em_radianos() {
        // O código `51` grava graus; a convenção do modelo é radianos
        // anti-horários, fixada no MT-KL-07.
        let (leitura, _) = ler(&viewport(&[(51, "90.0")]));

        let diferenca = (janela_lida(&leitura).twist - core::f64::consts::FRAC_PI_2).abs();
        assert!(
            diferenca < 1e-12,
            "giro lido: {}",
            janela_lida(&leitura).twist
        );
    }

    #[test]
    fn o_codigo_68_zero_desliga_a_janela() {
        let mut pares = viewport(&[]);
        let posicao = pares.iter().position(|(c, _)| *c == 68).expect("há 68");
        pares[posicao] = (68, "0");

        let (leitura, _) = ler(&pares);

        assert!(!janela_lida(&leitura).is_on);
    }

    #[test]
    fn janela_sem_o_codigo_68_e_tratada_como_ligada() {
        // Mostrar algo que o usuário pode desligar é melhor do que esconder algo
        // que ele não encontra.
        let pares: Vec<(u16, &str)> = viewport(&[])
            .into_iter()
            .filter(|(codigo, _)| *codigo != 68)
            .collect();

        let (leitura, _) = ler(&pares);

        assert!(janela_lida(&leitura).is_on);
    }

    #[test]
    fn o_viewport_de_identificador_1_e_a_folha_e_nao_vira_janela() {
        // O critério de aceite. Todo espaço-papel tem um, e transformá-lo em
        // entidade criaria uma moldura fantasma do tamanho da prancha sobre tudo.
        let mut pares = viewport(&[]);
        let posicao = pares.iter().position(|(c, _)| *c == 69).expect("há 69");
        pares[posicao] = (69, "1");

        let (leitura, _) = ler(&pares);

        assert!(leitura.entities.is_empty());
        // Compreendido e deliberadamente não modelado: não é "não suportado".
        assert!(leitura.unsupported.is_empty());
    }

    #[test]
    fn as_camadas_congeladas_saem_do_codigo_331() {
        // O `331` aponta por handle; sem o mapa da tabela de camadas ele não
        // resolve para camada nenhuma.
        let mut camadas = LayerTable::new();
        let cotas = camadas.create("Cotas").expect("nome válido");
        let eixos = camadas.create("Eixos").expect("nome válido");

        let handles: BTreeMap<String, String> = [
            (String::from("A1"), String::from("Cotas")),
            (String::from("A2"), String::from("Eixos")),
        ]
        .into_iter()
        .collect();

        let leitura = read_entities(
            &secao(&viewport(&[(331, "A1"), (331, "A2")])),
            &mut camadas,
            &handles,
        );

        let janela = janela_lida(&leitura);
        assert!(janela.is_layer_frozen(cotas));
        assert!(janela.is_layer_frozen(eixos));
        assert_eq!(janela.frozen_layers.len(), 2);
        assert_eq!(leitura.unresolved_frozen_layers, 0);
    }

    #[test]
    fn handle_de_camada_que_nao_resolve_e_contado() {
        let (leitura, _) = ler(&viewport(&[(331, "handle-perdido")]));

        assert!(janela_lida(&leitura).frozen_layers.is_empty());
        assert_eq!(leitura.unresolved_frozen_layers, 1);
    }

    #[test]
    fn o_recorte_por_entidade_e_contado_em_vez_de_sumir() {
        // O `340` aponta para a entidade que delimita a vista, e a leitura não
        // mantém mapa de handle para entidade. A janela sai retangular, e a
        // contagem é o que impede a diferença de passar em silêncio: prancha com
        // recorte perdido mostra mais do que deveria.
        let (leitura, _) = ler(&viewport(&[(340, "2F")]));

        assert_eq!(leitura.clipped_viewports, 1);
        assert_eq!(
            janela_lida(&leitura).clip,
            neocad_model::ViewportClip::Window
        );
    }

    #[test]
    fn janela_sem_medidas_e_contada_como_nao_representada() {
        // Sem centro, largura, altura ou altura de vista não há como situá-la no
        // papel, e inventar um retângulo seria desenhar o que o arquivo não diz.
        let pares: Vec<(u16, &str)> = viewport(&[])
            .into_iter()
            .filter(|(codigo, _)| *codigo != 45)
            .collect();

        let (leitura, _) = ler(&pares);

        assert!(leitura.entities.is_empty());
        assert_eq!(leitura.unsupported.get("VIEWPORT"), Some(&1));
    }
    use crate::sections;

    /// Monta uma seção `ENTITIES` a partir de pares.
    fn secao(pares: &[(u16, &str)]) -> Section {
        let mut texto = String::from("  0\nSECTION\n  2\nENTITIES\n");

        for (codigo, valor) in pares {
            texto.push_str(&format!("{codigo:>3}\n{valor}\n"));
        }

        texto.push_str("  0\nENDSEC\n  0\nEOF\n");

        sections(texto.as_bytes())
            .next()
            .expect("há seção")
            .expect("bem formada")
    }

    fn ler(pares: &[(u16, &str)]) -> (EntitiesReading, LayerTable) {
        let mut camadas = LayerTable::new();
        let leitura = read_entities(&secao(pares), &mut camadas, &BTreeMap::new());

        (leitura, camadas)
    }

    /// Lê uma das fixtures sintéticas do E2E, que é o critério de aceite.
    fn ler_fixture(nome: &str) -> EntitiesReading {
        let caminho = format!("{}/../../e2e/fixtures/{nome}", env!("CARGO_MANIFEST_DIR"));
        let bytes = std::fs::read(&caminho).expect("fixture existe");
        let mut camadas = LayerTable::new();
        let mut leitura = EntitiesReading::vazia();

        for secao in sections(&bytes) {
            let secao = secao.expect("fixture bem formada");

            if secao.kind == SectionKind::Entities {
                leitura = read_entities(&secao, &mut camadas, &BTreeMap::new());
            }
        }

        leitura
    }

    fn geometrias(leitura: &EntitiesReading) -> Vec<&Geometry> {
        leitura
            .entities
            .iter()
            .map(|e| &e.entity.geometry)
            .collect()
    }

    #[test]
    fn le_segmento_de_reta() {
        let (leitura, _) = ler(&[
            (0, "LINE"),
            (8, "Eixos"),
            (10, "0.0"),
            (20, "1.0"),
            (11, "10.0"),
            (21, "11.0"),
        ]);

        assert_eq!(
            geometrias(&leitura),
            [&Geometry::Line(Line {
                start: Point2::new(0.0, 1.0),
                end: Point2::new(10.0, 11.0)
            })]
        );
    }

    #[test]
    fn le_circunferencia_e_arco() {
        let (leitura, _) = ler(&[
            (0, "CIRCLE"),
            (10, "5.0"),
            (20, "5.0"),
            (40, "2.5"),
            (0, "ARC"),
            (10, "1.0"),
            (20, "2.0"),
            (40, "3.0"),
            (50, "0.0"),
            (51, "90.0"),
        ]);

        let lidas = geometrias(&leitura);
        assert_eq!(
            lidas[0],
            &Geometry::Circle(Circle {
                center: Point2::new(5.0, 5.0),
                radius: 2.5
            })
        );

        // O DXF grava graus; o modelo guarda radianos.
        let Geometry::Arc(arco) = lidas[1] else {
            panic!("o segundo é arco");
        };
        assert!((arco.start_angle - 0.0).abs() < 1e-12);
        assert!((arco.end_angle - core::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }

    #[test]
    fn le_texto_com_rotacao_em_graus() {
        let (leitura, _) = ler(&[
            (0, "TEXT"),
            (10, "1.0"),
            (20, "2.0"),
            (40, "2.5"),
            (1, "Fiação"),
            (50, "180.0"),
        ]);

        let Geometry::Text(texto) = &leitura.entities[0].entity.geometry else {
            panic!("é texto");
        };
        assert_eq!(texto.content, "Fiação");
        assert_eq!(texto.height, 2.5);
        assert!((texto.rotation - core::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn le_polilinha_leve_com_varios_vertices() {
        let (leitura, _) = ler(&[
            (0, "LWPOLYLINE"),
            (90, "3"),
            (70, "1"),
            (10, "0.0"),
            (20, "0.0"),
            (10, "5.0"),
            (20, "0.0"),
            (10, "5.0"),
            (20, "5.0"),
        ]);

        assert_eq!(
            geometrias(&leitura),
            [&Geometry::Polyline(Polyline {
                vertices: vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(5.0, 0.0),
                    Point2::new(5.0, 5.0)
                ],
                closed: true
            })]
        );
    }

    #[test]
    fn le_polilinha_de_estilo_antigo_com_vertices_separados() {
        let (leitura, _) = ler(&[
            (0, "POLYLINE"),
            (8, "Antiga"),
            (66, "1"),
            (70, "0"),
            (0, "VERTEX"),
            (10, "0.0"),
            (20, "0.0"),
            (0, "VERTEX"),
            (10, "50.0"),
            (20, "20.0"),
            (0, "SEQEND"),
        ]);

        assert_eq!(leitura.entities.len(), 1);
        assert_eq!(
            geometrias(&leitura),
            [&Geometry::Polyline(Polyline {
                vertices: vec![Point2::new(0.0, 0.0), Point2::new(50.0, 20.0)],
                closed: false
            })]
        );
    }

    #[test]
    fn polilinha_antiga_sem_seqend_nao_perde_os_vertices() {
        // Arquivo torto não é motivo para o desenho sumir.
        let (leitura, _) = ler(&[(0, "POLYLINE"), (0, "VERTEX"), (10, "1.0"), (20, "1.0")]);

        assert_eq!(leitura.entities.len(), 1);
    }

    #[test]
    fn polilinha_antiga_seguida_de_outra_entidade_e_fechada() {
        let (leitura, _) = ler(&[
            (0, "POLYLINE"),
            (0, "VERTEX"),
            (10, "1.0"),
            (20, "1.0"),
            (0, "LINE"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
        ]);

        assert_eq!(leitura.entities.len(), 2);
        assert!(matches!(
            leitura.entities[0].entity.geometry,
            Geometry::Polyline(_)
        ));
        assert!(matches!(
            leitura.entities[1].entity.geometry,
            Geometry::Line(_)
        ));
    }

    #[test]
    fn tipo_nao_modelado_e_contado_e_nao_interrompe() {
        let (leitura, _) = ler(&[
            (0, "HATCH"),
            (8, "0"),
            (0, "DIMENSION"),
            (8, "0"),
            (0, "HATCH"),
            (8, "0"),
            (0, "LINE"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
        ]);

        assert_eq!(leitura.unsupported.get("HATCH"), Some(&2));
        assert_eq!(leitura.unsupported.get("DIMENSION"), Some(&1));
        assert_eq!(leitura.unsupported_count(), 3);
        assert_eq!(leitura.entities.len(), 1);
    }

    #[test]
    fn entidade_de_espaco_papel_e_separada_pelo_codigo_67() {
        let (leitura, _) = ler(&[
            (0, "LINE"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
            (0, "LINE"),
            (67, "1"),
            (10, "2.0"),
            (20, "2.0"),
            (11, "3.0"),
            (21, "3.0"),
        ]);

        assert_eq!(leitura.entities.len(), 2);
        assert_eq!(leitura.entities[0].space, EntitySpace::Model);
        assert_eq!(
            leitura.entities[1].space,
            EntitySpace::Paper(String::from(DEFAULT_PAPER_SPACE))
        );
        assert_eq!(leitura.model_space_count(), 1);
    }

    #[test]
    fn aba_do_codigo_410_nomeia_o_layout() {
        let (leitura, _) = ler(&[
            (0, "LINE"),
            (67, "1"),
            (410, "Prancha A1"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
        ]);

        assert_eq!(
            leitura.entities[0].space,
            EntitySpace::Paper(String::from("Prancha A1"))
        );
    }

    #[test]
    fn aba_chamada_model_e_espaco_modelo() {
        // O `410` do espaço-modelo é literalmente "Model"; tratá-lo como papel
        // mandaria o desenho inteiro para um layout inexistente.
        let (leitura, _) = ler(&[
            (0, "LINE"),
            (410, "Model"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
        ]);

        assert_eq!(leitura.entities[0].space, EntitySpace::Model);
    }

    #[test]
    fn polilinha_antiga_herda_o_espaco_do_registro_que_a_abre() {
        let (leitura, _) = ler(&[
            (0, "POLYLINE"),
            (67, "1"),
            (410, "Prancha"),
            (0, "VERTEX"),
            (10, "1.0"),
            (20, "1.0"),
            (0, "SEQEND"),
        ]);

        assert_eq!(
            leitura.entities[0].space,
            EntitySpace::Paper(String::from("Prancha"))
        );
    }

    #[test]
    fn camada_citada_e_ausente_e_criada_e_registrada() {
        // É o que o AutoCAD faz. Descartar a entidade perderia desenho por causa
        // de inconsistência que o próprio formato tolera.
        let (leitura, camadas) = ler(&[
            (0, "LINE"),
            (8, "Inexistente"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
        ]);

        assert_eq!(leitura.created_layers, ["Inexistente"]);
        assert_eq!(leitura.entities.len(), 1);
        assert!(camadas.get_by_name("Inexistente").is_some());
    }

    #[test]
    fn camada_de_nome_impossivel_e_relatada() {
        let (leitura, _) = ler(&[
            (0, "LINE"),
            (8, "Proi/bida"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
        ]);

        assert!(leitura.entities.is_empty());
        assert_eq!(leitura.rejected.len(), 1);
        assert_eq!(leitura.rejected[0].layer_name, "Proi/bida");
        assert_eq!(leitura.rejected[0].entity_type, "LINE");
    }

    #[test]
    fn entidade_sem_camada_cai_na_camada_zero() {
        let (leitura, _) = ler(&[
            (0, "LINE"),
            (10, "0.0"),
            (20, "0.0"),
            (11, "1.0"),
            (21, "1.0"),
        ]);

        assert!(leitura.created_layers.is_empty());
        assert_eq!(leitura.entities.len(), 1);
    }

    #[test]
    fn cor_da_entidade_cobre_heranca_indice_e_cor_verdadeira() {
        let (leitura, _) = ler(&[
            (0, "CIRCLE"),
            (10, "0.0"),
            (20, "0.0"),
            (40, "1.0"),
            (62, "0"),
            (0, "CIRCLE"),
            (10, "0.0"),
            (20, "0.0"),
            (40, "1.0"),
            (62, "256"),
            (0, "CIRCLE"),
            (10, "0.0"),
            (20, "0.0"),
            (40, "1.0"),
            (62, "3"),
            (0, "CIRCLE"),
            (10, "0.0"),
            (20, "0.0"),
            (40, "1.0"),
            (420, "255"),
        ]);

        let cores: Vec<_> = leitura.entities.iter().map(|e| e.entity.color).collect();
        assert_eq!(
            cores,
            [
                EntityColor::ByBlock,
                EntityColor::ByLayer,
                EntityColor::Explicit(Color::Index(3)),
                EntityColor::Explicit(Color::Rgb {
                    red: 0,
                    green: 0,
                    blue: 255
                }),
            ]
        );
    }

    #[test]
    fn geometria_incompleta_nao_vira_entidade_torta() {
        // Círculo sem raio não pode virar círculo de raio zero em silêncio.
        let (leitura, _) = ler(&[(0, "CIRCLE"), (10, "1.0"), (20, "1.0")]);

        assert!(leitura.entities.is_empty());
        assert_eq!(leitura.unsupported.get("CIRCLE"), Some(&1));
    }

    #[test]
    fn secao_que_nao_e_entities_devolve_leitura_vazia() {
        let texto =
            "  0\nSECTION\n  2\nBLOCKS\n  0\nLINE\n 10\n0.0\n 20\n0.0\n  0\nENDSEC\n  0\nEOF\n";
        let secao = sections(texto.as_bytes())
            .next()
            .expect("há seção")
            .expect("bem formada");
        let mut camadas = LayerTable::new();

        let leitura = read_entities(&secao, &mut camadas, &BTreeMap::new());

        assert!(leitura.entities.is_empty());
    }

    #[test]
    fn fixture_minimal_entrega_a_reta_e_a_circunferencia() {
        let leitura = ler_fixture("minimal.dxf");

        assert_eq!(leitura.entities.len(), 2);
        assert_eq!(leitura.model_space_count(), 2);
        assert!(matches!(
            leitura.entities[0].entity.geometry,
            Geometry::Line(_)
        ));
        assert!(matches!(
            leitura.entities[1].entity.geometry,
            Geometry::Circle(_)
        ));
        assert!(leitura.unsupported.is_empty());
    }

    #[test]
    fn fixture_com_entidade_nao_modelada_abre_assim_mesmo() {
        let leitura = ler_fixture("with-unsupported.dxf");

        assert_eq!(leitura.entities.len(), 1);
        assert_eq!(leitura.unsupported.get("SOLID"), Some(&1));
    }

    #[test]
    fn fixture_de_polilinha_antiga_entrega_os_tres_vertices() {
        // É o construto que motivou a bissecção do desenho que não abria.
        let leitura = ler_fixture("legacy-polyline.dxf");

        assert_eq!(leitura.entities.len(), 3);
        let Geometry::Polyline(polilinha) = &leitura.entities[2].entity.geometry else {
            panic!("a terceira é polilinha");
        };
        assert_eq!(
            polilinha.vertices,
            [
                Point2::new(0.0, 0.0),
                Point2::new(50.0, 20.0),
                Point2::new(80.0, 60.0)
            ]
        );
    }
}
