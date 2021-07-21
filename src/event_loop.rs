use std::cell::{Cell, RefCell};
use std::thread::sleep;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Msg {
    pub what: i32,
    pub when: u32,
    pub arg0: i32,
    pub arg1: i32,
}

pub struct EDT {
    now: Cell<u32>,
    queue: RefCell<Vec<Msg>>,
}

impl EDT {
    pub fn create() -> EDT {
        EDT {
            now: Cell::new(0),
            queue: RefCell::new(Vec::new()),
        }
    }
}

/// TODO use a handler function instead of poll
/// TODO make sleep and now injectable
/// TODO no std
impl EDT {
    pub fn exit(&self) {
        // clearing the queue ends the loop
        self.queue.borrow_mut().clear();
    }

    pub fn poll(&self) -> Option<Msg> {
        self.queue
            .borrow_mut()
            .sort_by_key(|msg| { msg.when });

        return if self.queue.borrow().is_empty() {
            None
        } else {
            let first = self.queue
                .borrow_mut()
                .remove(0);

            let millis = first.when - self.now.get();

            // println!("Now sleeping: {}", millis);
            sleep(Duration::from_millis(millis as u64));

            self.now.set(first.when);

            Some(first)
        };
    }

    pub fn schedule(&self, delay: u32, what: i32) {
        self.schedule_with_args(delay, what, 0, 0);
    }

    pub fn schedule_with_args(&self, delay: u32, what: i32, arg0: i32, arg1: i32) {
        let msg = Msg {
            what,
            when: self.now.get() + delay,
            arg0,
            arg1,
        };
        self.queue.borrow_mut().push(msg);
    }

    pub fn remove_with_what(&self, what: i32) {
        self.queue.borrow_mut().retain(|msg| msg.what != what);
    }
}

#[cfg(test)]
mod tests {
    use crate::event_loop::EDT;
    use crate::event_loop::Msg;

    #[test]
    fn send_some_events() {
        let edt = EDT::create();
        edt.schedule(1000, 1);
        edt.schedule_with_args(3000, 1, 2, 3);
        assert_eq!(edt.now.get(), 0);
        assert_eq!(edt.queue.borrow_mut().remove(0),
                   Msg {
                       what: 1,
                       when: 1000,
                       arg0: 0,
                       arg1: 0,
                   });
        assert_eq!(edt.queue.borrow_mut().remove(0),
                   Msg {
                       what: 1,
                       when: 3000,
                       arg0: 2,
                       arg1: 3,
                   });
    }
}