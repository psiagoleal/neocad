// Caminho relativo: kernel/neocad-model/src/symbol_table.rs
//! \file kernel/neocad-model/src/symbol_table.rs
//! \brief Máquina comum às tabelas de símbolos do documento.
//! \author Iago Leal
//! \date 2026-08-18

use std::collections::BTreeMap;

use crate::arena::Arena;
use crate::id::EntityId;
use crate::symbol_name::{normalize, validate, validate_reserved, InvalidName};

/// O que a tabela precisa saber sobre o registro que guarda.
///
/// A tabela mantém um índice por nome normalizado, e para isso precisa ler e
/// escrever o nome do registro. Nada além disso: cor, ponto-base e altura de
/// texto são assunto de cada tabela, não desta.
pub(crate) trait SymbolRecord {
    /// Nome de exibição, com a caixa original.
    fn name(&self) -> &str;
    /// Substitui o nome. Só a tabela chama, ao renomear.
    fn set_name(&mut self, name: String);
}

/// Falha ao operar sobre uma tabela de símbolos.
///
/// Genérica de propósito: cada tabela a traduz para o seu próprio erro, que é
/// quem sabe dizer *qual* registro é protegido — a camada `0`, o `*Model_Space`
/// ou o estilo `Standard`. Uma mensagem única diria menos ao usuário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SymbolError {
    /// O nome não passa nas regras comuns aos formatos CAD.
    Invalid(InvalidName),
    /// Já existe registro com esse nome, ignorando caixa.
    Duplicate(String),
    /// O registro protegido da tabela não pode ser renomeado nem removido.
    Protected,
    /// O identificador não corresponde a nenhum registro vivo.
    NotFound,
}

impl From<InvalidName> for SymbolError {
    fn from(error: InvalidName) -> Self {
        Self::Invalid(error)
    }
}

/// Tabela de símbolos: registros nomeados, com nome único e ordem estável.
///
/// # Por que existe
///
/// Camadas, blocos e estilos de texto são, nos formatos CAD, o **mesmo
/// conceito** — registro de tabela de símbolos. As três tabelas do modelo
/// repetiam a mesma máquina de índice por nome, e três cópias de uma regra são
/// três oportunidades de elas divergirem: bastava uma corrigir a comparação de
/// caixa e as outras não.
///
/// # O que ela não decide
///
/// Qual registro é protegido, o que significa cada erro e o que cabe num
/// registro continuam sendo de cada tabela. Esta guarda a mecânica, não a
/// política.
///
/// # Ordem de iteração
///
/// Alfabética pelo nome normalizado, e não de criação. É determinística e
/// coincide com a ordenação que os aplicativos CAD apresentam — e é dela que a
/// escrita DXF tira o seu determinismo.
#[derive(Debug, Clone)]
pub(crate) struct SymbolTable<T> {
    records: Arena<T>,
    by_normalized_name: BTreeMap<String, EntityId>,
    protected: EntityId,
}

impl<T: SymbolRecord> SymbolTable<T> {
    /// Cria a tabela com o seu registro protegido dentro.
    ///
    /// O nome do registro protegido **não** passa pela validação: ele é do
    /// sistema, e é justamente por ser reservado que ninguém mais pode criá-lo.
    pub(crate) fn with_protected(record: T) -> Self {
        let normalized = normalize(record.name());
        let mut records = Arena::new();
        let protected = records.insert(record);
        let mut by_normalized_name = BTreeMap::new();
        by_normalized_name.insert(normalized, protected);

        Self {
            records,
            by_normalized_name,
            protected,
        }
    }

    /// Identificador do registro protegido, sempre presente.
    pub(crate) const fn protected(&self) -> EntityId {
        self.protected
    }

    /// Quantidade de registros. Nunca é zero.
    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }

    /// Cria um registro com o nome informado.
    pub(crate) fn create(
        &mut self,
        name: String,
        build: impl FnOnce(String) -> T,
    ) -> Result<EntityId, SymbolError> {
        self.insert(name, validate, build)
    }

    /// Cria um registro de nome **reservado**, iniciado por asterisco.
    pub(crate) fn create_reserved(
        &mut self,
        name: String,
        build: impl FnOnce(String) -> T,
    ) -> Result<EntityId, SymbolError> {
        self.insert(name, validate_reserved, build)
    }

    /// Caminho comum das duas criações, com a validação como parâmetro.
    fn insert(
        &mut self,
        name: String,
        validar: impl FnOnce(&str) -> Result<String, InvalidName>,
        build: impl FnOnce(String) -> T,
    ) -> Result<EntityId, SymbolError> {
        let normalized = validar(&name)?;

        if self.by_normalized_name.contains_key(&normalized) {
            return Err(SymbolError::Duplicate(name));
        }

        let id = self.records.insert(build(name));
        self.by_normalized_name.insert(normalized, id);

        Ok(id)
    }

    /// Devolve o registro de `id`, ou `None` se o identificador estiver obsoleto.
    pub(crate) fn get(&self, id: EntityId) -> Option<&T> {
        self.records.get(id)
    }

    /// Versão mutável de [`SymbolTable::get`].
    ///
    /// Não permite alterar o nome: renomear precisa manter o índice coerente, e
    /// por isso passa por [`SymbolTable::rename`].
    pub(crate) fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        self.records.get_mut(id)
    }

    /// Procura um registro pelo nome, ignorando caixa.
    pub(crate) fn id_of(&self, name: &str) -> Option<EntityId> {
        self.by_normalized_name.get(&normalize(name)).copied()
    }

    /// Procura um registro pelo nome, ignorando caixa.
    pub(crate) fn get_by_name(&self, name: &str) -> Option<&T> {
        self.id_of(name).and_then(|id| self.get(id))
    }

    /// Indica se o identificador corresponde a um registro vivo.
    pub(crate) fn contains(&self, id: EntityId) -> bool {
        self.records.contains(id)
    }

    /// Renomeia um registro, preservando o identificador.
    pub(crate) fn rename(&mut self, id: EntityId, name: String) -> Result<(), SymbolError> {
        if id == self.protected {
            return Err(SymbolError::Protected);
        }

        let normalized = validate(&name)?;

        // Renomear mudando apenas a caixa é permitido: o nome normalizado é o
        // mesmo, e o registro que o ocupa é este.
        if let Some(&existing) = self.by_normalized_name.get(&normalized) {
            if existing != id {
                return Err(SymbolError::Duplicate(name));
            }
        }

        let record = self.records.get_mut(id).ok_or(SymbolError::NotFound)?;
        let previous = normalize(record.name());
        record.set_name(name);

        self.by_normalized_name.remove(&previous);
        self.by_normalized_name.insert(normalized, id);

        Ok(())
    }

    /// Remove um registro e libera o seu nome.
    pub(crate) fn remove(&mut self, id: EntityId) -> Result<T, SymbolError> {
        if id == self.protected {
            return Err(SymbolError::Protected);
        }

        let record = self.records.remove(id).ok_or(SymbolError::NotFound)?;
        self.by_normalized_name.remove(&normalize(record.name()));

        Ok(record)
    }

    /// Itera em ordem alfabética de nome normalizado.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.by_normalized_name.values().filter_map(|&id| {
            let record = self.records.get(id)?;
            Some((id, record))
        })
    }

    /// Itera sobre os nomes de exibição, na mesma ordem.
    pub(crate) fn names(&self) -> impl Iterator<Item = &str> {
        self.iter().map(|(_, record)| record.name())
    }
}
