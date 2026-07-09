use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupRecord {
    pub tweak_id: String,
    pub original_value: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BackupState {
    /// Mapa onde a chave é o `tweak_id` e o valor é o registro de backup
    pub records: HashMap<String, BackupRecord>,
}

pub struct BackupManager {
    backup_file_path: PathBuf,
}

impl BackupManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            backup_file_path: path.as_ref().to_path_buf(),
        }
    }

    pub fn load_state(&self) -> Result<BackupState> {
        if !self.backup_file_path.exists() {
            return Ok(BackupState::default());
        }

        let content = fs::read_to_string(&self.backup_file_path)
            .context("Falha ao ler o arquivo de backup")?;
        
        let state: BackupState = serde_json::from_str(&content)
            .context("Falha ao desserializar o estado de backup")?;
            
        Ok(state)
    }

    pub fn save_state(&self, state: &BackupState) -> Result<()> {
        if let Some(parent) = self.backup_file_path.parent() {
            fs::create_dir_all(parent).context("Falha ao criar diretório de backup")?;
        }

        let content = serde_json::to_string_pretty(state)
            .context("Falha ao serializar estado de backup")?;
            
        fs::write(&self.backup_file_path, content)
            .context("Falha ao escrever no arquivo de backup")?;
            
        Ok(())
    }

    pub fn record_backup(&self, tweak_id: &str, original_value: &str) -> Result<()> {
        let mut state = self.load_state()?;
        
        // Só gravamos o backup se não houver um registro anterior, para proteger o 
        // valor original "real" antes de qualquer modificação repetida.
        if !state.records.contains_key(tweak_id) {
            state.records.insert(
                tweak_id.to_string(),
                BackupRecord {
                    tweak_id: tweak_id.to_string(),
                    original_value: original_value.to_string(),
                    timestamp: Utc::now(),
                },
            );
            self.save_state(&state)?;
        }
        
        Ok(())
    }

    pub fn remove_backup(&self, tweak_id: &str) -> Result<()> {
        let mut state = self.load_state()?;
        if state.records.remove(tweak_id).is_some() {
            self.save_state(&state)?;
        }
        Ok(())
    }
}
