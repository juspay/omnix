use nix_rs::command::NixCmd;
use tokio::sync::OnceCell;

pub mod flake_template;

static NIXCMD: OnceCell<NixCmd> = OnceCell::const_new();

pub async fn nixcmd() -> &'static NixCmd {
    NIXCMD
        .get_or_init(|| async { NixCmd::default().with_flakes().await.unwrap() })
        .await
}
