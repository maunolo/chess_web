use web_sys::Element;

use super::elements;

pub trait ElementPoolExt {
  fn soft_remove(&self);
  fn find_or_create() -> Element;
}

impl ElementPoolExt for Element {
  fn find_or_create() -> Element {
    if let Some(elem) = elements::query_selector(".element-pool") {
      elem.set_attribute("class", "").unwrap();
      elem
    } else {
      let elem = elements::document().create_element("div").unwrap();
      elem
    }
  }

  fn soft_remove(&self) {
    self.set_attribute("class", "element-pool").unwrap();
    self.set_attribute("data-square", "").unwrap();
    self.set_attribute("style", "").unwrap();
  }
}