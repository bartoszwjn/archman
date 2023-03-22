//! Managing the state of systemd services.

use std::{collections::HashSet, process::Command};

use anyhow::{anyhow, Context};

use crate::{args::ServiceArgs, config::Config};

struct ServiceStatus {
    /// The service is set to run at every boot.
    enabled: bool,
    /// The service is currently running.
    active: bool,
}

/// Synchronizes enabled systemd services with the service list.
pub fn synchronize_services(args: ServiceArgs, config: Config) -> anyhow::Result<()> {
    let services = config.services();
    warn_about_duplicate_services(&services.duplicates);

    if args.reset {
        systemctl_preset_all()
            .context("Failed to reset the enabled/disabled status of all services")?;
    }

    let to_enable = find_services_to_enable(&services.elements, args.start)
        .context("Failed to determine the set of services to enable")?;

    enable_services(&to_enable, args.start).context("Failed to enable declared services")?;

    Ok(())
}

fn warn_about_duplicate_services(duplicates: &HashSet<&str>) {
    for duplicate in duplicates {
        warn!("service {:?} declared multiple times", duplicate);
    }
}

fn systemctl_preset_all() -> anyhow::Result<()> {
    colour!("Resetting the enabled/disabled status of all services to their defaults");
    let mut cmd = Command::new("systemctl");
    cmd.arg("preset-all");
    run_for_status(cmd)
}

fn find_services_to_enable<'a>(
    declared: &HashSet<&'a str>,
    start: bool,
) -> anyhow::Result<Vec<&'a str>> {
    let mut to_enable = vec![];
    for service in declared.iter().copied() {
        let status = check_service_status(service)
            .with_context(|| format!("Failed to query for status of service {:?}", service))?;
        let should_enable = match (status.enabled, status.active) {
            (false, _) => true,
            (true, true) => false,
            (true, false) => start,
        };
        if should_enable {
            to_enable.push(service);
        }
    }
    to_enable.sort_unstable();
    Ok(to_enable)
}

fn check_service_status(service: &str) -> anyhow::Result<ServiceStatus> {
    let enabled = Command::new("systemctl")
        .args(["is-enabled", "-q", service])
        .status()
        .context("Failed to run systemctl")?
        .success();
    let active = Command::new("systemctl")
        .args(["is-active", "-q", service])
        .status()
        .context("Failed to run systemctl")?
        .success();
    Ok(ServiceStatus { enabled, active })
}

fn enable_services(services: &[&str], start: bool) -> anyhow::Result<()> {
    if services.is_empty() {
        return Ok(());
    }

    if start {
        colour!(
            "Enabling and starting {} {}",
            services.len(),
            services_str(services.len()),
        );
    } else {
        colour!(
            "Enabling {} {}",
            services.len(),
            services_str(services.len()),
        );
    }
    let mut cmd = Command::new("systemctl");
    cmd.arg("enable");
    if start {
        cmd.arg("--now");
    }
    cmd.args(services);
    run_for_status(cmd)
}

/// Runs the command, returning `Ok(())` if the command exits successfully.
///
/// The input and output streams of the command are inherited from the current process. Emits output
/// to mark the start and end of the command output.
fn run_for_status(mut cmd: Command) -> anyhow::Result<()> {
    bold!("======== RUNNING SYSTEMCTL ========");
    let status = cmd.status();
    bold!("===== END OF SYSTEMCTL OUTPUT =====");
    match status {
        Ok(exit_status) if exit_status.success() => Ok(()),
        Ok(_) => Err(anyhow!("systemctl did not exit successfully")),
        Err(err) => Err(err).context("Failed to run systemctl"),
    }
}

fn services_str(count: usize) -> &'static str {
    if count == 1 {
        "service"
    } else {
        "services"
    }
}
