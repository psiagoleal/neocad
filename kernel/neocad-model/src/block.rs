// Caminho relativo: kernel/neocad-model/src/block.rs
//! \file kernel/neocad-model/src/block.rs
//! \brief Tabela de blocos do documento, com o espaço-modelo como bloco raiz.
//! \author Iago Leal
//! \date 2026-08-07

use core::fmt;
use std::collections::BTreeMap;

use neocad_geometry::Point2;

use crate::arena::Arena;
use crate::id::EntityId;
use crate::symbol_name::{normalize, validate, InvalidName};

/// Nome interno do bloco que representa o espaço-modelo.
///
/// É o nome que os formatos DXF e DWG usam. Começa com asterisco porque, na
/// convenção desses formatos, o prefixo marca nomes reservados ao sistema — e é
/// justamente por isso que nomes com asterisco são recusados na criação.
pub const MODEL_SPACE_NAME: &str = "*Model_Space";

/// Identificador opaco de um bloco.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(EntityId);

impl fmt::Display for BlockId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "B{}", self.0)
    }
}

/// Registro de um bloco: um conjunto nomeado e ordenado de entidades.
///
/// O espaço-modelo é um bloco como qualquer outro — é assim que os formatos CAD
/// o representam, e adotar a mesma estrutura evita um caso especial em todo o
/// kernel.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockRecord {
    name: String,
    origin: Point2,
    entities: Vec<EntityId>,
}

impl BlockRecord {
    /// Nome de exibição, preservando a caixa com que o bloco foi criado.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Ponto base do bloco, usado como origem ao inserir uma referência.
    #[must_use]
    pub const fn origin(&self) -> Point2 {
        self.origin
    }

    /// Define o ponto base.
    pub fn set_origin(&mut self, origin: Point2) {
        self.origin = origin;
    }

    /// Entidades do bloco, na ordem de desenho.
    ///
    /// A ordem é significativa: define quem é desenhado por cima de quem.
    #[must_use]
    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    /// Quantidade de entidades no bloco.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Indica se o bloco não contém entidades.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Indica se a entidade pertence a este bloco.
    #[must_use]
    pub fn contains_entity(&self, entity: EntityId) -> bool {
        self.entities.contains(&entity)
    }

    /// Acrescenta uma entidade ao fim da ordem de desenho.
    ///
    /// Devolve `false`, sem alterar nada, se a entidade já estiver no bloco —
    /// uma entidade pertence a exatamente um bloco e aparece nele uma só vez.
    ///
    /// Não valida se o identificador existe na arena do documento: essa
    /// checagem exige o documento inteiro e entra em MT-K1-07.
    pub fn push_entity(&mut self, entity: EntityId) -> bool {
        if self.contains_entity(entity) {
            return false;
        }

        self.entities.push(entity);
        true
    }

    /// Insere uma entidade em uma posição específica da ordem de desenho.
    ///
    /// Existe para que desfazer uma remoção devolva a entidade ao lugar de onde
    /// ela saiu: reacrescentá-la ao fim mudaria silenciosamente quem é desenhado
    /// por cima de quem.
    ///
    /// Devolve `false`, sem alterar nada, se a entidade já estiver no bloco ou
    /// se a posição estiver além do fim da lista.
    pub fn insert_entity_at(&mut self, position: usize, entity: EntityId) -> bool {
        if position > self.entities.len() || self.contains_entity(entity) {
            return false;
        }

        self.entities.insert(position, entity);
        true
    }

    /// Posição da entidade na ordem de desenho, se ela pertencer ao bloco.
    #[must_use]
    pub fn position_of(&self, entity: EntityId) -> Option<usize> {
        self.entities.iter().position(|&item| item == entity)
    }

    /// Remove uma entidade do bloco, preservando a ordem das demais.
    ///
    /// Devolve `false` se a entidade não pertencia ao bloco.
    pub fn remove_entity(&mut self, entity: EntityId) -> bool {
        let Some(position) = self.entities.iter().position(|&item| item == entity) else {
            return false;
        };

        self.entities.remove(position);
        true
    }
}

/// Falha ao operar sobre a tabela de blocos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    /// O nome informado é vazio ou só contém espaços.
    EmptyName,
    /// O nome contém um caractere que os formatos CAD não aceitam.
    ///
    /// Inclui o asterisco, reservado aos nomes internos do formato.
    ForbiddenCharacter(char),
    /// Já existe bloco com esse nome. A comparação ignora caixa.
    DuplicateName(String),
    /// O espaço-modelo não pode ser removido nem renomeado.
    ModelSpaceIsProtected,
    /// O identificador não corresponde a nenhum bloco vivo.
    NotFound,
}

impl From<InvalidName> for BlockError {
    fn from(error: InvalidName) -> Self {
        match error {
            InvalidName::Empty => Self::EmptyName,
            InvalidName::Forbidden(character) => Self::ForbiddenCharacter(character),
        }
    }
}

impl fmt::Display for BlockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "o nome do bloco não pode ser vazio"),
            Self::ForbiddenCharacter(character) => write!(
                formatter,
                "o caractere {character:?} não é aceito em nome de bloco"
            ),
            Self::DuplicateName(name) => {
                write!(formatter, "já existe um bloco chamado {name:?}")
            }
            Self::ModelSpaceIsProtected => write!(
                formatter,
                "o espaço-modelo {MODEL_SPACE_NAME:?} não pode ser removido nem renomeado"
            ),
            Self::NotFound => write!(formatter, "bloco não encontrado"),
        }
    }
}

impl core::error::Error for BlockError {}

/// Tabela de blocos de um documento.
///
/// # Espaço-modelo como bloco raiz
///
/// Toda tabela nasce com o bloco `*Model_Space`, onde vivem as entidades do
/// desenho principal. Tratar o espaço-modelo como bloco — e não como uma lista
/// separada de entidades — é o que permite que criação, consulta e ordem de
/// desenho tenham um caminho único no kernel, tanto para o desenho quanto para
/// definições de bloco.
///
/// # Nomes reservados
///
/// Nomes iniciados por asterisco pertencem ao formato e são recusados na
/// criação, o que impede colisão com `*Model_Space` e com os blocos anônimos
/// gerados por hachuras e cotas.
///
/// # Exemplo
///
/// ```
/// use neocad_model::{BlockError, BlockTable};
///
/// let mut blocks = BlockTable::new();
///
/// // O espaço-modelo já existe.
/// assert!(blocks.get(blocks.model_space()).is_some());
///
/// let porta = blocks.create("Porta")?;
/// assert_eq!(blocks.get(porta).map(|b| b.entity_count()), Some(0));
///
/// // Nomes reservados do formato são recusados.
/// assert_eq!(
///     blocks.create("*Meu_Bloco"),
///     Err(BlockError::ForbiddenCharacter('*')),
/// );
/// # Ok::<(), BlockError>(())
/// ```
#[derive(Debug, Clone)]
pub struct BlockTable {
    records: Arena<BlockRecord>,
    by_normalized_name: BTreeMap<String, BlockId>,
    model_space: BlockId,
}

impl BlockTable {
    /// Cria uma tabela contendo apenas o espaço-modelo, vazio.
    #[must_use]
    pub fn new() -> Self {
        let mut records = Arena::new();
        // O espaço-modelo contorna a validação de nome de propósito: o nome dele
        // é reservado justamente para que ninguém mais possa criá-lo.
        let model_space = BlockId(records.insert(BlockRecord {
            name: String::from(MODEL_SPACE_NAME),
            origin: Point2::ORIGIN,
            entities: Vec::new(),
        }));

        let mut by_normalized_name = BTreeMap::new();
        by_normalized_name.insert(normalize(MODEL_SPACE_NAME), model_space);

        Self {
            records,
            by_normalized_name,
            model_space,
        }
    }

    /// Identificador do espaço-modelo, sempre presente.
    #[must_use]
    pub const fn model_space(&self) -> BlockId {
        self.model_space
    }

    /// Quantidade de blocos. Nunca é zero.
    #[must_use]
    #[expect(
        clippy::len_without_is_empty,
        reason = "a tabela nunca é vazia: o espaço-modelo não pode ser removido"
    )]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Cria um bloco vazio com o nome informado.
    ///
    /// # Errors
    ///
    /// Falha se o nome for inválido — incluindo nomes com asterisco, reservados
    /// ao formato — ou se colidir com um bloco existente, ignorando caixa.
    pub fn create(&mut self, name: impl Into<String>) -> Result<BlockId, BlockError> {
        let name = name.into();
        let normalized = validate(&name)?;

        if self.by_normalized_name.contains_key(&normalized) {
            return Err(BlockError::DuplicateName(name));
        }

        let id = BlockId(self.records.insert(BlockRecord {
            name,
            origin: Point2::ORIGIN,
            entities: Vec::new(),
        }));
        self.by_normalized_name.insert(normalized, id);

        Ok(id)
    }

    /// Devolve o bloco de `id`, ou `None` se o identificador estiver obsoleto.
    #[must_use]
    pub fn get(&self, id: BlockId) -> Option<&BlockRecord> {
        self.records.get(id.0)
    }

    /// Versão mutável de [`BlockTable::get`].
    ///
    /// Não permite alterar o nome: renomear passa por [`BlockTable::rename`],
    /// que mantém o índice coerente.
    #[must_use]
    pub fn get_mut(&mut self, id: BlockId) -> Option<&mut BlockRecord> {
        self.records.get_mut(id.0)
    }

    /// Atalho para o registro do espaço-modelo.
    #[must_use]
    pub fn model_space_record(&self) -> &BlockRecord {
        self.get(self.model_space)
            .expect("o espaço-modelo é indestrutível")
    }

    /// Atalho mutável para o registro do espaço-modelo.
    pub fn model_space_record_mut(&mut self) -> &mut BlockRecord {
        let model_space = self.model_space;

        self.get_mut(model_space)
            .expect("o espaço-modelo é indestrutível")
    }

    /// Procura um bloco pelo nome, ignorando caixa.
    #[must_use]
    pub fn id_of(&self, name: &str) -> Option<BlockId> {
        self.by_normalized_name.get(&normalize(name)).copied()
    }

    /// Procura um bloco pelo nome, ignorando caixa, devolvendo o registro.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&BlockRecord> {
        self.get(self.id_of(name)?)
    }

    /// Indica se `id` referencia um bloco vivo.
    #[must_use]
    pub fn contains(&self, id: BlockId) -> bool {
        self.records.contains(id.0)
    }

    /// Procura o bloco que contém a entidade.
    ///
    /// Percorre os blocos, o que é aceitável no volume de blocos de um desenho.
    /// Se virar gargalo, o caminho é um índice reverso mantido pelo documento.
    #[must_use]
    pub fn owner_of(&self, entity: EntityId) -> Option<BlockId> {
        self.iter()
            .find(|(_, record)| record.contains_entity(entity))
            .map(|(id, _)| id)
    }

    /// Renomeia um bloco, preservando seu identificador.
    ///
    /// # Errors
    ///
    /// Falha se o bloco for o espaço-modelo, se o identificador estiver
    /// obsoleto, ou se o nome novo for inválido ou colidir com outro bloco.
    pub fn rename(&mut self, id: BlockId, name: impl Into<String>) -> Result<(), BlockError> {
        if id == self.model_space {
            return Err(BlockError::ModelSpaceIsProtected);
        }

        let name = name.into();
        let normalized = validate(&name)?;

        if let Some(&existing) = self.by_normalized_name.get(&normalized) {
            if existing != id {
                return Err(BlockError::DuplicateName(name));
            }
        }

        let record = self.records.get_mut(id.0).ok_or(BlockError::NotFound)?;
        let previous = normalize(&record.name);
        record.name = name;

        self.by_normalized_name.remove(&previous);
        self.by_normalized_name.insert(normalized, id);

        Ok(())
    }

    /// Remove um bloco e devolve o registro removido, com suas entidades.
    ///
    /// # Errors
    ///
    /// Falha se o bloco for o espaço-modelo ou se o identificador estiver
    /// obsoleto.
    ///
    /// As entidades listadas no registro **não** são removidas da arena do
    /// documento: quem coordena as duas estruturas é o documento, em MT-K1-07.
    /// O registro devolvido carrega a lista justamente para que o chamador possa
    /// fazê-lo.
    pub fn remove(&mut self, id: BlockId) -> Result<BlockRecord, BlockError> {
        if id == self.model_space {
            return Err(BlockError::ModelSpaceIsProtected);
        }

        let record = self.records.remove(id.0).ok_or(BlockError::NotFound)?;
        self.by_normalized_name.remove(&normalize(&record.name));

        Ok(record)
    }

    /// Itera sobre os blocos em ordem alfabética de nome.
    pub fn iter(&self) -> impl Iterator<Item = (BlockId, &BlockRecord)> {
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

impl Default for BlockTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::Arena;

    /// Produz identificadores de entidade válidos para exercitar a associação.
    fn entity_ids(count: usize) -> Vec<EntityId> {
        let mut arena = Arena::new();
        (0..count).map(|value| arena.insert(value)).collect()
    }

    #[test]
    fn tabela_nova_contem_o_espaco_modelo_vazio() {
        let blocks = BlockTable::new();

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks.names().collect::<Vec<_>>(), vec![MODEL_SPACE_NAME]);
        assert!(blocks.contains(blocks.model_space()));
        assert!(blocks.model_space_record().is_empty());
        assert_eq!(blocks.model_space_record().origin(), Point2::ORIGIN);
    }

    #[test]
    fn espaco_modelo_e_alcancavel_por_nome() {
        let blocks = BlockTable::new();

        assert_eq!(blocks.id_of(MODEL_SPACE_NAME), Some(blocks.model_space()));
        assert_eq!(blocks.id_of("*model_space"), Some(blocks.model_space()));
    }

    #[test]
    fn cria_bloco_vazio() {
        let mut blocks = BlockTable::new();
        let id = blocks.create("Porta").expect("nome válido");

        let record = blocks.get(id).expect("bloco recém-criado");
        assert_eq!(record.name(), "Porta");
        assert!(record.is_empty());
        assert_eq!(record.entity_count(), 0);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn nome_reservado_com_asterisco_e_recusado() {
        let mut blocks = BlockTable::new();

        assert_eq!(
            blocks.create("*Model_Space"),
            Err(BlockError::ForbiddenCharacter('*')),
            "o nome do espaço-modelo é inalcançável pela criação"
        );
        assert_eq!(
            blocks.create("*U1"),
            Err(BlockError::ForbiddenCharacter('*'))
        );
    }

    #[test]
    fn nome_duplicado_e_rejeitado_ignorando_caixa() {
        let mut blocks = BlockTable::new();
        blocks.create("Porta").expect("nome válido");

        assert_eq!(
            blocks.create("PORTA"),
            Err(BlockError::DuplicateName(String::from("PORTA")))
        );
    }

    #[test]
    fn nome_invalido_segue_as_mesmas_regras_das_demais_tabelas() {
        let mut blocks = BlockTable::new();

        assert_eq!(blocks.create(" "), Err(BlockError::EmptyName));
        assert_eq!(
            blocks.create("Porta/Dupla"),
            Err(BlockError::ForbiddenCharacter('/'))
        );
    }

    #[test]
    fn acrescenta_entidades_preservando_a_ordem_de_desenho() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(3);
        let record = blocks.model_space_record_mut();

        for &id in &ids {
            assert!(record.push_entity(id));
        }

        assert_eq!(record.entities(), ids.as_slice());
        assert_eq!(record.entity_count(), 3);
        assert!(!record.is_empty());
    }

    #[test]
    fn entidade_repetida_nao_e_acrescentada_duas_vezes() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(1);
        let record = blocks.model_space_record_mut();

        assert!(record.push_entity(ids[0]));
        assert!(
            !record.push_entity(ids[0]),
            "uma entidade pertence ao bloco uma só vez"
        );
        assert_eq!(record.entity_count(), 1);
    }

    #[test]
    fn remove_entidade_preservando_a_ordem_das_demais() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(3);
        let record = blocks.model_space_record_mut();
        for &id in &ids {
            record.push_entity(id);
        }

        assert!(record.remove_entity(ids[1]));

        assert_eq!(record.entities(), [ids[0], ids[2]]);
        assert!(!record.contains_entity(ids[1]));
    }

    #[test]
    fn insere_entidade_em_posicao_especifica() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(3);
        let record = blocks.model_space_record_mut();
        record.push_entity(ids[0]);
        record.push_entity(ids[2]);

        assert!(record.insert_entity_at(1, ids[1]));

        assert_eq!(record.entities(), ids.as_slice());
    }

    #[test]
    fn insercao_posicional_recusa_posicao_invalida_ou_repetida() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(2);
        let record = blocks.model_space_record_mut();
        record.push_entity(ids[0]);

        assert!(!record.insert_entity_at(5, ids[1]), "posição além do fim");
        assert!(!record.insert_entity_at(0, ids[0]), "entidade já presente");
        assert_eq!(record.entity_count(), 1);
    }

    #[test]
    fn ciclo_remover_reinserir_preserva_a_ordem_de_desenho() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(3);
        let record = blocks.model_space_record_mut();
        for &id in &ids {
            record.push_entity(id);
        }

        let posicao = record.position_of(ids[1]).expect("entidade presente");
        record.remove_entity(ids[1]);
        record.insert_entity_at(posicao, ids[1]);

        assert_eq!(
            record.entities(),
            ids.as_slice(),
            "a entidade tem de voltar ao lugar de onde saiu"
        );
    }

    #[test]
    fn posicao_de_entidade_ausente_e_none() {
        let blocks = BlockTable::new();
        let ids = entity_ids(1);

        assert_eq!(blocks.model_space_record().position_of(ids[0]), None);
    }

    #[test]
    fn remover_entidade_ausente_devolve_false() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(2);
        let record = blocks.model_space_record_mut();
        record.push_entity(ids[0]);

        assert!(!record.remove_entity(ids[1]));
        assert_eq!(record.entity_count(), 1);
    }

    #[test]
    fn descobre_o_bloco_dono_de_uma_entidade() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(2);
        let porta = blocks.create("Porta").expect("nome válido");

        blocks.model_space_record_mut().push_entity(ids[0]);
        blocks
            .get_mut(porta)
            .expect("bloco existe")
            .push_entity(ids[1]);

        assert_eq!(blocks.owner_of(ids[0]), Some(blocks.model_space()));
        assert_eq!(blocks.owner_of(ids[1]), Some(porta));
    }

    #[test]
    fn entidade_sem_bloco_nao_tem_dono() {
        let blocks = BlockTable::new();
        let ids = entity_ids(1);

        assert_eq!(blocks.owner_of(ids[0]), None);
    }

    #[test]
    fn ponto_base_do_bloco_pode_ser_definido() {
        let mut blocks = BlockTable::new();
        let id = blocks.create("Porta").expect("nome válido");

        blocks
            .get_mut(id)
            .expect("bloco existe")
            .set_origin(Point2::new(1.0, 2.0));

        assert_eq!(
            blocks.get(id).expect("bloco existe").origin(),
            Point2::new(1.0, 2.0)
        );
    }

    #[test]
    fn espaco_modelo_e_protegido() {
        let mut blocks = BlockTable::new();
        let model_space = blocks.model_space();

        assert_eq!(
            blocks.remove(model_space),
            Err(BlockError::ModelSpaceIsProtected)
        );
        assert_eq!(
            blocks.rename(model_space, "Principal"),
            Err(BlockError::ModelSpaceIsProtected)
        );
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn remover_bloco_devolve_as_entidades_para_o_chamador_tratar() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(2);
        let porta = blocks.create("Porta").expect("nome válido");
        let record = blocks.get_mut(porta).expect("bloco existe");
        record.push_entity(ids[0]);
        record.push_entity(ids[1]);

        let removido = blocks.remove(porta).expect("bloco existe");

        assert_eq!(
            removido.entities(),
            ids.as_slice(),
            "o chamador precisa da lista para remover as entidades da arena"
        );
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks.id_of("Porta"), None);
    }

    #[test]
    fn renomear_preserva_o_identificador_e_as_entidades() {
        let mut blocks = BlockTable::new();
        let ids = entity_ids(1);
        let id = blocks.create("Porta").expect("nome válido");
        blocks
            .get_mut(id)
            .expect("bloco existe")
            .push_entity(ids[0]);

        blocks.rename(id, "PortaDupla").expect("nome válido");

        let record = blocks.get(id).expect("bloco continua existindo");
        assert_eq!(record.name(), "PortaDupla");
        assert_eq!(record.entities(), [ids[0]]);
        assert_eq!(blocks.id_of("Porta"), None);
    }

    #[test]
    fn identificador_obsoleto_e_rejeitado() {
        let mut blocks = BlockTable::new();
        let id = blocks.create("Porta").expect("nome válido");
        blocks.remove(id).expect("bloco existe");

        assert_eq!(blocks.get(id), None);
        assert_eq!(blocks.remove(id), Err(BlockError::NotFound));
        assert_eq!(blocks.rename(id, "Outro"), Err(BlockError::NotFound));
    }

    #[test]
    fn iteracao_segue_ordem_alfabetica() {
        let mut blocks = BlockTable::new();
        blocks.create("Porta").expect("nome válido");
        blocks.create("Janela").expect("nome válido");

        assert_eq!(
            blocks.names().collect::<Vec<_>>(),
            vec![MODEL_SPACE_NAME, "Janela", "Porta"],
            "o asterisco ordena antes das letras"
        );
    }

    #[test]
    fn default_equivale_a_new() {
        assert_eq!(BlockTable::default().len(), 1);
    }
}
