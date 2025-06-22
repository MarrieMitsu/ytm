use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{IFRAME_API_URL, LOCAL_WIDGET_API_PATH, utils::fetch_url};

/// YouTube
#[derive(Clone, Debug)]
pub struct YouTube {
    pub iframe_api_script: String,
    pub widgetapi_script: String,
}

/// Load YouTube components
///
/// Retrieve YouTube Iframe API script once and serve it locally for the rest
/// of the program's lifetime, reducing outbound network requests
///
/// This function will panic if cannot extract `www-widgetapi.js` URL from
/// `iframe_api`, which is most likely due to the `iframe_api` structure has
/// been changed from the YouTube side
pub async fn load_youtube_components() -> Result<YouTube> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"var scriptUrl = '(.*?)';"#).unwrap());

    log::debug!("Retrieve `iframe_api` script");

    let iframe_api_script = fetch_url(IFRAME_API_URL).await?;
    let iframe_api_script = String::from_utf8(iframe_api_script.to_vec())?;

    log::debug!("Extract `www-widgetapi.js` URL from `iframe_api` script");

    let origin_url = RE
        .captures(&iframe_api_script)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().replace(r"\/", "/"))
        .expect("Cannot extract `www-widgetapi.js` URL from `frame_api`, which is most likely due to the `iframe_api` structure has been changed from the YouTube side");

    log::debug!("Modify `iframe_api` script");

    let new_url = LOCAL_WIDGET_API_PATH.replace("/", r"\/");
    let iframe_api_script = RE
        .replace(
            &iframe_api_script,
            format!("var scriptUrl = '{}';", new_url),
        )
        .to_string();

    log::debug!("Retrieve `www-widgetapi.js` script");

    let widgetapi_script = fetch_url(&origin_url).await?;
    let widgetapi_script = String::from_utf8(widgetapi_script.to_vec())?;

    let yt = YouTube {
        iframe_api_script,
        widgetapi_script,
    };

    Ok(yt)
}
