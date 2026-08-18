// Caminho relativo: kernel/neocad-model/src/lib.rs
//! \file kernel/neocad-model/src/lib.rs
//! \brief Modelo de documento do kernel CAD do NeoCAD.
//! \author Iago Leal
//! \date 2026-08-06
//!
//! Responsabilidade: identificadores de entidade, arena, entidades de desenho e
//! tabelas de símbolos (camadas, blocos, estilos), agregados em um documento.
//!
//! É o núcleo da fase K1 e passa a ser a fonte de verdade sobre o desenho, no
//! lugar do modelo do upstream. Recebe conteúdo de MT-K1-03 a MT-K1-07.
//!
//! A mutação do documento é fechada atrás de `neocad-transaction` em MT-K1-10:
//! esta crate não expõe caminho público de escrita que escape do journal.
//!
//! Conforme o ADR 0003, esta crate não conhece Tauri, Svelte, DOM nem tipos do
//! NeoCAD, e não pode receber dependências copyleft.

mod arena;
mod block;
mod change;
mod document;
mod entity;
mod id;
mod layer;
mod layout;
mod text_style;

pub use arena::{Arena, RestoreError};
pub use block::{BlockError, BlockId, BlockRecord, BlockTable, MODEL_SPACE_NAME};
pub use change::{Change, ChangeError};
pub use document::{Document, DocumentEditor, DocumentError, EntityPlacement};
pub use entity::{Arc, Circle, Entity, EntityColor, Geometry, Line, Polyline, Text};
pub use id::EntityId;
pub use layer::{
    Color, LayerError, LayerId, LayerRecord, LayerTable, LineWeight, DEFAULT_LAYER_NAME,
};
pub use layout::{
    LayoutError, LayoutId, LayoutRecord, LayoutTable, PageSetup, PlotMargins, PlotRotation,
    PlotUnits, MODEL_LAYOUT_NAME,
};
pub use text_style::{
    TextStyleError, TextStyleId, TextStyleRecord, TextStyleTable, STANDARD_TEXT_STYLE_NAME,
};

pub(crate) mod symbol_table;

/// Regras de nome comuns às tabelas de símbolos.
///
/// Camadas, blocos e estilos de texto são, nos formatos CAD, o mesmo conceito —
/// registros de tabela de símbolos — e compartilham as mesmas restrições de
/// nome. Centralizar aqui garante que as três não divirjam: se um nome é
/// rejeitado como camada, também é como bloco.
pub(crate) mod symbol_name {
    /// Caracteres que os formatos DXF e DWG não aceitam em nome de símbolo.
    ///
    /// O asterisco é reservado aos nomes internos do próprio formato, como o
    /// bloco `*Model_Space`, e por isso é proibido em nomes criados pelo usuário.
    const FORBIDDEN_CHARS: [char; 11] = ['<', '>', '/', '\\', '"', ':', ';', '?', '*', '|', '='];

    /// Motivo pelo qual um nome de símbolo foi recusado.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum InvalidName {
        /// Vazio ou composto apenas de espaços.
        Empty,
        /// Contém um caractere não aceito pelos formatos CAD.
        Forbidden(char),
    }

    /// Normaliza um nome para efeito de unicidade e ordenação.
    ///
    /// A comparação ignora caixa e espaços nas bordas, como nos formatos CAD.
    pub(crate) fn normalize(name: &str) -> String {
        name.trim().to_uppercase()
    }

    /// Prefixo que os formatos DXF e DWG reservam aos nomes do próprio sistema.
    pub(crate) const RESERVED_PREFIX: char = '*';

    /// Valida um nome **reservado**, que começa por asterisco.
    ///
    /// É a via pela qual a própria crate cria `*Model_Space` e os blocos de
    /// espaço-papel. O asterisco inicial é dispensado da lista de proibidos; o
    /// resto do nome segue as mesmas regras de qualquer símbolo, para que um
    /// nome reservado não possa carregar barra ou dois-pontos só por ser
    /// reservado.
    ///
    /// Não é `pub`: fora da crate, nome com asterisco continua recusado, e é o
    /// compilador que garante isso (ADR 0005).
    pub(crate) fn validate_reserved(name: &str) -> Result<String, InvalidName> {
        let trimmed = name.trim();
        let Some(rest) = trimmed.strip_prefix(RESERVED_PREFIX) else {
            return Err(InvalidName::Forbidden(RESERVED_PREFIX));
        };

        if rest.is_empty() {
            return Err(InvalidName::Empty);
        }

        if let Some(character) = rest.chars().find(|c| FORBIDDEN_CHARS.contains(c)) {
            return Err(InvalidName::Forbidden(character));
        }

        Ok(normalize(trimmed))
    }

    /// Valida um nome e devolve sua forma normalizada.
    pub(crate) fn validate(name: &str) -> Result<String, InvalidName> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(InvalidName::Empty);
        }

        if let Some(character) = trimmed.chars().find(|c| FORBIDDEN_CHARS.contains(c)) {
            return Err(InvalidName::Forbidden(character));
        }

        Ok(normalize(trimmed))
    }
}
