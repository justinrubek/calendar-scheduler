use minidom::Element;
use reqwest::{header::CONTENT_TYPE, Method, Result};
use url::Url;

use crate::util::find_element;

use super::principal::Principal;

static DAVCLIENT_BODY: &str = r#"
    <d:propfind xmlns:d="DAV:">
        <d:prop>
            <d:current-user-principal />
        </d:prop>
    </d:propfind>
"#;

#[derive(Clone, Debug)]
pub struct DavCredentials {
    pub(super) username: String,
    pub(super) password: String,
}

impl DavCredentials {
    pub fn new(username: String, password: String) -> DavCredentials {
        DavCredentials { username, password }
    }
}

#[derive(Clone, Debug)]
pub struct DavClient {
    url: Url,
    pub(super) credentials: DavCredentials,
}

impl DavClient {
    pub fn new(url: String, credentials: DavCredentials) -> Self {
        let url = Url::parse(&url).expect("failed to parse url");

        DavClient { url, credentials }
    }

    pub fn create_request(
        &self,
        client: &reqwest::Client,
        method: Method,
        url: &Url,
        depth: u32,
    ) -> Result<reqwest::RequestBuilder> {
        let req = client
            .request(method, url.as_str())
            .header("Depth", format!("{depth}"))
            .header(CONTENT_TYPE, "application/xml")
            .basic_auth(&self.credentials.username, Some(&self.credentials.password));

        Ok(req)
    }

    pub async fn get_principal(&self, client: &reqwest::Client) -> Result<Principal> {
        let method = Method::from_bytes(b"PROPFIND").expect("failed to create PROPFIND method");

        let res = self
            .create_request(client, method, &self.url, 0)?
            .body(DAVCLIENT_BODY)
            .send()
            .await?;

        let text = res.text().await?;

        let root: Element = text.parse().expect("failed to parse xml");
        let principal = find_element(&root, "current-user-principal".to_string())
            .expect("failed to find current-user-principal");
        let principal_href =
            find_element(principal, "href".to_string()).expect("failed to find principal's href");

        let href = principal_href.text();

        let mut url = self.url.clone();
        url.set_path(&href);

        Ok(Principal::new(self.clone(), url))
    }
}
