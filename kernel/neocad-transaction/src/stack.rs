// Caminho relativo: kernel/neocad-transaction/src/stack.rs
//! \file kernel/neocad-transaction/src/stack.rs
//! \brief Pilha de comandos com desfazer e refazer.
//! \author Iago Leal
//! \date 2026-08-07

use std::collections::VecDeque;

use neocad_model::{Document, DocumentEditor, DocumentError};

use crate::transaction::{Transaction, TransactionError};

/// Profundidade padrão do histórico de desfazer.
pub const DEFAULT_UNDO_LIMIT: usize = 256;

/// Pilha de comandos que sustenta desfazer e refazer.
///
/// # Como funciona
///
/// A pilha guarda, para cada ação já aplicada, a **transação que a desfaz**.
/// Desfazer é aplicar essa transação — o que produz a transação que refaz, e
/// assim por diante. `undo` e `redo` são, portanto, a mesma operação em sentidos
/// opostos, e não dois caminhos de código que precisam ser mantidos coerentes
/// entre si.
///
/// # Ramo de refazer
///
/// Confirmar uma transação nova **descarta** o ramo de refazer. Depois de
/// desfazer e fazer outra coisa, as ações desfeitas partiam de um estado que não
/// existe mais: reaplicá-las produziria um resultado que o usuário nunca viu.
///
/// # Limite
///
/// O histórico é limitado para que um desenho longo não consuma memória sem
/// teto. Ao estourar o limite, a transação **mais antiga** é descartada — ela é
/// a menos provável de ser desfeita. Limite zero desativa o histórico: as
/// transações continuam sendo aplicadas, mas nada fica desfazível.
///
/// # Exemplo
///
/// ```
/// use neocad_model::{Document, Entity, Geometry, Line};
/// use neocad_geometry::Point2;
/// use neocad_transaction::{Change, CommandStack, Transaction};
///
/// let mut document = Document::new();
/// let zero = document.layers().default_layer();
///
/// let mut editor = document.edit();
/// let id = editor.insert_in_model_space(Entity::new(
///     zero,
///     Geometry::Line(Line { start: Point2::ORIGIN, end: Point2::new(1.0, 0.0) }),
/// ))?;
/// let _ = editor.finish();
///
/// let mut stack = CommandStack::new();
/// stack.commit(
///     &mut document,
///     Transaction::new("Apagar linha", vec![Change::RemoveEntity { entity: id }]),
/// )?;
///
/// assert!(document.entity(id).is_none());
/// assert_eq!(stack.undo_name(), Some("Apagar linha"));
///
/// stack.undo(&mut document)?;
/// assert!(document.entity(id).is_some());
/// # Ok::<(), Box<dyn core::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct CommandStack {
    /// Transações que desfazem, da mais antiga para a mais recente.
    undoable: VecDeque<Transaction>,
    /// Transações que refazem, da mais antiga para a mais recente.
    redoable: Vec<Transaction>,
    limit: usize,
}

impl CommandStack {
    /// Cria uma pilha com o limite padrão.
    #[must_use]
    pub fn new() -> Self {
        Self::with_limit(DEFAULT_UNDO_LIMIT)
    }

    /// Cria uma pilha com o limite informado.
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self {
            undoable: VecDeque::new(),
            redoable: Vec::new(),
            limit,
        }
    }

    /// Profundidade máxima do histórico.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Define a profundidade máxima, descartando desde já o excedente mais
    /// antigo.
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
        self.enforce_limit();
    }

    /// Aplica uma transação e a registra no histórico.
    ///
    /// Uma transação vazia é aplicada — sem efeito — e **não** entra no
    /// histórico, nem descarta o ramo de refazer: ela não mudou nada, então não
    /// deve consumir um passo de desfazer nem invalidar o que estava refazível.
    ///
    /// # Errors
    ///
    /// Propaga a falha da transação. Nesse caso o documento fica como estava e o
    /// histórico não é tocado.
    pub fn commit(
        &mut self,
        document: &mut Document,
        transaction: Transaction,
    ) -> Result<(), TransactionError> {
        if transaction.is_empty() {
            return Ok(());
        }

        let undo = transaction.apply(document)?;

        self.redoable.clear();
        self.undoable.push_back(undo);
        self.enforce_limit();

        Ok(())
    }

    /// Executa uma edição e a registra no histórico como uma ação nomeada.
    ///
    /// É a via normal de alterar o desenho. A closure recebe o
    /// [`DocumentEditor`] — o único caminho público de mutação — e o que ela
    /// fizer vira um passo de desfazer.
    ///
    /// Resolve o que [`CommandStack::commit`] não alcança: criar entidade nova
    /// exige que o documento **emita** o identificador, e uma [`Transaction`]
    /// pré-montada só sabe operar sobre identificadores que já existem.
    ///
    /// Uma edição que não altera nada não entra no histórico nem descarta o ramo
    /// de refazer, pela mesma razão de uma transação vazia.
    ///
    /// # Errors
    ///
    /// Se a closure falhar, tudo o que ela já tiver feito é revertido e o erro é
    /// propagado — a edição é atômica como qualquer transação.
    ///
    /// # Exemplo
    ///
    /// ```
    /// use neocad_model::{Document, Entity, Geometry, Line};
    /// use neocad_geometry::Point2;
    /// use neocad_transaction::CommandStack;
    ///
    /// let mut document = Document::new();
    /// let zero = document.layers().default_layer();
    /// let mut stack = CommandStack::new();
    ///
    /// let id = stack.edit(&mut document, "Desenhar linha", |editor| {
    ///     editor.insert_in_model_space(Entity::new(
    ///         zero,
    ///         Geometry::Line(Line { start: Point2::ORIGIN, end: Point2::new(5.0, 0.0) }),
    ///     ))
    /// })?;
    ///
    /// assert!(document.entity(id).is_some());
    /// assert_eq!(stack.undo_name(), Some("Desenhar linha"));
    ///
    /// stack.undo(&mut document)?;
    /// assert!(document.entity(id).is_none(), "a criação foi desfeita");
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    pub fn edit<F, T>(
        &mut self,
        document: &mut Document,
        name: impl Into<String>,
        build: F,
    ) -> Result<T, TransactionError>
    where
        F: FnOnce(&mut DocumentEditor<'_>) -> Result<T, DocumentError>,
    {
        let mut editor = document.edit();

        match build(&mut editor) {
            Ok(value) => {
                let undo = editor.finish();

                if !undo.is_empty() {
                    self.redoable.clear();
                    self.undoable.push_back(Transaction::new(name, undo));
                    self.enforce_limit();
                }

                Ok(value)
            }
            Err(error) => {
                editor
                    .rollback()
                    .map_err(|source| TransactionError::RollbackFailed { index: 0, source })?;

                Err(TransactionError::Edit(error))
            }
        }
    }

    /// Desfaz a última transação aplicada.
    ///
    /// Devolve `false` se não houver o que desfazer.
    ///
    /// # Errors
    ///
    /// Propaga a falha da transação. A transação permanece no histórico para que
    /// o estado da pilha continue refletindo o do documento.
    pub fn undo(&mut self, document: &mut Document) -> Result<bool, TransactionError> {
        let Some(transaction) = self.undoable.pop_back() else {
            return Ok(false);
        };

        match transaction.clone().apply(document) {
            Ok(redo) => {
                self.redoable.push(redo);
                Ok(true)
            }
            Err(error) => {
                self.undoable.push_back(transaction);
                Err(error)
            }
        }
    }

    /// Refaz a última transação desfeita.
    ///
    /// Devolve `false` se não houver o que refazer.
    ///
    /// # Errors
    ///
    /// Propaga a falha da transação, preservando o histórico.
    pub fn redo(&mut self, document: &mut Document) -> Result<bool, TransactionError> {
        let Some(transaction) = self.redoable.pop() else {
            return Ok(false);
        };

        match transaction.clone().apply(document) {
            Ok(undo) => {
                self.undoable.push_back(undo);
                Ok(true)
            }
            Err(error) => {
                self.redoable.push(transaction);
                Err(error)
            }
        }
    }

    /// Indica se há transação a desfazer.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undoable.is_empty()
    }

    /// Indica se há transação a refazer.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redoable.is_empty()
    }

    /// Nome da ação que `undo` desfaria.
    ///
    /// É o que a interface exibe em `Desfazer <nome>`.
    #[must_use]
    pub fn undo_name(&self) -> Option<&str> {
        self.undoable.back().map(Transaction::name)
    }

    /// Nome da ação que `redo` refaria.
    #[must_use]
    pub fn redo_name(&self) -> Option<&str> {
        self.redoable.last().map(Transaction::name)
    }

    /// Quantidade de transações desfazíveis.
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.undoable.len()
    }

    /// Quantidade de transações refazíveis.
    #[must_use]
    pub fn redo_depth(&self) -> usize {
        self.redoable.len()
    }

    /// Esvazia o histórico, sem alterar o documento.
    ///
    /// Usado ao abrir ou salvar um documento, quando o histórico anterior deixa
    /// de fazer sentido.
    pub fn clear(&mut self) {
        self.undoable.clear();
        self.redoable.clear();
    }

    /// Descarta as transações mais antigas que excedem o limite.
    fn enforce_limit(&mut self) {
        while self.undoable.len() > self.limit {
            self.undoable.pop_front();
        }
    }
}

impl Default for CommandStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neocad_geometry::Point2;
    use neocad_model::Change;
    use neocad_model::{Entity, EntityId, Geometry, LayerId, Line};

    fn line(layer: LayerId, x: f64) -> Entity {
        Entity::new(
            layer,
            Geometry::Line(Line {
                start: Point2::new(x, 0.0),
                end: Point2::new(x + 1.0, 1.0),
            }),
        )
    }

    /// Documento com quatro entidades no espaço-modelo.
    fn documento() -> (Document, Vec<EntityId>) {
        let mut document = Document::new();
        let zero = document.layers().default_layer();

        let mut editor = document.edit();
        let ids = (0..4)
            .map(|index| {
                editor
                    .insert_in_model_space(line(zero, f64::from(index) * 5.0))
                    .expect("insere")
            })
            .collect();
        let _ = editor.finish();

        (document, ids)
    }

    fn apagar(name: &str, entity: EntityId) -> Transaction {
        Transaction::new(name, vec![Change::RemoveEntity { entity }])
    }

    #[test]
    fn pilha_nova_nao_tem_nada_a_desfazer() {
        let stack = CommandStack::new();

        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(stack.undo_name(), None);
        assert_eq!(stack.redo_name(), None);
        assert_eq!(stack.limit(), DEFAULT_UNDO_LIMIT);
    }

    #[test]
    fn commit_aplica_e_torna_desfazivel() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();

        stack
            .commit(&mut document, apagar("Apagar", ids[0]))
            .expect("aplica");

        assert!(document.entity(ids[0]).is_none());
        assert!(stack.can_undo());
        assert_eq!(stack.undo_name(), Some("Apagar"));
        assert_eq!(stack.undo_depth(), 1);
    }

    #[test]
    fn undo_e_redo_encadeados_percorrem_o_historico() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();

        // `estados[n]` é o documento após n transações confirmadas.
        let mut estados = vec![document.clone()];
        for (index, &id) in ids.iter().enumerate().take(3) {
            stack
                .commit(&mut document, apagar(&format!("Apagar {index}"), id))
                .expect("aplica");
            estados.push(document.clone());
        }

        // Desfazer percorre o histórico de trás para frente, e cada passo tem de
        // reproduzir exatamente o estado correspondente.
        for esperado in estados.iter().rev().skip(1) {
            assert!(stack.undo(&mut document).expect("desfaz"));
            assert_eq!(document, *esperado);
        }
        assert!(!stack.can_undo());
        assert_eq!(stack.redo_depth(), 3);

        // Refazer percorre o mesmo caminho na ordem direta.
        for esperado in estados.iter().skip(1) {
            assert!(stack.redo(&mut document).expect("refaz"));
            assert_eq!(document, *esperado);
        }
        assert!(!stack.can_redo());
        assert_eq!(stack.undo_depth(), 3);
    }

    #[test]
    fn undo_sem_historico_devolve_false() {
        let (mut document, _) = documento();
        let mut stack = CommandStack::new();

        assert!(!stack.undo(&mut document).expect("sem erro"));
    }

    #[test]
    fn redo_sem_historico_devolve_false() {
        let (mut document, _) = documento();
        let mut stack = CommandStack::new();

        assert!(!stack.redo(&mut document).expect("sem erro"));
    }

    #[test]
    fn nova_transacao_descarta_o_ramo_de_refazer() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();

        stack
            .commit(&mut document, apagar("Primeira", ids[0]))
            .expect("aplica");
        stack.undo(&mut document).expect("desfaz");
        assert!(stack.can_redo());

        stack
            .commit(&mut document, apagar("Segunda", ids[1]))
            .expect("aplica");

        assert!(
            !stack.can_redo(),
            "o ramo de refazer partia de um estado que não existe mais"
        );
        assert_eq!(stack.redo_depth(), 0);
        assert_eq!(stack.undo_name(), Some("Segunda"));
    }

    #[test]
    fn transacao_vazia_nao_entra_no_historico() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();
        stack
            .commit(&mut document, apagar("Apagar", ids[0]))
            .expect("aplica");

        stack
            .commit(&mut document, Transaction::named("Nada"))
            .expect("aplica");

        assert_eq!(stack.undo_depth(), 1);
        assert_eq!(
            stack.undo_name(),
            Some("Apagar"),
            "a transação vazia não pode ocupar um passo de desfazer"
        );
    }

    #[test]
    fn transacao_vazia_preserva_o_ramo_de_refazer() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();
        stack
            .commit(&mut document, apagar("Apagar", ids[0]))
            .expect("aplica");
        stack.undo(&mut document).expect("desfaz");

        stack
            .commit(&mut document, Transaction::named("Nada"))
            .expect("aplica");

        assert!(
            stack.can_redo(),
            "nada mudou, então o ramo de refazer continua válido"
        );
    }

    #[test]
    fn limite_descarta_a_transacao_mais_antiga() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::with_limit(2);

        for (index, &id) in ids.iter().enumerate().take(3) {
            stack
                .commit(&mut document, apagar(&format!("Apagar {index}"), id))
                .expect("aplica");
        }

        assert_eq!(stack.undo_depth(), 2);
        assert_eq!(stack.undo_name(), Some("Apagar 2"));

        stack.undo(&mut document).expect("desfaz");
        stack.undo(&mut document).expect("desfaz");

        assert!(
            !stack.can_undo(),
            "a transação mais antiga saiu do histórico"
        );
        assert!(
            document.entity(ids[0]).is_none(),
            "o efeito da transação descartada permanece: ela não é desfazível"
        );
    }

    #[test]
    fn limite_zero_desativa_o_historico_sem_impedir_a_aplicacao() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::with_limit(0);

        stack
            .commit(&mut document, apagar("Apagar", ids[0]))
            .expect("aplica");

        assert!(document.entity(ids[0]).is_none(), "a ação acontece");
        assert!(!stack.can_undo(), "mas não fica desfazível");
    }

    #[test]
    fn reduzir_o_limite_descarta_o_excedente_na_hora() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();
        for (index, &id) in ids.iter().enumerate().take(3) {
            stack
                .commit(&mut document, apagar(&format!("Apagar {index}"), id))
                .expect("aplica");
        }

        stack.set_limit(1);

        assert_eq!(stack.undo_depth(), 1);
        assert_eq!(stack.undo_name(), Some("Apagar 2"));
        assert_eq!(stack.limit(), 1);
    }

    #[test]
    fn commit_com_falha_nao_toca_no_historico() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();
        stack
            .commit(&mut document, apagar("Apagar", ids[0]))
            .expect("aplica");
        stack.undo(&mut document).expect("desfaz");
        let antes = document.clone();

        // A segunda mudança falha porque a primeira já removeu a entidade.
        let invalida = Transaction::new(
            "Inválida",
            vec![
                Change::RemoveEntity { entity: ids[1] },
                Change::RemoveEntity { entity: ids[1] },
            ],
        );
        stack
            .commit(&mut document, invalida)
            .expect_err("deve falhar");

        assert_eq!(document, antes, "documento intacto");
        assert!(
            stack.can_redo(),
            "o ramo de refazer não pode ter sido perdido"
        );
        assert_eq!(stack.undo_depth(), 0);
    }

    #[test]
    fn clear_esvazia_o_historico_sem_alterar_o_documento() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();
        stack
            .commit(&mut document, apagar("Apagar", ids[0]))
            .expect("aplica");
        let depois = document.clone();

        stack.clear();

        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(document, depois);
    }

    #[test]
    fn nomes_acompanham_o_passo_corrente() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();
        stack
            .commit(&mut document, apagar("Primeira", ids[0]))
            .expect("aplica");
        stack
            .commit(&mut document, apagar("Segunda", ids[1]))
            .expect("aplica");

        assert_eq!(stack.undo_name(), Some("Segunda"));

        stack.undo(&mut document).expect("desfaz");

        assert_eq!(stack.undo_name(), Some("Primeira"));
        assert_eq!(stack.redo_name(), Some("Segunda"));
    }

    #[test]
    fn edit_cria_entidade_nova_e_torna_a_criacao_desfazivel() {
        let (mut document, _) = documento();
        let zero = document.layers().default_layer();
        let antes = document.clone();
        let mut stack = CommandStack::new();

        let id = stack
            .edit(&mut document, "Desenhar linha", |editor| {
                editor.insert_in_model_space(line(zero, 99.0))
            })
            .expect("edição válida");

        assert!(document.entity(id).is_some());
        assert_eq!(stack.undo_name(), Some("Desenhar linha"));

        stack.undo(&mut document).expect("desfaz");

        assert_eq!(
            document, antes,
            "desfazer uma criação tem de devolver o documento ao estado anterior"
        );
    }

    #[test]
    fn edit_agrupa_varias_operacoes_em_um_passo() {
        let (mut document, ids) = documento();
        let zero = document.layers().default_layer();
        let antes = document.clone();
        let mut stack = CommandStack::new();

        stack
            .edit(&mut document, "Editar", |editor| {
                editor.remove_entity(ids[0])?;
                editor.insert_in_model_space(line(zero, 99.0))?;

                Ok(())
            })
            .expect("edição válida");

        assert_eq!(stack.undo_depth(), 1, "duas operações, um só passo");

        stack.undo(&mut document).expect("desfaz");

        assert_eq!(document, antes);
    }

    #[test]
    fn edit_que_falha_no_meio_reverte_o_que_ja_fez() {
        let (mut document, ids) = documento();
        let zero = document.layers().default_layer();
        let antes = document.clone();
        let mut stack = CommandStack::new();

        let erro = stack
            .edit(&mut document, "Inválida", |editor| {
                editor.insert_in_model_space(line(zero, 99.0))?;
                // A segunda falha: a entidade é removida duas vezes.
                editor.remove_entity(ids[0])?;
                editor.remove_entity(ids[0])?;

                Ok(())
            })
            .expect_err("deve falhar");

        assert!(matches!(erro, TransactionError::Edit(_)), "erro: {erro}");
        assert_eq!(document, antes, "nada pode ter sobrado da edição parcial");
        assert!(!stack.can_undo(), "uma edição revertida não vira histórico");
    }

    #[test]
    fn edit_sem_efeito_nao_ocupa_passo_de_desfazer() {
        let (mut document, ids) = documento();
        let mut stack = CommandStack::new();
        stack
            .commit(&mut document, apagar("Apagar", ids[0]))
            .expect("aplica");
        stack.undo(&mut document).expect("desfaz");

        stack
            .edit(&mut document, "Nada", |editor| {
                let _ = editor.document().entity_count();

                Ok(())
            })
            .expect("edição vazia");

        assert_eq!(stack.undo_depth(), 0);
        assert!(stack.can_redo(), "o ramo de refazer continua válido");
    }

    #[test]
    fn default_equivale_a_new() {
        assert_eq!(CommandStack::default().limit(), DEFAULT_UNDO_LIMIT);
    }
}
