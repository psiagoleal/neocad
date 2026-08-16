// Caminho relativo: kernel/neocad-io/src/lib.rs
//! \file kernel/neocad-io/src/lib.rs
//! \brief Leitura e escrita de formatos CAD para o kernel do NeoCAD.
//! \author Iago Leal
//! \date 2026-08-06
//!
//! Responsabilidade: serialização e desserialização entre o modelo do kernel e
//! os formatos de arquivo — DXF em K2, DWG em K6, STEP/IGES em K9.
//!
//! A leitura DXF própria está sendo construída em K2, de baixo para cima: o
//! fluxo de pares código/valor primeiro, depois seções, tabelas e entidades. O
//! plano está em `docs/tickets/k2-dxf-nativo.md`.
//!
//! # Fronteira de licenciamento
//!
//! Esta é a **única** crate do kernel autorizada a depender de bibliotecas
//! copyleft, conforme o ADR 0003. As crates de geometria, topologia, modelo e
//! transações permanecem livres de copyleft para que o kernel possa ser
//! licenciado de forma independente do aplicativo e reaproveitado em outros
//! projetos.

mod dxf;

pub use dxf::{
    formatar_real, pairs, read_blocks, read_dxf, read_entities, read_layer_table, sections,
    write_dxf, BlockDefinition, BlocksReading, DxfPair, DxfPairError, DxfPairs, DxfReading,
    DxfReport, DxfSectionError, DxfValue, EntitiesReading, EntitySpace, LayerTableReading,
    ReadEntity, RejectedEntity, RejectedLayer, Section, SectionKind, Sections, ACAD_VERSION,
    DEFAULT_PAPER_SPACE,
};
