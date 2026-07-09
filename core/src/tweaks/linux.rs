#![allow(clippy::module_name_repetitions)]

#[cfg(target_os = "linux")]
use crate::tweaks::{OsPlatform, Tweak};
#[cfg(target_os = "linux")]
use anyhow::{Context, Result};
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::process::Command;

#[cfg(target_os = "linux")]
fn read_sysctl(path: &str) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(content.trim().to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Erro ao ler {}: {}", path, e)),
    }
}

#[cfg(target_os = "linux")]
fn write_sysctl(path: &str, value: &str) -> Result<()> {
    fs::write(path, value.as_bytes())
        .with_context(|| format!("Falha ao escrever em {}. Você tem privilégios de root/sudo?", path))
}

#[cfg(target_os = "linux")]
pub struct TcpCongestionControlTweak;

#[cfg(target_os = "linux")]
impl Tweak for TcpCongestionControlTweak {
    fn id(&self) -> &'static str {
        "linux_tcp_bbr"
    }

    fn name(&self) -> &'static str {
        "TCP BBR Congestion Control"
    }

    fn description(&self) -> &'static str {
        "Altera o controle de congestionamento para BBR, que otimiza banda e minimiza latência em redes variadas."
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::Linux]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        read_sysctl("/proc/sys/net/ipv4/tcp_congestion_control")
    }

    fn apply(&self, _option_value: Option<&str>) -> Result<()> {
        write_sysctl("/proc/sys/net/core/default_qdisc", "fq")?;
        write_sysctl("/proc/sys/net/ipv4/tcp_congestion_control", "bbr")?;
        Ok(())
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        // Assume pfifo_fast or fq_codel as default for qdisc if reverting, but let's just revert congestion control
        write_sysctl("/proc/sys/net/ipv4/tcp_congestion_control", original_value)?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub struct NetdevMaxBacklogTweak;

#[cfg(target_os = "linux")]
impl Tweak for NetdevMaxBacklogTweak {
    fn id(&self) -> &'static str {
        "linux_netdev_max_backlog"
    }

    fn name(&self) -> &'static str {
        "Netdev Max Backlog"
    }

    fn description(&self) -> &'static str {
        "Aumenta a fila de recepção do kernel para lidar melhor com picos de pacotes sem descartá-los (jitter)."
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::Linux]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        read_sysctl("/proc/sys/net/core/netdev_max_backlog")
    }

    fn apply(&self, option_value: Option<&str>) -> Result<()> {
        let val = option_value.unwrap_or("5000"); // 5000 is a good tuned value
        write_sysctl("/proc/sys/net/core/netdev_max_backlog", val)?;
        Ok(())
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        write_sysctl("/proc/sys/net/core/netdev_max_backlog", original_value)?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub struct TcpLowLatencyTweak;

#[cfg(target_os = "linux")]
impl Tweak for TcpLowLatencyTweak {
    fn id(&self) -> &'static str {
        "linux_tcp_low_latency"
    }

    fn name(&self) -> &'static str {
        "TCP Low Latency"
    }

    fn description(&self) -> &'static str {
        "Instrui a pilha TCP a priorizar baixa latência sobre throughput bruto (desativa processamento em batch)."
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::Linux]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        read_sysctl("/proc/sys/net/ipv4/tcp_low_latency")
    }

    fn apply(&self, _option_value: Option<&str>) -> Result<()> {
        write_sysctl("/proc/sys/net/ipv4/tcp_low_latency", "1")?;
        Ok(())
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        write_sysctl("/proc/sys/net/ipv4/tcp_low_latency", original_value)?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub struct InterruptCoalescingTweak;

#[cfg(target_os = "linux")]
impl Tweak for InterruptCoalescingTweak {
    fn id(&self) -> &'static str {
        "linux_interrupt_coalescing"
    }

    fn name(&self) -> &'static str {
        "Interrupt Coalescing (ethtool)"
    }

    fn description(&self) -> &'static str {
        "Desativa o rx-usecs usando ethtool na interface principal. Processa pacotes IMEDIATAMENTE."
    }

    fn supported_os(&self) -> Vec<OsPlatform> {
        vec![OsPlatform::Linux]
    }

    fn read_current_value(&self) -> Result<Option<String>> {
        let default_iface = self.get_default_interface()?;
        if default_iface.is_empty() {
            return Ok(None);
        }

        let output = Command::new("ethtool").arg("-c").arg(&default_iface).output()?;
        if !output.status.success() {
            return Ok(None); // Ethtool might not be installed or interface doesn't support it
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.trim().starts_with("rx-usecs:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    return Ok(Some(parts[1].trim().to_string()));
                }
            }
        }
        Ok(None)
    }

    fn apply(&self, _option_value: Option<&str>) -> Result<()> {
        let default_iface = self.get_default_interface()?;
        if default_iface.is_empty() {
            anyhow::bail!("Nenhuma interface padrão encontrada para aplicar ethtool.");
        }

        let status = Command::new("ethtool")
            .args(["-C", &default_iface, "rx-usecs", "0"])
            .status()
            .context("Falha ao executar ethtool. Ele está instalado e executado como root?")?;

        if !status.success() {
            anyhow::bail!("O comando ethtool falhou. Sua placa pode não suportar Coalescing ajustável.");
        }
        Ok(())
    }

    fn revert(&self, original_value: &str) -> Result<()> {
        let default_iface = self.get_default_interface()?;
        if default_iface.is_empty() {
            return Ok(());
        }

        Command::new("ethtool")
            .args(["-C", &default_iface, "rx-usecs", original_value])
            .status()?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl InterruptCoalescingTweak {
    fn get_default_interface(&self) -> Result<String> {
        let output = Command::new("ip").args(["route", "show", "default"]).output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = stdout.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|&x| x == "dev") {
                if pos + 1 < parts.len() {
                    return Ok(parts[pos + 1].to_string());
                }
            }
        }
        Ok(String::new())
    }
}
