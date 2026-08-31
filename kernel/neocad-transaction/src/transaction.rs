// Caminho relativo: kernel/neocad-transaction/src/transaction.rs
//! \file kernel/neocad-transaction/src/transaction.rs
//! \brief Agrupamento nomeado de mudanças, aplicado como unidade atômica.
//! \author Iago Leal
//! \date 2026-08-07

use core::fmt;

use neocad_model::{Change, ChangeError, Document};

/// Falha ao aplicar uma transação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    /// Uma mudança falhou. O documento foi devolvido ao estado anterior.
    Change {
        /// Posição da mudança que falhou, dentro da transação.
        index: usize,
        /// Motivo da falha.
        source: ChangeError,
    },
    /// A edição foi recusada pelo documento.
    Edit(neocad_model::DocumentError),
    /// Uma mudança falhou e a reversão das anteriores também falhou.
    ///
    /// Indica quebra de invariante do kernel: as inversas são construídas a
    /// partir do estado observado e deveriam sempre aplicar. O documento pode
    /// ter ficado em estado intermediário.
    RollbackFailed {
        /// Posição da mudança que falhou originalmente.
        index: usize,
        /// Motivo da falha original.
        source: ChangeError,
    },
}

impl fmt::Display for TransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Change { index, source } => {
                write!(formatter, "mudança {index} da transação falhou: {source}")
            }
            Self::Edit(error) => write!(formatter, "a edição foi recusada: {error}"),
            Self::RollbackFailed { index, source } => write!(
                formatter,
                "mudança {index} falhou ({source}) e a reversão não pôde ser concluída"
            ),
        }
    }
}

impl core::error::Error for TransactionError {}

/// Conjunto nomeado de mudanças aplicado como uma unidade.
///
/// # Por que agrupar
///
/// Uma única ação do usuário costuma corresponder a várias mudanças — apagar uma
/// seleção remove N entidades; mover altera N geometrias. O que precisa ser
/// desfeito é a **ação**, não cada mudança isolada. O nome existe para a
/// interface poder dizer o que será desfeito.
///
/// # Atomicidade
///
/// [`Transaction::apply`] é tudo ou nada: se uma mudança falhar, as anteriores
/// são revertidas e o documento fica como estava. Uma transação parcialmente
/// aplicada não teria como ser desfeita de forma confiável.
///
/// # Inversão
///
/// Aplicar devolve a transação que desfaz — mesmo nome, mudanças invertidas e em
/// ordem contrária. Aplicar essa inversa reconstrói a transação original, o que
/// faz `undo` e `redo` serem a mesma operação em sentidos opostos.
///
/// # Construção
///
/// Uma transação carrega mudanças com identificadores explícitos e **não aloca**
/// nada. Criar entidade nova exige pedir o identificador ao documento primeiro;
/// a API que faz isso e grava a transação correspondente chega em MT-K1-10, ao
/// fechar a mutação do documento atrás das transações.
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    name: String,
    changes: Vec<Change>,
}

impl Transaction {
    /// Cria uma transação nomeada com as mudanças informadas, na ordem em que
    /// devem ser aplicadas.
    #[must_use]
    pub fn new(name: impl Into<String>, changes: Vec<Change>) -> Self {
        Self {
            name: name.into(),
            changes,
        }
    }

    /// Cria uma transação vazia, apenas com nome.
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        Self::new(name, Vec::new())
    }

    /// Nome exibível da ação.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Mudanças que compõem a transação, na ordem de aplicação.
    #[must_use]
    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    /// Quantidade de mudanças.
    #[must_use]
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Indica se a transação não contém mudança alguma.
    ///
    /// Uma transação vazia não altera o documento e não deve entrar na pilha de
    /// desfazer: registrá-la faria o usuário apertar `Desfazer` e nada acontecer.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Acrescenta uma mudança ao fim da transação.
    pub fn push(&mut self, change: Change) {
        self.changes.push(change);
    }

    /// Aplica todas as mudanças e devolve a transação que as desfaz.
    ///
    /// # Errors
    ///
    /// Falha se alguma mudança não puder ser aplicada. As mudanças já aplicadas
    /// são revertidas antes do retorno, de modo que o documento fica como
    /// estava.
    pub fn apply(self, document: &mut Document) -> Result<Self, TransactionError> {
        let name = self.name;
        let mut inverses: Vec<Change> = Vec::with_capacity(self.changes.len());

        for (index, change) in self.changes.into_iter().enumerate() {
            match change.apply(document) {
                Ok(inverse) => inverses.push(inverse),
                Err(source) => {
                    // Reverte o que já foi aplicado, de trás para frente.
                    for inverse in inverses.into_iter().rev() {
                        if inverse.apply(document).is_err() {
                            return Err(TransactionError::RollbackFailed { index, source });
                        }
                    }

                    return Err(TransactionError::Change { index, source });
                }
            }
        }

        // A inversa aplica na ordem contrária à da aplicação original.
        inverses.reverse();

        Ok(Self {
            name,
            changes: inverses,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neocad_geometry::Point2;
    use neocad_model::{Circle, Color, Entity, EntityId, Geometry, LayerId, Line};

    fn line(layer: LayerId, x: f64) -> Entity {
        Entity::new(
            layer,
            Geometry::Line(Line {
                start: Point2::new(x, 0.0),
                end: Point2::new(x + 1.0, 1.0),
            }),
        )
    }

    fn circle(layer: LayerId) -> Entity {
        Entity::new(
            layer,
            Geometry::Circle(Circle {
                center: Point2::ORIGIN,
                radius: 5.0,
            }),
        )
    }

    fn documento() -> (Document, LayerId, Vec<EntityId>) {
        let mut document = Document::new();
        let parede = document.create_layer("Parede").expect("nome válido");
        let zero = document.layers().default_layer();

        let mut editor = document.edit();
        let ids = vec![
            editor
                .insert_in_model_space(line(zero, 0.0))
                .expect("insere"),
            editor
                .insert_in_model_space(line(parede, 5.0))
                .expect("insere"),
            editor
                .insert_in_model_space(line(zero, 10.0))
                .expect("insere"),
        ];
        let _ = editor.finish();

        (document, parede, ids)
    }

    #[test]
    fn transacao_vazia_e_reconhecivel() {
        let transaction = Transaction::named("Nada");

        assert!(transaction.is_empty());
        assert_eq!(transaction.len(), 0);
        assert_eq!(transaction.name(), "Nada");
    }

    #[test]
    fn aplicar_transacao_vazia_nao_altera_o_documento() {
        let (mut document, _, _) = documento();
        let inicial = document.clone();

        let inversa = Transaction::named("Nada")
            .apply(&mut document)
            .expect("aplica");

        assert_eq!(document, inicial);
        assert!(inversa.is_empty());
    }

    #[test]
    fn aplica_varias_mudancas_como_unidade() {
        let (mut document, _, ids) = documento();

        let transaction = Transaction::new(
            "Apagar seleção",
            vec![
                Change::RemoveEntity { entity: ids[0] },
                Change::RemoveEntity { entity: ids[2] },
            ],
        );

        let desfazer = transaction.apply(&mut document).expect("aplica");

        assert_eq!(document.entity_count(), 1);
        assert_eq!(desfazer.name(), "Apagar seleção");
        assert_eq!(desfazer.len(), 2);
    }

    #[test]
    fn desfazer_restaura_o_documento_por_inteiro() {
        let (mut document, parede, ids) = documento();
        let inicial = document.clone();

        let mut record = document.layers().get(parede).expect("existe").clone();
        record.set_color(Color::Index(5));

        let transaction = Transaction::new(
            "Editar",
            vec![
                Change::RemoveEntity { entity: ids[0] },
                Change::ReplaceEntity {
                    entity: ids[1],
                    content: Box::new(circle(parede)),
                },
                Change::SetLayerRecord {
                    layer: parede,
                    record: Box::new(record),
                },
            ],
        );

        let desfazer = transaction.apply(&mut document).expect("aplica");
        assert_ne!(document, inicial);

        desfazer.apply(&mut document).expect("desfaz");

        assert_eq!(document, inicial);
    }

    #[test]
    fn refazer_reconstroi_a_transacao_original() {
        let (mut document, _, ids) = documento();

        let original = Transaction::new(
            "Apagar",
            vec![
                Change::RemoveEntity { entity: ids[0] },
                Change::RemoveEntity { entity: ids[1] },
            ],
        );
        let aplicado = document.clone();

        let desfazer = original.clone().apply(&mut document).expect("aplica");
        let depois_de_aplicar = document.clone();

        let refazer = desfazer.apply(&mut document).expect("desfaz");
        assert_eq!(document, aplicado);

        refazer.apply(&mut document).expect("refaz");
        assert_eq!(
            document, depois_de_aplicar,
            "refazer tem de reproduzir exatamente o efeito original"
        );
    }

    #[test]
    fn a_inversa_aplica_na_ordem_contraria() {
        let (mut document, _, ids) = documento();

        // Remover ids[0] e depois ids[1]; desfazer tem de restaurar ids[1]
        // antes de ids[0], para que cada um volte à sua posição.
        let transaction = Transaction::new(
            "Apagar",
            vec![
                Change::RemoveEntity { entity: ids[0] },
                Change::RemoveEntity { entity: ids[1] },
            ],
        );

        let desfazer = transaction.apply(&mut document).expect("aplica");
        desfazer.apply(&mut document).expect("desfaz");

        let ordem: Vec<_> = document
            .entities_in_block(document.model_space())
            .map(|(id, _)| id)
            .collect();

        assert_eq!(ordem, ids, "a ordem de desenho tem de ser reconstruída");
    }

    #[test]
    fn falha_no_meio_reverte_as_mudancas_anteriores() {
        let (mut document, _, ids) = documento();
        let inicial = document.clone();

        // A segunda mudança falha: a entidade já foi removida pela primeira.
        let transaction = Transaction::new(
            "Inválida",
            vec![
                Change::RemoveEntity { entity: ids[0] },
                Change::RemoveEntity { entity: ids[0] },
            ],
        );

        let erro = transaction.apply(&mut document).expect_err("deve falhar");

        assert!(
            matches!(erro, TransactionError::Change { index: 1, .. }),
            "erro inesperado: {erro}"
        );
        assert_eq!(
            document, inicial,
            "uma transação que falha não pode deixar efeito pela metade"
        );
    }

    #[test]
    fn falha_na_primeira_mudanca_nao_deixa_efeito() {
        let (mut document, _, ids) = documento();
        let _ = document.edit().remove_entity(ids[0]).expect("existe");
        let inicial = document.clone();

        let erro = Transaction::new("Inválida", vec![Change::RemoveEntity { entity: ids[0] }])
            .apply(&mut document)
            .expect_err("deve falhar");

        assert!(matches!(erro, TransactionError::Change { index: 0, .. }));
        assert_eq!(document, inicial);
    }

    #[test]
    fn push_acrescenta_mudanca() {
        let (_, _, ids) = documento();
        let mut transaction = Transaction::named("Apagar");

        transaction.push(Change::RemoveEntity { entity: ids[0] });

        assert_eq!(transaction.len(), 1);
        assert_eq!(transaction.changes().len(), 1);
    }

    #[test]
    fn nome_sobrevive_a_inversao() {
        let (mut document, _, ids) = documento();

        let desfazer = Transaction::new(
            "Apagar linha",
            vec![Change::RemoveEntity { entity: ids[0] }],
        )
        .apply(&mut document)
        .expect("aplica");

        assert_eq!(desfazer.name(), "Apagar linha");
    }
}
