// Caminho relativo: kernel/neocad-model/src/change.rs
//! \file kernel/neocad-model/src/change.rs
//! \brief Mudança atômica e reversível sobre o documento.
//! \author Iago Leal
//! \date 2026-08-07

use core::fmt;

use crate::document::{Document, DocumentError, EntityPlacement};
use crate::entity::Entity;
use crate::id::EntityId;
use crate::layer::{LayerId, LayerRecord};

/// Falha ao aplicar uma mudança.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeError {
    /// A entidade referenciada não existe ou o identificador está obsoleto.
    UnknownEntity(EntityId),
    /// A entidade existe, mas não está em bloco algum.
    ///
    /// Não deveria ocorrer sob as invariantes do documento; é relatado em vez de
    /// ignorado para que uma quebra de invariante apareça onde acontece.
    EntityNotPlaced(EntityId),
    /// A operação foi recusada pelo documento.
    Document(DocumentError),
}

impl From<DocumentError> for ChangeError {
    fn from(error: DocumentError) -> Self {
        Self::Document(error)
    }
}

impl fmt::Display for ChangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownEntity(id) => write!(formatter, "entidade {id} não existe no documento"),
            Self::EntityNotPlaced(id) => {
                write!(formatter, "entidade {id} não pertence a bloco algum")
            }
            Self::Document(error) => write!(formatter, "{error}"),
        }
    }
}

impl core::error::Error for ChangeError {}

/// Mudança atômica e reversível sobre o documento.
///
/// # O journal descreve, não decide
///
/// Uma `Change` nunca aloca identificador: ela sempre carrega o identificador
/// sobre o qual atua. É o registro de uma mutação que **já aconteceu** — ou que
/// está sendo reproduzida — e por isso pode ser reaplicada nas duas direções sem
/// que o resultado dependa do estado do alocador.
///
/// Criar uma entidade nova é operação da camada acima ([`crate::Transaction`],
/// em MT-K1-09): ela pede a inserção ao documento, recebe o identificador
/// emitido e registra a `Change` correspondente.
///
/// # Inversão exata
///
/// [`Change::apply`] devolve a mudança que a desfaz, construída a partir do
/// estado observado no momento da aplicação — e não deduzida do enunciado. É o
/// que garante que desfazer restaure identificador, bloco, posição na ordem de
/// desenho e conteúdo anterior, em vez de apenas algo equivalente.
///
/// Aplicar uma mudança e em seguida a sua inversa devolve o documento a um
/// estado igual ao inicial, no sentido de [`Document`]: mesmo conteúdo
/// observável, ainda que reste um slot vago na arena.
///
/// # Exemplo
///
/// ```
/// use neocad_model::{Change, Document, Entity, Geometry, Line};
/// use neocad_geometry::Point2;
///
/// let mut document = Document::new();
/// let zero = document.layers().default_layer();
///
/// let mut editor = document.edit();
/// let id = editor.insert_in_model_space(Entity::new(
///     zero,
///     Geometry::Line(Line { start: Point2::ORIGIN, end: Point2::new(1.0, 1.0) }),
/// ))?;
/// let _ = editor.finish();
///
/// let antes = document.clone();
///
/// let desfazer = Change::RemoveEntity { entity: id }.apply(&mut document)?;
/// assert!(document.entity(id).is_none());
///
/// desfazer.apply(&mut document)?;
/// assert_eq!(document, antes, "o documento voltou ao estado inicial");
/// # Ok::<(), Box<dyn core::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    /// Restaura uma entidade em identificador, bloco e posição determinados.
    InsertEntity {
        /// Identificador a restaurar.
        entity: EntityId,
        /// Conteúdo da entidade.
        content: Box<Entity>,
        /// Bloco e posição na ordem de desenho.
        placement: EntityPlacement,
    },
    /// Remove uma entidade do documento.
    RemoveEntity {
        /// Identificador a remover.
        entity: EntityId,
    },
    /// Substitui o conteúdo de uma entidade, preservando identificador e posição.
    ReplaceEntity {
        /// Identificador a alterar.
        entity: EntityId,
        /// Conteúdo novo.
        content: Box<Entity>,
    },
    /// Move uma entidade para outro bloco e posição na ordem de desenho.
    MoveEntity {
        /// Identificador a mover.
        entity: EntityId,
        /// Bloco e posição de destino.
        placement: EntityPlacement,
    },
    /// Substitui o registro de uma camada por inteiro.
    SetLayerRecord {
        /// Camada a alterar.
        layer: LayerId,
        /// Registro novo.
        record: Box<LayerRecord>,
    },
}

impl Change {
    /// Aplica a mudança e devolve a mudança que a desfaz.
    ///
    /// # Errors
    ///
    /// Falha se a mudança não puder ser aplicada ao estado atual — entidade
    /// inexistente, camada ou bloco ausentes, identificador já ocupado. Em caso
    /// de falha o documento permanece inalterado.
    pub fn apply(self, document: &mut Document) -> Result<Self, ChangeError> {
        match self {
            Self::InsertEntity {
                entity,
                content,
                placement,
            } => {
                document.restore_entity(entity, *content, placement)?;

                Ok(Self::RemoveEntity { entity })
            }

            Self::RemoveEntity { entity } => {
                // A inversa é montada a partir do estado observado agora, antes
                // de remover: é isso que a torna exata.
                let placement = document
                    .entity_placement(entity)
                    .ok_or(ChangeError::EntityNotPlaced(entity))?;
                let content = document
                    .remove_entity(entity)
                    .ok_or(ChangeError::UnknownEntity(entity))?;

                Ok(Self::InsertEntity {
                    entity,
                    content: Box::new(content),
                    placement,
                })
            }

            Self::ReplaceEntity { entity, content } => {
                let previous = document.replace_entity(entity, *content)?;

                Ok(Self::ReplaceEntity {
                    entity,
                    content: Box::new(previous),
                })
            }

            Self::MoveEntity { entity, placement } => {
                let previous = document.set_entity_placement(entity, placement)?;

                Ok(Self::MoveEntity {
                    entity,
                    placement: previous,
                })
            }

            Self::SetLayerRecord { layer, record } => {
                let previous = document.set_layer_record(layer, *record)?;

                Ok(Self::SetLayerRecord {
                    layer,
                    record: Box::new(previous),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Circle, Geometry, Line};
    use crate::layer::Color;
    use neocad_geometry::Point2;

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

    /// Documento com três entidades no espaço-modelo e uma camada extra.
    fn documento() -> (Document, LayerId, Vec<EntityId>) {
        let mut document = Document::new();
        let parede = document.create_layer("Parede").expect("nome válido");
        let zero = document.layers().default_layer();

        let ids = vec![
            document
                .insert_in_model_space(line(zero, 0.0))
                .expect("insere"),
            document
                .insert_in_model_space(line(parede, 5.0))
                .expect("insere"),
            document
                .insert_in_model_space(line(zero, 10.0))
                .expect("insere"),
        ];

        (document, parede, ids)
    }

    /// Aplica a mudança, aplica a inversa e exige o estado inicial de volta.
    fn assert_round_trip(document: &mut Document, change: Change) {
        let inicial = document.clone();

        let inverse = change
            .clone()
            .apply(document)
            .expect("a mudança deve aplicar");
        assert_ne!(
            *document, inicial,
            "a mudança precisa ter efeito observável, senão o teste não prova nada"
        );

        let reapplied = inverse.apply(document).expect("a inversa deve aplicar");

        assert_eq!(*document, inicial, "a inversa deve restaurar o estado");
        assert_eq!(
            reapplied, change,
            "inverter a inversa deve reconstruir a mudança original"
        );
    }

    #[test]
    fn remover_entidade_e_desfazer_restaura_o_documento() {
        let (mut document, _, ids) = documento();

        assert_round_trip(&mut document, Change::RemoveEntity { entity: ids[1] });
    }

    #[test]
    fn desfazer_remocao_devolve_o_mesmo_identificador() {
        let (mut document, _, ids) = documento();
        let alvo = ids[1];

        let desfazer = Change::RemoveEntity { entity: alvo }
            .apply(&mut document)
            .expect("aplica");
        assert!(document.entity(alvo).is_none());

        desfazer.apply(&mut document).expect("desfaz");

        assert!(
            document.entity(alvo).is_some(),
            "o identificador antigo tem de voltar a resolver"
        );
    }

    #[test]
    fn desfazer_remocao_devolve_a_posicao_na_ordem_de_desenho() {
        let (mut document, _, ids) = documento();

        let desfazer = Change::RemoveEntity { entity: ids[1] }
            .apply(&mut document)
            .expect("aplica");
        desfazer.apply(&mut document).expect("desfaz");

        let ordem: Vec<_> = document
            .entities_in_block(document.model_space())
            .map(|(id, _)| id)
            .collect();

        assert_eq!(
            ordem, ids,
            "a entidade tem de voltar ao meio, não ao fim da ordem de desenho"
        );
    }

    #[test]
    fn inserir_entidade_e_desfazer_restaura_o_documento() {
        let (mut document, _, ids) = documento();

        // Para restaurar é preciso um identificador conhecido e vago: removemos
        // primeiro, que é exatamente o caminho de um redo.
        let placement = document.entity_placement(ids[1]).expect("está no bloco");
        let content = document.edit().remove_entity(ids[1]).expect("existe");

        assert_round_trip(
            &mut document,
            Change::InsertEntity {
                entity: ids[1],
                content: Box::new(content),
                placement,
            },
        );
    }

    #[test]
    fn substituir_entidade_e_desfazer_restaura_o_conteudo() {
        let (mut document, parede, ids) = documento();

        assert_round_trip(
            &mut document,
            Change::ReplaceEntity {
                entity: ids[0],
                content: Box::new(circle(parede)),
            },
        );
    }

    #[test]
    fn substituir_entidade_preserva_identificador_e_posicao() {
        let (mut document, parede, ids) = documento();

        Change::ReplaceEntity {
            entity: ids[0],
            content: Box::new(circle(parede)),
        }
        .apply(&mut document)
        .expect("aplica");

        assert_eq!(
            document.entity_placement(ids[0]),
            Some(EntityPlacement {
                block: document.model_space(),
                position: 0,
            })
        );
        assert_eq!(
            document.entity(ids[0]).map(|entity| entity.layer),
            Some(parede)
        );
    }

    #[test]
    fn alterar_camada_e_desfazer_restaura_o_registro() {
        let (mut document, parede, _) = documento();

        let mut record = document
            .layers()
            .get(parede)
            .expect("camada existe")
            .clone();
        record.set_color(Color::Index(3));
        record.set_locked(true);

        assert_round_trip(
            &mut document,
            Change::SetLayerRecord {
                layer: parede,
                record: Box::new(record),
            },
        );
    }

    #[test]
    fn alterar_camada_aplica_de_fato_as_propriedades() {
        let (mut document, parede, _) = documento();
        let mut record = document
            .layers()
            .get(parede)
            .expect("camada existe")
            .clone();
        record.set_color(Color::Index(3));

        Change::SetLayerRecord {
            layer: parede,
            record: Box::new(record),
        }
        .apply(&mut document)
        .expect("aplica");

        assert_eq!(
            document.layers().get(parede).expect("existe").color(),
            Color::Index(3)
        );
    }

    #[test]
    fn sequencia_de_mudancas_desfeita_em_ordem_inversa_restaura_tudo() {
        let (mut document, parede, ids) = documento();
        let inicial = document.clone();

        let mut record = document.layers().get(parede).expect("existe").clone();
        record.set_off(true);

        let mudancas = vec![
            Change::RemoveEntity { entity: ids[0] },
            Change::ReplaceEntity {
                entity: ids[2],
                content: Box::new(circle(parede)),
            },
            Change::SetLayerRecord {
                layer: parede,
                record: Box::new(record),
            },
        ];

        let mut inversas = Vec::new();
        for change in mudancas {
            inversas.push(change.apply(&mut document).expect("aplica"));
        }

        assert_ne!(document, inicial);

        // Desfazer é percorrer o journal de trás para frente.
        for inverse in inversas.into_iter().rev() {
            inverse.apply(&mut document).expect("desfaz");
        }

        assert_eq!(document, inicial);
    }

    #[test]
    fn remover_entidade_inexistente_falha_sem_alterar_o_documento() {
        let (mut document, _, ids) = documento();
        Change::RemoveEntity { entity: ids[0] }
            .apply(&mut document)
            .expect("primeira remoção aplica");
        let depois = document.clone();

        let erro = Change::RemoveEntity { entity: ids[0] }
            .apply(&mut document)
            .expect_err("a entidade já saiu");

        assert_eq!(erro, ChangeError::EntityNotPlaced(ids[0]));
        assert_eq!(document, depois, "documento intacto após a falha");
    }

    #[test]
    fn substituir_entidade_inexistente_falha_sem_alterar_o_documento() {
        let (mut document, parede, ids) = documento();
        let _ = document.edit().remove_entity(ids[0]).expect("existe");
        let depois = document.clone();

        let erro = Change::ReplaceEntity {
            entity: ids[0],
            content: Box::new(circle(parede)),
        }
        .apply(&mut document)
        .expect_err("entidade removida");

        assert_eq!(
            erro,
            ChangeError::Document(DocumentError::UnknownEntity),
            "erro: {erro}"
        );
        assert_eq!(document, depois);
    }

    #[test]
    fn restaurar_sobre_identificador_ocupado_falha() {
        let (mut document, _, ids) = documento();
        let placement = document.entity_placement(ids[0]).expect("está no bloco");
        let content = document.entity(ids[0]).expect("existe").clone();
        let depois = document.clone();

        let erro = Change::InsertEntity {
            entity: ids[0],
            content: Box::new(content),
            placement,
        }
        .apply(&mut document)
        .expect_err("o slot está ocupado");

        assert!(matches!(erro, ChangeError::Document(_)), "erro: {erro}");
        assert_eq!(document, depois);
    }

    #[test]
    fn mensagens_de_erro_sao_legiveis() {
        let (mut document, _, ids) = documento();
        let _ = document.edit().remove_entity(ids[0]).expect("existe");

        let erro = ChangeError::UnknownEntity(ids[0]);
        assert!(erro.to_string().contains("não existe no documento"));
    }
}
