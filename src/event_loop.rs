use std::cell::{Cell, RefCell};
use std::thread::sleep;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Msg<T> {
    pub when: u32,
    pub payload: T,
}

pub struct EDT<T> {
    now: Cell<u32>,
    queue: RefCell<Vec<Msg<T>>>,
}

impl<T> EDT<T> {
    pub fn create() -> EDT<T> {
        EDT {
            now: Cell::new(0),
            queue: RefCell::new(Vec::new()),
        }
    }
}

/// TODO use a handler function instead of poll
/// TODO make sleep and now injectable
/// TODO no std
impl<T> EDT<T> {
    pub fn exit(&self) {
        // clearing the queue ends the loop
        self.queue.borrow_mut().clear();
    }

    pub fn poll(&self) -> Option<Msg<T>> {
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

    pub fn schedule(&self, delay: u32, payload: T) {
        let msg = Msg {
            when: self.now.get() + delay,
            payload,
        };
        self.queue.borrow_mut().push(msg);
    }

    pub fn remove<F>(&self, mut predicate: F) where F: FnMut(&T) -> bool {
        self.queue.borrow_mut().retain(|msg| !predicate(&msg.payload));
    }
}

#[cfg(test)]
mod tests {
    use crate::event_loop::EDT;
    use crate::event_loop::Msg;

    #[test]
    fn send_some_events() {
        let edt: EDT<u32> = EDT::create();
        edt.schedule(1000, 1);
        edt.schedule(3000, 2);
        assert_eq!(edt.now.get(), 0);
        assert_eq!(
            edt.queue.borrow_mut().remove(0),
            Msg { when: 1000, payload: 1 }
        );
        assert_eq!(
            edt.queue.borrow_mut().remove(0),
            Msg { when: 3000, payload: 2 }
        );
    }
}