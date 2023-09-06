use nix_health::{traits::Check, NixHealth};
use nix_rs::{command::NixCmd, info::NixInfo};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    let info = NixInfo::from_nix(&NixCmd::default()).await?;
    let health = NixHealth::check(&info);
    for check in &health {
        println!("{}: {:?}", check.name(), check.report());
    }
    Ok(())
}
