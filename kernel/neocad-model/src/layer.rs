// Caminho relativo: kernel/neocad-model/src/layer.rs
//! \file kernel/neocad-model/src/layer.rs
//! \brief Tabela de camadas do documento.
//! \author Iago Leal
//! \date 2026-08-07

use core::fmt;
use std::collections::BTreeMap;

use crate::arena::Arena;
use crate::id::EntityId;
use crate::symbol_name::{normalize, validate, InvalidName};

/// Nome da camada que todo documento CAD possui e que não pode ser removida.
pub const DEFAULT_LAYER_NAME: &str = "0";

/// Identificador opaco de uma camada.
///
/// Distinto de [`EntityId`] de propósito: o compilador passa a impedir que um
/// identificador de camada seja usado onde se espera uma entidade, e vice-versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerId(EntityId);

impl LayerId {
    /// Representação escalar opaca, adequada a transporte.
    ///
    /// Delega a [`EntityId::to_bits`]; ver lá o motivo de existir.
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0.to_bits()
    }

    /// Reconstrói um identificador de camada a partir de [`LayerId::to_bits`].
    #[must_use]
    pub const fn from_bits(bits: u64) -> Option<Self> {
        match EntityId::from_bits(bits) {
            Some(id) => Some(Self(id)),
            None => None,
        }
    }
}

impl fmt::Display for LayerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "L{}", self.0)
    }
}

/// Cor de uma camada.
///
/// Cobre o índice ACI clássico e a cor verdadeira introduzida depois. A tradução
/// fiel para os códigos de grupo do DXF acontece em K2, na crate de I/O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Índice na paleta ACI (`AutoCAD Color Index`).
    Index(u8),
    /// Cor verdadeira, em componentes vermelho, verde e azul.
    Rgb {
        /// Componente vermelha.
        red: u8,
        /// Componente verde.
        green: u8,
        /// Componente azul.
        blue: u8,
    },
}

impl Default for Color {
    /// Índice 7, o padrão de desenho novo — renderizado como preto ou branco
    /// conforme o fundo.
    fn default() -> Self {
        Self::Index(7)
    }
}

/// Espessura de linha de uma camada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineWeight {
    /// Espessura padrão do documento.
    #[default]
    Default,
    /// Espessura explícita, em centésimos de milímetro.
    Hundredths(u16),
}

/// Registro de uma camada.
///
/// Os campos são privados: a mutação direta existe apenas enquanto o command
/// stack não está pronto. Em MT-K1-10 os modificadores passam a `pub(crate)` e
/// toda alteração passa a exigir uma transação (ADR 0003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerRecord {
    name: String,
    color: Color,
    linetype: String,
    line_weight: LineWeight,
    is_off: bool,
    is_frozen: bool,
    is_locked: bool,
}

impl LayerRecord {
    /// Nome de exibição, preservando a caixa com que a camada foi criada.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Cor da camada.
    #[must_use]
    pub const fn color(&self) -> Color {
        self.color
    }

    /// Nome do tipo de linha.
    ///
    /// Guardado como texto enquanto não existe tabela de tipos de linha; vira
    /// referência quando ela for criada.
    #[must_use]
    pub fn linetype(&self) -> &str {
        &self.linetype
    }

    /// Espessura de linha.
    #[must_use]
    pub const fn line_weight(&self) -> LineWeight {
        self.line_weight
    }

    /// Camada desligada: não é exibida, mas continua sendo regenerada.
    #[must_use]
    pub const fn is_off(&self) -> bool {
        self.is_off
    }

    /// Camada congelada: não é exibida nem regenerada.
    #[must_use]
    pub const fn is_frozen(&self) -> bool {
        self.is_frozen
    }

    /// Camada bloqueada: visível, porém não editável.
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Indica se as entidades da camada são desenhadas.
    ///
    /// Uma camada desligada **ou** congelada não aparece.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        !self.is_off && !self.is_frozen
    }

    /// Define a cor.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Define o tipo de linha.
    pub fn set_linetype(&mut self, linetype: impl Into<String>) {
        self.linetype = linetype.into();
    }

    /// Define a espessura de linha.
    pub fn set_line_weight(&mut self, line_weight: LineWeight) {
        self.line_weight = line_weight;
    }

    /// Liga ou desliga a camada.
    pub fn set_off(&mut self, is_off: bool) {
        self.is_off = is_off;
    }

    /// Congela ou descongela a camada.
    pub fn set_frozen(&mut self, is_frozen: bool) {
        self.is_frozen = is_frozen;
    }

    /// Bloqueia ou desbloqueia a camada.
    pub fn set_locked(&mut self, is_locked: bool) {
        self.is_locked = is_locked;
    }
}

/// Falha ao operar sobre a tabela de camadas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerError {
    /// O nome informado é vazio ou só contém espaços.
    EmptyName,
    /// O nome contém um caractere que os formatos CAD não aceitam.
    ForbiddenCharacter(char),
    /// Já existe camada com esse nome. A comparação ignora caixa.
    DuplicateName(String),
    /// A camada `0` não pode ser removida nem renomeada.
    DefaultLayerIsProtected,
    /// O identificador não corresponde a nenhuma camada viva.
    NotFound,
}

impl From<InvalidName> for LayerError {
    fn from(error: InvalidName) -> Self {
        match error {
            InvalidName::Empty => Self::EmptyName,
            InvalidName::Forbidden(character) => Self::ForbiddenCharacter(character),
        }
    }
}

impl fmt::Display for LayerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "o nome da camada não pode ser vazio"),
            Self::ForbiddenCharacter(character) => write!(
                formatter,
                "o caractere {character:?} não é aceito em nome de camada"
            ),
            Self::DuplicateName(name) => {
                write!(formatter, "já existe uma camada chamada {name:?}")
            }
            Self::DefaultLayerIsProtected => write!(
                formatter,
                "a camada {DEFAULT_LAYER_NAME:?} não pode ser removida nem renomeada"
            ),
            Self::NotFound => write!(formatter, "camada não encontrada"),
        }
    }
}

impl core::error::Error for LayerError {}

/// Tabela de camadas de um documento.
///
/// # Identidade estável
///
/// As camadas são referenciadas por [`LayerId`], não por nome. Em DXF, cada
/// entidade guarda o **nome** da camada como texto, o que faz renomear custar uma
/// varredura por todas as entidades. Aqui o nome é atributo da camada, não a sua
/// identidade: renomear é uma operação local, e a resolução para nome acontece
/// apenas na escrita do arquivo (K2).
///
/// # Unicidade de nome
///
/// Nomes são únicos **ignorando caixa**, como nos formatos CAD: criar `PAREDE`
/// quando já existe `Parede` é rejeitado. A caixa original é preservada para
/// exibição.
///
/// # Ordem de iteração
///
/// [`LayerTable::iter`] percorre em ordem alfabética do nome normalizado, e não
/// em ordem de criação. É determinística e coincide com a ordenação que os
/// aplicativos CAD apresentam.
///
/// # Camada padrão
///
/// A camada `0` existe em toda tabela, desde a construção, e não pode ser
/// removida nem renomeada.
///
/// # Exemplo
///
/// ```
/// use neocad_model::{LayerError, LayerTable};
///
/// let mut layers = LayerTable::new();
/// let parede = layers.create("Parede")?;
///
/// assert_eq!(layers.get(parede).map(|layer| layer.name()), Some("Parede"));
/// assert_eq!(
///     layers.create("PAREDE"),
///     Err(LayerError::DuplicateName(String::from("PAREDE"))),
/// );
/// # Ok::<(), LayerError>(())
/// ```
#[derive(Debug, Clone)]
pub struct LayerTable {
    records: Arena<LayerRecord>,
    by_normalized_name: BTreeMap<String, LayerId>,
    default_layer: LayerId,
}

impl LayerTable {
    /// Cria uma tabela contendo apenas a camada `0`.
    #[must_use]
    pub fn new() -> Self {
        let mut records = Arena::new();
        let default_layer = LayerId(records.insert(LayerRecord {
            name: String::from(DEFAULT_LAYER_NAME),
            color: Color::default(),
            linetype: String::from("Continuous"),
            line_weight: LineWeight::Default,
            is_off: false,
            is_frozen: false,
            is_locked: false,
        }));

        let mut by_normalized_name = BTreeMap::new();
        by_normalized_name.insert(normalize(DEFAULT_LAYER_NAME), default_layer);

        Self {
            records,
            by_normalized_name,
            default_layer,
        }
    }

    /// Identificador da camada `0`, sempre presente.
    #[must_use]
    pub const fn default_layer(&self) -> LayerId {
        self.default_layer
    }

    /// Quantidade de camadas. Nunca é zero.
    ///
    /// Não existe `is_empty` correspondente porque a camada `0` é indestrutível:
    /// um método que devolvesse sempre `false` seria ruído na API pública.
    #[must_use]
    #[expect(
        clippy::len_without_is_empty,
        reason = "a tabela nunca é vazia: a camada 0 não pode ser removida"
    )]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Cria uma camada com o nome informado e valores padrão.
    ///
    /// # Errors
    ///
    /// Falha se o nome for vazio, contiver caractere proibido pelos formatos CAD,
    /// ou colidir com uma camada existente ignorando caixa.
    pub fn create(&mut self, name: impl Into<String>) -> Result<LayerId, LayerError> {
        let name = name.into();
        let normalized = validate(&name)?;

        if self.by_normalized_name.contains_key(&normalized) {
            return Err(LayerError::DuplicateName(name));
        }

        let id = LayerId(self.records.insert(LayerRecord {
            name,
            color: Color::default(),
            linetype: String::from("Continuous"),
            line_weight: LineWeight::Default,
            is_off: false,
            is_frozen: false,
            is_locked: false,
        }));
        self.by_normalized_name.insert(normalized, id);

        Ok(id)
    }

    /// Devolve a camada de `id`, ou `None` se o identificador estiver obsoleto.
    #[must_use]
    pub fn get(&self, id: LayerId) -> Option<&LayerRecord> {
        self.records.get(id.0)
    }

    /// Versão mutável de [`LayerTable::get`].
    ///
    /// Não permite alterar o nome: renomear precisa manter o índice por nome
    /// coerente, e por isso passa por [`LayerTable::rename`].
    #[must_use]
    pub fn get_mut(&mut self, id: LayerId) -> Option<&mut LayerRecord> {
        self.records.get_mut(id.0)
    }

    /// Procura uma camada pelo nome, ignorando caixa.
    #[must_use]
    pub fn id_of(&self, name: &str) -> Option<LayerId> {
        self.by_normalized_name.get(&normalize(name)).copied()
    }

    /// Procura uma camada pelo nome, ignorando caixa, devolvendo o registro.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&LayerRecord> {
        self.get(self.id_of(name)?)
    }

    /// Indica se `id` referencia uma camada viva.
    #[must_use]
    pub fn contains(&self, id: LayerId) -> bool {
        self.records.contains(id.0)
    }

    /// Renomeia uma camada, preservando seu identificador.
    ///
    /// # Errors
    ///
    /// Falha se a camada for a `0`, se o identificador estiver obsoleto, ou se o
    /// nome novo for inválido ou colidir com outra camada. Renomear uma camada
    /// para o próprio nome, mudando apenas a caixa, é permitido.
    pub fn rename(&mut self, id: LayerId, name: impl Into<String>) -> Result<(), LayerError> {
        if id == self.default_layer {
            return Err(LayerError::DefaultLayerIsProtected);
        }

        let name = name.into();
        let normalized = validate(&name)?;

        match self.by_normalized_name.get(&normalized) {
            Some(&existing) if existing != id => {
                return Err(LayerError::DuplicateName(name));
            }
            _ => {}
        }

        let record = self.records.get_mut(id.0).ok_or(LayerError::NotFound)?;
        let previous = normalize(&record.name);
        record.name = name;

        self.by_normalized_name.remove(&previous);
        self.by_normalized_name.insert(normalized, id);

        Ok(())
    }

    /// Remove uma camada e devolve o registro removido.
    ///
    /// # Errors
    ///
    /// Falha se a camada for a `0` ou se o identificador estiver obsoleto.
    ///
    /// Não verifica se há entidades na camada: essa checagem depende da tabela de
    /// entidades e entra junto com o documento, em MT-K1-07.
    pub fn remove(&mut self, id: LayerId) -> Result<LayerRecord, LayerError> {
        if id == self.default_layer {
            return Err(LayerError::DefaultLayerIsProtected);
        }

        let record = self.records.remove(id.0).ok_or(LayerError::NotFound)?;
        self.by_normalized_name.remove(&normalize(&record.name));

        Ok(record)
    }

    /// Itera sobre as camadas em ordem alfabética de nome.
    pub fn iter(&self) -> impl Iterator<Item = (LayerId, &LayerRecord)> {
        self.by_normalized_name.values().filter_map(|&id| {
            let record = self.records.get(id.0)?;
            Some((id, record))
        })
    }

    /// Itera sobre os nomes de exibição em ordem alfabética.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.iter().map(|(_, record)| record.name())
    }
}

impl Default for LayerTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabela_nova_contem_apenas_a_camada_zero() {
        let layers = LayerTable::new();

        assert_eq!(layers.len(), 1);
        assert_eq!(layers.names().collect::<Vec<_>>(), vec![DEFAULT_LAYER_NAME]);
        assert!(layers.contains(layers.default_layer()));
    }

    #[test]
    fn camada_zero_tem_valores_padrao() {
        let layers = LayerTable::new();
        let zero = layers
            .get(layers.default_layer())
            .expect("camada 0 deve existir");

        assert_eq!(zero.name(), "0");
        assert_eq!(zero.color(), Color::Index(7));
        assert_eq!(zero.linetype(), "Continuous");
        assert_eq!(zero.line_weight(), LineWeight::Default);
        assert!(zero.is_visible());
        assert!(!zero.is_off());
        assert!(!zero.is_frozen());
        assert!(!zero.is_locked());
    }

    #[test]
    fn cria_camada_e_recupera_por_id_e_por_nome() {
        let mut layers = LayerTable::new();
        let id = layers.create("Parede").expect("nome válido");

        assert_eq!(layers.get(id).map(LayerRecord::name), Some("Parede"));
        assert_eq!(layers.id_of("Parede"), Some(id));
        assert_eq!(layers.len(), 2);
    }

    #[test]
    fn busca_por_nome_ignora_caixa() {
        let mut layers = LayerTable::new();
        let id = layers.create("Parede").expect("nome válido");

        assert_eq!(layers.id_of("PAREDE"), Some(id));
        assert_eq!(layers.id_of("parede"), Some(id));
        assert_eq!(
            layers.get_by_name("pArEdE").map(LayerRecord::name),
            Some("Parede"),
            "a caixa original é preservada para exibição"
        );
    }

    #[test]
    fn nome_duplicado_e_rejeitado() {
        let mut layers = LayerTable::new();
        layers.create("Parede").expect("nome válido");

        assert_eq!(
            layers.create("Parede"),
            Err(LayerError::DuplicateName(String::from("Parede")))
        );
        assert_eq!(layers.len(), 2, "a tabela não pode ter crescido");
    }

    #[test]
    fn nome_duplicado_e_rejeitado_ignorando_caixa() {
        let mut layers = LayerTable::new();
        layers.create("Parede").expect("nome válido");

        assert_eq!(
            layers.create("PAREDE"),
            Err(LayerError::DuplicateName(String::from("PAREDE")))
        );
    }

    #[test]
    fn nome_da_camada_zero_e_reservado() {
        let mut layers = LayerTable::new();

        assert_eq!(
            layers.create("0"),
            Err(LayerError::DuplicateName(String::from("0")))
        );
    }

    #[test]
    fn nome_vazio_ou_so_espacos_e_rejeitado() {
        let mut layers = LayerTable::new();

        assert_eq!(layers.create(""), Err(LayerError::EmptyName));
        assert_eq!(layers.create("   "), Err(LayerError::EmptyName));
    }

    #[test]
    fn caractere_proibido_e_rejeitado() {
        let mut layers = LayerTable::new();

        assert_eq!(
            layers.create("Parede/Externa"),
            Err(LayerError::ForbiddenCharacter('/'))
        );
        assert_eq!(
            layers.create("Corte:1"),
            Err(LayerError::ForbiddenCharacter(':'))
        );
    }

    #[test]
    fn espacos_nas_bordas_sao_removidos() {
        let mut layers = LayerTable::new();
        let id = layers.create("  Parede  ").expect("nome válido após trim");

        assert_eq!(layers.id_of("Parede"), Some(id));
    }

    #[test]
    fn iteracao_segue_ordem_alfabetica_e_nao_de_criacao() {
        let mut layers = LayerTable::new();
        layers.create("Texto").expect("nome válido");
        layers.create("Cotas").expect("nome válido");
        layers.create("parede").expect("nome válido");

        assert_eq!(
            layers.names().collect::<Vec<_>>(),
            vec!["0", "Cotas", "parede", "Texto"],
            "ordem alfabética do nome normalizado, com a caixa original preservada"
        );
    }

    #[test]
    fn iteracao_e_estavel_entre_chamadas() {
        let mut layers = LayerTable::new();
        layers.create("B").expect("nome válido");
        layers.create("A").expect("nome válido");

        let primeira: Vec<_> = layers.names().collect();
        let segunda: Vec<_> = layers.names().collect();

        assert_eq!(primeira, segunda);
    }

    #[test]
    fn altera_propriedades_da_camada() {
        let mut layers = LayerTable::new();
        let id = layers.create("Parede").expect("nome válido");

        let record = layers.get_mut(id).expect("camada recém-criada");
        record.set_color(Color::Rgb {
            red: 200,
            green: 30,
            blue: 30,
        });
        record.set_line_weight(LineWeight::Hundredths(35));
        record.set_locked(true);

        let record = layers.get(id).expect("camada deve continuar existindo");
        assert_eq!(
            record.color(),
            Color::Rgb {
                red: 200,
                green: 30,
                blue: 30
            }
        );
        assert_eq!(record.line_weight(), LineWeight::Hundredths(35));
        assert!(record.is_locked());
        assert!(record.is_visible(), "bloquear não esconde a camada");
    }

    #[test]
    fn camada_desligada_ou_congelada_nao_e_visivel() {
        let mut layers = LayerTable::new();
        let id = layers.create("Parede").expect("nome válido");

        layers.get_mut(id).expect("camada").set_off(true);
        assert!(!layers.get(id).expect("camada").is_visible());

        layers.get_mut(id).expect("camada").set_off(false);
        layers.get_mut(id).expect("camada").set_frozen(true);
        assert!(!layers.get(id).expect("camada").is_visible());
    }

    #[test]
    fn renomear_preserva_o_identificador() {
        let mut layers = LayerTable::new();
        let id = layers.create("Parede").expect("nome válido");

        layers.rename(id, "Alvenaria").expect("nome novo é válido");

        assert_eq!(layers.get(id).map(LayerRecord::name), Some("Alvenaria"));
        assert_eq!(layers.id_of("Alvenaria"), Some(id));
        assert_eq!(layers.id_of("Parede"), None, "o nome antigo foi liberado");
    }

    #[test]
    fn renomear_para_nome_ocupado_e_rejeitado() {
        let mut layers = LayerTable::new();
        let parede = layers.create("Parede").expect("nome válido");
        layers.create("Cotas").expect("nome válido");

        assert_eq!(
            layers.rename(parede, "COTAS"),
            Err(LayerError::DuplicateName(String::from("COTAS")))
        );
        assert_eq!(layers.get(parede).map(LayerRecord::name), Some("Parede"));
    }

    #[test]
    fn renomear_mudando_apenas_a_caixa_e_permitido() {
        let mut layers = LayerTable::new();
        let id = layers.create("Parede").expect("nome válido");

        layers.rename(id, "PAREDE").expect("mesma camada");

        assert_eq!(layers.get(id).map(LayerRecord::name), Some("PAREDE"));
        assert_eq!(layers.len(), 2);
    }

    #[test]
    fn camada_zero_nao_pode_ser_renomeada() {
        let mut layers = LayerTable::new();
        let zero = layers.default_layer();

        assert_eq!(
            layers.rename(zero, "Base"),
            Err(LayerError::DefaultLayerIsProtected)
        );
        assert_eq!(layers.get(zero).map(LayerRecord::name), Some("0"));
    }

    #[test]
    fn remove_camada_e_libera_o_nome() {
        let mut layers = LayerTable::new();
        let id = layers.create("Parede").expect("nome válido");

        let removida = layers.remove(id).expect("camada existe");

        assert_eq!(removida.name(), "Parede");
        assert_eq!(layers.len(), 1);
        assert_eq!(layers.id_of("Parede"), None);
        assert!(!layers.contains(id));
        assert!(
            layers.create("Parede").is_ok(),
            "o nome deve voltar a ficar disponível"
        );
    }

    #[test]
    fn camada_zero_nao_pode_ser_removida() {
        let mut layers = LayerTable::new();
        let zero = layers.default_layer();

        assert_eq!(
            layers.remove(zero),
            Err(LayerError::DefaultLayerIsProtected)
        );
        assert_eq!(layers.len(), 1);
        assert!(layers.contains(zero));
    }

    #[test]
    fn identificador_obsoleto_e_rejeitado_apos_remocao() {
        let mut layers = LayerTable::new();
        let id = layers.create("Parede").expect("nome válido");
        layers.remove(id).expect("camada existe");

        assert_eq!(layers.get(id), None);
        assert_eq!(layers.remove(id), Err(LayerError::NotFound));
        assert_eq!(layers.rename(id, "Outra"), Err(LayerError::NotFound));
    }

    #[test]
    fn camada_recriada_nao_reaproveita_identificador_antigo() {
        let mut layers = LayerTable::new();
        let antiga = layers.create("Parede").expect("nome válido");
        layers.remove(antiga).expect("camada existe");
        let nova = layers.create("Parede").expect("nome liberado");

        assert_ne!(antiga, nova);
        assert_eq!(layers.get(antiga), None);
        assert_eq!(layers.get(nova).map(LayerRecord::name), Some("Parede"));
    }

    #[test]
    fn exibicao_do_erro_e_legivel() {
        assert_eq!(
            LayerError::ForbiddenCharacter('/').to_string(),
            "o caractere '/' não é aceito em nome de camada"
        );
        assert_eq!(
            LayerError::DefaultLayerIsProtected.to_string(),
            "a camada \"0\" não pode ser removida nem renomeada"
        );
    }

    #[test]
    fn default_equivale_a_new() {
        let layers = LayerTable::default();

        assert_eq!(layers.len(), 1);
        assert!(layers.get_by_name("0").is_some());
    }
}
