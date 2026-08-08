// Caminho relativo: kernel/neocad-model/src/text_style.rs
//! \file kernel/neocad-model/src/text_style.rs
//! \brief Tabela de estilos de texto do documento.
//! \author Iago Leal
//! \date 2026-08-07

use core::fmt;
use std::collections::BTreeMap;

use crate::arena::Arena;
use crate::id::EntityId;
use crate::symbol_name::{normalize, validate, InvalidName};

/// Nome do estilo de texto que todo documento possui e que não pode ser removido.
pub const STANDARD_TEXT_STYLE_NAME: &str = "Standard";

/// Arquivo de fonte adotado pelo estilo padrão.
const DEFAULT_FONT_FILE: &str = "txt.shx";

/// Identificador opaco de um estilo de texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextStyleId(EntityId);

impl fmt::Display for TextStyleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "S{}", self.0)
    }
}

/// Registro de um estilo de texto.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyleRecord {
    name: String,
    font_file: String,
    fixed_height: f64,
    width_factor: f64,
    oblique_angle: f64,
}

impl TextStyleRecord {
    /// Nome de exibição, preservando a caixa com que o estilo foi criado.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Arquivo de fonte associado ao estilo.
    #[must_use]
    pub fn font_file(&self) -> &str {
        &self.font_file
    }

    /// Altura fixa do estilo, em unidades do desenho.
    ///
    /// Zero significa **altura variável**: cada texto informa a sua. É a
    /// convenção do DXF, e não um valor ausente.
    #[must_use]
    pub const fn fixed_height(&self) -> f64 {
        self.fixed_height
    }

    /// Indica se o estilo impõe altura a todos os textos que o usam.
    #[must_use]
    pub fn has_fixed_height(&self) -> bool {
        self.fixed_height > 0.0
    }

    /// Fator de largura: acima de 1 alarga os caracteres, abaixo os estreita.
    #[must_use]
    pub const fn width_factor(&self) -> f64 {
        self.width_factor
    }

    /// Inclinação dos caracteres, em radianos. Zero é vertical.
    #[must_use]
    pub const fn oblique_angle(&self) -> f64 {
        self.oblique_angle
    }

    /// Resolve a altura efetiva de um texto que usa este estilo.
    ///
    /// Um estilo de altura fixa **sobrepõe** a altura informada pela entidade —
    /// é o que faz alterar o estilo redimensionar todos os textos que o adotam.
    /// Um estilo de altura variável devolve a altura da própria entidade.
    #[must_use]
    pub fn effective_height(&self, entity_height: f64) -> f64 {
        if self.has_fixed_height() {
            self.fixed_height
        } else {
            entity_height
        }
    }

    /// Define o arquivo de fonte.
    pub fn set_font_file(&mut self, font_file: impl Into<String>) {
        self.font_file = font_file.into();
    }

    /// Define a altura fixa. Zero volta o estilo a altura variável.
    pub fn set_fixed_height(&mut self, fixed_height: f64) {
        self.fixed_height = fixed_height;
    }

    /// Define o fator de largura.
    pub fn set_width_factor(&mut self, width_factor: f64) {
        self.width_factor = width_factor;
    }

    /// Define a inclinação, em radianos.
    pub fn set_oblique_angle(&mut self, oblique_angle: f64) {
        self.oblique_angle = oblique_angle;
    }
}

/// Falha ao operar sobre a tabela de estilos de texto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextStyleError {
    /// O nome informado é vazio ou só contém espaços.
    EmptyName,
    /// O nome contém um caractere que os formatos CAD não aceitam.
    ForbiddenCharacter(char),
    /// Já existe estilo com esse nome. A comparação ignora caixa.
    DuplicateName(String),
    /// O estilo `Standard` não pode ser removido nem renomeado.
    StandardStyleIsProtected,
    /// O identificador não corresponde a nenhum estilo vivo.
    NotFound,
}

impl From<InvalidName> for TextStyleError {
    fn from(error: InvalidName) -> Self {
        match error {
            InvalidName::Empty => Self::EmptyName,
            InvalidName::Forbidden(character) => Self::ForbiddenCharacter(character),
        }
    }
}

impl fmt::Display for TextStyleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "o nome do estilo não pode ser vazio"),
            Self::ForbiddenCharacter(character) => write!(
                formatter,
                "o caractere {character:?} não é aceito em nome de estilo"
            ),
            Self::DuplicateName(name) => {
                write!(formatter, "já existe um estilo chamado {name:?}")
            }
            Self::StandardStyleIsProtected => write!(
                formatter,
                "o estilo {STANDARD_TEXT_STYLE_NAME:?} não pode ser removido nem renomeado"
            ),
            Self::NotFound => write!(formatter, "estilo de texto não encontrado"),
        }
    }
}

impl core::error::Error for TextStyleError {}

/// Tabela de estilos de texto de um documento.
///
/// Segue as mesmas regras das demais tabelas de símbolos: identidade por
/// [`TextStyleId`], nomes únicos ignorando caixa, iteração alfabética
/// determinística e um registro padrão indestrutível — aqui, `Standard`.
///
/// # Exemplo
///
/// ```
/// use neocad_model::{TextStyleError, TextStyleTable};
///
/// let mut styles = TextStyleTable::new();
/// let cotas = styles.create("Cotas")?;
///
/// styles.get_mut(cotas).expect("estilo recém-criado").set_fixed_height(2.5);
///
/// // Um estilo de altura fixa sobrepõe a altura informada pelo texto.
/// let estilo = styles.get(cotas).expect("estilo existe");
/// assert_eq!(estilo.effective_height(10.0), 2.5);
/// # Ok::<(), TextStyleError>(())
/// ```
#[derive(Debug, Clone)]
pub struct TextStyleTable {
    records: Arena<TextStyleRecord>,
    by_normalized_name: BTreeMap<String, TextStyleId>,
    standard: TextStyleId,
}

impl TextStyleTable {
    /// Cria uma tabela contendo apenas o estilo `Standard`.
    #[must_use]
    pub fn new() -> Self {
        let mut records = Arena::new();
        let standard = TextStyleId(records.insert(TextStyleRecord {
            name: String::from(STANDARD_TEXT_STYLE_NAME),
            font_file: String::from(DEFAULT_FONT_FILE),
            fixed_height: 0.0,
            width_factor: 1.0,
            oblique_angle: 0.0,
        }));

        let mut by_normalized_name = BTreeMap::new();
        by_normalized_name.insert(normalize(STANDARD_TEXT_STYLE_NAME), standard);

        Self {
            records,
            by_normalized_name,
            standard,
        }
    }

    /// Identificador do estilo `Standard`, sempre presente.
    #[must_use]
    pub const fn standard(&self) -> TextStyleId {
        self.standard
    }

    /// Quantidade de estilos. Nunca é zero.
    #[must_use]
    #[expect(
        clippy::len_without_is_empty,
        reason = "a tabela nunca é vazia: o estilo Standard não pode ser removido"
    )]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Cria um estilo com o nome informado e valores padrão.
    ///
    /// # Errors
    ///
    /// Falha se o nome for inválido ou colidir com um estilo existente,
    /// ignorando caixa.
    pub fn create(&mut self, name: impl Into<String>) -> Result<TextStyleId, TextStyleError> {
        let name = name.into();
        let normalized = validate(&name)?;

        if self.by_normalized_name.contains_key(&normalized) {
            return Err(TextStyleError::DuplicateName(name));
        }

        let id = TextStyleId(self.records.insert(TextStyleRecord {
            name,
            font_file: String::from(DEFAULT_FONT_FILE),
            fixed_height: 0.0,
            width_factor: 1.0,
            oblique_angle: 0.0,
        }));
        self.by_normalized_name.insert(normalized, id);

        Ok(id)
    }

    /// Resolve um identificador para o registro correspondente.
    ///
    /// Devolve `None` quando o identificador está obsoleto — o estilo foi
    /// removido depois de a referência ter sido obtida.
    #[must_use]
    pub fn get(&self, id: TextStyleId) -> Option<&TextStyleRecord> {
        self.records.get(id.0)
    }

    /// Versão mutável de [`TextStyleTable::get`].
    ///
    /// Não permite alterar o nome: renomear passa por
    /// [`TextStyleTable::rename`], que mantém o índice coerente.
    #[must_use]
    pub fn get_mut(&mut self, id: TextStyleId) -> Option<&mut TextStyleRecord> {
        self.records.get_mut(id.0)
    }

    /// Procura um estilo pelo nome, ignorando caixa.
    #[must_use]
    pub fn id_of(&self, name: &str) -> Option<TextStyleId> {
        self.by_normalized_name.get(&normalize(name)).copied()
    }

    /// Procura um estilo pelo nome, ignorando caixa, devolvendo o registro.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&TextStyleRecord> {
        self.get(self.id_of(name)?)
    }

    /// Indica se `id` referencia um estilo vivo.
    #[must_use]
    pub fn contains(&self, id: TextStyleId) -> bool {
        self.records.contains(id.0)
    }

    /// Renomeia um estilo, preservando seu identificador.
    ///
    /// # Errors
    ///
    /// Falha se o estilo for o `Standard`, se o identificador estiver obsoleto,
    /// ou se o nome novo for inválido ou colidir com outro estilo.
    pub fn rename(
        &mut self,
        id: TextStyleId,
        name: impl Into<String>,
    ) -> Result<(), TextStyleError> {
        if id == self.standard {
            return Err(TextStyleError::StandardStyleIsProtected);
        }

        let name = name.into();
        let normalized = validate(&name)?;

        if let Some(&existing) = self.by_normalized_name.get(&normalized) {
            if existing != id {
                return Err(TextStyleError::DuplicateName(name));
            }
        }

        let record = self.records.get_mut(id.0).ok_or(TextStyleError::NotFound)?;
        let previous = normalize(&record.name);
        record.name = name;

        self.by_normalized_name.remove(&previous);
        self.by_normalized_name.insert(normalized, id);

        Ok(())
    }

    /// Remove um estilo e devolve o registro removido.
    ///
    /// # Errors
    ///
    /// Falha se o estilo for o `Standard` ou se o identificador estiver obsoleto.
    ///
    /// Não verifica se há textos usando o estilo: essa checagem depende da
    /// tabela de entidades e entra com o documento, em MT-K1-07.
    pub fn remove(&mut self, id: TextStyleId) -> Result<TextStyleRecord, TextStyleError> {
        if id == self.standard {
            return Err(TextStyleError::StandardStyleIsProtected);
        }

        let record = self.records.remove(id.0).ok_or(TextStyleError::NotFound)?;
        self.by_normalized_name.remove(&normalize(&record.name));

        Ok(record)
    }

    /// Itera sobre os estilos em ordem alfabética de nome.
    pub fn iter(&self) -> impl Iterator<Item = (TextStyleId, &TextStyleRecord)> {
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

impl Default for TextStyleTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabela_nova_contem_apenas_o_estilo_standard() {
        let styles = TextStyleTable::new();

        assert_eq!(styles.len(), 1);
        assert_eq!(
            styles.names().collect::<Vec<_>>(),
            vec![STANDARD_TEXT_STYLE_NAME]
        );
        assert!(styles.contains(styles.standard()));
    }

    #[test]
    fn estilo_standard_tem_valores_padrao() {
        let styles = TextStyleTable::new();
        let standard = styles.get(styles.standard()).expect("Standard existe");

        assert_eq!(standard.name(), "Standard");
        assert_eq!(standard.font_file(), "txt.shx");
        assert_eq!(standard.fixed_height(), 0.0);
        assert!(!standard.has_fixed_height());
        assert_eq!(standard.width_factor(), 1.0);
        assert_eq!(standard.oblique_angle(), 0.0);
    }

    #[test]
    fn cria_estilo_e_resolve_a_referencia() {
        let mut styles = TextStyleTable::new();
        let id = styles.create("Cotas").expect("nome válido");

        assert_eq!(styles.get(id).map(TextStyleRecord::name), Some("Cotas"));
        assert_eq!(styles.id_of("cotas"), Some(id));
        assert_eq!(styles.len(), 2);
    }

    #[test]
    fn referencia_obsoleta_nao_resolve() {
        let mut styles = TextStyleTable::new();
        let id = styles.create("Cotas").expect("nome válido");
        styles.remove(id).expect("estilo existe");

        assert_eq!(styles.get(id), None);
        assert!(!styles.contains(id));
        assert_eq!(styles.remove(id), Err(TextStyleError::NotFound));
    }

    #[test]
    fn nome_duplicado_e_rejeitado_ignorando_caixa() {
        let mut styles = TextStyleTable::new();
        styles.create("Cotas").expect("nome válido");

        assert_eq!(
            styles.create("COTAS"),
            Err(TextStyleError::DuplicateName(String::from("COTAS")))
        );
    }

    #[test]
    fn nome_invalido_e_rejeitado_com_as_mesmas_regras_das_camadas() {
        let mut styles = TextStyleTable::new();

        assert_eq!(styles.create("  "), Err(TextStyleError::EmptyName));
        assert_eq!(
            styles.create("Cotas/Grande"),
            Err(TextStyleError::ForbiddenCharacter('/'))
        );
    }

    #[test]
    fn altura_fixa_sobrepoe_a_altura_da_entidade() {
        let mut styles = TextStyleTable::new();
        let id = styles.create("Cotas").expect("nome válido");
        styles
            .get_mut(id)
            .expect("estilo existe")
            .set_fixed_height(2.5);

        let estilo = styles.get(id).expect("estilo existe");

        assert!(estilo.has_fixed_height());
        assert_eq!(estilo.effective_height(10.0), 2.5);
    }

    #[test]
    fn altura_variavel_preserva_a_altura_da_entidade() {
        let styles = TextStyleTable::new();
        let standard = styles.get(styles.standard()).expect("Standard existe");

        assert_eq!(standard.effective_height(7.0), 7.0);
    }

    #[test]
    fn altura_fixa_zerada_volta_a_ser_variavel() {
        let mut styles = TextStyleTable::new();
        let id = styles.create("Cotas").expect("nome válido");

        let record = styles.get_mut(id).expect("estilo existe");
        record.set_fixed_height(3.0);
        record.set_fixed_height(0.0);

        assert!(!styles.get(id).expect("estilo existe").has_fixed_height());
        assert_eq!(
            styles.get(id).expect("estilo existe").effective_height(9.0),
            9.0
        );
    }

    #[test]
    fn altera_fonte_largura_e_inclinacao() {
        let mut styles = TextStyleTable::new();
        let id = styles.create("Titulo").expect("nome válido");

        let record = styles.get_mut(id).expect("estilo existe");
        record.set_font_file("arial.ttf");
        record.set_width_factor(0.8);
        record.set_oblique_angle(0.26);

        let record = styles.get(id).expect("estilo existe");
        assert_eq!(record.font_file(), "arial.ttf");
        assert_eq!(record.width_factor(), 0.8);
        assert_eq!(record.oblique_angle(), 0.26);
    }

    #[test]
    fn estilo_standard_e_protegido() {
        let mut styles = TextStyleTable::new();
        let standard = styles.standard();

        assert_eq!(
            styles.remove(standard),
            Err(TextStyleError::StandardStyleIsProtected)
        );
        assert_eq!(
            styles.rename(standard, "Base"),
            Err(TextStyleError::StandardStyleIsProtected)
        );
        assert_eq!(styles.len(), 1);
    }

    #[test]
    fn renomear_preserva_o_identificador() {
        let mut styles = TextStyleTable::new();
        let id = styles.create("Cotas").expect("nome válido");

        styles.rename(id, "Dimensoes").expect("nome novo é válido");

        assert_eq!(styles.get(id).map(TextStyleRecord::name), Some("Dimensoes"));
        assert_eq!(styles.id_of("Cotas"), None);
    }

    #[test]
    fn iteracao_segue_ordem_alfabetica() {
        let mut styles = TextStyleTable::new();
        styles.create("Titulo").expect("nome válido");
        styles.create("Cotas").expect("nome válido");

        assert_eq!(
            styles.names().collect::<Vec<_>>(),
            vec!["Cotas", "Standard", "Titulo"]
        );
    }

    #[test]
    fn default_equivale_a_new() {
        assert_eq!(TextStyleTable::default().len(), 1);
    }
}
