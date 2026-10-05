//! A speech backend for tests: it keeps what it was given to say.

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::speech::Backend;

#[derive(Default)]
pub struct Recorder {
    pub spoken: RefCell<Vec<String>>,
    /// What was said cutting off what came before.
    pub interrupting: RefCell<Vec<String>>,
}

impl Recorder {
    pub fn new() -> Rc<Self> {
        Rc::new(Recorder::default())
    }

    pub fn spoken(&self) -> Vec<String> {
        self.spoken.borrow().clone()
    }

    pub fn interrupting(&self) -> Vec<String> {
        self.interrupting.borrow().clone()
    }
}

impl Backend for Recorder {
    fn speak(&self, text: &str, interrupt: bool) {
        self.spoken.borrow_mut().push(text.to_string());
        if interrupt {
            self.interrupting.borrow_mut().push(text.to_string());
        }
    }

    fn braille(&self, _text: &str) {}
}
