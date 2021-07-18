//! Managing the state of systemd services.

use crate::{args::ServiceArgs, config::Config};

/// Synchronizes enabled systemd services with the service list.
pub(crate) fn synchronize_services(args: ServiceArgs, config: Config) -> anyhow::Result<()> {
    todo!()
}
