// Caminho relativo: kernel/neocad-model/src/layout.rs
//! \file kernel/neocad-model/src/layout.rs
//! \brief Tabela de layouts do documento — o espaço-modelo e as pranchas.
//! \author Iago Leal
//! \date 2026-08-18

use core::fmt;

use crate::block::BlockId;
use crate::id::EntityId;
use crate::symbol_name::InvalidName;
use crate::symbol_table::{SymbolError, SymbolRecord, SymbolTable};

/// Nome da aba do espaço-modelo, como o AutoCAD a exibe.
pub const MODEL_LAYOUT_NAME: &str = "Model";

/// Prefixo dos blocos de espaço-papel, na convenção dos formatos CAD.
pub(crate) const PAPER_SPACE_PREFIX: &str = "*Paper_Space";

/// Identificador opaco de um layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayoutId(EntityId);

impl fmt::Display for LayoutId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "T{}", self.0)
    }
}

/// Unidade em que a folha é medida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlotUnits {
    /// Milímetros. É o padrão do desenho técnico brasileiro.
    #[default]
    Millimeters,
    /// Polegadas.
    Inches,
}

/// Rotação da folha ao plotar, em quartos de volta.
///
/// O formato só admite estes quatro valores; um ângulo livre não existe, e
/// representá-lo como número convidaria a gravar algo que o arquivo não aceita.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlotRotation {
    /// Sem rotação.
    #[default]
    None,
    /// Noventa graus no sentido anti-horário.
    Quarter,
    /// Cento e oitenta graus.
    Half,
    /// Duzentos e setenta graus.
    ThreeQuarters,
}

/// Margens não imperssas da folha, na unidade da configuração.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PlotMargins {
    /// Margem esquerda.
    pub left: f64,
    /// Margem inferior.
    pub bottom: f64,
    /// Margem direita.
    pub right: f64,
    /// Margem superior.
    pub top: f64,
}

/// Configuração de página de um layout.
///
/// # Escala é razão, não número
///
/// A escala de plotagem é gravada como razão — `1:100`, `1:50` — e é assim que o
/// projetista a nomeia. Guardá-la como um `f64` já dividido perderia a distinção
/// entre `1:3` e o decimal que o aproxima, e o carimbo do desenho mostra a razão,
/// não o quociente.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSetup {
    /// Largura da folha.
    pub paper_width: f64,
    /// Altura da folha.
    pub paper_height: f64,
    /// Unidade em que a folha e as margens são medidas.
    pub units: PlotUnits,
    /// Margens não impressas.
    pub margins: PlotMargins,
    /// Numerador da escala de plotagem.
    pub scale_numerator: f64,
    /// Denominador da escala de plotagem.
    pub scale_denominator: f64,
    /// Rotação da folha.
    pub rotation: PlotRotation,
}

impl PageSetup {
    /// Escala como quociente, quando o denominador permite.
    ///
    /// `None` para denominador zero, que é arquivo defeituoso: devolver infinito
    /// espalharia o defeito para dentro do desenho.
    #[must_use]
    pub fn scale(&self) -> Option<f64> {
        (self.scale_denominator != 0.0).then(|| self.scale_numerator / self.scale_denominator)
    }
}

impl Default for PageSetup {
    /// A4 em pé, em milímetros, escala 1:1.
    fn default() -> Self {
        Self {
            paper_width: 210.0,
            paper_height: 297.0,
            units: PlotUnits::Millimeters,
            margins: PlotMargins::default(),
            scale_numerator: 1.0,
            scale_denominator: 1.0,
            rotation: PlotRotation::None,
        }
    }
}

/// Registro de um layout: uma aba do documento.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutRecord {
    name: String,
    block: BlockId,
    tab_order: u16,
    page_setup: PageSetup,
}

impl LayoutRecord {
    /// Nome da aba, preservando a caixa com que foi criada.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Bloco onde moram as entidades deste layout.
    ///
    /// É o vínculo que o ADR 0005 fixa: o espaço de uma entidade é derivado do
    /// bloco dono, e não de um campo paralelo que poderia divergir.
    #[must_use]
    pub const fn block(&self) -> BlockId {
        self.block
    }

    /// Posição da aba na barra.
    #[must_use]
    pub const fn tab_order(&self) -> u16 {
        self.tab_order
    }

    /// Configuração de página.
    #[must_use]
    pub const fn page_setup(&self) -> PageSetup {
        self.page_setup
    }

    /// Define a posição da aba.
    pub fn set_tab_order(&mut self, tab_order: u16) {
        self.tab_order = tab_order;
    }

    /// Define a configuração de página.
    pub fn set_page_setup(&mut self, page_setup: PageSetup) {
        self.page_setup = page_setup;
    }
}

impl SymbolRecord for LayoutRecord {
    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

/// Falha ao operar sobre a tabela de layouts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// O nome informado é vazio ou só contém espaços.
    EmptyName,
    /// O nome contém um caractere que os formatos CAD não aceitam.
    ForbiddenCharacter(char),
    /// Já existe layout com esse nome. A comparação ignora caixa.
    DuplicateName(String),
    /// A aba do espaço-modelo não pode ser removida nem renomeada.
    ModelLayoutIsProtected,
    /// O identificador não corresponde a nenhum layout vivo.
    NotFound,
}

impl From<InvalidName> for LayoutError {
    fn from(error: InvalidName) -> Self {
        match error {
            InvalidName::Empty => Self::EmptyName,
            InvalidName::Forbidden(character) => Self::ForbiddenCharacter(character),
        }
    }
}

impl From<SymbolError> for LayoutError {
    /// Traduz o erro genérico da tabela para o vocabulário dos layouts.
    fn from(error: SymbolError) -> Self {
        match error {
            SymbolError::Invalid(invalid) => invalid.into(),
            SymbolError::Duplicate(name) => Self::DuplicateName(name),
            SymbolError::Protected => Self::ModelLayoutIsProtected,
            SymbolError::NotFound => Self::NotFound,
        }
    }
}

impl fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "o nome do layout não pode ser vazio"),
            Self::ForbiddenCharacter(character) => write!(
                formatter,
                "o caractere {character:?} não é aceito em nome de layout"
            ),
            Self::DuplicateName(name) => {
                write!(formatter, "já existe um layout chamado {name:?}")
            }
            Self::ModelLayoutIsProtected => write!(
                formatter,
                "a aba {MODEL_LAYOUT_NAME:?} não pode ser removida nem renomeada"
            ),
            Self::NotFound => write!(formatter, "layout não encontrado"),
        }
    }
}

impl core::error::Error for LayoutError {}

/// Tabela de layouts de um documento.
///
/// # O espaço-modelo é uma aba
///
/// A aba `Model` existe em toda tabela e aponta para o bloco `*Model_Space`.
/// Tratá-la como layout, e não como exceção, é o que permite à interface
/// percorrer uma lista só — no AutoCAD ela é a primeira aba, não um modo à parte.
///
/// # Ordem das abas
///
/// [`LayoutTable::iter`] percorre em ordem alfabética, herdada da tabela de
/// símbolos. Quem quer a ordem da **barra de abas** usa
/// [`LayoutTable::in_tab_order`], que ordena por `tab_order` com o nome como
/// desempate — determinística, e portanto adequada à escrita do arquivo.
#[derive(Debug, Clone)]
pub struct LayoutTable {
    symbols: SymbolTable<LayoutRecord>,
}

impl LayoutTable {
    /// Cria a tabela com a aba do espaço-modelo apontando para `model_space`.
    #[must_use]
    pub(crate) fn new(model_space: BlockId) -> Self {
        Self {
            symbols: SymbolTable::with_protected(LayoutRecord {
                name: String::from(MODEL_LAYOUT_NAME),
                block: model_space,
                tab_order: 0,
                page_setup: PageSetup::default(),
            }),
        }
    }

    /// Identificador da aba do espaço-modelo, sempre presente.
    #[must_use]
    pub const fn model_layout(&self) -> LayoutId {
        LayoutId(self.symbols.protected())
    }

    /// Quantidade de layouts. Nunca é zero.
    #[must_use]
    #[expect(
        clippy::len_without_is_empty,
        reason = "a tabela nunca é vazia: a aba do espaço-modelo é indestrutível"
    )]
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Cria um layout ligado ao bloco informado.
    ///
    /// O bloco é criado por quem chama — o documento —, porque só ele pode usar
    /// a via reservada da tabela de blocos.
    pub(crate) fn create(
        &mut self,
        name: impl Into<String>,
        block: BlockId,
    ) -> Result<LayoutId, LayoutError> {
        let tab_order = self.next_tab_order();

        self.symbols
            .create(name.into(), |name| LayoutRecord {
                name,
                block,
                tab_order,
                page_setup: PageSetup::default(),
            })
            .map(LayoutId)
            .map_err(LayoutError::from)
    }

    /// Próxima posição livre na barra de abas.
    fn next_tab_order(&self) -> u16 {
        self.symbols
            .iter()
            .map(|(_, record)| record.tab_order)
            .max()
            .map_or(0, |maior| maior.saturating_add(1))
    }

    /// Devolve o layout de `id`, ou `None` se o identificador estiver obsoleto.
    #[must_use]
    pub fn get(&self, id: LayoutId) -> Option<&LayoutRecord> {
        self.symbols.get(id.0)
    }

    /// Versão mutável de [`LayoutTable::get`].
    ///
    /// Não permite alterar o nome nem o bloco: renomear passa por
    /// [`LayoutTable::rename`], e o bloco é vínculo estrutural, não propriedade.
    #[must_use]
    pub fn get_mut(&mut self, id: LayoutId) -> Option<&mut LayoutRecord> {
        self.symbols.get_mut(id.0)
    }

    /// Procura um layout pelo nome da aba, ignorando caixa.
    #[must_use]
    pub fn id_of(&self, name: &str) -> Option<LayoutId> {
        self.symbols.id_of(name).map(LayoutId)
    }

    /// Procura um layout pelo nome da aba, ignorando caixa.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&LayoutRecord> {
        self.symbols.get_by_name(name)
    }

    /// Indica se o identificador corresponde a um layout vivo.
    #[must_use]
    pub fn contains(&self, id: LayoutId) -> bool {
        self.symbols.contains(id.0)
    }

    /// Procura o layout a que um bloco pertence.
    #[must_use]
    pub fn of_block(&self, block: BlockId) -> Option<LayoutId> {
        self.iter()
            .find(|(_, record)| record.block == block)
            .map(|(id, _)| id)
    }

    /// Renomeia a aba, preservando o identificador.
    ///
    /// # Errors
    ///
    /// Falha se for a aba do espaço-modelo, se o nome for inválido ou se já
    /// estiver ocupado.
    pub(crate) fn rename(
        &mut self,
        id: LayoutId,
        name: impl Into<String>,
    ) -> Result<(), LayoutError> {
        self.symbols
            .rename(id.0, name.into())
            .map_err(LayoutError::from)
    }

    /// Remove um layout e devolve o registro.
    ///
    /// Quem chama é responsável por remover o bloco associado — a tabela de
    /// layouts não alcança as entidades.
    pub(crate) fn remove(&mut self, id: LayoutId) -> Result<LayoutRecord, LayoutError> {
        self.symbols.remove(id.0).map_err(LayoutError::from)
    }

    /// Itera em ordem alfabética de nome.
    pub fn iter(&self) -> impl Iterator<Item = (LayoutId, &LayoutRecord)> {
        self.symbols
            .iter()
            .map(|(id, record)| (LayoutId(id), record))
    }

    /// Itera na ordem da barra de abas.
    ///
    /// Ordena por `tab_order`, com o nome como desempate para a saída não
    /// depender da ordem de criação — é dessa estabilidade que a escrita do
    /// arquivo tira o seu determinismo.
    #[must_use]
    pub fn in_tab_order(&self) -> Vec<(LayoutId, &LayoutRecord)> {
        let mut abas: Vec<(LayoutId, &LayoutRecord)> = self.iter().collect();

        abas.sort_by(|(_, um), (_, outro)| {
            um.tab_order
                .cmp(&outro.tab_order)
                .then_with(|| um.name.cmp(&outro.name))
        });

        abas
    }

    /// Itera sobre os nomes das abas, em ordem alfabética.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.symbols.names()
    }
}
