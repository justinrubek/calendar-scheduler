use minidom::Element;

/// Recursively find elements in the root that match the given tag name
pub fn find_elements(root: &Element, tag: String) -> Vec<&Element> {
    let mut elements: Vec<&Element> = Vec::new();

    for elem in root.children() {
        if elem.name() == tag {
            elements.push(elem);
        } else {
            let ret = find_elements(elem, tag.clone());
            elements.extend(ret);
        }
    }

    elements
}

/// Recursively search for the first element that matches the given tag name
pub fn find_element(root: &Element, tag: String) -> Option<&Element> {
    if root.name() == tag {
        return Some(root);
    }

    for elem in root.children() {
        if elem.name() == tag {
            return Some(elem);
        } else {
            let ret = find_element(elem, tag.clone());
            if ret.is_some() {
                return ret;
            }
        }
    }

    None
}
