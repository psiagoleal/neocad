// Caminho relativo: src-tauri/src/lib.rs
//! \file src-tauri/src/lib.rs
//! \brief Inicialização do backend Tauri do NeoCAD.
//! \author Iago Leal
//! \date 2026-05-12

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running NeoCAD tauri application");
}
