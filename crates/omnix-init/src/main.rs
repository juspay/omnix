use std::{collections::HashMap, path::Path};

use omnix_init::core::initialize_template;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    initialize_template(Path::new("/tmp/init"), None, &HashMap::new()).await
}
