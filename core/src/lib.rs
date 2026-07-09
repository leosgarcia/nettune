#![warn(clippy::pedantic)]

pub mod backup;
pub mod tweaks;

use crate::tweaks::Tweak;

/// Retorna a lista de todos os tweaks disponíveis para o sistema operacional atual.
#[must_use]
pub fn get_all_tweaks() -> Vec<Box<dyn Tweak>> {
    let mut list: Vec<Box<dyn Tweak>> = Vec::new();

    #[cfg(target_os = "windows")]
    {
        list.push(Box::new(tweaks::windows::NetworkThrottlingTweak));
        list.push(Box::new(tweaks::windows::NagleAlgorithmTweak));
        list.push(Box::new(tweaks::windows::InterruptModerationTweak));
    }

    #[cfg(target_os = "linux")]
    {
        list.push(Box::new(tweaks::linux::TcpCongestionControlTweak));
        list.push(Box::new(tweaks::linux::NetdevMaxBacklogTweak));
        list.push(Box::new(tweaks::linux::TcpLowLatencyTweak));
        list.push(Box::new(tweaks::linux::InterruptCoalescingTweak));
    }

    #[cfg(target_os = "macos")]
    {
        list.push(Box::new(tweaks::macos::DelayedAckTweak));
        list.push(Box::new(tweaks::macos::TcpWindowSizeTweak));
    }

    list
}
