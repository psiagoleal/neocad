// Caminho relativo: kernel/neocad-io/src/dxf/mod.rs
//! \file kernel/neocad-io/src/dxf/mod.rs
//! \brief Leitura e escrita do formato DXF.
//! \author Iago Leal
//! \date 2026-08-11
//!
//! O DXF é lido em camadas, de baixo para cima: o fluxo de pares código/valor
//! ([`pairs`]), depois as seções, depois as tabelas e entidades. Cada camada só
//! conhece a anterior, o que permite testá-las isoladamente e trocar uma sem
//! mexer nas outras.
//!
//! Esta é a leitura **própria**, que substitui a do upstream. A motivação está
//! registrada em `docs/tickets/k2-dxf-nativo.md`: o parser upstream não lê
//! arquivos cuja seção `BLOCKS` contenha bloco com entidades.

mod pairs;
mod sections;
mod tables;

pub use pairs::{pairs, DxfPair, DxfPairError, DxfPairs, DxfValue};
pub use sections::{sections, DxfSectionError, Section, SectionKind, Sections};
pub use tables::{read_layer_table, LayerTableReading, RejectedLayer};
