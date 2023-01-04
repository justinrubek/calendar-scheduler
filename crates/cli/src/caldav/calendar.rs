use url::Url;

#[derive(Clone, Debug)]
pub struct Calendar {
    url: Url,
    pub display_name: String,
    pub identifier: String,
}

impl Calendar {
    pub fn new(url: Url, display_name: String, identifier: String) -> Calendar {
        Calendar {
            url,
            display_name,
            identifier,
        }
    }

    pub fn get_events(&self) -> Result<Vec<()>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }
}
