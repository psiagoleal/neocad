// Caminho relativo: kernel/neocad-transaction/src/effect.rs
//! \file kernel/neocad-transaction/src/effect.rs
//! \brief O que uma operação da pilha fez ao documento, em identificadores.
//! \author Iago Leal
//! \date 2026-10-30

use std::collections::BTreeSet;

use neocad_model::{Change, EntityId, LayerId};

/// O que a última operação da pilha fez ao documento.
///
/// Existe para que quem desenha a tela atualize só o que mudou, em vez de listar
/// o documento inteiro a cada edição (ADR 0008).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Effect {
    /// Entidades que passaram a existir, em ordem crescente.
    pub created: Vec<EntityId>,
    /// Entidades que continuam existindo com conteúdo ou posição novos.
    pub modified: Vec<EntityId>,
    /// Entidades que deixaram de existir.
    pub removed: Vec<EntityId>,
    /// Camadas cujo registro mudou.
    pub layers: Vec<LayerId>,
}

impl Effect {
    /// Verdadeiro quando a operação não mudou nada.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.created.is_empty()
            && self.modified.is_empty()
            && self.removed.is_empty()
            && self.layers.is_empty()
    }
}

impl Effect {
    /// Deduz o efeito a partir das mudanças que **desfazem** a operação.
    ///
    /// A pilha só guarda as inversas, e elas bastam: desfazer uma inserção é
    /// remover, desfazer uma remoção é reinserir, e substituir ou mover se desfaz
    /// do mesmo jeito. Entidade que nasceu e morreu dentro da mesma operação não
    /// aparece: para a tela, nada mudou.
    #[must_use]
    pub fn from_inverse(changes: &[Change]) -> Self {
        let mut inseridas = BTreeSet::new();
        let mut removidas = BTreeSet::new();
        let mut tocadas = BTreeSet::new();
        let mut camadas = BTreeSet::new();

        for change in changes {
            match change {
                Change::RemoveEntity { entity } => {
                    inseridas.insert(*entity);
                }
                Change::InsertEntity { entity, .. } => {
                    removidas.insert(*entity);
                }
                Change::ReplaceEntity { entity, .. } | Change::MoveEntity { entity, .. } => {
                    tocadas.insert(*entity);
                }
                Change::SetLayerRecord { layer, .. } => {
                    camadas.insert(*layer);
                }
            }
        }

        Self {
            created: inseridas.difference(&removidas).copied().collect(),
            removed: removidas.difference(&inseridas).copied().collect(),
            modified: tocadas
                .into_iter()
                .filter(|id| !inseridas.contains(id) && !removidas.contains(id))
                .collect(),
            layers: camadas.into_iter().collect(),
        }
    }
}
