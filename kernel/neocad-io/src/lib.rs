// Caminho relativo: kernel/neocad-io/src/lib.rs
//! \file kernel/neocad-io/src/lib.rs
//! \brief Leitura e escrita de formatos CAD para o kernel do NeoCAD.
//! \author Iago Leal
//! \date 2026-08-06
//!
//! Responsabilidade: serialização e desserialização entre o modelo do kernel e
//! os formatos de arquivo — DXF em K2, DWG em K6, STEP/IGES em K9.
//!
//! Permanece vazia durante K1: em K1 o modelo é populado a partir do documento
//! já aberto pelo upstream, no frontend (MT-K1-14).
//!
//! # Fronteira de licenciamento
//!
//! Esta é a **única** crate do kernel autorizada a depender de bibliotecas
//! copyleft, conforme o ADR 0003. As crates de geometria, topologia, modelo e
//! transações permanecem livres de copyleft para que o kernel possa ser
//! licenciado de forma independente do aplicativo e reaproveitado em outros
//! projetos.
