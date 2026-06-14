use crate::settings::Settings;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct AppState {
    pub settings: Settings,
}
pub type SharedState = Rc<RefCell<AppState>>;

impl AppState {
    pub fn new() -> SharedState {
        Rc::new(RefCell::new(AppState {
            settings: crate::settings::load(),
        }))
    }
}
