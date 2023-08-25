use thirtyfour::{
    components::{Component, ElementResolver},
    prelude::*,
};
use std::time::Duration;

#[derive(Debug, Clone, Component)]
pub struct NixInfoCard {
    base: WebElement,
    #[by(tag = "div", first)]
    version: ElementResolver<WebElement>,
}

impl NixInfoCard {
    pub async fn get_version_text(&self) -> WebDriverResult<String> {
        let version_elem = self.version.resolve().await?;
        // timeout + interval for hydration
        let _ = version_elem.wait_until().wait(Duration::new(1, 0), Duration::new(5, 0)).displayed().await?;
        let text = version_elem.text().await?;
        Ok(text)
    }
}
