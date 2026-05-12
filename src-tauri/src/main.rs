// Caminho relativo: src-tauri/src/main.rs
//! \file src-tauri/src/main.rs
//! \brief Ponto de entrada nativo do aplicativo NeoCAD.
//! \author Iago Leal
//! \date 2026-05-12

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    neocad_lib::run();
}
