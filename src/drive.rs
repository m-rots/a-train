use crate::{autoscan::create_payload, Atrain, Result};
use bernard::SyncKind;
use futures::prelude::*;
use tracing::warn;

const CONCURRENCY: usize = 5;

impl Atrain {
    async fn sync_drive(&self, drive_id: &str) -> Result<()> {
        match self.bernard.sync_drive(drive_id).await {
            // Do not send a payload to Autoscan on a full scan
            Ok(SyncKind::Full) => (),
            Ok(SyncKind::Partial(changes)) => {
                let changed_paths = changes.paths().await?;
                let payload = create_payload(changed_paths);

                if !payload.is_empty() {
                    self.autoscan.send_payload(drive_id, &payload).await?;
                }
            }
            Err(err) => {
                // Can ignore a Partial Change List as it should recover eventually.
                if !err.is_partial_change_list() {
                    return Err(err.into());
                }

                warn!(%drive_id, "Encountered a Partial Change List.")
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn sync(&self) -> Result<()> {
        // also fetch changes here and create+send response to Autoscan for each individual Drive.
        // https://stackoverflow.com/questions/51044467
        stream::iter(&self.drives)
            .map(|drive_id| self.sync_drive(drive_id))
            .buffer_unordered(CONCURRENCY)
            .try_collect()
            .await
    }

    pub async fn close(self) {
        self.bernard.close().await
    }
}
