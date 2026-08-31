// Caminho relativo: kernel/neocad-model/src/arena.rs
//! \file kernel/neocad-model/src/arena.rs
//! \brief Arena geracional que armazena entidades e detecta identificadores obsoletos.
//! \author Iago Leal
//! \date 2026-08-07

use core::fmt;
use core::mem;
use core::num::NonZeroU32;

use crate::id::EntityId;

/// Geração de um slot recém-criado.
const FIRST_GENERATION: NonZeroU32 = NonZeroU32::MIN;

/// Falha ao restaurar um valor em um identificador já conhecido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreError {
    /// O slot referenciado nunca existiu nesta arena.
    UnknownSlot,
    /// O slot está ocupado: restaurar sobre ele descartaria um valor vivo.
    SlotOccupied,
    /// O slot foi aposentado por esgotamento de geração e não aceita valores.
    SlotRetired,
}

impl fmt::Display for RestoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSlot => write!(formatter, "identificador não pertence a esta arena"),
            Self::SlotOccupied => write!(formatter, "o slot está ocupado"),
            Self::SlotRetired => write!(formatter, "o slot foi aposentado"),
        }
    }
}

impl core::error::Error for RestoreError {}

/// Estado de um slot da arena.
#[derive(Debug, Clone)]
enum Entry<T> {
    /// Slot em uso, guardando um valor.
    Occupied(T),
    /// Slot livre, encadeado na lista de reuso.
    Vacant { next_free: Option<u32> },
    /// Slot aposentado: a geração esgotou e ele nunca será reutilizado, para que
    /// nenhum identificador já emitido possa ser reemitido para outro valor.
    Retired,
}

#[derive(Debug, Clone)]
struct Slot<T> {
    generation: NonZeroU32,
    entry: Entry<T>,
}

/// Armazenamento geracional de valores endereçados por [`EntityId`].
///
/// A arena é a estrutura de base do modelo de documento: entidades vivem aqui e
/// são referenciadas por identificador, nunca por índice cru nem por ponteiro.
///
/// # Por que geracional
///
/// Reutilizar slots é necessário para não vazar memória à medida que entidades
/// são criadas e apagadas — mas reutilizar um índice puro faria um identificador
/// antigo passar a apontar para a entidade nova, corrompendo silenciosamente
/// seleções, histórico de comandos e referências entre entidades. A geração,
/// incrementada a cada remoção, transforma esse erro silencioso em um `None`
/// explícito.
///
/// # Determinismo
///
/// [`Arena::iter`] percorre por índice crescente, e a lista de reuso é mantida
/// em ordem crescente de índice. Duas sequências idênticas de operações produzem
/// os mesmos identificadores — propriedade da qual as transações e os testes de
/// regressão dependem.
///
/// # Igualdade
///
/// A arena deliberadamente **não** implementa `PartialEq`. Comparar duas arenas
/// exige decidir se a lista de reuso e as gerações fazem parte do estado
/// observável, e essa definição pertence à camada de transações, não aqui.
///
/// # Exemplo
///
/// ```
/// use neocad_model::Arena;
///
/// let mut arena = Arena::new();
/// let id = arena.insert("linha");
///
/// assert_eq!(arena.get(id), Some(&"linha"));
///
/// assert_eq!(arena.remove(id), Some("linha"));
/// assert_eq!(arena.get(id), None, "o identificador ficou obsoleto");
/// ```
#[derive(Debug, Clone)]
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free_head: Option<u32>,
    len: usize,
}

impl<T> Arena<T> {
    /// Cria uma arena vazia.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_head: None,
            len: 0,
        }
    }

    /// Cria uma arena vazia com capacidade reservada para `capacity` slots.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_head: None,
            len: 0,
        }
    }

    /// Quantidade de valores atualmente armazenados.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Indica se a arena não contém nenhum valor.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Insere um valor e devolve o identificador emitido para ele.
    ///
    /// Reutiliza o slot livre de menor índice quando há algum; caso contrário,
    /// cresce a arena.
    ///
    /// # Panics
    ///
    /// Entra em pânico se a arena exceder `u32::MAX` slots.
    pub fn insert(&mut self, value: T) -> EntityId {
        match self.free_head {
            Some(index) => {
                let slot = &mut self.slots[index as usize];
                let next_free = match slot.entry {
                    Entry::Vacant { next_free } => next_free,
                    Entry::Occupied(_) | Entry::Retired => {
                        unreachable!("slot na lista de reuso não está vago")
                    }
                };

                slot.entry = Entry::Occupied(value);
                let generation = slot.generation;

                self.free_head = next_free;
                self.len += 1;
                EntityId::new(index, generation)
            }
            None => {
                let index = u32::try_from(self.slots.len()).expect("arena excedeu u32::MAX slots");

                self.slots.push(Slot {
                    generation: FIRST_GENERATION,
                    entry: Entry::Occupied(value),
                });
                self.len += 1;
                EntityId::new(index, FIRST_GENERATION)
            }
        }
    }

    /// Devolve o valor associado a `id`, ou `None` se o identificador estiver
    /// obsoleto ou nunca ter sido emitido por esta arena.
    #[must_use]
    pub fn get(&self, id: EntityId) -> Option<&T> {
        match &self.resolve(id)?.entry {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant { .. } | Entry::Retired => None,
        }
    }

    /// Versão mutável de [`Arena::get`].
    #[must_use]
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index() as usize)?;
        if slot.generation != id.generation() {
            return None;
        }

        match &mut slot.entry {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant { .. } | Entry::Retired => None,
        }
    }

    /// Indica se `id` referencia um valor vivo nesta arena.
    #[must_use]
    pub fn contains(&self, id: EntityId) -> bool {
        self.get(id).is_some()
    }

    /// Remove e devolve o valor associado a `id`.
    ///
    /// Devolve `None` se o identificador já estiver obsoleto. Após a remoção, o
    /// identificador nunca mais resolve: a geração do slot é incrementada antes
    /// de ele voltar à lista de reuso.
    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        let index = id.index();
        let free_head = self.free_head;

        let slot = self.slots.get_mut(index as usize)?;
        if slot.generation != id.generation() {
            return None;
        }
        if !matches!(slot.entry, Entry::Occupied(_)) {
            return None;
        }

        // Sem geração seguinte, o slot é aposentado em vez de voltar ao reuso —
        // reutilizá-lo reemitiria um identificador já entregue.
        let next_generation = slot.generation.checked_add(1);
        let replacement = match next_generation {
            Some(_) => Entry::Vacant {
                next_free: free_head,
            },
            None => Entry::Retired,
        };

        let value = match mem::replace(&mut slot.entry, replacement) {
            Entry::Occupied(value) => value,
            Entry::Vacant { .. } | Entry::Retired => {
                unreachable!("ocupação verificada logo acima")
            }
        };

        if let Some(generation) = next_generation {
            slot.generation = generation;
            self.free_head = Some(index);
        }

        self.len -= 1;
        Some(value)
    }

    /// Restaura um valor **no identificador exato** informado.
    ///
    /// # Por que isto existe
    ///
    /// Desfazer uma remoção não pode reinserir o valor com um identificador
    /// novo: seleções, referências entre entidades e as demais mudanças da mesma
    /// transação apontam para o identificador antigo. Sem restaurar o mesmo
    /// identificador, desfazer conserta o desenho e quebra tudo que o
    /// referenciava.
    ///
    /// Por isso a geração do slot volta ao valor carregado pelo identificador.
    /// Isso é seguro porque um slot vago não tem identificador vivo: a geração
    /// intermediária nunca foi entregue a ninguém — `insert` só devolve gerações
    /// de slots que passa a ocupar.
    ///
    /// # Uso pretendido
    ///
    /// Primitiva de baixo nível para a reprodução de um journal de transações
    /// (`neocad-transaction`). Fora desse contexto, prefira
    /// [`Arena::insert`], que aloca um identificador novo.
    ///
    /// # Errors
    ///
    /// Falha se o slot não existir, estiver ocupado ou tiver sido aposentado.
    pub fn insert_at(&mut self, id: EntityId, value: T) -> Result<(), RestoreError> {
        let index = id.index();
        let slot = self
            .slots
            .get(index as usize)
            .ok_or(RestoreError::UnknownSlot)?;

        match slot.entry {
            Entry::Occupied(_) => return Err(RestoreError::SlotOccupied),
            Entry::Retired => return Err(RestoreError::SlotRetired),
            Entry::Vacant { .. } => {}
        }

        self.unlink_from_free_list(index);

        let slot = &mut self.slots[index as usize];
        slot.generation = id.generation();
        slot.entry = Entry::Occupied(value);
        self.len += 1;

        Ok(())
    }

    /// Retira um slot da lista de reuso, preservando o encadeamento dos demais.
    fn unlink_from_free_list(&mut self, index: u32) {
        let next_of = |slots: &Vec<Slot<T>>, at: u32| match slots[at as usize].entry {
            Entry::Vacant { next_free } => next_free,
            Entry::Occupied(_) | Entry::Retired => None,
        };

        if self.free_head == Some(index) {
            self.free_head = next_of(&self.slots, index);
            return;
        }

        let mut cursor = self.free_head;

        while let Some(current) = cursor {
            let next = next_of(&self.slots, current);

            if next == Some(index) {
                let after = next_of(&self.slots, index);

                if let Entry::Vacant { next_free } = &mut self.slots[current as usize].entry {
                    *next_free = after;
                }

                return;
            }

            cursor = next;
        }
    }

    /// Remove todos os valores, invalidando todos os identificadores emitidos.
    pub fn clear(&mut self) {
        let mut free_head = None;

        // Percorre do fim para o começo para que a lista de reuso resultante
        // fique em ordem crescente de índice.
        for index in (0..self.slots.len()).rev() {
            let slot = &mut self.slots[index];

            if matches!(slot.entry, Entry::Retired) {
                continue;
            }

            let occupied = matches!(slot.entry, Entry::Occupied(_));
            let next_generation = if occupied {
                slot.generation.checked_add(1)
            } else {
                Some(slot.generation)
            };

            match next_generation {
                Some(generation) => {
                    slot.generation = generation;
                    slot.entry = Entry::Vacant {
                        next_free: free_head,
                    };
                    free_head = Some(u32::try_from(index).expect("índice excede u32::MAX"));
                }
                None => slot.entry = Entry::Retired,
            }
        }

        self.free_head = free_head;
        self.len = 0;
    }

    /// Itera sobre os valores vivos em ordem crescente de índice.
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| match &slot.entry {
                Entry::Occupied(value) => {
                    let index = u32::try_from(index).expect("índice excede u32::MAX");
                    Some((EntityId::new(index, slot.generation), value))
                }
                Entry::Vacant { .. } | Entry::Retired => None,
            })
    }

    /// Versão mutável de [`Arena::iter`].
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(index, slot)| {
                let generation = slot.generation;
                match &mut slot.entry {
                    Entry::Occupied(value) => {
                        let index = u32::try_from(index).expect("índice excede u32::MAX");
                        Some((EntityId::new(index, generation), value))
                    }
                    Entry::Vacant { .. } | Entry::Retired => None,
                }
            })
    }

    /// Itera sobre os identificadores vivos em ordem crescente de índice.
    pub fn ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.iter().map(|(id, _)| id)
    }

    /// Resolve `id` para o slot correspondente, checando a geração.
    fn resolve(&self, id: EntityId) -> Option<&Slot<T>> {
        let slot = self.slots.get(id.index() as usize)?;
        (slot.generation == id.generation()).then_some(slot)
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_nova_esta_vazia() {
        let arena: Arena<i32> = Arena::new();

        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);
        assert_eq!(arena.iter().count(), 0);
    }

    #[test]
    fn insere_e_recupera_valor() {
        let mut arena = Arena::new();
        let id = arena.insert(42);

        assert_eq!(arena.get(id), Some(&42));
        assert!(arena.contains(id));
        assert_eq!(arena.len(), 1);
        assert!(!arena.is_empty());
    }

    #[test]
    fn insercoes_produzem_identificadores_distintos() {
        let mut arena = Arena::new();
        let primeiro = arena.insert("a");
        let segundo = arena.insert("b");

        assert_ne!(primeiro, segundo);
        assert_eq!(arena.get(primeiro), Some(&"a"));
        assert_eq!(arena.get(segundo), Some(&"b"));
    }

    #[test]
    fn get_mut_permite_alterar_valor() {
        let mut arena = Arena::new();
        let id = arena.insert(1);

        *arena
            .get_mut(id)
            .expect("valor recém-inserido deve existir") = 2;

        assert_eq!(arena.get(id), Some(&2));
    }

    #[test]
    fn remove_devolve_o_valor_e_esvazia_o_slot() {
        let mut arena = Arena::new();
        let id = arena.insert(String::from("linha"));

        assert_eq!(arena.remove(id), Some(String::from("linha")));
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn identificador_obsoleto_e_rejeitado_apos_remocao() {
        let mut arena = Arena::new();
        let id = arena.insert(7);
        arena.remove(id);

        assert_eq!(arena.get(id), None);
        assert_eq!(arena.get_mut(id), None);
        assert!(!arena.contains(id));
    }

    #[test]
    fn remover_duas_vezes_devolve_none_na_segunda() {
        let mut arena = Arena::new();
        let id = arena.insert(7);

        assert_eq!(arena.remove(id), Some(7));
        assert_eq!(arena.remove(id), None);
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn slot_e_reutilizado_apos_remocao() {
        let mut arena = Arena::new();
        let antigo = arena.insert("antigo");
        arena.remove(antigo);
        let novo = arena.insert("novo");

        assert_eq!(arena.len(), 1);
        assert_eq!(
            arena.iter().count(),
            1,
            "o slot deve ter sido reaproveitado, não duplicado"
        );
        assert_eq!(arena.get(novo), Some(&"novo"));
    }

    #[test]
    fn identificador_obsoleto_nao_alcanca_o_valor_que_ocupou_o_slot() {
        let mut arena = Arena::new();
        let antigo = arena.insert("antigo");
        arena.remove(antigo);
        let novo = arena.insert("novo");

        // Esta é a razão de existir da geração: sem ela, `antigo` devolveria
        // "novo" silenciosamente.
        assert_ne!(antigo, novo);
        assert_eq!(arena.get(antigo), None);
        assert_eq!(arena.get(novo), Some(&"novo"));
    }

    #[test]
    fn reuso_segue_ordem_crescente_de_indice() {
        let mut arena = Arena::new();
        let ids: Vec<_> = (0..4).map(|value| arena.insert(value)).collect();

        arena.remove(ids[3]);
        arena.remove(ids[1]);

        // A lista de reuso deve entregar o menor índice livre primeiro.
        let reutilizado = arena.insert(10);
        assert_eq!(arena.get(ids[0]), Some(&0));
        assert_eq!(arena.get(reutilizado), Some(&10));
        assert_eq!(arena.len(), 3);
    }

    #[test]
    fn iteracao_e_deterministica_por_indice() {
        let mut arena = Arena::new();
        let ids: Vec<_> = ["a", "b", "c"].map(|value| arena.insert(value)).into();
        arena.remove(ids[1]);

        let observado: Vec<_> = arena.iter().map(|(id, value)| (id, *value)).collect();

        assert_eq!(observado, vec![(ids[0], "a"), (ids[2], "c")]);
        assert_eq!(arena.ids().collect::<Vec<_>>(), vec![ids[0], ids[2]]);
    }

    #[test]
    fn iter_mut_permite_alterar_todos_os_valores() {
        let mut arena = Arena::new();
        arena.insert(1);
        arena.insert(2);

        for (_, value) in arena.iter_mut() {
            *value *= 10;
        }

        assert_eq!(
            arena.iter().map(|(_, v)| *v).collect::<Vec<_>>(),
            vec![10, 20]
        );
    }

    #[test]
    fn clear_invalida_todos_os_identificadores() {
        let mut arena = Arena::new();
        let primeiro = arena.insert("a");
        let segundo = arena.insert("b");

        arena.clear();

        assert!(arena.is_empty());
        assert_eq!(arena.get(primeiro), None);
        assert_eq!(arena.get(segundo), None);
        assert_eq!(arena.iter().count(), 0);
    }

    #[test]
    fn insercao_apos_clear_nao_reemite_identificador_antigo() {
        let mut arena = Arena::new();
        let antigo = arena.insert("a");
        arena.clear();
        let novo = arena.insert("b");

        assert_ne!(antigo, novo);
        assert_eq!(arena.get(antigo), None);
        assert_eq!(arena.get(novo), Some(&"b"));
    }

    #[test]
    fn identificador_de_indice_inexistente_e_rejeitado() {
        let arena: Arena<i32> = Arena::new();
        let forjado = EntityId::new(99, FIRST_GENERATION);

        assert_eq!(arena.get(forjado), None);
        assert!(!arena.contains(forjado));
    }

    #[test]
    fn geracao_esgotada_aposenta_o_slot_em_vez_de_reemitir_id() {
        let mut arena = Arena::new();
        let id = arena.insert("a");

        // Força a geração ao limite para exercitar o caminho de esgotamento sem
        // precisar de bilhões de remoções.
        arena.slots[id.index() as usize].generation = NonZeroU32::MAX;
        let no_limite = EntityId::new(id.index(), NonZeroU32::MAX);

        assert_eq!(arena.remove(no_limite), Some("a"));
        assert!(matches!(arena.slots[0].entry, Entry::Retired));
        assert_eq!(
            arena.free_head, None,
            "slot aposentado não pode voltar à lista de reuso"
        );

        // A próxima inserção precisa de um slot novo, não do aposentado.
        let novo = arena.insert("b");
        assert_ne!(novo, no_limite);
        assert_eq!(arena.get(no_limite), None);
        assert_eq!(arena.get(novo), Some(&"b"));
    }

    #[test]
    fn clear_preserva_slot_aposentado() {
        let mut arena = Arena::new();
        let id = arena.insert("a");
        arena.slots[id.index() as usize].generation = NonZeroU32::MAX;
        arena.remove(EntityId::new(id.index(), NonZeroU32::MAX));

        arena.clear();

        assert!(matches!(arena.slots[0].entry, Entry::Retired));
        assert_eq!(arena.free_head, None);
    }

    #[test]
    fn insert_at_restaura_no_mesmo_identificador() {
        let mut arena = Arena::new();
        let id = arena.insert("a");
        arena.remove(id);

        arena.insert_at(id, "a").expect("slot vago");

        assert_eq!(
            arena.get(id),
            Some(&"a"),
            "o identificador antigo tem de voltar a resolver"
        );
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn insert_at_recusa_slot_ocupado() {
        let mut arena = Arena::new();
        let id = arena.insert("a");

        assert_eq!(arena.insert_at(id, "b"), Err(RestoreError::SlotOccupied));
        assert_eq!(arena.get(id), Some(&"a"), "o valor vivo é preservado");
    }

    #[test]
    fn insert_at_recusa_slot_inexistente() {
        let mut arena: Arena<&str> = Arena::new();

        assert_eq!(
            arena.insert_at(EntityId::new(3, FIRST_GENERATION), "a"),
            Err(RestoreError::UnknownSlot)
        );
    }

    #[test]
    fn insert_at_recusa_slot_aposentado() {
        let mut arena = Arena::new();
        let id = arena.insert("a");
        arena.slots[id.index() as usize].generation = NonZeroU32::MAX;
        let no_limite = EntityId::new(id.index(), NonZeroU32::MAX);
        arena.remove(no_limite);

        assert_eq!(
            arena.insert_at(no_limite, "a"),
            Err(RestoreError::SlotRetired)
        );
    }

    #[test]
    fn insert_at_retira_o_slot_da_lista_de_reuso() {
        let mut arena = Arena::new();
        let ids: Vec<_> = (0..3).map(|value| arena.insert(value)).collect();
        arena.remove(ids[0]);
        arena.remove(ids[1]);

        // Restaura o do meio da lista de reuso.
        arena.insert_at(ids[0], 0).expect("slot vago");

        // Se o slot restaurado tivesse ficado na lista, esta inserção o
        // sobrescreveria e o `unreachable!` de `insert` dispararia.
        let novo = arena.insert(99);

        assert_eq!(arena.get(ids[0]), Some(&0));
        assert_eq!(arena.get(novo), Some(&99));
        assert_eq!(arena.len(), 3);
    }

    #[test]
    fn insert_at_no_inicio_da_lista_de_reuso() {
        let mut arena = Arena::new();
        let ids: Vec<_> = (0..2).map(|value| arena.insert(value)).collect();
        arena.remove(ids[1]);
        arena.remove(ids[0]);

        // `ids[0]` está na cabeça da lista de reuso.
        arena.insert_at(ids[0], 0).expect("slot vago");
        let novo = arena.insert(99);

        assert_eq!(arena.get(ids[0]), Some(&0));
        assert_eq!(arena.get(novo), Some(&99));
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn ciclo_remover_restaurar_preserva_o_conteudo_vivo() {
        let mut arena = Arena::new();
        let ids: Vec<_> = (0..3).map(|value| arena.insert(value)).collect();
        let antes: Vec<_> = arena.iter().map(|(id, value)| (id, *value)).collect();

        let removido = arena.remove(ids[1]).expect("existe");
        arena.insert_at(ids[1], removido).expect("slot vago");

        let depois: Vec<_> = arena.iter().map(|(id, value)| (id, *value)).collect();
        assert_eq!(antes, depois);
    }

    #[test]
    fn with_capacity_nao_cria_valores() {
        let arena: Arena<i32> = Arena::with_capacity(16);

        assert!(arena.is_empty());
        assert_eq!(arena.iter().count(), 0);
    }

    #[test]
    fn default_equivale_a_new() {
        let arena: Arena<i32> = Arena::default();

        assert!(arena.is_empty());
    }
}
