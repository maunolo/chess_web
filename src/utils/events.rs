pub trait EventTargetExt {
    fn target_element(&self) -> Option<web_sys::Element>;
}

impl EventTargetExt for web_sys::Event {
    fn target_element(&self) -> Option<web_sys::Element> {
        super::js_cast::<web_sys::Element, _>(self.target()?)
    }
}

impl EventTargetExt for web_sys::MouseEvent {
    fn target_element(&self) -> Option<web_sys::Element> {
        super::js_cast::<web_sys::Element, _>(self.target()?)
    }
}

impl EventTargetExt for web_sys::TouchEvent {
    fn target_element(&self) -> Option<web_sys::Element> {
        super::js_cast::<web_sys::Element, _>(self.target()?)
    }
}

pub trait EventPositionExt {
    fn position(&self) -> (i32, i32);
}

impl EventPositionExt for web_sys::MouseEvent {
    fn position(&self) -> (i32, i32) {
        (self.client_x(), self.client_y())
    }
}

impl EventPositionExt for web_sys::TouchEvent {
    fn position(&self) -> (i32, i32) {
        let mut touches = self.touches();
        let mut index = 0;

        log::debug!("Touches: {}", touches.length());

        if touches.length() == 0 {
            touches = self.changed_touches();
            if touches.length() > 0 {
                index = touches.length() - 1;
            }
        }

        log::debug!("Changed Touches: {}", touches.length());

        if let Some(touch) = touches.get(index) {
            (touch.client_x(), touch.client_y())
        } else {
            (0, 0)
        }
    }
}
