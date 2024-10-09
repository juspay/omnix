use nix_rs::config::NixConfig;
use omnix_init::registry::BUILTIN_REGISTRY;

/// `om init` runs and successfully initializes a template
#[tokio::test]
async fn om_init() -> anyhow::Result<()> {
    let registry = BUILTIN_REGISTRY.clone();
    let cfg = NixConfig::get().await.as_ref()?;
    let current_system = &cfg.system.value;
    for url in registry.0.values() {
        // TODO: Refactor(DRY) with src/core.rs:run_tests
        // TODO: Make this test (and other tests) use tracing!
        println!("🕍 Testing template: {}", url);
        let templates = omnix_init::config::load_templates(url).await?;
        for template in templates {
            let tests = &template.template.tests;
            for (name, test) in tests {
                if test.can_run_on(current_system) {
                    println!(
                        "🧪 [{}#{}] Running test: {}",
                        url, template.template_name, name
                    );
                    test.run_test(
                        &url.with_attr(&format!("{}.{}", template.template_name, name)),
                        &template,
                    )
                    .await?;
                } else {
                    println!(
                        "⚠️ Skipping test: {} (cannot run on {})",
                        name, current_system
                    );
                }
            }
        }
    }
    Ok(())
}
