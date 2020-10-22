use std::cell::{Cell, RefCell};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Msg<T> {
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

impl<T> EDT<T> {
    pub fn exit(&self) {
        // clearing the queue ends the loop
        self.queue.borrow_mut().clear();
    }

    /// Processes all events which are due within the advance_time_by from now
    /// returns the time until the next event, which can be used to sleep
    pub fn process_events(&self, advance_time_by: u32, handler: &dyn Fn(T)) -> u32 {
        let target_time = self.now.get() + advance_time_by;
        loop {
            if self.queue.borrow().is_empty() {
                return 0;
            }

            self.queue.borrow_mut().sort_by_key(|msg| { msg.when });

            let next_when = self.queue.borrow().first().unwrap().when;

            if next_when > target_time {
                // processed all messages due before target_time
                self.now.set(target_time);
                return next_when - target_time;
            } else {
                let next_msg = self.queue.borrow_mut().remove(0);
                self.now.set(next_msg.when);
                handler(next_msg.payload);
            }
        }
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