// Caminho relativo: kernel/neocad-model/src/id.rs
//! \file kernel/neocad-model/src/id.rs
//! \brief Identificador opaco e estável de entidade.
//! \author Iago Leal
//! \date 2026-08-07

use core::fmt;
use core::num::NonZeroU32;

/// Identificador opaco de uma entidade dentro de um documento.
///
/// Internamente combina o índice do slot na arena com a **geração** desse slot.
/// A geração é incrementada a cada remoção, de modo que um identificador obtido
/// antes de uma remoção deixa de resolver — em vez de passar a apontar,
/// silenciosamente, para a entidade que ocupou o slot depois.
///
/// O tipo é deliberadamente opaco: nem o índice nem a geração são públicos. Isso
/// permite mudar a representação sem quebrar quem depende do kernel.
///
/// # Escopo de validade
///
/// Um identificador só tem significado dentro da arena que o emitiu. Compará-lo
/// com um identificador de outra arena, ou usá-lo para consultar outra arena, é
/// um erro de uso — a consulta pode até resolver, mas para uma entidade
/// arbitrária.
///
/// # Ordenação
///
/// A ordem é por índice e depois por geração, o que dá iteração e ordenação
/// determinísticas. Não corresponde à ordem de criação das entidades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId {
    index: u32,
    generation: NonZeroU32,
}

impl EntityId {
    /// Constrói um identificador a partir de índice e geração.
    ///
    /// Restrito à crate: identificadores só devem ser emitidos por uma arena.
    pub(crate) const fn new(index: u32, generation: NonZeroU32) -> Self {
        Self { index, generation }
    }

    /// Índice do slot que este identificador referencia.
    pub(crate) const fn index(self) -> u32 {
        self.index
    }

    /// Geração do slot no momento em que este identificador foi emitido.
    pub(crate) const fn generation(self) -> NonZeroU32 {
        self.generation
    }

    /// Representação escalar opaca, adequada a transporte.
    ///
    /// Existe porque o identificador precisa atravessar fronteiras que não
    /// conhecem tipos Rust — a ponte WebAssembly com o frontend e, mais adiante,
    /// a serialização de arquivo. Segue opaco: o número não carrega significado
    /// além de identificar, e só volta a ser um identificador por
    /// [`EntityId::from_bits`].
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        ((self.index as u64) << 32) | self.generation.get() as u64
    }

    /// Reconstrói um identificador a partir de [`EntityId::to_bits`].
    ///
    /// Devolve `None` se os bits não puderem corresponder a um identificador:
    /// geração zero nunca é emitida.
    #[must_use]
    pub const fn from_bits(bits: u64) -> Option<Self> {
        let index = (bits >> 32) as u32;

        match NonZeroU32::new(bits as u32) {
            Some(generation) => Some(Self { index, generation }),
            None => None,
        }
    }
}

impl fmt::Display for EntityId {
    /// Formata como `#índice.geração`, por exemplo `#12.3`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "#{}.{}", self.index, self.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: NonZeroU32 = NonZeroU32::MIN;

    fn generation(value: u32) -> NonZeroU32 {
        NonZeroU32::new(value).expect("geração de teste deve ser não-zero")
    }

    #[test]
    fn identificadores_iguais_quando_indice_e_geracao_coincidem() {
        assert_eq!(EntityId::new(7, FIRST), EntityId::new(7, FIRST));
    }

    #[test]
    fn geracao_diferente_produz_identificador_diferente() {
        assert_ne!(EntityId::new(7, FIRST), EntityId::new(7, generation(2)));
    }

    #[test]
    fn ordena_por_indice_e_depois_por_geracao() {
        let mut ids = [
            EntityId::new(2, FIRST),
            EntityId::new(1, generation(9)),
            EntityId::new(1, FIRST),
        ];
        ids.sort();

        assert_eq!(
            ids,
            [
                EntityId::new(1, FIRST),
                EntityId::new(1, generation(9)),
                EntityId::new(2, FIRST),
            ]
        );
    }

    #[test]
    fn bits_fazem_ida_e_volta() {
        let id = EntityId::new(12, generation(3));

        assert_eq!(EntityId::from_bits(id.to_bits()), Some(id));
    }

    #[test]
    fn bits_distinguem_indice_de_geracao() {
        assert_ne!(
            EntityId::new(1, generation(2)).to_bits(),
            EntityId::new(2, generation(1)).to_bits()
        );
    }

    #[test]
    fn bits_com_geracao_zero_sao_recusados() {
        // Geração zero nunca é emitida: um valor assim só pode vir de fora.
        assert_eq!(EntityId::from_bits(0), None);
        assert_eq!(EntityId::from_bits(7 << 32), None);
    }

    #[test]
    fn bits_preservam_os_extremos() {
        let extremo = EntityId::new(u32::MAX, NonZeroU32::MAX);

        assert_eq!(EntityId::from_bits(extremo.to_bits()), Some(extremo));
    }

    #[test]
    fn exibicao_mostra_indice_e_geracao() {
        assert_eq!(EntityId::new(12, generation(3)).to_string(), "#12.3");
    }
}
