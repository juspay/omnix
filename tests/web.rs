use leptos::get_configuration;
use nix_browser::nix;
use std::env;
use std::str::FromStr;
use thirtyfour::prelude::*;

mod dom;

struct TestSession {
    app_url: String,
    web_driver: WebDriver,
}

impl TestSession {
    async fn new() -> WebDriverResult<Self> {
        let mut caps = DesiredCapabilities::chrome();

        if env::var("CHROME_DRIVER_HEADLESS").unwrap_or("true".to_owned()) == "true" {
            let _ = caps.set_headless();
        }

        let binary_path = env::var("CHROME_BINARY_PATH");
        if binary_path.is_ok() {
            let _ = caps.set_binary(&binary_path.unwrap());
        }

        let driver_url =
            env::var("CHROME_DRIVER_URL").unwrap_or("http://127.0.0.1:9515".to_owned());
        let driver = WebDriver::new(&driver_url, caps).await?;

        let conf = get_configuration(None).await.unwrap();
        let mut socket_conf = conf.leptos_options.site_addr;
        let test_port = env::var("TEST_PORT").unwrap_or("3000".to_owned());

        socket_conf.set_port(
            u16::from_str(&test_port)
                .map_err(|_| WebDriverError::CustomError("test port invalid".to_owned()))?,
        );

        Ok(TestSession {
            web_driver: driver,
            app_url: format!("http://{}", &socket_conf),
        })
    }

    async fn navigate(&self, path: &str) -> WebDriverResult<()> {
        self.web_driver
            .goto(format!("{}/{}", self.app_url, path))
            .await?;
        Ok(())
    }
}

#[tokio::test]
async fn test_nix_version() -> WebDriverResult<()> {
    let session = TestSession::new().await?;
    session.navigate("info").await?;

    let driver = session.web_driver;

    let elem = driver
        .query(By::XPath("//div[./b[contains(text(), 'Nix Version')]]"))
        .first()
        .await?;

    let nix_info_card = dom::NixInfoCard::new(elem);
    let nix_version_text = nix_info_card.get_version_text().await?;
    let current_nix_version =
        nix::version::NixVersion::from_str(&format!("nix (Nix) {}", nix_version_text))
            .map_err(|_| WebDriverError::CustomError("cannot parse nix version".to_owned()))?;
    let actual_nix_version = nix::info::get_nix_info(())
        .await
        .map_err(|_| WebDriverError::CustomError("cannot get nix info from api".to_owned()))?;

    assert_eq!(current_nix_version, actual_nix_version.nix_version);
    Ok(())
}
