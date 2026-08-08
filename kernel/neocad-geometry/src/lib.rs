// Caminho relativo: kernel/neocad-geometry/src/lib.rs
//! \file kernel/neocad-geometry/src/lib.rs
//! \brief Camada de geometria do kernel CAD do NeoCAD.
//! \author Iago Leal
//! \date 2026-08-06
//!
//! Responsabilidade: primitivas geométricas, curvas e superfícies, sem qualquer
//! noção de documento, entidade de desenho ou topologia.
//!
//! É a crate de mais baixo nível do kernel e não depende de nenhuma outra.
//! Recebe conteúdo a partir de MT-K1-05 (ponto e caixa envolvente) e é ampliada
//! em K3 (operações 2D) e K7 (NURBS).
//!
//! Conforme o ADR 0003, esta crate não conhece Tauri, Svelte, DOM nem tipos do
//! NeoCAD, e não pode receber dependências copyleft.

mod aabb;
mod point;

pub use aabb::Aabb;
pub use point::{point_on_circle, Point2};
