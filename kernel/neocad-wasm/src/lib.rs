// Caminho relativo: kernel/neocad-wasm/src/lib.rs
//! \file kernel/neocad-wasm/src/lib.rs
//! \brief Fachada WebAssembly do kernel CAD do NeoCAD.
//! \author Iago Leal
//! \date 2026-08-07
//!
//! Responsabilidade: expor ao frontend a criação de documento, a consulta de
//! camadas e entidades, a execução de edições como transações nomeadas e o
//! controle de `undo`/`redo`.
//!
//! É a única crate do kernel autorizada a conhecer o ambiente de execução do
//! frontend. As demais permanecem agnósticas, conforme o ADR 0003.
//!
//! # Fronteira de tipos
//!
//! JavaScript não tem os tipos do kernel, então nada atravessa a ponte por
//! referência: as consultas devolvem cópias serializadas e os identificadores
//! viajam como **texto decimal** de [`neocad_model::EntityId::to_bits`]. Texto,
//! e não número, porque um `u64` viraria `BigInt` no lado JavaScript, o que
//! complica comparação e serialização sem nenhum ganho — o identificador é
//! opaco de qualquer modo.

use neocad_geometry::{Aabb, Point2};
use neocad_io::{
    build_document, read_dxf, write_dxf, BlockDefinition, DocumentBuild, DxfContents, EntitySpace,
    LayoutDefinition, ReadEntity,
};
use neocad_model::{
    Arc, Circle, Color, Document, Entity, EntityId, Geometry, LayerId, LayerRecord, Line, Polyline,
    Text, Viewport, ViewportClip,
};
use neocad_transaction::CommandStack;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Ponto, na forma que atravessa a ponte.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PointView {
    /// Coordenada X.
    pub x: f64,
    /// Coordenada Y.
    pub y: f64,
}

impl From<Point2> for PointView {
    fn from(point: Point2) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

/// Caixa envolvente, na forma que atravessa a ponte.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundsView {
    /// Menor X.
    pub min_x: f64,
    /// Menor Y.
    pub min_y: f64,
    /// Maior X.
    pub max_x: f64,
    /// Maior Y.
    pub max_y: f64,
}

impl From<Aabb> for BoundsView {
    fn from(bounds: Aabb) -> Self {
        Self {
            min_x: bounds.min().x,
            min_y: bounds.min().y,
            max_x: bounds.max().x,
            max_y: bounds.max().y,
        }
    }
}

/// Cor, na forma que atravessa a ponte.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ColorView {
    /// Herda a cor do bloco.
    ByBlock,
    /// Herda a cor da camada.
    ByLayer,
    /// Índice na paleta ACI, de 1 a 255.
    Index {
        /// Índice.
        index: u8,
    },
    /// Cor verdadeira.
    Rgb {
        /// Componente vermelha.
        red: u8,
        /// Componente verde.
        green: u8,
        /// Componente azul.
        blue: u8,
    },
}

impl From<Color> for ColorView {
    fn from(color: Color) -> Self {
        match color {
            Color::ByBlock => Self::ByBlock,
            Color::ByLayer => Self::ByLayer,
            Color::Index(index) => Self::Index { index },
            Color::Rgb { red, green, blue } => Self::Rgb { red, green, blue },
        }
    }
}

/// Camada, na forma que atravessa a ponte.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerView {
    /// Identificador opaco, em texto decimal.
    pub id: String,
    /// Nome de exibição.
    pub name: String,
    /// Cor.
    pub color: ColorView,
    /// Se as entidades da camada são desenhadas.
    pub visible: bool,
    /// Se a camada está desligada.
    pub off: bool,
    /// Se a camada está congelada.
    pub frozen: bool,
    /// Se a camada está bloqueada.
    pub locked: bool,
}

/// Geometria, na forma que atravessa a ponte.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GeometryView {
    /// Segmento de reta.
    Line {
        /// Ponto inicial.
        start: PointView,
        /// Ponto final.
        end: PointView,
    },
    /// Circunferência.
    Circle {
        /// Centro.
        center: PointView,
        /// Raio.
        radius: f64,
    },
    /// Arco.
    #[serde(rename_all = "camelCase")]
    Arc {
        /// Centro.
        center: PointView,
        /// Raio.
        radius: f64,
        /// Ângulo inicial, em radianos.
        start_angle: f64,
        /// Ângulo final, em radianos.
        end_angle: f64,
    },
    /// Polilinha.
    Polyline {
        /// Vértices, em ordem de percurso.
        vertices: Vec<PointView>,
        /// Se há segmento ligando o último vértice ao primeiro.
        closed: bool,
    },
    /// Texto.
    Text {
        /// Ponto de inserção.
        position: PointView,
        /// Conteúdo.
        content: String,
        /// Altura dos caracteres.
        height: f64,
        /// Rotação, em radianos.
        rotation: f64,
    },
    /// Janela de espaço-papel que mostra uma vista do espaço-modelo.
    Viewport {
        /// Centro da janela, no papel.
        center: PointView,
        /// Largura da janela no papel.
        width: f64,
        /// Altura da janela no papel.
        height: f64,
        /// Ponto do modelo que aparece no centro da janela.
        view_center: PointView,
        /// Altura da vista, em unidades do modelo.
        view_height: f64,
        /// Giro do conteúdo na folha, em radianos.
        twist: f64,
        /// Escala da janela, quando existe.
        ///
        /// Derivada, e não guardada: é a razão entre a altura da janela e a da
        /// vista. Atravessa a ponte pronta porque quem desenha o carimbo precisa
        /// dela, e recalcular do outro lado convidaria as duas contas a
        /// divergirem.
        scale: Option<f64>,
        /// Janela desligada não mostra nada, mas ocupa lugar na folha.
        is_on: bool,
        /// Camadas congeladas nesta janela, em texto decimal.
        ///
        /// Sai do kernel para que a interface possa dizer **por que** algo não
        /// aparece nesta prancha. Não volta: identificador de camada só faz
        /// sentido dentro do documento que o emitiu, e aceitá-lo de fora
        /// congelaria camada errada.
        frozen_layers: Vec<String>,
    },
}

impl From<&Geometry> for GeometryView {
    fn from(geometry: &Geometry) -> Self {
        match geometry {
            Geometry::Line(line) => Self::Line {
                start: line.start.into(),
                end: line.end.into(),
            },
            Geometry::Circle(circle) => Self::Circle {
                center: circle.center.into(),
                radius: circle.radius,
            },
            Geometry::Arc(arc) => Self::Arc {
                center: arc.center.into(),
                radius: arc.radius,
                start_angle: arc.start_angle,
                end_angle: arc.end_angle,
            },
            Geometry::Polyline(polyline) => Self::Polyline {
                vertices: polyline.vertices.iter().copied().map(Into::into).collect(),
                closed: polyline.closed,
            },
            Geometry::Text(text) => Self::Text {
                position: text.position.into(),
                content: text.content.clone(),
                height: text.height,
                rotation: text.rotation,
            },
            Geometry::Viewport(viewport) => Self::Viewport {
                center: viewport.center.into(),
                width: viewport.width,
                height: viewport.height,
                view_center: viewport.view_center.into(),
                view_height: viewport.view_height,
                twist: viewport.twist,
                scale: viewport.scale(),
                is_on: viewport.is_on,
                frozen_layers: viewport
                    .frozen_layers
                    .iter()
                    .copied()
                    .map(encode_layer)
                    .collect(),
            },
        }
    }
}

impl From<PointView> for Point2 {
    fn from(point: PointView) -> Self {
        Self::new(point.x, point.y)
    }
}

impl From<ColorView> for Color {
    fn from(color: ColorView) -> Self {
        match color {
            ColorView::ByBlock => Self::ByBlock,
            ColorView::ByLayer => Self::ByLayer,
            ColorView::Index { index } => Self::Index(index),
            ColorView::Rgb { red, green, blue } => Self::Rgb { red, green, blue },
        }
    }
}

impl From<GeometryView> for Geometry {
    fn from(geometry: GeometryView) -> Self {
        match geometry {
            GeometryView::Line { start, end } => Self::Line(Line {
                start: start.into(),
                end: end.into(),
            }),
            GeometryView::Circle { center, radius } => Self::Circle(Circle {
                center: center.into(),
                radius,
            }),
            GeometryView::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            } => Self::Arc(Arc {
                center: center.into(),
                radius,
                start_angle,
                end_angle,
            }),
            GeometryView::Polyline { vertices, closed } => Self::Polyline(Polyline {
                vertices: vertices.into_iter().map(Into::into).collect(),
                closed,
            }),
            GeometryView::Text {
                position,
                content,
                height,
                rotation,
            } => Self::Text(Text {
                position: position.into(),
                content,
                height,
                rotation,
            }),
            // A escala que atravessa a ponte é derivada, e por isso é descartada
            // na volta: recebê-la de fora e guardá-la abriria a porta para uma
            // janela cuja escala declarada não bate com a desenhada.
            GeometryView::Viewport {
                center,
                width,
                height,
                view_center,
                view_height,
                twist,
                scale: _,
                is_on,
                frozen_layers: _,
            } => Self::Viewport(Viewport {
                center: center.into(),
                width,
                height,
                view_center: view_center.into(),
                view_height,
                twist,
                clip: ViewportClip::Window,
                is_on,
                // O congelamento não volta pela ponte: identificador de camada só
                // vale dentro do documento que o emitiu.
                frozen_layers: std::collections::BTreeSet::new(),
            }),
        }
    }
}

/// Camada de um documento a carregar.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerInput {
    /// Nome da camada.
    pub name: String,
    /// Cor.
    pub color: ColorView,
    /// Se a camada está desligada.
    pub off: bool,
    /// Se a camada está congelada.
    pub frozen: bool,
    /// Se a camada está bloqueada.
    pub locked: bool,
}

/// Entidade de um documento a carregar.
///
/// Referencia a camada por **nome**, e não por identificador: quem produz o
/// documento é um parser externo, que não conhece os identificadores que este
/// kernel ainda vai emitir. É também como o próprio DXF representa a relação.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityInput {
    /// Nome da camada a que a entidade pertence.
    pub layer_name: String,
    /// Geometria.
    pub geometry: GeometryView,
}

/// Documento a carregar no kernel.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInput {
    /// Camadas.
    pub layers: Vec<LayerInput>,
    /// Entidades, na ordem de desenho.
    pub entities: Vec<EntityInput>,
}

/// Resultado de um carregamento.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadReport {
    /// Camadas presentes no documento após o carregamento.
    pub layer_count: usize,
    /// Entidades carregadas.
    pub entity_count: usize,
    /// Entidades recusadas por referenciarem camada inexistente.
    pub skipped_count: usize,
}

/// Um tipo de entidade que o modelo ainda não representa, com sua contagem.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsupportedView {
    /// Tipo como veio do arquivo.
    pub entity_type: String,
    /// Quantas vezes apareceu.
    pub count: usize,
}

/// Um layout de espaço-papel encontrado no arquivo.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperSpaceView {
    /// Nome da aba.
    pub name: String,
    /// Entidades que moram nela.
    pub entity_count: usize,
}

/// O que uma gravação **descartaria** do que foi lido.
///
/// Existe para que a perda apareça antes de acontecer. Salvar por cima de um
/// original sem avisar que a prancha não vai junto é destruição silenciosa de
/// trabalho alheio — o ADR 0005 proíbe, e este relatório é o que permite
/// cumprir a proibição na interface.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLossView {
    /// Entidades de tipo que o modelo não representa, por tipo.
    pub unsupported: Vec<UnsupportedView>,
    /// Total delas.
    pub unsupported_count: usize,
    /// Layouts de espaço-papel, que o documento ainda não guarda (fase KL).
    pub paper_space: Vec<PaperSpaceView>,
    /// Total de entidades em espaço-papel.
    pub paper_space_count: usize,
    /// Blocos que são referência externa, cujo caminho o modelo não guarda.
    pub xref_count: usize,
    /// Verdadeiro quando nada se perderia.
    pub is_lossless: bool,
}

impl Default for SaveLossView {
    /// Sessão sem arquivo aberto não perde nada.
    ///
    /// Escrito à mão porque o `bool` padrão é `false`, e derivar faria uma
    /// sessão recém-criada anunciar perda que não existe — o tipo de defeito
    /// que leva o usuário a desconfiar de um aviso verdadeiro depois.
    fn default() -> Self {
        Self {
            unsupported: Vec::new(),
            unsupported_count: 0,
            paper_space: Vec::new(),
            paper_space_count: 0,
            xref_count: 0,
            is_lossless: true,
        }
    }
}

/// Resultado da abertura de um DXF.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DxfOpenReport {
    /// Camadas do documento após a abertura.
    pub layer_count: usize,
    /// Entidades de espaço-modelo carregadas.
    pub entity_count: usize,
    /// Entidades recusadas por referenciarem camada inexistente.
    pub skipped_count: usize,
    /// Definições de bloco lidas.
    pub block_count: usize,
    /// Entidades dentro de definições de bloco.
    pub block_entity_count: usize,
    /// Camadas que o arquivo citava sem definir, criadas na leitura.
    pub created_layers: Vec<String>,
    /// Falhas locais de percurso, que não impediram a abertura.
    pub errors: Vec<String>,
    /// O que uma gravação descartaria.
    pub loss: SaveLossView,
    /// Resumo de uma linha, pronto para mensagem de interface.
    pub summary: String,
}

/// Entidade, na forma que atravessa a ponte.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityView {
    /// Identificador opaco, em texto decimal.
    pub id: String,
    /// Identificador da camada, em texto decimal.
    pub layer: String,
    /// Geometria.
    pub geometry: GeometryView,
    /// Caixa envolvente.
    pub bounds: BoundsView,
}

/// Estado da pilha de comandos, na forma que atravessa a ponte.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryView {
    /// Se há ação a desfazer.
    pub can_undo: bool,
    /// Se há ação a refazer.
    pub can_redo: bool,
    /// Nome da ação que seria desfeita.
    pub undo_name: Option<String>,
    /// Nome da ação que seria refeita.
    pub redo_name: Option<String>,
    /// Quantidade de ações desfazíveis.
    pub undo_depth: usize,
    /// Quantidade de ações refazíveis.
    pub redo_depth: usize,
}

/// Sessão de edição: um documento e o seu histórico.
///
/// Documento e pilha andam juntos porque separá-los permitiria desfazer contra o
/// documento errado. Do lado JavaScript existe um objeto só.
///
/// # Uso
///
/// ```js
/// import init, { CadSession } from '$lib/kernel/pkg/neocad_wasm.js';
///
/// await init();
/// const session = new CadSession();
/// const layer = session.createLayer('Parede');
/// session.addLine(layer, 0, 0, 100, 0);
///
/// session.entities();  // [{ id, layer, geometry, bounds }]
/// session.undo();      // desfaz o traçado da linha
/// ```
#[wasm_bindgen]
#[derive(Debug)]
pub struct CadSession {
    document: Document,
    stack: CommandStack,
    /// O que a última abertura de DXF traria a perder numa gravação.
    ///
    /// Guardado na sessão porque a pergunta "o que eu perco ao salvar?" é feita
    /// no momento de salvar, não no de abrir.
    loss: SaveLossView,
}

#[wasm_bindgen]
impl CadSession {
    /// Cria uma sessão com documento vazio e histórico limpo.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            stack: CommandStack::new(),
            loss: SaveLossView::default(),
        }
    }

    /// Camadas do documento, em ordem alfabética.
    ///
    /// # Errors
    ///
    /// Falha se a serialização para JavaScript falhar.
    pub fn layers(&self) -> Result<JsValue, JsError> {
        let layers: Vec<LayerView> = self
            .document
            .layers()
            .iter()
            .map(|(id, record)| layer_view(id, record))
            .collect();

        to_js(&layers)
    }

    /// Entidades do espaço-modelo, na ordem de desenho.
    ///
    /// # Errors
    ///
    /// Falha se a serialização para JavaScript falhar.
    pub fn entities(&self) -> Result<JsValue, JsError> {
        let entities: Vec<EntityView> = self
            .document
            .entities_in_block(self.document.model_space())
            .map(|(id, entity)| entity_view(id, entity))
            .collect();

        to_js(&entities)
    }

    /// Quantidade de entidades do documento, somando todos os blocos.
    #[wasm_bindgen(js_name = entityCount)]
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.document.entity_count()
    }

    /// Caixa envolvente do espaço-modelo, ou `null` se não houver entidades.
    ///
    /// É o que o ajuste de vista consome.
    ///
    /// # Errors
    ///
    /// Falha se a serialização para JavaScript falhar.
    #[wasm_bindgen(js_name = boundingBox)]
    pub fn bounding_box(&self) -> Result<JsValue, JsError> {
        let bounds = self
            .document
            .block_bounding_box(self.document.model_space())
            .map(BoundsView::from);

        to_js(&bounds)
    }

    /// Cria uma camada e devolve seu identificador.
    ///
    /// # Errors
    ///
    /// Falha se o nome for inválido ou já estiver em uso.
    #[wasm_bindgen(js_name = createLayer)]
    pub fn create_layer(&mut self, name: &str) -> Result<String, JsError> {
        self.try_create_layer(name).map_err(js_error)
    }

    /// Desenha um segmento de reta, como uma ação desfazível.
    ///
    /// # Errors
    ///
    /// Falha se a camada não existir.
    #[wasm_bindgen(js_name = addLine)]
    pub fn add_line(
        &mut self,
        layer: &str,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    ) -> Result<String, JsError> {
        self.try_add_line(layer, start_x, start_y, end_x, end_y)
            .map_err(js_error)
    }

    /// Apaga uma entidade, como uma ação desfazível.
    ///
    /// # Errors
    ///
    /// Falha se a entidade não existir.
    #[wasm_bindgen(js_name = removeEntity)]
    pub fn remove_entity(&mut self, entity: &str) -> Result<(), JsError> {
        self.try_remove_entity(entity).map_err(js_error)
    }

    /// Liga ou desliga uma camada, como uma ação desfazível.
    ///
    /// # Errors
    ///
    /// Falha se a camada não existir.
    #[wasm_bindgen(js_name = setLayerOff)]
    pub fn set_layer_off(&mut self, layer: &str, off: bool) -> Result<(), JsError> {
        self.try_set_layer_off(layer, off).map_err(js_error)
    }

    /// Desfaz a última ação. Devolve `false` se não houver o que desfazer.
    ///
    /// # Errors
    ///
    /// Falha se a transação não puder ser aplicada.
    pub fn undo(&mut self) -> Result<bool, JsError> {
        self.try_undo().map_err(js_error)
    }

    /// Refaz a última ação desfeita. Devolve `false` se não houver o que refazer.
    ///
    /// # Errors
    ///
    /// Falha se a transação não puder ser aplicada.
    pub fn redo(&mut self) -> Result<bool, JsError> {
        self.try_redo().map_err(js_error)
    }

    /// Substitui o documento pelo conteúdo informado, zerando o histórico.
    ///
    /// É o caminho de abertura de arquivo: um parser externo produz camadas e
    /// entidades, e o kernel passa a ser a fonte de verdade sobre elas. O
    /// histórico é zerado porque desfazer para antes da abertura não faz
    /// sentido.
    ///
    /// Entidades que referenciam camada inexistente são **contadas e
    /// ignoradas**, não abortam o carregamento: um arquivo real não pode deixar
    /// de abrir por causa de uma entidade defeituosa.
    ///
    /// # Errors
    ///
    /// Falha se o documento não puder ser interpretado ou se uma camada tiver
    /// nome inválido.
    pub fn load(&mut self, document: JsValue) -> Result<JsValue, JsError> {
        let input: DocumentInput = serde_wasm_bindgen::from_value(document)
            .map_err(|error| js_error(error.to_string()))?;
        let report = self.try_load(input).map_err(js_error)?;

        to_js(&report)
    }

    /// Abre um DXF a partir dos bytes do arquivo, substituindo o documento.
    ///
    /// O histórico é zerado: desfazer para antes da abertura não faz sentido.
    ///
    /// Só o **espaço-modelo** entra no documento. As entidades de espaço-papel
    /// são lidas, contadas e relatadas, mas ainda não têm onde morar — os blocos
    /// `*Paper_Space*` chegam na fase KL. Elas aparecem em
    /// [`DxfOpenReport::loss`] justamente para que a interface possa dizer ao
    /// usuário que a prancha dele existe e ainda não é exibida.
    ///
    /// # Errors
    ///
    /// Falha se o documento não puder ser montado ou se a serialização para
    /// JavaScript falhar. **Arquivo malformado não é erro:** o que não for
    /// compreendido é contado no relatório.
    #[wasm_bindgen(js_name = openDxf)]
    pub fn open_dxf(&mut self, bytes: &[u8]) -> Result<JsValue, JsError> {
        let report = self.try_open_dxf(bytes).map_err(js_error)?;

        to_js(&report)
    }

    /// Serializa o documento para DXF.
    ///
    /// A saída é determinística: o mesmo documento produz os mesmos bytes
    /// (ADR 0004).
    ///
    /// # Errors
    ///
    /// Falha se o documento estiver inconsistente.
    #[wasm_bindgen(js_name = toDxf)]
    pub fn to_dxf(&self) -> Result<Vec<u8>, JsError> {
        Ok(self.try_to_dxf())
    }

    /// O que uma gravação descartaria do arquivo aberto.
    ///
    /// Deve ser consultado **antes** de sobrescrever um original.
    ///
    /// # Errors
    ///
    /// Falha se a serialização para JavaScript falhar.
    #[wasm_bindgen(js_name = saveLoss)]
    pub fn save_loss(&self) -> Result<JsValue, JsError> {
        to_js(&self.loss)
    }

    /// Estado da pilha de comandos, para alimentar o menu `Editar`.
    ///
    /// # Errors
    ///
    /// Falha se a serialização para JavaScript falhar.
    pub fn history(&self) -> Result<JsValue, JsError> {
        to_js(&HistoryView {
            can_undo: self.stack.can_undo(),
            can_redo: self.stack.can_redo(),
            undo_name: self.stack.undo_name().map(str::to_owned),
            redo_name: self.stack.redo_name().map(str::to_owned),
            undo_depth: self.stack.undo_depth(),
            redo_depth: self.stack.redo_depth(),
        })
    }
}

/// Operações sem contato com o runtime JavaScript.
///
/// `JsError` só pode ser construído dentro do WebAssembly — instanciá-lo em
/// outro alvo entra em pânico. Manter a lógica aqui, devolvendo `String`, deixa
/// a fachada inteira exercitável por `cargo test` no host, inclusive os
/// caminhos de erro.
impl CadSession {
    fn try_create_layer(&mut self, name: &str) -> Result<String, String> {
        self.document
            .create_layer(name)
            .map(encode_layer)
            .map_err(|error| error.to_string())
    }

    fn try_add_line(
        &mut self,
        layer: &str,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    ) -> Result<String, String> {
        let layer = decode_layer(layer)?;
        let entity = Entity::new(
            layer,
            Geometry::Line(Line {
                start: Point2::new(start_x, start_y),
                end: Point2::new(end_x, end_y),
            }),
        );

        self.stack
            .edit(&mut self.document, "Desenhar linha", |editor| {
                editor.insert_in_model_space(entity)
            })
            .map(encode_entity)
            .map_err(|error| error.to_string())
    }

    fn try_remove_entity(&mut self, entity: &str) -> Result<(), String> {
        let entity = decode_entity(entity)?;

        self.stack
            .edit(&mut self.document, "Apagar", |editor| {
                editor.remove_entity(entity).map(|_| ())
            })
            .map_err(|error| error.to_string())
    }

    fn try_set_layer_off(&mut self, layer: &str, off: bool) -> Result<(), String> {
        let layer = decode_layer(layer)?;
        let mut record = self
            .document
            .layers()
            .get(layer)
            .ok_or_else(|| String::from("camada inexistente no documento"))?
            .clone();
        record.set_off(off);

        self.stack
            .edit(&mut self.document, "Alterar camada", |editor| {
                editor.set_layer_record(layer, record).map(|_| ())
            })
            .map_err(|error| error.to_string())
    }

    fn try_load(&mut self, input: DocumentInput) -> Result<LoadReport, String> {
        let mut document = Document::new();

        for layer in input.layers {
            // A camada `0` já existe em todo documento; o arquivo apenas
            // redefine as propriedades dela.
            let id = match document.layers().id_of(&layer.name) {
                Some(existing) => existing,
                None => document
                    .create_layer(layer.name.as_str())
                    .map_err(|error| error.to_string())?,
            };

            let mut record = document
                .layers()
                .get(id)
                .ok_or_else(|| String::from("camada recém-criada desapareceu"))?
                .clone();
            record.set_color(layer.color.into());
            record.set_off(layer.off);
            record.set_frozen(layer.frozen);
            record.set_locked(layer.locked);

            document
                .edit()
                .set_layer_record(id, record)
                .map_err(|error| error.to_string())?;
        }

        let mut entity_count = 0;
        let mut skipped_count = 0;

        {
            let mut editor = document.edit();

            for entity in input.entities {
                let Some(layer) = editor.document().layers().id_of(&entity.layer_name) else {
                    skipped_count += 1;
                    continue;
                };

                editor
                    .insert_in_model_space(Entity::new(layer, entity.geometry.into()))
                    .map_err(|error| error.to_string())?;
                entity_count += 1;
            }

            let _ = editor.finish();
        }

        let layer_count = document.layers().len();

        self.document = document;
        self.stack = CommandStack::new();

        Ok(LoadReport {
            layer_count,
            entity_count,
            skipped_count,
        })
    }

    fn try_open_dxf(&mut self, bytes: &[u8]) -> Result<DxfOpenReport, String> {
        let leitura = read_dxf(bytes);
        let montagem = build_document(&leitura).map_err(|error| error.to_string())?;

        let loss = loss_view(&leitura, &montagem);
        let skipped_count = montagem.skipped_count;
        let document = montagem.document;
        let summary = leitura.report.to_string();
        let entity_count = document.entities_in_block(document.model_space()).count();
        let block_count = document
            .blocks()
            .len()
            .saturating_sub(document.layouts().len());
        let block_entity_count = document
            .blocks()
            .iter()
            .filter(|(id, _)| document.layouts().of_block(*id).is_none())
            .map(|(_, registro)| registro.entity_count())
            .sum();

        self.loss = loss.clone();
        self.stack = CommandStack::new();

        let report = DxfOpenReport {
            layer_count: document.layers().len(),
            entity_count,
            skipped_count,
            block_count,
            block_entity_count,
            created_layers: leitura.report.created_layers.clone(),
            errors: leitura
                .report
                .section_errors
                .iter()
                .map(ToString::to_string)
                .collect(),
            loss,
            summary,
        };

        self.document = document;

        Ok(report)
    }

    fn try_to_dxf(&self) -> Vec<u8> {
        let layouts = self.document.layouts();
        let modelo = layouts.model_layout();

        // As entidades saem na ordem em que o desenho as guarda: primeiro o
        // espaço-modelo, depois cada aba. Cada uma leva o seu espaço, e é dele
        // que a gravação tira os códigos `67` e `410`.
        let mut entidades: Vec<ReadEntity> = self
            .document
            .entities_in_block(self.document.model_space())
            .map(|(_, entidade)| ReadEntity {
                space: EntitySpace::Model,
                entity: entidade.clone(),
            })
            .collect();

        let mut declarados: Vec<LayoutDefinition> = Vec::new();

        for (id, registro) in layouts.in_tab_order() {
            if id == modelo {
                continue;
            }

            let bloco = self
                .document
                .blocks()
                .get(registro.block())
                .map(|b| b.name().to_owned());

            entidades.extend(self.document.entities_in_block(registro.block()).map(
                |(_, entidade)| ReadEntity {
                    space: EntitySpace::Paper(registro.name().to_owned()),
                    entity: entidade.clone(),
                },
            ));

            declarados.push(LayoutDefinition {
                name: registro.name().to_owned(),
                tab_order: registro.tab_order(),
                block_name: bloco,
                page_setup: registro.page_setup(),
            });
        }

        // Bloco de aba não é bloco do desenho: o conteúdo dele já saiu como
        // entidade de espaço-papel, e gravá-lo aqui o duplicaria.
        let blocos: Vec<BlockDefinition> = self
            .document
            .blocks()
            .iter()
            .filter(|(id, _)| *id != self.document.model_space())
            .filter(|(id, _)| layouts.of_block(*id).is_none())
            .map(|(_, registro)| BlockDefinition {
                name: registro.name().to_owned(),
                base_point: registro.origin(),
                entities: registro
                    .entities()
                    .iter()
                    .filter_map(|id| self.document.entity(*id).cloned())
                    .collect(),
                // O modelo não guarda caminho de referência externa; ver a
                // perda declarada em `SaveLossView::xref_count`.
                xref_path: None,
            })
            .collect();

        write_dxf(&DxfContents {
            layers: self.document.layers(),
            entities: &entidades,
            blocks: &blocos,
            layouts: &declarados,
        })
    }

    fn try_undo(&mut self) -> Result<bool, String> {
        self.stack
            .undo(&mut self.document)
            .map_err(|error| error.to_string())
    }

    fn try_redo(&mut self) -> Result<bool, String> {
        self.stack
            .redo(&mut self.document)
            .map_err(|error| error.to_string())
    }
}

impl Default for CadSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Monta o relatório de perda a partir de uma leitura e da montagem.
///
/// # O espaço-papel saiu da lista de perdas
///
/// Até o MT-KL-09 as entidades de papel eram lidas e não tinham onde morar, e
/// por isso figuravam aqui. Agora entram no documento, em abas de verdade — o
/// que resta é informar **quais** abas existem, e não avisar que se perdem.
fn loss_view(leitura: &neocad_io::DxfReading, montagem: &DocumentBuild) -> SaveLossView {
    let unsupported: Vec<UnsupportedView> = leitura
        .report
        .unsupported_by_frequency()
        .into_iter()
        .map(|(entity_type, count)| UnsupportedView {
            entity_type: entity_type.to_owned(),
            count,
        })
        .collect();

    let paper_space: Vec<PaperSpaceView> = montagem
        .document
        .layouts()
        .in_tab_order()
        .iter()
        .filter(|(id, _)| *id != montagem.document.layouts().model_layout())
        .map(|(_, registro)| PaperSpaceView {
            name: registro.name().to_owned(),
            entity_count: montagem
                .document
                .entities_in_block(registro.block())
                .count(),
        })
        .collect();

    let unsupported_count = unsupported.iter().map(|item| item.count).sum();
    let paper_space_count = paper_space.iter().map(|item| item.entity_count).sum();
    let xref_count = leitura.blocks.iter().filter(|b| b.is_xref()).count();

    SaveLossView {
        unsupported,
        unsupported_count,
        paper_space,
        paper_space_count,
        xref_count,
        // O papel não conta mais como perda: ele entra no documento e é gravado.
        // O que ainda se perde é o que o modelo não representa e o caminho das
        // referências externas.
        is_lossless: unsupported_count == 0 && xref_count == 0,
    }
}

/// Converte uma mensagem em erro do lado JavaScript.
fn js_error(message: String) -> JsError {
    JsError::new(&message)
}

/// Serializa um valor para o lado JavaScript.
fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsError::new(&error.to_string()))
}

/// Codifica o identificador de uma camada.
fn encode_layer(layer: LayerId) -> String {
    layer.to_bits().to_string()
}

/// Codifica o identificador de uma entidade.
fn encode_entity(entity: EntityId) -> String {
    entity.to_bits().to_string()
}

/// Decodifica o identificador de uma camada vindo do JavaScript.
fn decode_layer(raw: &str) -> Result<LayerId, String> {
    raw.parse::<u64>()
        .ok()
        .and_then(LayerId::from_bits)
        .ok_or_else(|| String::from("identificador de camada inválido"))
}

/// Decodifica o identificador de uma entidade vindo do JavaScript.
fn decode_entity(raw: &str) -> Result<EntityId, String> {
    raw.parse::<u64>()
        .ok()
        .and_then(EntityId::from_bits)
        .ok_or_else(|| String::from("identificador de entidade inválido"))
}

/// Converte uma camada do kernel para a forma de transporte.
fn layer_view(id: LayerId, record: &LayerRecord) -> LayerView {
    LayerView {
        id: encode_layer(id),
        name: record.name().to_owned(),
        color: record.color().into(),
        visible: record.is_visible(),
        off: record.is_off(),
        frozen: record.is_frozen(),
        locked: record.is_locked(),
    }
}

/// Converte uma entidade do kernel para a forma de transporte.
fn entity_view(id: EntityId, entity: &Entity) -> EntityView {
    EntityView {
        id: encode_entity(id),
        layer: encode_layer(entity.layer),
        geometry: (&entity.geometry).into(),
        bounds: entity.bounding_box().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Carrega uma fixture sintética do E2E.
    fn fixture(nome: &str) -> Vec<u8> {
        let caminho = format!("{}/../../e2e/fixtures/{nome}", env!("CARGO_MANIFEST_DIR"));

        std::fs::read(&caminho).unwrap_or_else(|erro| panic!("{caminho}: {erro}"))
    }

    #[test]
    fn abre_dxf_e_carrega_o_espaco_modelo() {
        let mut session = CadSession::new();

        let relatorio = session
            .try_open_dxf(&fixture("minimal.dxf"))
            .expect("fixture bem formada");

        assert_eq!(relatorio.entity_count, 2);
        assert_eq!(session.entity_count(), 2);
        assert_eq!(relatorio.skipped_count, 0);
        assert!(relatorio.loss.is_lossless);
        assert_eq!(relatorio.summary, "arquivo compreendido por inteiro");
    }

    #[test]
    fn abrir_zera_o_historico() {
        // Desfazer para antes da abertura não faz sentido.
        let mut session = CadSession::new();
        let layer = encode_layer(session.document.layers().default_layer());
        session
            .try_add_line(&layer, 0.0, 0.0, 1.0, 1.0)
            .expect("camada existe");
        assert!(session.stack.can_undo());

        session
            .try_open_dxf(&fixture("minimal.dxf"))
            .expect("fixture bem formada");

        assert!(!session.stack.can_undo());
    }

    #[test]
    fn abre_o_arquivo_que_o_upstream_nao_le() {
        // A fixture que motivou a fase K2 inteira.
        let mut session = CadSession::new();

        let relatorio = session
            .try_open_dxf(&fixture("block-with-entities.dxf"))
            .expect("fixture bem formada");

        assert_eq!(relatorio.entity_count, 2);
        assert_eq!(relatorio.block_count, 1);
        assert_eq!(relatorio.block_entity_count, 1);
    }

    #[test]
    fn entidade_nao_modelada_aparece_na_perda_em_vez_de_sumir() {
        let mut session = CadSession::new();

        let relatorio = session
            .try_open_dxf(&fixture("with-unsupported.dxf"))
            .expect("fixture bem formada");

        assert!(!relatorio.loss.is_lossless);
        assert_eq!(relatorio.loss.unsupported_count, 1);
        assert_eq!(relatorio.loss.unsupported[0].entity_type, "SOLID");
        // E continua consultável no momento de salvar.
        assert_eq!(session.loss.unsupported_count, 1);
    }

    #[test]
    fn desenho_montado_no_papel_entra_na_aba_e_deixa_de_ser_perda() {
        // O caso dos 8% do acervo. Até o MT-KL-09 a prancha era lida e não tinha
        // onde morar, então figurava como perda; agora entra numa aba de verdade
        // e o documento a grava. O relatório continua nomeando a aba, porque a
        // interface precisa dizer que o desenho está no papel — mas dizer isso é
        // informação, e não mais aviso de destruição.
        let arquivo = b"  0\nSECTION\n  2\nENTITIES\n\
                        0\nLINE\n  8\n0\n 67\n1\n410\nPrancha A1\n\
                        10\n0.0\n 20\n0.0\n 11\n1.0\n 21\n1.0\n\
                        0\nENDSEC\n  0\nEOF\n";
        let mut session = CadSession::new();

        let relatorio = session.try_open_dxf(arquivo).expect("bem formado");

        assert_eq!(relatorio.entity_count, 0);
        assert_eq!(relatorio.loss.paper_space_count, 1);
        assert_eq!(relatorio.loss.paper_space[0].name, "Prancha A1");
        assert!(relatorio.loss.is_lossless);
    }

    #[test]
    fn arquivo_malformado_nao_impede_a_abertura() {
        // `ENDSEC` faltando: relatado, e o que veio depois ainda entra.
        let arquivo = b"  0\nSECTION\n  2\nHEADER\n  9\n$ACADVER\n\
                        0\nSECTION\n  2\nENTITIES\n\
                        0\nLINE\n  8\n0\n 10\n0.0\n 20\n0.0\n 11\n1.0\n 21\n1.0\n\
                        0\nENDSEC\n  0\nEOF\n";
        let mut session = CadSession::new();

        let relatorio = session.try_open_dxf(arquivo).expect("abre assim mesmo");

        assert_eq!(relatorio.entity_count, 1);
        assert_eq!(relatorio.errors.len(), 1);
    }

    #[test]
    fn grava_o_documento_e_reabre_com_o_mesmo_conteudo() {
        let mut session = CadSession::new();
        session
            .try_open_dxf(&fixture("block-with-entities.dxf"))
            .expect("fixture bem formada");

        let bytes = session.try_to_dxf();
        let mut outra = CadSession::new();
        let relatorio = outra.try_open_dxf(&bytes).expect("o que gravamos, abrimos");

        assert_eq!(relatorio.entity_count, 2);
        assert_eq!(relatorio.block_count, 1);
        assert_eq!(relatorio.block_entity_count, 1);
        assert!(relatorio.loss.is_lossless);
    }

    #[test]
    fn a_gravacao_e_deterministica() {
        let mut session = CadSession::new();
        session
            .try_open_dxf(&fixture("legacy-polyline.dxf"))
            .expect("fixture bem formada");

        assert_eq!(session.try_to_dxf(), session.try_to_dxf());
    }

    #[test]
    fn desenho_feito_na_sessao_atravessa_a_gravacao() {
        let mut session = CadSession::new();
        let layer = session.try_create_layer("Eixos").expect("nome válido");
        session
            .try_add_line(&layer, 0.0, 0.0, 10.0, 5.0)
            .expect("camada existe");

        let mut outra = CadSession::new();
        let relatorio = outra
            .try_open_dxf(&session.try_to_dxf())
            .expect("o que gravamos, abrimos");

        assert_eq!(relatorio.entity_count, 1);
        assert!(outra.document.layers().id_of("Eixos").is_some());
    }

    #[test]
    fn sessao_sem_arquivo_aberto_nao_tem_perda() {
        let session = CadSession::new();

        assert!(session.loss.is_lossless);
        assert_eq!(session.loss.unsupported_count, 0);
    }

    #[test]
    fn sessao_nova_tem_camada_zero_e_nenhuma_entidade() {
        let session = CadSession::new();

        assert_eq!(session.entity_count(), 0);
        assert_eq!(session.document.layers().len(), 1);
        assert!(!session.stack.can_undo());
    }

    #[test]
    fn desenhar_linha_e_desfazer_volta_ao_documento_vazio() {
        let mut session = CadSession::new();
        let layer = encode_layer(session.document.layers().default_layer());

        session
            .try_add_line(&layer, 0.0, 0.0, 10.0, 0.0)
            .expect("camada existe");
        assert_eq!(session.entity_count(), 1);
        assert!(session.stack.can_undo());

        assert!(session.try_undo().expect("desfaz"));

        assert_eq!(session.entity_count(), 0);
        assert!(session.stack.can_redo());
    }

    #[test]
    fn refazer_reconstroi_a_linha() {
        let mut session = CadSession::new();
        let layer = encode_layer(session.document.layers().default_layer());
        let id = session
            .try_add_line(&layer, 0.0, 0.0, 10.0, 0.0)
            .expect("camada existe");

        session.try_undo().expect("desfaz");
        assert!(session.try_redo().expect("refaz"));

        assert_eq!(session.entity_count(), 1);
        assert!(
            session
                .document
                .entity(decode_entity(&id).expect("id válido"))
                .is_some(),
            "refazer tem de devolver o mesmo identificador"
        );
    }

    #[test]
    fn apagar_entidade_e_desfazivel() {
        let mut session = CadSession::new();
        let layer = encode_layer(session.document.layers().default_layer());
        let id = session
            .try_add_line(&layer, 0.0, 0.0, 10.0, 0.0)
            .expect("camada existe");

        session.try_remove_entity(&id).expect("entidade existe");
        assert_eq!(session.entity_count(), 0);

        session.try_undo().expect("desfaz");
        assert_eq!(session.entity_count(), 1);
    }

    #[test]
    fn alterar_camada_e_desfazivel() {
        let mut session = CadSession::new();
        let layer = session.try_create_layer("Parede").expect("nome válido");

        session
            .try_set_layer_off(&layer, true)
            .expect("camada existe");
        let id = decode_layer(&layer).expect("id válido");
        assert!(session.document.layers().get(id).expect("existe").is_off());

        session.try_undo().expect("desfaz");

        assert!(!session.document.layers().get(id).expect("existe").is_off());
    }

    #[test]
    fn criar_camada_nao_entra_no_historico() {
        let mut session = CadSession::new();

        session.try_create_layer("Parede").expect("nome válido");

        // Operações de estrutura de tabela ainda não são reversíveis; está
        // registrado como pendência no handoff do MT-K1-10.
        assert!(!session.stack.can_undo());
    }

    #[test]
    fn identificadores_fazem_ida_e_volta_pela_ponte() {
        let mut session = CadSession::new();
        let layer = encode_layer(session.document.layers().default_layer());
        let id = session
            .try_add_line(&layer, 1.0, 2.0, 3.0, 4.0)
            .expect("camada existe");

        let decodificado = decode_entity(&id).expect("id válido");

        assert!(session.document.entity(decodificado).is_some());
    }

    #[test]
    fn identificador_malformado_e_recusado() {
        assert!(decode_entity("abc").is_err());
        assert!(decode_entity("0").is_err(), "geração zero nunca é emitida");
        assert!(decode_layer("").is_err());
    }

    #[test]
    fn camada_inexistente_e_recusada_ao_desenhar() {
        let mut session = CadSession::new();
        let mut outra = CadSession::new();
        let estranha = outra.try_create_layer("Fantasma").expect("nome válido");

        assert!(session.try_add_line(&estranha, 0.0, 0.0, 1.0, 1.0).is_err());
        assert_eq!(session.entity_count(), 0);
    }

    #[test]
    fn conversao_de_geometria_preserva_o_tipo() {
        let mut session = CadSession::new();
        let layer = encode_layer(session.document.layers().default_layer());
        session
            .try_add_line(&layer, 0.0, 0.0, 3.0, 4.0)
            .expect("camada existe");

        let (id, entity) = session.document.entities().next().expect("há uma entidade");
        let view = entity_view(id, entity);

        assert!(matches!(view.geometry, GeometryView::Line { .. }));
        assert_eq!(view.bounds.max_x, 3.0);
        assert_eq!(view.bounds.max_y, 4.0);
    }

    fn documento_de_teste() -> DocumentInput {
        DocumentInput {
            layers: vec![
                LayerInput {
                    name: String::from("0"),
                    color: ColorView::Index { index: 7 },
                    off: false,
                    frozen: false,
                    locked: false,
                },
                LayerInput {
                    name: String::from("Parede"),
                    color: ColorView::Rgb {
                        red: 200,
                        green: 30,
                        blue: 30,
                    },
                    off: true,
                    frozen: false,
                    locked: false,
                },
            ],
            entities: vec![
                EntityInput {
                    layer_name: String::from("Parede"),
                    geometry: GeometryView::Line {
                        start: PointView { x: 0.0, y: 0.0 },
                        end: PointView { x: 10.0, y: 0.0 },
                    },
                },
                EntityInput {
                    layer_name: String::from("0"),
                    geometry: GeometryView::Circle {
                        center: PointView { x: 5.0, y: 5.0 },
                        radius: 2.0,
                    },
                },
            ],
        }
    }

    #[test]
    fn load_reconstroi_camadas_e_entidades() {
        let mut session = CadSession::new();

        let report = session
            .try_load(documento_de_teste())
            .expect("documento válido");

        assert_eq!(report.layer_count, 2);
        assert_eq!(report.entity_count, 2);
        assert_eq!(report.skipped_count, 0);
        assert_eq!(session.entity_count(), 2);
    }

    #[test]
    fn load_aplica_propriedades_da_camada() {
        let mut session = CadSession::new();
        session
            .try_load(documento_de_teste())
            .expect("documento válido");

        let parede = session
            .document
            .layers()
            .get_by_name("Parede")
            .expect("camada carregada");

        assert!(parede.is_off());
        assert!(!parede.is_visible());
    }

    #[test]
    fn load_redefine_a_camada_zero_em_vez_de_duplicar() {
        let mut session = CadSession::new();

        session
            .try_load(documento_de_teste())
            .expect("documento válido");

        assert_eq!(
            session.document.layers().len(),
            2,
            "a camada 0 do arquivo é a mesma que já existe"
        );
    }

    #[test]
    fn load_preserva_a_ordem_de_desenho() {
        let mut session = CadSession::new();
        session
            .try_load(documento_de_teste())
            .expect("documento válido");

        let tipos: Vec<_> = session
            .document
            .entities_in_block(session.document.model_space())
            .map(|(_, entity)| matches!(entity.geometry, Geometry::Line(_)))
            .collect();

        assert_eq!(tipos, vec![true, false], "a linha vem antes do círculo");
    }

    #[test]
    fn load_ignora_entidade_de_camada_inexistente_sem_abortar() {
        let mut session = CadSession::new();
        let mut input = documento_de_teste();
        input.entities.push(EntityInput {
            layer_name: String::from("Fantasma"),
            geometry: GeometryView::Circle {
                center: PointView { x: 0.0, y: 0.0 },
                radius: 1.0,
            },
        });

        let report = session
            .try_load(input)
            .expect("o carregamento não pode abortar");

        assert_eq!(report.entity_count, 2);
        assert_eq!(report.skipped_count, 1);
    }

    #[test]
    fn load_zera_o_historico() {
        let mut session = CadSession::new();
        let layer = encode_layer(session.document.layers().default_layer());
        session
            .try_add_line(&layer, 0.0, 0.0, 1.0, 1.0)
            .expect("desenha");
        assert!(session.stack.can_undo());

        session
            .try_load(documento_de_teste())
            .expect("documento válido");

        assert!(
            !session.stack.can_undo(),
            "desfazer para antes da abertura não faz sentido"
        );
    }

    #[test]
    fn load_substitui_o_documento_anterior() {
        let mut session = CadSession::new();
        let layer = encode_layer(session.document.layers().default_layer());
        session
            .try_add_line(&layer, 0.0, 0.0, 1.0, 1.0)
            .expect("desenha");

        session
            .try_load(documento_de_teste())
            .expect("documento válido");

        assert_eq!(
            session.entity_count(),
            2,
            "o desenho anterior não pode sobrar"
        );
    }

    #[test]
    fn load_de_documento_vazio_produz_documento_valido() {
        let mut session = CadSession::new();

        let report = session
            .try_load(DocumentInput {
                layers: Vec::new(),
                entities: Vec::new(),
            })
            .expect("documento vazio é válido");

        assert_eq!(report.entity_count, 0);
        assert_eq!(report.layer_count, 1, "a camada 0 existe mesmo assim");
    }

    #[test]
    fn default_equivale_a_new() {
        assert_eq!(CadSession::default().entity_count(), 0);
    }
}
