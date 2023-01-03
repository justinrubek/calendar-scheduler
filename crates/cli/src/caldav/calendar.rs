#[derive(Clone, Debug)]
pub struct Calendar {
    pub display_name: String,
    pub identifier: String,
}

impl Calendar {
    pub fn new(display_name: String, identifier: String) -> Calendar {
        Calendar {
            display_name,
            identifier,
        }
    }
}
