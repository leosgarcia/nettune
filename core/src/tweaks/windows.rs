#![allow(clippy::module_name_repetitions)]

#[cfg(target_os = "windows")]
use crate::tweaks::{OsPlatform, Tweak};
#[cfg(target_os = "windows")]
use anyhow::{Context, Result};
#[cfg(target_os = "windows")]
use winreg::{enums::HKEY_LOCAL_MACHINE, enums::KEY_READ, enums::KEY_WRITE, RegKey};
#[cfg(target_os = "windows")]
use std::process::Command;

#[cfg(target_os = "windows")]
pub struct NetworkThrottlingTweak;

#[cfg(target_os = "windows")]
impl Tweak for NetworkThrottlingTweak {
    fn id(&self) -> &'static str {
        "win_network_throttling"
    }

    fn name(&self) -> &'static str {
        "Desabilitar Network Throttling"
    }

    fn description(&self) -> &'static str {
        "Desativa o limite de taxa de pacotes multimidia do Windows, o que pode reduzir a latência em jogos."
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::Windows]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        match hklm.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile", KEY_READ) {
            Ok(key) => {
                match key.get_value::<u32, _>("NetworkThrottlingIndex") {
                    Ok(val) => Ok(Some(val.to_string())),
                    Err(_) => Ok(None),
                }
            },
            Err(_) => Ok(None)
        }
    }

    fn apply(&self, _option_value: Option<&str>) -> Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (key, _) = hklm.create_subkey_with_flags("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile", KEY_WRITE)
            .context("Falha ao abrir chave de registro. Execute como Administrador.")?;
        
        // 0xFFFFFFFF (4294967295) desabilita o throttling
        key.set_value("NetworkThrottlingIndex", &0xFFFFFFFF_u32)?;
        Ok(())
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        if let Ok(val) = original_value.parse::<u32>() {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let (key, _) = hklm.create_subkey_with_flags("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile", KEY_WRITE)
                .context("Falha ao abrir chave de registro. Execute como Administrador.")?;
            key.set_value("NetworkThrottlingIndex", &val)?;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub struct NagleAlgorithmTweak;

#[cfg(target_os = "windows")]
impl Tweak for NagleAlgorithmTweak {
    fn id(&self) -> &'static str {
        "win_nagle_algorithm"
    }

    fn name(&self) -> &'static str {
        "Desabilitar Nagle's Algorithm (TcpAckFrequency & TCPNoDelay)"
    }

    fn description(&self) -> &'static str {
        "Desabilita o algoritmo de Nagle em todas as interfaces de rede para enviar pacotes imediatamente, reduzindo o delay (MS)."
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::Windows]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        // Para simplificar, verificaremos apenas a primeira interface que tem IP atribuído.
        // Em um app real complexo, poderíamos iterar e retornar um JSON com os estados.
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let interfaces = hklm.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces", KEY_READ)?;
        
        for interface_name in interfaces.enum_keys().flatten() {
            if let Ok(iface_key) = interfaces.open_subkey_with_flags(&interface_name, KEY_READ) {
                // Checa se tem DHCP ou IP
                if iface_key.get_value::<String, _>("DhcpIPAddress").is_ok() || iface_key.get_value::<Vec<String>, _>("IPAddress").is_ok() {
                    let ack = iface_key.get_value::<u32, _>("TcpAckFrequency").unwrap_or(0);
                    let no_delay = iface_key.get_value::<u32, _>("TCPNoDelay").unwrap_or(0);
                    return Ok(Some(format!("{}:{}", ack, no_delay)));
                }
            }
        }
        Ok(None)
    }

    fn apply(&self, _option_value: Option<&str>) -> Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let interfaces = hklm.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces", KEY_READ)?;
        
        for interface_name in interfaces.enum_keys().flatten() {
            if let Ok(iface_key) = interfaces.open_subkey_with_flags(&interface_name, KEY_WRITE | KEY_READ) {
                if iface_key.get_value::<String, _>("DhcpIPAddress").is_ok() || iface_key.get_value::<Vec<String>, _>("IPAddress").is_ok() {
                    iface_key.set_value("TcpAckFrequency", &1_u32)?;
                    iface_key.set_value("TCPNoDelay", &1_u32)?;
                }
            }
        }
        Ok(())
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        // original_value no formato "ack:nodelay"
        let parts: Vec<&str> = original_value.split(':').collect();
        if parts.len() == 2 {
            let ack: u32 = parts[0].parse().unwrap_or(0);
            let no_delay: u32 = parts[1].parse().unwrap_or(0);

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let interfaces = hklm.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces", KEY_READ)?;
            
            for interface_name in interfaces.enum_keys().flatten() {
                if let Ok(iface_key) = interfaces.open_subkey_with_flags(&interface_name, KEY_WRITE | KEY_READ) {
                    if iface_key.get_value::<String, _>("DhcpIPAddress").is_ok() || iface_key.get_value::<Vec<String>, _>("IPAddress").is_ok() {
                        iface_key.set_value("TcpAckFrequency", &ack)?;
                        iface_key.set_value("TCPNoDelay", &no_delay)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn requires_reboot(&self) -> bool {
        true
    }
}

#[cfg(target_os = "windows")]
pub struct InterruptModerationTweak;

#[cfg(target_os = "windows")]
impl Tweak for InterruptModerationTweak {
    fn id(&self) -> &'static str {
        "win_interrupt_moderation"
    }

    fn name(&self) -> &'static str {
        "Interrupt Moderation (Placa de Rede)"
    }

    fn description(&self) -> &'static str {
        "Desabilita a Moderação de Interrupção. Reduz CPU overhead mas aumenta imediatamente a resposta da rede."
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::Windows]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-NetAdapterAdvancedProperty -DisplayName 'Interrupt Moderation' | Select-Object -ExpandProperty DisplayValue"])
            .output()?;

        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !val.is_empty() {
                return Ok(Some(val));
            }
        }
        Ok(None)
    }

    fn apply(&self, _option_value: Option<&str>) -> Result<()> {
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Set-NetAdapterAdvancedProperty -DisplayName 'Interrupt Moderation' -DisplayValue 'Disabled' -NoRestart"])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Falha ao alterar Interrupt Moderation via PowerShell. Execute como Administrador.");
        }
        Ok(())
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("Set-NetAdapterAdvancedProperty -DisplayName 'Interrupt Moderation' -DisplayValue '{}' -NoRestart", original_value)])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Falha ao reverter Interrupt Moderation via PowerShell.");
        }
        Ok(())
    }
}
