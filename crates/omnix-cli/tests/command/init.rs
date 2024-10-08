use omnix_init::registry::BUILTIN_REGISTRY;

/// `om init` runs and successfully initializes a template
#[tokio::test]
async fn om_init() -> anyhow::Result<()> {
    let registry = BUILTIN_REGISTRY.clone();
    for url in registry.0.values() {
        println!("🕍 Testing template: {}", url);
        let templates = omnix_init::config::load_templates(url).await?;
        for template in templates {
            let tests = &template.template.tests;
            for (name, test) in tests {
                println!(
                    "🧪 [{}#{}] Running test: {}",
                    url, template.template_name, name
                );
                test.run_test(name, &template).await?;
            }
        }
    }
    Ok(())
}
