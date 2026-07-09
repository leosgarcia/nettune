use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OsPlatform {
    Windows,
    Linux,
    MacOS,
}

pub mod mock;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;

pub trait Tweak {
    /// Identificador único do ajuste (ex: "tcp_nodelay")
    fn id(&self) -> &'static str;
    
    /// Nome legível do ajuste
    fn name(&self) -> &'static str;
    
    /// Descrição do que o ajuste faz
    fn description(&self) -> &'static str;
    
    /// Lista de sistemas operacionais suportados por este ajuste
    fn supported_os(&self) -> Vec<OsPlatform>;
    
    /// Lê o valor atual do sistema. Retorna None se não for possível ler ou se não existir
    fn read_current_value(&self) -> Result<Option<String>>;
    
    /// Aplica o ajuste. O parâmetro option_value pode ser usado se o ajuste suportar
    /// múltiplos valores (ex: habilitar/desabilitar, ou um número específico).
    fn apply(&self, option_value: Option<&str>) -> Result<()>;
    
    /// Reverte o ajuste para o valor especificado.
    fn revert(&self, original_value: &str) -> Result<()>;
    
    /// Informa se a máquina precisa ser reiniciada após a aplicação
    fn requires_reboot(&self) -> bool {
        false
    }
}
