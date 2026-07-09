use crate::tweaks::{OsPlatform, Tweak};
use anyhow::Result;
use std::cell::RefCell;

pub struct MockTweak {
    id: &'static str,
    name: &'static str,
    // RefCell para permitir mutabilidade em testes sem precisar mutar o Tweak (que é lido por &self)
    state: RefCell<String>,
}

impl MockTweak {
    #[must_use]
    pub fn new(id: &'static str, name: &'static str, initial_state: &str) -> Self {
        Self {
            id,
            name,
            state: RefCell::new(initial_state.to_string()),
        }
    }
}

impl Tweak for MockTweak {
    fn id(&self) -> &'static str {
        self.id
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        "Ajuste falso para testes unitários"
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::Windows, OsPlatform::Linux, OsPlatform::MacOS]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        Ok(Some(self.state.borrow().clone()))
    }

    fn apply(&self, option_value: Option<&str>) -> Result<()> {
        let val = option_value.unwrap_or("applied");
        *self.state.borrow_mut() = val.to_string();
        Ok(())
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        *self.state.borrow_mut() = original_value.to_string();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::BackupManager;
    use tempfile::tempdir;

    #[test]
    fn test_mock_tweak_apply_and_revert() {
        let tweak = MockTweak::new("test_tweak", "Test Tweak", "default_val");
        
        assert_eq!(tweak.read_current_value().unwrap().unwrap(), "default_val");
        
        tweak.apply(Some("new_val")).unwrap();
        assert_eq!(tweak.read_current_value().unwrap().unwrap(), "new_val");
        
        tweak.revert("default_val").unwrap();
        assert_eq!(tweak.read_current_value().unwrap().unwrap(), "default_val");
    }

    #[test]
    fn test_backup_manager() {
        let dir = tempdir().unwrap();
        let backup_file = dir.path().join("backup.json");
        let manager = BackupManager::new(&backup_file);

        // Record a backup
        manager.record_backup("tweak1", "orig1").unwrap();
        
        let state = manager.load_state().unwrap();
        assert!(state.records.contains_key("tweak1"));
        assert_eq!(state.records.get("tweak1").unwrap().original_value, "orig1");

        // Try recording again, should not overwrite
        manager.record_backup("tweak1", "changed_orig").unwrap();
        let state2 = manager.load_state().unwrap();
        assert_eq!(state2.records.get("tweak1").unwrap().original_value, "orig1"); // Ainda deve ser orig1

        // Remove backup
        manager.remove_backup("tweak1").unwrap();
        let state3 = manager.load_state().unwrap();
        assert!(!state3.records.contains_key("tweak1"));
    }
}
