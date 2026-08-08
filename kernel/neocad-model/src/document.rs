// Caminho relativo: kernel/neocad-model/src/document.rs
//! \file kernel/neocad-model/src/document.rs
//! \brief Documento: arena de entidades e tabelas de símbolos, com as invariantes que as ligam.
//! \author Iago Leal
//! \date 2026-08-07

use core::fmt;

use neocad_geometry::Aabb;

use crate::arena::{Arena, RestoreError};
use crate::block::{BlockError, BlockId, BlockRecord, BlockTable};
use crate::change::{Change, ChangeError};
use crate::entity::Entity;
use crate::id::EntityId;
use crate::layer::{LayerError, LayerId, LayerRecord, LayerTable};
use crate::text_style::{TextStyleError, TextStyleTable};
use neocad_geometry::Point2;

/// Falha ao operar sobre o documento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    /// A camada informada não existe ou o identificador está obsoleto.
    UnknownLayer,
    /// O bloco informado não existe ou o identificador está obsoleto.
    UnknownBlock,
    /// A entidade informada não existe ou o identificador está obsoleto.
    UnknownEntity,
    /// A camada não pode ser removida porque ainda há entidades nela.
    LayerInUse {
        /// Camada que se tentou remover.
        layer: LayerId,
        /// Quantidade de entidades que ainda a referenciam.
        entity_count: usize,
    },
    /// Falha originada na tabela de camadas.
    Layer(LayerError),
    /// Falha originada na tabela de blocos.
    Block(BlockError),
    /// Falha originada na tabela de estilos de texto.
    TextStyle(TextStyleError),
    /// Falha ao restaurar uma entidade em um identificador já conhecido.
    Restore(RestoreError),
}

impl From<RestoreError> for DocumentError {
    fn from(error: RestoreError) -> Self {
        Self::Restore(error)
    }
}

/// Onde uma entidade está, dentro do documento.
///
/// Bloco **e** posição, porque a ordem dentro do bloco é a ordem de desenho:
/// devolver uma entidade ao bloco certo, mas ao fim da lista, mudaria quem
/// aparece por cima de quem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityPlacement {
    /// Bloco que contém a entidade.
    pub block: BlockId,
    /// Índice na ordem de desenho do bloco.
    pub position: usize,
}

impl From<LayerError> for DocumentError {
    fn from(error: LayerError) -> Self {
        Self::Layer(error)
    }
}

impl From<BlockError> for DocumentError {
    fn from(error: BlockError) -> Self {
        Self::Block(error)
    }
}

impl From<TextStyleError> for DocumentError {
    fn from(error: TextStyleError) -> Self {
        Self::TextStyle(error)
    }
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLayer => write!(formatter, "camada inexistente no documento"),
            Self::UnknownBlock => write!(formatter, "bloco inexistente no documento"),
            Self::UnknownEntity => write!(formatter, "entidade inexistente no documento"),
            Self::LayerInUse {
                layer,
                entity_count,
            } => write!(
                formatter,
                "a camada {layer} ainda contém {entity_count} entidade(s) e não pode ser removida"
            ),
            Self::Layer(error) => write!(formatter, "{error}"),
            Self::Block(error) => write!(formatter, "{error}"),
            Self::TextStyle(error) => write!(formatter, "{error}"),
            Self::Restore(error) => write!(formatter, "{error}"),
        }
    }
}

impl core::error::Error for DocumentError {}

/// Documento CAD: as entidades e as tabelas de símbolos que elas referenciam.
///
/// # Por que o documento existe
///
/// As tabelas isoladas não conseguem manter as invariantes que **atravessam**
/// estruturas, e por isso não as verificam:
///
/// - uma entidade referencia uma camada que precisa existir;
/// - uma entidade pertence a exatamente um bloco, e a arena e a lista do bloco
///   têm de contar a mesma história;
/// - remover uma camada que ainda tem entidades deixaria referências penduradas.
///
/// O documento é o dono das três estruturas e o único lugar onde essas regras
/// podem ser aplicadas. Por isso as tabelas são expostas apenas para **leitura**,
/// e toda alteração que cruza estruturas passa por um método daqui.
///
/// # Mutação fechada atrás do registro
///
/// Conforme o ADR 0003, alterar o **desenho** — entidades e propriedades de
/// camada — só é possível por [`Document::edit`], que devolve um
/// [`DocumentEditor`] e registra a inversa de cada operação. Os métodos diretos
/// correspondentes são privados à crate, de modo que a regra é verificada pelo
/// compilador e não apenas documentada.
///
/// A **estrutura** das tabelas de símbolos — criar, renomear e remover camada,
/// bloco ou estilo — continua pública e **ainda não é reversível**: `Change` não
/// tem variantes para essas operações, e criá-las exige restauração por
/// identificador exato nas três tabelas, como a `Arena` já faz para entidades.
/// Fica registrado como pendência no handoff.
///
/// # Exemplo
///
/// ```
/// use neocad_model::{Document, Entity, Geometry, Line};
/// use neocad_geometry::Point2;
///
/// let mut document = Document::new();
/// let parede = document.create_layer("Parede")?;
///
/// let mut editor = document.edit();
/// let id = editor.insert_in_model_space(Entity::new(
///     parede,
///     Geometry::Line(Line {
///         start: Point2::ORIGIN,
///         end: Point2::new(10.0, 0.0),
///     }),
/// ))?;
/// let _ = editor.finish();
///
/// assert_eq!(document.entities_in_layer(parede).count(), 1);
/// assert_eq!(document.entity(id).map(|e| e.layer), Some(parede));
/// # Ok::<(), neocad_model::DocumentError>(())
/// ```
#[derive(Debug, Clone)]
pub struct Document {
    entities: Arena<Entity>,
    layers: LayerTable,
    blocks: BlockTable,
    text_styles: TextStyleTable,
}

impl Document {
    /// Cria um documento vazio e válido.
    ///
    /// Válido significa que as garantias mínimas dos formatos CAD já valem: a
    /// camada `0`, o bloco `*Model_Space` e o estilo `Standard` existem.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: Arena::new(),
            layers: LayerTable::new(),
            blocks: BlockTable::new(),
            text_styles: TextStyleTable::new(),
        }
    }

    /// Tabela de camadas, para leitura.
    #[must_use]
    pub const fn layers(&self) -> &LayerTable {
        &self.layers
    }

    /// Tabela de blocos, para leitura.
    #[must_use]
    pub const fn blocks(&self) -> &BlockTable {
        &self.blocks
    }

    /// Tabela de estilos de texto, para leitura.
    #[must_use]
    pub const fn text_styles(&self) -> &TextStyleTable {
        &self.text_styles
    }

    /// Tabela de estilos de texto, para alteração.
    ///
    /// Exposta diretamente porque nenhuma entidade referencia estilo ainda. Ao
    /// ganhar essa referência, a remoção de estilo passa a precisar da mesma
    /// proteção que [`Document::remove_layer`] aplica às camadas.
    pub const fn text_styles_mut(&mut self) -> &mut TextStyleTable {
        &mut self.text_styles
    }

    /// Identificador do bloco do espaço-modelo.
    #[must_use]
    pub const fn model_space(&self) -> BlockId {
        self.blocks.model_space()
    }

    // -- Camadas ------------------------------------------------------------

    /// Cria uma camada.
    ///
    /// # Errors
    ///
    /// Falha se o nome for inválido ou já estiver em uso.
    pub fn create_layer(&mut self, name: impl Into<String>) -> Result<LayerId, DocumentError> {
        Ok(self.layers.create(name)?)
    }

    /// Renomeia uma camada.
    ///
    /// # Errors
    ///
    /// Falha se a camada for a `0`, se o identificador estiver obsoleto ou se o
    /// nome novo for inválido ou já estiver em uso.
    pub fn rename_layer(
        &mut self,
        layer: LayerId,
        name: impl Into<String>,
    ) -> Result<(), DocumentError> {
        Ok(self.layers.rename(layer, name)?)
    }

    /// Remove uma camada vazia.
    ///
    /// # Errors
    ///
    /// Falha com [`DocumentError::LayerInUse`] se ainda houver entidades na
    /// camada — removê-la deixaria essas entidades apontando para uma camada
    /// inexistente. Falha também se a camada for a `0` ou se o identificador
    /// estiver obsoleto.
    pub fn remove_layer(&mut self, layer: LayerId) -> Result<LayerRecord, DocumentError> {
        let entity_count = self.entities_in_layer(layer).count();

        if entity_count > 0 {
            return Err(DocumentError::LayerInUse {
                layer,
                entity_count,
            });
        }

        Ok(self.layers.remove(layer)?)
    }

    // -- Blocos -------------------------------------------------------------

    /// Cria um bloco vazio.
    ///
    /// # Errors
    ///
    /// Falha se o nome for inválido — incluindo nomes reservados, iniciados por
    /// asterisco — ou já estiver em uso.
    pub fn create_block(&mut self, name: impl Into<String>) -> Result<BlockId, DocumentError> {
        Ok(self.blocks.create(name)?)
    }

    /// Renomeia um bloco.
    ///
    /// # Errors
    ///
    /// Falha se o bloco for o espaço-modelo, se o identificador estiver obsoleto
    /// ou se o nome novo for inválido ou já estiver em uso.
    pub fn rename_block(
        &mut self,
        block: BlockId,
        name: impl Into<String>,
    ) -> Result<(), DocumentError> {
        Ok(self.blocks.rename(block, name)?)
    }

    /// Define o ponto base de um bloco.
    ///
    /// # Errors
    ///
    /// Falha se o identificador estiver obsoleto.
    pub fn set_block_origin(
        &mut self,
        block: BlockId,
        origin: Point2,
    ) -> Result<(), DocumentError> {
        self.blocks
            .get_mut(block)
            .ok_or(DocumentError::UnknownBlock)?
            .set_origin(origin);

        Ok(())
    }

    /// Remove um bloco **e as entidades que ele contém**.
    ///
    /// A cascata é deliberada: uma entidade pertence a exatamente um bloco, de
    /// modo que deixá-la na arena após remover o bloco produziria entidade órfã,
    /// invisível e impossível de alcançar.
    ///
    /// O registro devolvido lista as entidades removidas.
    ///
    /// # Errors
    ///
    /// Falha se o bloco for o espaço-modelo ou se o identificador estiver
    /// obsoleto.
    pub fn remove_block(&mut self, block: BlockId) -> Result<BlockRecord, DocumentError> {
        let record = self.blocks.remove(block)?;

        for &entity in record.entities() {
            self.entities.remove(entity);
        }

        Ok(record)
    }

    // -- Entidades ----------------------------------------------------------

    /// Insere uma entidade em um bloco, ao fim da ordem de desenho.
    ///
    /// # Errors
    ///
    /// Falha se o bloco ou a camada da entidade não existirem — validar antes de
    /// inserir evita que uma referência pendurada entre no documento.
    pub(crate) fn insert_entity(
        &mut self,
        entity: Entity,
        block: BlockId,
    ) -> Result<EntityId, DocumentError> {
        if !self.layers.contains(entity.layer) {
            return Err(DocumentError::UnknownLayer);
        }
        if !self.blocks.contains(block) {
            return Err(DocumentError::UnknownBlock);
        }

        let id = self.entities.insert(entity);
        self.blocks
            .get_mut(block)
            .ok_or(DocumentError::UnknownBlock)?
            .push_entity(id);

        Ok(id)
    }

    /// Insere uma entidade no espaço-modelo.
    ///
    /// # Errors
    ///
    /// Falha se a camada da entidade não existir.
    pub(crate) fn insert_in_model_space(
        &mut self,
        entity: Entity,
    ) -> Result<EntityId, DocumentError> {
        self.insert_entity(entity, self.model_space())
    }

    /// Remove uma entidade do documento e do bloco que a contém.
    ///
    /// Devolve `None` se o identificador estiver obsoleto.
    pub(crate) fn remove_entity(&mut self, entity: EntityId) -> Option<Entity> {
        let removed = self.entities.remove(entity)?;

        if let Some(owner) = self.blocks.owner_of(entity) {
            if let Some(record) = self.blocks.get_mut(owner) {
                record.remove_entity(entity);
            }
        }

        Some(removed)
    }

    /// Abre uma sessão de edição — a única via pública de mutação do desenho.
    ///
    /// Ver [`DocumentEditor`] para o motivo de a mutação direta ser privada.
    pub fn edit(&mut self) -> DocumentEditor<'_> {
        DocumentEditor::new(self)
    }

    /// Move uma entidade para um bloco e posição determinados, devolvendo onde
    /// ela estava.
    pub(crate) fn set_entity_placement(
        &mut self,
        entity: EntityId,
        placement: EntityPlacement,
    ) -> Result<EntityPlacement, DocumentError> {
        let previous = self
            .entity_placement(entity)
            .ok_or(DocumentError::UnknownEntity)?;

        if !self.blocks.contains(placement.block) {
            return Err(DocumentError::UnknownBlock);
        }

        self.blocks
            .get_mut(previous.block)
            .ok_or(DocumentError::UnknownBlock)?
            .remove_entity(entity);

        let inserted = self
            .blocks
            .get_mut(placement.block)
            .ok_or(DocumentError::UnknownBlock)?
            .insert_entity_at(placement.position, entity);

        if !inserted {
            // Devolve a entidade ao lugar de origem para não perdê-la.
            self.blocks
                .get_mut(previous.block)
                .ok_or(DocumentError::UnknownBlock)?
                .insert_entity_at(previous.position, entity);

            return Err(DocumentError::UnknownBlock);
        }

        Ok(previous)
    }

    /// Onde a entidade está: bloco e posição na ordem de desenho.
    ///
    /// Devolve `None` se a entidade não pertencer a nenhum bloco — situação que
    /// as invariantes do documento não produzem, mas que a consulta reporta em
    /// vez de mascarar.
    #[must_use]
    pub fn entity_placement(&self, entity: EntityId) -> Option<EntityPlacement> {
        let block = self.blocks.owner_of(entity)?;
        let position = self.blocks.get(block)?.position_of(entity)?;

        Some(EntityPlacement { block, position })
    }

    /// Restaura uma entidade **no identificador e na posição exatos**.
    ///
    /// É a operação inversa de [`Document::remove_entity`], e existe para que
    /// desfazer uma remoção devolva a entidade como ela estava: mesmo
    /// identificador, mesmo bloco, mesma posição na ordem de desenho. Reinserir
    /// com identificador novo quebraria toda referência a ela; reinserir ao fim
    /// da lista mudaria o que aparece por cima.
    ///
    /// # Errors
    ///
    /// Falha se a camada ou o bloco não existirem, se o identificador já estiver
    /// ocupado, ou se a posição estiver além do fim da lista do bloco.
    pub(crate) fn restore_entity(
        &mut self,
        id: EntityId,
        entity: Entity,
        placement: EntityPlacement,
    ) -> Result<(), DocumentError> {
        if !self.layers.contains(entity.layer) {
            return Err(DocumentError::UnknownLayer);
        }
        if !self.blocks.contains(placement.block) {
            return Err(DocumentError::UnknownBlock);
        }

        self.entities.insert_at(id, entity)?;

        let restored = self
            .blocks
            .get_mut(placement.block)
            .ok_or(DocumentError::UnknownBlock)?
            .insert_entity_at(placement.position, id);

        if !restored {
            // Desfaz a inserção na arena para não deixar entidade órfã.
            self.entities.remove(id);
            return Err(DocumentError::UnknownBlock);
        }

        Ok(())
    }

    /// Substitui o conteúdo de uma entidade, preservando identificador e posição.
    ///
    /// Devolve o conteúdo anterior, que é exatamente o necessário para desfazer.
    ///
    /// # Errors
    ///
    /// Falha se a entidade não existir ou se a camada da entidade nova não
    /// existir.
    pub(crate) fn replace_entity(
        &mut self,
        id: EntityId,
        entity: Entity,
    ) -> Result<Entity, DocumentError> {
        if !self.layers.contains(entity.layer) {
            return Err(DocumentError::UnknownLayer);
        }

        let slot = self
            .entities
            .get_mut(id)
            .ok_or(DocumentError::UnknownEntity)?;

        Ok(core::mem::replace(slot, entity))
    }

    /// Substitui o registro de uma camada por inteiro.
    ///
    /// Devolve o registro anterior, que é o necessário para desfazer. Se o nome
    /// tiver mudado, o índice por nome é atualizado junto.
    ///
    /// # Errors
    ///
    /// Falha se a camada não existir, ou se a troca implicar renomear a camada
    /// `0` ou colidir com o nome de outra camada.
    pub(crate) fn set_layer_record(
        &mut self,
        layer: LayerId,
        record: LayerRecord,
    ) -> Result<LayerRecord, DocumentError> {
        let previous = self
            .layers
            .get(layer)
            .ok_or(DocumentError::UnknownLayer)?
            .clone();

        if previous.name() != record.name() {
            self.layers.rename(layer, record.name().to_owned())?;
        }

        let slot = self
            .layers
            .get_mut(layer)
            .ok_or(DocumentError::UnknownLayer)?;
        *slot = record;

        Ok(previous)
    }

    /// Devolve a entidade de `id`, ou `None` se o identificador estiver obsoleto.
    #[must_use]
    pub fn entity(&self, entity: EntityId) -> Option<&Entity> {
        self.entities.get(entity)
    }

    /// Quantidade de entidades no documento, somando todos os blocos.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Itera sobre todas as entidades, em ordem determinística de identificador.
    pub fn entities(&self) -> impl Iterator<Item = (EntityId, &Entity)> {
        self.entities.iter()
    }

    /// Itera sobre as entidades de uma camada.
    ///
    /// A ordem é a do identificador, não a de desenho: a camada atravessa
    /// blocos, e ordem de desenho só faz sentido dentro de um bloco.
    pub fn entities_in_layer(&self, layer: LayerId) -> impl Iterator<Item = (EntityId, &Entity)> {
        self.entities
            .iter()
            .filter(move |(_, entity)| entity.layer == layer)
    }

    /// Itera sobre as entidades de um bloco, **na ordem de desenho**.
    ///
    /// Devolve iterador vazio se o bloco não existir.
    pub fn entities_in_block(&self, block: BlockId) -> impl Iterator<Item = (EntityId, &Entity)> {
        self.blocks
            .get(block)
            .map(BlockRecord::entities)
            .unwrap_or_default()
            .iter()
            .filter_map(move |&id| Some((id, self.entities.get(id)?)))
    }

    /// Caixa envolvente de todas as entidades do documento.
    ///
    /// Devolve `None` quando não há entidades.
    #[must_use]
    pub fn bounding_box(&self) -> Option<Aabb> {
        Aabb::union_all(
            self.entities
                .iter()
                .map(|(_, entity)| entity.bounding_box()),
        )
    }

    /// Caixa envolvente das entidades de um bloco.
    ///
    /// É o que o ajuste de vista consome, aplicado ao espaço-modelo. Devolve
    /// `None` quando o bloco não existe ou está vazio.
    #[must_use]
    pub fn block_bounding_box(&self, block: BlockId) -> Option<Aabb> {
        Aabb::union_all(
            self.entities_in_block(block)
                .map(|(_, entity)| entity.bounding_box()),
        )
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Única via pública de mutação do desenho.
///
/// # Por que existe
///
/// O ADR 0003 determina que toda alteração do desenho ocorra por comando
/// reversível. Uma regra assim, se ficar só na documentação, é violada no
/// primeiro atalho conveniente. O editor a torna verificável pelo compilador:
/// os métodos de mutação de [`Document`] são privados à crate, e este guarda é
/// o único caminho de fora — e ele **sempre** registra a inversa.
///
/// Não há como alterar uma entidade sem produzir o que a desfaz.
///
/// # A restrição é verificada pelo compilador
///
/// Inserir uma entidade sem passar pelo editor não compila:
///
/// ```compile_fail,E0624
/// use neocad_model::{Document, Entity, Geometry, Line};
/// use neocad_geometry::Point2;
///
/// let mut document = Document::new();
/// let zero = document.layers().default_layer();
///
/// // `insert_in_model_space` é privado à crate: a mutação não registrada
/// // não tem caminho a partir de fora.
/// document.insert_in_model_space(Entity::new(
///     zero,
///     Geometry::Line(Line { start: Point2::ORIGIN, end: Point2::new(1.0, 0.0) }),
/// ));
/// ```
///
/// Remover também não:
///
/// ```compile_fail,E0624
/// use neocad_model::{Document, EntityId};
///
/// fn tenta(document: &mut Document, id: EntityId) {
///     document.remove_entity(id);
/// }
/// ```
///
/// Nem alterar uma entidade já existente por referência mutável — o método que
/// permitia isso deixou de existir:
///
/// ```compile_fail
/// use neocad_model::{Document, EntityId};
///
/// fn tenta(document: &mut Document, id: EntityId) {
///     document.entity_mut(id);
/// }
/// ```
///
/// Nem substituir o registro de uma camada:
///
/// ```compile_fail,E0624
/// use neocad_model::{Document, LayerId, LayerRecord};
///
/// fn tenta(document: &mut Document, layer: LayerId, record: LayerRecord) {
///     document.set_layer_record(layer, record);
/// }
/// ```
///
/// # Uso
///
/// Cada operação devolve o que seria de esperar dela; o registro acontece por
/// baixo. Ao final, [`DocumentEditor::finish`] entrega as mudanças que desfazem
/// tudo, já na ordem em que devem ser aplicadas.
///
/// A camada de transações (`neocad-transaction`) empacota essas mudanças em uma
/// transação nomeada e a empilha no histórico.
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
///     Geometry::Line(Line { start: Point2::ORIGIN, end: Point2::new(1.0, 0.0) }),
/// ))?;
/// let desfazer = editor.finish();
///
/// assert!(document.entity(id).is_some());
/// assert_eq!(desfazer.len(), 1, "a inserção produziu o que a desfaz");
/// # Ok::<(), neocad_model::DocumentError>(())
/// ```
#[derive(Debug)]
pub struct DocumentEditor<'a> {
    document: &'a mut Document,
    undo: Vec<Change>,
}

impl<'a> DocumentEditor<'a> {
    fn new(document: &'a mut Document) -> Self {
        Self {
            document,
            undo: Vec::new(),
        }
    }

    /// Documento sob edição, para consulta.
    #[must_use]
    pub fn document(&self) -> &Document {
        self.document
    }

    /// Insere uma entidade em um bloco, ao fim da ordem de desenho.
    ///
    /// # Errors
    ///
    /// Falha se o bloco ou a camada da entidade não existirem.
    pub fn insert_entity(
        &mut self,
        entity: Entity,
        block: BlockId,
    ) -> Result<EntityId, DocumentError> {
        let id = self.document.insert_entity(entity, block)?;
        self.undo.push(Change::RemoveEntity { entity: id });

        Ok(id)
    }

    /// Insere uma entidade no espaço-modelo.
    ///
    /// # Errors
    ///
    /// Falha se a camada da entidade não existir.
    pub fn insert_in_model_space(&mut self, entity: Entity) -> Result<EntityId, DocumentError> {
        let id = self.document.insert_in_model_space(entity)?;
        self.undo.push(Change::RemoveEntity { entity: id });

        Ok(id)
    }

    /// Remove uma entidade.
    ///
    /// # Errors
    ///
    /// Falha se a entidade não existir ou não estiver em bloco algum.
    pub fn remove_entity(&mut self, entity: EntityId) -> Result<Entity, DocumentError> {
        let placement = self
            .document
            .entity_placement(entity)
            .ok_or(DocumentError::UnknownEntity)?;
        let content = self
            .document
            .remove_entity(entity)
            .ok_or(DocumentError::UnknownEntity)?;

        self.undo.push(Change::InsertEntity {
            entity,
            content: Box::new(content.clone()),
            placement,
        });

        Ok(content)
    }

    /// Substitui o conteúdo de uma entidade, preservando identificador e posição.
    ///
    /// # Errors
    ///
    /// Falha se a entidade ou a camada da entidade nova não existirem.
    pub fn replace_entity(
        &mut self,
        entity: EntityId,
        content: Entity,
    ) -> Result<Entity, DocumentError> {
        let previous = self.document.replace_entity(entity, content)?;

        self.undo.push(Change::ReplaceEntity {
            entity,
            content: Box::new(previous.clone()),
        });

        Ok(previous)
    }

    /// Move uma entidade para outro bloco e posição na ordem de desenho.
    ///
    /// # Errors
    ///
    /// Falha se a entidade ou o bloco de destino não existirem, ou se a posição
    /// estiver além do fim da lista do bloco.
    pub fn set_entity_placement(
        &mut self,
        entity: EntityId,
        placement: EntityPlacement,
    ) -> Result<EntityPlacement, DocumentError> {
        let previous = self.document.set_entity_placement(entity, placement)?;

        self.undo.push(Change::MoveEntity {
            entity,
            placement: previous,
        });

        Ok(previous)
    }

    /// Substitui o registro de uma camada por inteiro.
    ///
    /// # Errors
    ///
    /// Falha se a camada não existir, ou se a troca implicar renomear a camada
    /// `0` ou colidir com o nome de outra camada.
    pub fn set_layer_record(
        &mut self,
        layer: LayerId,
        record: LayerRecord,
    ) -> Result<LayerRecord, DocumentError> {
        let previous = self.document.set_layer_record(layer, record)?;

        self.undo.push(Change::SetLayerRecord {
            layer,
            record: Box::new(previous.clone()),
        });

        Ok(previous)
    }

    /// Encerra a edição e devolve as mudanças que a desfazem.
    ///
    /// As mudanças vêm na ordem em que devem ser aplicadas para desfazer, ou
    /// seja, na ordem contrária à das operações realizadas.
    #[must_use]
    pub fn finish(mut self) -> Vec<Change> {
        self.undo.reverse();
        self.undo
    }

    /// Desfaz tudo o que foi feito nesta edição e devolve o documento ao estado
    /// anterior.
    ///
    /// # Errors
    ///
    /// Falha se alguma inversa não puder ser aplicada, o que indicaria quebra de
    /// invariante do kernel.
    pub fn rollback(self) -> Result<(), ChangeError> {
        let Self { document, mut undo } = self;

        // Desfazer percorre as inversas na ordem contrária à das operações.
        undo.reverse();

        for change in undo {
            change.apply(&mut *document)?;
        }

        Ok(())
    }
}

/// Igualdade **semântica** entre documentos.
///
/// Compara o conteúdo observável — entidades vivas com seus identificadores,
/// camadas, blocos com sua ordem de desenho e estilos de texto — e **ignora**
/// resíduo interno de alocação, como slots vagos deixados por remoções e a
/// ordem da lista de reuso.
///
/// A distinção importa para as transações: desfazer uma inserção remove a
/// entidade, mas o slot que ela ocupou continua existindo, vago. O documento é
/// equivalente ao original para qualquer efeito observável, ainda que a memória
/// não esteja byte a byte igual. Comparar a representação bruta reprovaria
/// desfazimentos corretos.
impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.entities.iter().eq(other.entities.iter())
            && self.layers.iter().eq(other.layers.iter())
            && self.blocks.iter().eq(other.blocks.iter())
            && self.text_styles.iter().eq(other.text_styles.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::MODEL_SPACE_NAME;
    use crate::entity::{Circle, Geometry, Line};
    use crate::layer::DEFAULT_LAYER_NAME;
    use crate::text_style::STANDARD_TEXT_STYLE_NAME;

    fn line(layer: LayerId, x: f64) -> Entity {
        Entity::new(
            layer,
            Geometry::Line(Line {
                start: Point2::new(x, 0.0),
                end: Point2::new(x + 1.0, 1.0),
            }),
        )
    }

    fn circle(layer: LayerId, radius: f64) -> Entity {
        Entity::new(
            layer,
            Geometry::Circle(Circle {
                center: Point2::ORIGIN,
                radius,
            }),
        )
    }

    #[test]
    fn documento_novo_e_vazio_porem_valido() {
        let document = Document::new();

        assert_eq!(document.entity_count(), 0);
        assert_eq!(document.bounding_box(), None);

        // As três garantias mínimas dos formatos CAD.
        assert!(document.layers().get_by_name(DEFAULT_LAYER_NAME).is_some());
        assert!(document.blocks().get_by_name(MODEL_SPACE_NAME).is_some());
        assert!(document
            .text_styles()
            .get_by_name(STANDARD_TEXT_STYLE_NAME)
            .is_some());

        assert_eq!(document.layers().len(), 1);
        assert_eq!(document.blocks().len(), 1);
        assert_eq!(document.text_styles().len(), 1);
    }

    #[test]
    fn default_equivale_a_new() {
        assert_eq!(Document::default().entity_count(), 0);
    }

    #[test]
    fn insere_entidade_no_espaco_modelo() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();

        let id = document
            .insert_in_model_space(line(zero, 0.0))
            .expect("camada e bloco existem");

        assert_eq!(document.entity_count(), 1);
        assert!(document.entity(id).is_some());
        assert_eq!(
            document.blocks().model_space_record().entities().to_owned(),
            vec![id],
            "a entidade tem de entrar também na lista do bloco"
        );
    }

    #[test]
    fn entidade_com_camada_inexistente_e_recusada() {
        let mut document = Document::new();
        let mut outro = Document::new();
        let camada_de_outro = outro.create_layer("Fantasma").expect("nome válido");

        assert_eq!(
            document.insert_in_model_space(line(camada_de_outro, 0.0)),
            Err(DocumentError::UnknownLayer),
            "referência pendurada não pode entrar no documento"
        );
        assert_eq!(document.entity_count(), 0);
    }

    #[test]
    fn entidade_em_bloco_inexistente_e_recusada() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        let bloco = document.create_block("Porta").expect("nome válido");
        document.remove_block(bloco).expect("bloco existe");

        assert_eq!(
            document.insert_entity(line(zero, 0.0), bloco),
            Err(DocumentError::UnknownBlock)
        );
        assert_eq!(document.entity_count(), 0);
    }

    #[test]
    fn consulta_entidades_por_camada() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        let parede = document.create_layer("Parede").expect("nome válido");

        let a = document
            .insert_in_model_space(line(parede, 0.0))
            .expect("insere");
        document
            .insert_in_model_space(line(zero, 5.0))
            .expect("insere");
        let c = document
            .insert_in_model_space(line(parede, 10.0))
            .expect("insere");

        let na_parede: Vec<_> = document
            .entities_in_layer(parede)
            .map(|(id, _)| id)
            .collect();

        assert_eq!(na_parede, vec![a, c]);
        assert_eq!(document.entities_in_layer(zero).count(), 1);
        assert_eq!(document.entities().count(), 3);
    }

    #[test]
    fn consulta_por_camada_vazia_devolve_nada() {
        let mut document = Document::new();
        let vazia = document.create_layer("Vazia").expect("nome válido");

        assert_eq!(document.entities_in_layer(vazia).count(), 0);
    }

    #[test]
    fn entidades_do_bloco_saem_na_ordem_de_desenho() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();

        let primeira = document
            .insert_in_model_space(line(zero, 0.0))
            .expect("insere");
        let segunda = document
            .insert_in_model_space(line(zero, 1.0))
            .expect("insere");

        let ordem: Vec<_> = document
            .entities_in_block(document.model_space())
            .map(|(id, _)| id)
            .collect();

        assert_eq!(ordem, vec![primeira, segunda]);
    }

    #[test]
    fn entidades_de_bloco_inexistente_devolve_iterador_vazio() {
        let mut document = Document::new();
        let bloco = document.create_block("Porta").expect("nome válido");
        document.remove_block(bloco).expect("bloco existe");

        assert_eq!(document.entities_in_block(bloco).count(), 0);
    }

    #[test]
    fn remover_entidade_tira_da_arena_e_do_bloco() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        let id = document
            .insert_in_model_space(line(zero, 0.0))
            .expect("insere");

        let removida = document.remove_entity(id).expect("entidade existe");

        assert_eq!(removida.layer, zero);
        assert_eq!(document.entity_count(), 0);
        assert!(
            document.blocks().model_space_record().is_empty(),
            "a lista do bloco não pode ficar com identificador morto"
        );
        assert!(document.entity(id).is_none());
    }

    #[test]
    fn remover_entidade_obsoleta_devolve_none() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        let id = document
            .insert_in_model_space(line(zero, 0.0))
            .expect("insere");
        document.remove_entity(id);

        assert_eq!(document.remove_entity(id), None);
    }

    #[test]
    fn camada_com_entidades_nao_pode_ser_removida() {
        let mut document = Document::new();
        let parede = document.create_layer("Parede").expect("nome válido");
        document
            .insert_in_model_space(line(parede, 0.0))
            .expect("insere");

        assert_eq!(
            document.remove_layer(parede),
            Err(DocumentError::LayerInUse {
                layer: parede,
                entity_count: 1,
            })
        );
        assert!(document.layers().contains(parede));
    }

    #[test]
    fn camada_esvaziada_pode_ser_removida() {
        let mut document = Document::new();
        let parede = document.create_layer("Parede").expect("nome válido");
        let id = document
            .insert_in_model_space(line(parede, 0.0))
            .expect("insere");

        document.remove_entity(id);
        let removida = document.remove_layer(parede).expect("camada agora vazia");

        assert_eq!(removida.name(), "Parede");
        assert!(!document.layers().contains(parede));
    }

    #[test]
    fn camada_zero_continua_protegida_pelo_documento() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();

        assert_eq!(
            document.remove_layer(zero),
            Err(DocumentError::Layer(LayerError::DefaultLayerIsProtected))
        );
    }

    #[test]
    fn remover_bloco_leva_junto_as_entidades() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        let porta = document.create_block("Porta").expect("nome válido");

        let dentro = document
            .insert_entity(line(zero, 0.0), porta)
            .expect("insere");
        let fora = document
            .insert_in_model_space(line(zero, 5.0))
            .expect("insere");

        let record = document.remove_block(porta).expect("bloco existe");

        assert_eq!(record.entities(), [dentro]);
        assert!(
            document.entity(dentro).is_none(),
            "entidade de bloco removido não pode ficar órfã na arena"
        );
        assert!(document.entity(fora).is_some());
        assert_eq!(document.entity_count(), 1);
    }

    #[test]
    fn espaco_modelo_continua_protegido_pelo_documento() {
        let mut document = Document::new();
        let model_space = document.model_space();

        assert_eq!(
            document.remove_block(model_space),
            Err(DocumentError::Block(BlockError::ModelSpaceIsProtected))
        );
    }

    #[test]
    fn move_entidade_entre_blocos_preservando_o_identificador() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        let porta = document.create_block("Porta").expect("nome válido");
        let id = document
            .insert_in_model_space(line(zero, 0.0))
            .expect("insere");

        document
            .edit()
            .set_entity_placement(
                id,
                EntityPlacement {
                    block: porta,
                    position: 0,
                },
            )
            .expect("entidade e bloco existem");

        assert!(document.blocks().model_space_record().is_empty());
        assert_eq!(
            document
                .entities_in_block(porta)
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            vec![id]
        );
        assert_eq!(document.entity_count(), 1, "mover não duplica");
    }

    #[test]
    fn mover_para_o_mesmo_bloco_nao_muda_nada() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        let id = document
            .insert_in_model_space(line(zero, 0.0))
            .expect("insere");

        let model_space = document.model_space();
        document
            .edit()
            .set_entity_placement(
                id,
                EntityPlacement {
                    block: model_space,
                    position: 0,
                },
            )
            .expect("mesmo bloco");

        assert_eq!(document.blocks().model_space_record().entities(), [id]);
    }

    #[test]
    fn mover_entidade_obsoleta_e_recusado() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        let porta = document.create_block("Porta").expect("nome válido");
        let id = document
            .insert_in_model_space(line(zero, 0.0))
            .expect("insere");
        document.remove_entity(id);

        assert_eq!(
            document.edit().set_entity_placement(
                id,
                EntityPlacement {
                    block: porta,
                    position: 0,
                },
            ),
            Err(DocumentError::UnknownEntity)
        );
    }

    #[test]
    fn caixa_envolvente_do_documento_une_todas_as_entidades() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        document
            .insert_in_model_space(circle(zero, 1.0))
            .expect("insere");
        document
            .insert_in_model_space(circle(zero, 3.0))
            .expect("insere");

        let caixa = document.bounding_box().expect("há entidades");

        assert_eq!(caixa.min(), Point2::new(-3.0, -3.0));
        assert_eq!(caixa.max(), Point2::new(3.0, 3.0));
    }

    #[test]
    fn caixa_do_bloco_ignora_entidades_de_outros_blocos() {
        let mut document = Document::new();
        let zero = document.layers().default_layer();
        let porta = document.create_block("Porta").expect("nome válido");

        document
            .insert_in_model_space(circle(zero, 1.0))
            .expect("insere");
        document
            .insert_entity(circle(zero, 10.0), porta)
            .expect("insere");

        let caixa = document
            .block_bounding_box(document.model_space())
            .expect("espaço-modelo tem entidade");

        assert_eq!(caixa.max(), Point2::new(1.0, 1.0));
        assert_eq!(
            document.bounding_box().expect("há entidades").max(),
            Point2::new(10.0, 10.0),
            "a caixa do documento inclui as definições de bloco"
        );
    }

    #[test]
    fn caixa_de_bloco_vazio_e_none() {
        let document = Document::new();

        assert_eq!(document.block_bounding_box(document.model_space()), None);
    }

    #[test]
    fn propriedades_de_camada_sao_editaveis_pelo_documento() {
        let mut document = Document::new();
        let parede = document.create_layer("Parede").expect("nome válido");

        let mut record = document
            .layers()
            .get(parede)
            .expect("camada existe")
            .clone();
        record.set_locked(true);
        document
            .edit()
            .set_layer_record(parede, record)
            .expect("camada existe");

        assert!(document
            .layers()
            .get(parede)
            .expect("camada existe")
            .is_locked());
    }

    #[test]
    fn renomeia_camada_e_bloco_pelo_documento() {
        let mut document = Document::new();
        let parede = document.create_layer("Parede").expect("nome válido");
        let porta = document.create_block("Porta").expect("nome válido");

        document.rename_layer(parede, "Alvenaria").expect("válido");
        document.rename_block(porta, "PortaDupla").expect("válido");

        assert_eq!(document.layers().id_of("Alvenaria"), Some(parede));
        assert_eq!(document.blocks().id_of("PortaDupla"), Some(porta));
    }

    #[test]
    fn ponto_base_do_bloco_definido_pelo_documento() {
        let mut document = Document::new();
        let porta = document.create_block("Porta").expect("nome válido");

        document
            .set_block_origin(porta, Point2::new(1.0, 2.0))
            .expect("bloco existe");

        assert_eq!(
            document.blocks().get(porta).expect("bloco existe").origin(),
            Point2::new(1.0, 2.0)
        );
    }

    #[test]
    fn estilo_de_texto_e_alcancavel_para_alteracao() {
        let mut document = Document::new();
        let id = document
            .text_styles_mut()
            .create("Cotas")
            .expect("nome válido");

        document
            .text_styles_mut()
            .get_mut(id)
            .expect("estilo existe")
            .set_fixed_height(2.5);

        assert_eq!(
            document
                .text_styles()
                .get(id)
                .expect("estilo existe")
                .effective_height(9.0),
            2.5
        );
    }
}
