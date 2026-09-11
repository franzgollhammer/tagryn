use tauri::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle,
};
pub fn install(app: &AppHandle, german: bool) -> tauri::Result<()> {
    let t = |de: &'static str, en: &'static str| if german { de } else { en };
    let menu = Menu::with_items(
        app,
        &[
            &Submenu::with_items(
                app,
                "Tagryn",
                true,
                &[
                    &PredefinedMenuItem::about(
                        app,
                        Some(t("Über Tagryn", "About Tagryn")),
                        Some(AboutMetadata {
                            name: Some("Tagryn".into()),
                            version: Some(env!("CARGO_PKG_VERSION").into()),
                            ..Default::default()
                        }),
                    )?,
                    &MenuItem::with_id(
                        app,
                        "settings",
                        t("Einstellungen …", "Settings …"),
                        true,
                        Some("CmdOrCtrl+,"),
                    )?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, None)?,
                    &PredefinedMenuItem::hide_others(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(
                        app,
                        "quit",
                        t("Tagryn beenden", "Quit Tagryn"),
                        true,
                        Some("CmdOrCtrl+Q"),
                    )?,
                ],
            )?,
            &Submenu::with_items(
                app,
                t("Datei", "File"),
                true,
                &[
                    &MenuItem::with_id(
                        app,
                        "open",
                        t("Dateien öffnen …", "Open files …"),
                        true,
                        Some("CmdOrCtrl+O"),
                    )?,
                    &MenuItem::with_id(
                        app,
                        "folder",
                        t("Ordner öffnen …", "Open folder …"),
                        true,
                        Some("CmdOrCtrl+Shift+O"),
                    )?,
                    &MenuItem::with_id(
                        app,
                        "save",
                        t("Änderungen prüfen …", "Review changes …"),
                        true,
                        Some("CmdOrCtrl+S"),
                    )?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::close_window(app, None)?,
                ],
            )?,
            &Submenu::with_items(
                app,
                t("Bearbeiten", "Edit"),
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?,
            &Submenu::with_items(
                app,
                t("Ansicht", "View"),
                true,
                &[
                    &MenuItem::with_id(
                        app,
                        "palette",
                        t("Befehlspalette …", "Command palette …"),
                        true,
                        Some("CmdOrCtrl+K"),
                    )?,
                    &MenuItem::with_id(
                        app,
                        "compare",
                        t("Dateien vergleichen …", "Compare files …"),
                        true,
                        Some("CmdOrCtrl+D"),
                    )?,
                    &PredefinedMenuItem::fullscreen(app, None)?,
                ],
            )?,
            &Submenu::with_items(
                app,
                t("Fenster", "Window"),
                true,
                &[
                    &PredefinedMenuItem::minimize(app, None)?,
                    &PredefinedMenuItem::maximize(app, None)?,
                ],
            )?,
            &Submenu::with_items(
                app,
                t("Hilfe", "Help"),
                true,
                &[&MenuItem::with_id(
                    app,
                    "formats",
                    t("Formate und Grenzen …", "Formats and limits …"),
                    true,
                    None::<&str>,
                )?],
            )?,
        ],
    )?;
    app.set_menu(menu)?;
    Ok(())
}
