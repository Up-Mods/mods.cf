use serde::Serialize;
use twilight_model::channel::message::Component;

#[derive(Serialize, Clone)]
pub(crate) struct ComponentHolder {
    component: Component,
}

impl ComponentHolder {
    pub(crate) fn new(component: impl Into<Component>) -> Self {
        Self {
            component: component.into(),
        }
    }
}

impl From<Component> for ComponentHolder {
    fn from(value: Component) -> Self {
        Self::new(value)
    }
}
