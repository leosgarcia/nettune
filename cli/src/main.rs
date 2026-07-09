use clap::{Parser, Subcommand};
use nettune_core::backup::BackupManager;
use nettune_core::get_all_tweaks;
use nettune_core::tweaks::Tweak;
use std::path::PathBuf;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "NetTune")]
#[command(about = "Otimizador de rede local seguro e nativo (sem túneis)", version)]
struct Cli {
    /// Simula a execução sem aplicar mudanças no SO
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lista todos os ajustes suportados pelo seu sistema
    List,
    /// Mostra o status atual (quais ajustes estão aplicados)
    Status,
    /// Aplica um ajuste específico
    Apply { id: String },
    /// Aplica TODOS os ajustes suportados
    ApplyAll,
    /// Reverte um ajuste específico para o valor original
    Revert { id: String },
    /// Reverte TODOS os ajustes aplicados (Restaurar)
    RevertAll,
}

fn get_backup_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".nettune");
    path.push("backup.json");
    path
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let cli = Cli::parse();
    let tweaks = get_all_tweaks();
    let backup_manager = BackupManager::new(get_backup_path());

    match &cli.command {
        Commands::List => {
            println!("== Ajustes Suportados no seu SO ==");
            for tweak in &tweaks {
                println!("- {} ({}):\n  {}", tweak.id(), tweak.name(), tweak.description());
            }
        }
        Commands::Status => {
            let state = backup_manager.load_state().unwrap_or_default();
            println!("== Status do Sistema ==");
            if state.records.is_empty() {
                println!("Nenhum ajuste do NetTune está ativo.");
            } else {
                for (id, record) in &state.records {
                    let name = tweaks.iter().find(|t| t.id() == id).map(|t| t.name()).unwrap_or("Desconhecido");
                    println!("- [ATIVO] {} ({}). Aplicado em: {}", id, name, record.timestamp);
                }
            }
        }
        Commands::Apply { id } => {
            if let Some(tweak) = tweaks.iter().find(|t| t.id() == id) {
                apply_tweak(tweak.as_ref(), &backup_manager, cli.dry_run)?;
            } else {
                error!("Ajuste '{}' não encontrado para este SO.", id);
            }
        }
        Commands::ApplyAll => {
            for tweak in &tweaks {
                apply_tweak(tweak.as_ref(), &backup_manager, cli.dry_run)?;
            }
        }
        Commands::Revert { id } => {
            if let Some(tweak) = tweaks.iter().find(|t| t.id() == id) {
                revert_tweak(tweak.as_ref(), &backup_manager, cli.dry_run)?;
            } else {
                error!("Ajuste '{}' não encontrado para este SO.", id);
            }
        }
        Commands::RevertAll => {
            for tweak in &tweaks {
                revert_tweak(tweak.as_ref(), &backup_manager, cli.dry_run).unwrap_or_else(|e| {
                    error!("Falha ao reverter {}: {}", tweak.id(), e);
                });
            }
        }
    }

    Ok(())
}

fn apply_tweak(tweak: &dyn Tweak, backup_manager: &BackupManager, dry_run: bool) -> anyhow::Result<()> {
    info!("Iniciando aplicação do ajuste: {}", tweak.name());
    
    // Leitura original para backup
    let original = tweak.read_current_value()?;
    if let Some(orig) = original {
        if dry_run {
            info!("[DRY-RUN] Backup salvo: '{}' = '{}'", tweak.id(), orig);
            info!("[DRY-RUN] Ajuste '{}' aplicado virtualmente.", tweak.id());
        } else {
            backup_manager.record_backup(tweak.id(), &orig)?;
            tweak.apply(None)?;
            info!("Ajuste aplicado com sucesso!");
            if tweak.requires_reboot() {
                warn!("Este ajuste requer a reinicialização do sistema para surtir efeito.");
            }
        }
    } else {
        warn!("Não foi possível ler o estado original do ajuste {}. Pode não ser suportado neste hardware.", tweak.id());
    }
    
    Ok(())
}

fn revert_tweak(tweak: &dyn Tweak, backup_manager: &BackupManager, dry_run: bool) -> anyhow::Result<()> {
    let state = backup_manager.load_state()?;
    
    if let Some(record) = state.records.get(tweak.id()) {
        info!("Iniciando reversão do ajuste: {}", tweak.name());
        if dry_run {
            info!("[DRY-RUN] Ajuste revertido para o valor original: {}", record.original_value);
        } else {
            tweak.revert(&record.original_value)?;
            backup_manager.remove_backup(tweak.id())?;
            info!("Reversão concluída com sucesso.");
            if tweak.requires_reboot() {
                warn!("Este ajuste requer a reinicialização do sistema para surtir efeito.");
            }
        }
    } else {
        info!("Nenhum backup encontrado para o ajuste {}. Ele provavelmente não foi aplicado.", tweak.id());
    }
    
    Ok(())
}
