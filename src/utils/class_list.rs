use web_sys::Element;

pub struct ClassList {
    pub class_name: String,
}

impl ClassList {
    pub fn new(class_name: &str) -> ClassList {
        ClassList {
            class_name: String::from(class_name),
        }
    }

    pub fn add(&mut self, class: &str) {
        let new_class_name: Vec<&str> = self.class_name.split(" ").collect();
        if !new_class_name.contains(&class) {
            self.class_name.push_str(format!(" {}", class).as_str());
        }
    }

    pub fn remove(&mut self, class: &str) {
        let mut class_list: Vec<&str> = self.class_name.split(" ").collect();
        class_list.retain(|c| *c != class);
        self.class_name = class_list.join(" ");
    }
}

pub trait ClassListExt {
    fn class_list_add(&self, class: &str);
    fn class_list_remove(&self, class: &str);
    fn class_list_include(&self, class: &str) -> bool;
}

impl ClassListExt for Element {
    fn class_list_add(&self, class: &str) {
        let mut class_list = ClassList::new(&self.class_name());
        class_list.add(class);
        self.set_class_name(&class_list.class_name);
    }

    fn class_list_remove(&self, class: &str) {
        let mut class_list = ClassList::new(&self.class_name());
        class_list.remove(class);
        self.set_class_name(&class_list.class_name);
    }

    fn class_list_include(&self, class: &str) -> bool {
        let class_list = ClassList::new(&self.class_name());
        class_list.class_name.split(" ").any(|c| c == class)
    }
}
