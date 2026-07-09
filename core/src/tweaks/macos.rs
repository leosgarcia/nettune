#![allow(clippy::module_name_repetitions)]

#[cfg(target_os = "macos")]
use crate::tweaks::{OsPlatform, Tweak};
#[cfg(target_os = "macos")]
use anyhow::{Context, Result};
#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "macos")]
fn read_sysctl_mac(key: &str) -> Result<Option<String>> {
    let output = Command::new("sysctl").args(["-n", key]).output()?;
    if output.status.success() {
        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !val.is_empty() {
            return Ok(Some(val));
        }
    }
    Ok(None)
}

#[cfg(target_os = "macos")]
fn write_sysctl_mac(key: &str, value: &str) -> Result<()> {
    let status = Command::new("sysctl")
        .args(["-w", &format!("{}={}", key, value)])
        .status()
        .context("Falha ao executar sysctl.")?;
        
    if !status.success() {
        anyhow::bail!("Falha ao alterar {}. Você tem permissões de root/sudo?", key);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub struct DelayedAckTweak;

#[cfg(target_os = "macos")]
impl Tweak for DelayedAckTweak {
    fn id(&self) -> &'static str {
        "macos_delayed_ack"
    }

    fn name(&self) -> &'static str {
        "Desabilitar TCP Delayed ACK"
    }

    fn description(&self) -> &'static str {
        "Otimiza o tráfego interativo (jogos) respondendo imediatamente aos pacotes TCP recebidos, reduzindo a latência artificial."
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::MacOS]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        read_sysctl_mac("net.inet.tcp.delayed_ack")
    }

    fn apply(&self, _option_value: Option<&str>) -> Result<()> {
        // 0 desabilita o delayed ACK
        write_sysctl_mac("net.inet.tcp.delayed_ack", "0")
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        write_sysctl_mac("net.inet.tcp.delayed_ack", original_value)
    }
}

#[cfg(target_os = "macos")]
pub struct TcpWindowSizeTweak;

#[cfg(target_os = "macos")]
impl Tweak for TcpWindowSizeTweak {
    fn id(&self) -> &'static str {
        "macos_tcp_window_size"
    }

    fn name(&self) -> &'static str {
        "TCP Send/Recv Space (Window Size)"
    }

    fn description(&self) -> &'static str {
        "Aumenta o tamanho da janela do TCP. (Nota: O macOS moderno faz auto-tuning bem, o ganho de jitter aqui é marginal/análise)."
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::MacOS]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        let send = read_sysctl_mac("net.inet.tcp.sendspace")?.unwrap_or_else(|| "0".to_string());
        let recv = read_sysctl_mac("net.inet.tcp.recvspace")?.unwrap_or_else(|| "0".to_string());
        Ok(Some(format!("{}:{}", send, recv)))
    }

    fn apply(&self, _option_value: Option<&str>) -> Result<()> {
        // Valores generosos (ex: 524288 ou 1048576) podem prevenir drop de pacotes em conexões gigabit
        write_sysctl_mac("net.inet.tcp.sendspace", "524288")?;
        write_sysctl_mac("net.inet.tcp.recvspace", "524288")?;
        Ok(())
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        let parts: Vec<&str> = original_value.split(':').collect();
        if parts.len() == 2 {
            write_sysctl_mac("net.inet.tcp.sendspace", parts[0])?;
            write_sysctl_mac("net.inet.tcp.recvspace", parts[1])?;
        }
        Ok(())
    }
}
