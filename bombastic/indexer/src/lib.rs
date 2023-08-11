use std::process::ExitCode;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use trustification_event_bus::EventBusConfig;
use trustification_index::{IndexConfig, IndexStore};
use trustification_indexer::{actix::configure, Indexer, IndexerStatus};
use trustification_infrastructure::{Infrastructure, InfrastructureConfig};
use trustification_storage::StorageConfig;

#[derive(clap::Args, Debug)]
#[command(about = "Run the indexer", args_conflicts_with_subcommands = true)]
pub struct Run {
    #[arg(long = "stored-topic", default_value = "sbom-stored")]
    pub stored_topic: String,

    #[arg(long = "indexed-topic", default_value = "sbom-indexed")]
    pub indexed_topic: String,

    #[arg(long = "failed-topic", default_value = "sbom-failed")]
    pub failed_topic: String,

    #[arg(long = "devmode", default_value_t = false)]
    pub devmode: bool,

    #[command(flatten)]
    pub bus: EventBusConfig,

    #[command(flatten)]
    pub index: IndexConfig,

    #[command(flatten)]
    pub storage: StorageConfig,

    #[command(flatten)]
    pub infra: InfrastructureConfig,
}

impl Run {
    pub async fn run(mut self) -> anyhow::Result<ExitCode> {
        let (command_sender, command_receiver) = mpsc::channel(1);
        let status = Arc::new(Mutex::new(IndexerStatus::Running));
        let s = status.clone();
        Infrastructure::from(self.infra)
            .run_with_config(
                "bombastic-indexer",
                |metrics| async move {
                    let index = IndexStore::new(&self.index, bombastic_index::Index::new(), metrics.registry())?;
                    let storage = self.storage.create("bombastic", self.devmode, metrics.registry())?;

                    let interval = self.index.sync_interval.into();
                    let bus = self.bus.create(metrics.registry()).await?;
                    if self.devmode {
                        bus.create(&[self.stored_topic.as_str()]).await?;
                    }

                    let mut indexer = Indexer {
                        index,
                        storage,
                        bus,
                        stored_topic: self.stored_topic.as_str(),
                        indexed_topic: self.indexed_topic.as_str(),
                        failed_topic: self.failed_topic.as_str(),
                        sync_interval: interval,
                        status: s.clone(),
                        commands: command_receiver,
                    };
                    indexer.run().await
                },
                move |config| {
                    configure(status, command_sender, config);
                },
            )
            .await?;
        Ok(ExitCode::SUCCESS)
    }
}
