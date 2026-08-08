// Caminho relativo: kernel/neocad-transaction/src/lib.rs
//! \file kernel/neocad-transaction/src/lib.rs
//! \brief Transações reversíveis e command stack do kernel CAD do NeoCAD.
//! \author Iago Leal
//! \date 2026-08-06
//!
//! Responsabilidade: journal de mudanças atômicas reversíveis, agrupamento em
//! transações nomeadas e pilha de `undo`/`redo`.
//!
//! Recebe conteúdo de MT-K1-08 a MT-K1-10. A partir de MT-K1-10, é a **única**
//! via de mutação do documento — a diretriz do ADR 0003 que proíbe alterar o
//! modelo fora de transação é verificada por teste de compilação negativa.
//!
//! Conforme o ADR 0003, esta crate não conhece Tauri, Svelte, DOM nem tipos do
//! NeoCAD, e não pode receber dependências copyleft.

mod stack;
mod transaction;

// O primitivo de mudança vive no `neocad-model`, junto do documento: é a única
// forma de o compilador poder exigir que toda mutação seja registrada (MT-K1-10).
pub use neocad_model::{Change, ChangeError};
pub use stack::{CommandStack, DEFAULT_UNDO_LIMIT};
pub use transaction::{Transaction, TransactionError};
