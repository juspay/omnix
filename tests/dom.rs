use thirtyfour::{
    components::{Component, ElementResolver},
    prelude::*,
};

#[derive(Debug, Clone, Component)]
pub struct NixInfoCard {
    base: WebElement,
    #[by(tag = "div", first)]
    version: ElementResolver<WebElement>,
}

impl NixInfoCard {
    pub async fn get_version_text(&self) -> WebDriverResult<String> {
        let version_elem = self.version.resolve().await?;
        let text = version_elem.text().await?;
        Ok(text)
    }
}
