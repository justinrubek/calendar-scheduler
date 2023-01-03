use minidom::Element;
use reqwest::{
    header::CONTENT_TYPE,
    Client, Method, Result
};
use url::Url;

use super::principal::Principal;
use super::util::find_element;

static DAVCLIENT_BODY: &str = r#"
    <d:propfind xmlns:d="DAV:">
        <d:prop>
            <d:current-user-principal />
        </d:prop>
    </d:propfind>
"#;

#[derive(Clone, Debug)]
pub struct DavCredentials {
    pub (super) username: String,
    pub (super) password: String,
}

impl DavCredentials {
    pub fn new(username: String, password: String) -> DavCredentials {
        DavCredentials {
            username,
            password,
        }
    }
}

#[derive(Debug)]
pub struct DavClient {
    url: Url,
    credentials: DavCredentials,
}

impl DavClient {
    pub fn new(url: String, credentials: DavCredentials) -> Self {
        let url = Url::parse(&url).expect("failed to parse url");

        DavClient { 
            url,
            credentials,
        }
    }

    pub async fn get_principal(&self, client: &Client) -> Result<Principal> {
        let method = Method::from_bytes(b"PROPFIND")
            .expect("failed to create PROPFIND method");

        let res = client
            .request(method, self.url.as_str())
            .header("Depth", "0")
            .header(CONTENT_TYPE, "application/xml")
            .basic_auth(&self.credentials.username, Some(&self.credentials.password))
            .body(DAVCLIENT_BODY)
            .send()
            .await?;

        let text = res.text().await?;

        let root: Element = text.parse().expect("failed to parse xml");
        let principal = find_element(&root, "current-user-principal".to_string())
            .expect("failed to find current-user-principal");
        let principal_href = find_element(principal, "href".to_string())
            .expect("failed to find principal's href");

        let href = principal_href.text();

        let mut url = self.url.clone();
        url.set_path(&href);

        Ok(Principal::new(url, self.credentials.clone()))
    }
}
