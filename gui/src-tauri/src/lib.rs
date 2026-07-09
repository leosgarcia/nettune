use core::backup::BackupManager;
use core::get_all_tweaks;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize)]
pub struct TweakInfo {
    id: String,
    name: String,
    description: String,
    active: bool,
}

fn get_backup_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".nettune");
    path.push("backup.json");
    path
}

#[tauri::command]
fn get_tweaks() -> Result<Vec<TweakInfo>, String> {
    let tweaks = get_all_tweaks();
    let backup_manager = BackupManager::new(get_backup_path());
    let state = backup_manager.load_state().unwrap_or_default();
    
    let mut info_list = Vec::new();
    for tweak in tweaks {
        let is_active = state.records.contains_key(tweak.id());
        info_list.push(TweakInfo {
            id: tweak.id().to_string(),
            name: tweak.name().to_string(),
            description: tweak.description().to_string(),
            active: is_active,
        });
    }
    
    Ok(info_list)
}

#[tauri::command]
fn apply_tweak(id: String, dry_run: bool) -> Result<String, String> {
    let tweaks = get_all_tweaks();
    let backup_manager = BackupManager::new(get_backup_path());
    
    if let Some(tweak) = tweaks.iter().find(|t| t.id() == id) {
        let original = tweak.read_current_value().map_err(|e| e.to_string())?;
        if let Some(orig) = original {
            if !dry_run {
                backup_manager.record_backup(tweak.id(), &orig).map_err(|e| e.to_string())?;
                tweak.apply(None).map_err(|e| e.to_string())?;
            }
            let reboot_msg = if tweak.requires_reboot() && !dry_run {
                " (Requer reinicialização)"
            } else {
                ""
            };
            return Ok(format!("Aplicado com sucesso.{}", reboot_msg));
        }
        return Err("Não foi possível ler o estado original do SO.".into());
    }
    
    Err("Ajuste não encontrado".into())
}

#[tauri::command]
fn revert_tweak(id: String, dry_run: bool) -> Result<String, String> {
    let tweaks = get_all_tweaks();
    let backup_manager = BackupManager::new(get_backup_path());
    let state = backup_manager.load_state().map_err(|e| e.to_string())?;
    
    if let Some(tweak) = tweaks.iter().find(|t| t.id() == id) {
        if let Some(record) = state.records.get(tweak.id()) {
            if !dry_run {
                tweak.revert(&record.original_value).map_err(|e| e.to_string())?;
                backup_manager.remove_backup(tweak.id()).map_err(|e| e.to_string())?;
            }
            return Ok("Revertido com sucesso.".into());
        }
        return Err("Nenhum backup encontrado. O ajuste não estava ativo.".into());
    }
    
    Err("Ajuste não encontrado".into())
}

#[tauri::command]
fn revert_all(dry_run: bool) -> Result<String, String> {
    let tweaks = get_all_tweaks();
    let backup_manager = BackupManager::new(get_backup_path());
    
    let mut count = 0;
    for tweak in tweaks {
        let state = backup_manager.load_state().unwrap_or_default();
        if let Some(record) = state.records.get(tweak.id()) {
            if !dry_run {
                if tweak.revert(&record.original_value).is_ok() {
                    let _ = backup_manager.remove_backup(tweak.id());
                    count += 1;
                }
            } else {
                count += 1;
            }
        }
    }
    
    Ok(format!("{} ajustes revertidos.", count))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_tweaks,
            apply_tweak,
            revert_tweak,
            revert_all
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
