use alloc::vec;
use alloc::vec::Vec;

use no_std_compat::cell::{Cell, RefCell};
use no_std_compat::cmp::Ordering::Equal;

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct Msg<T: Sized> {
    pub when: u32,
    pub order: u32,
    pub payload: T,
}

pub struct EDT<T> {
    now: Cell<u32>,
    queue: RefCell<Vec<Msg<T>>>,
}

impl<T> EDT<T> {}

impl<T: Copy> EDT<T> {
    pub fn create() -> EDT<T> {
        EDT {
            now: Cell::new(0),
            queue: RefCell::new(vec![]),
        }
    }
}

pub enum Event<T> {
    Execute { msg: T },
    Wait { ms: u32 },
    Halt,
}

impl<T: Copy> EDT<T> {
    pub fn now(&self) -> u32 {
        self.now.get()
    }

    pub fn poll(&self) -> Event<T> {
        let head_option = self.peek_head();

        if let Some(head) = head_option {
            let to_wait = head.when - self.now.get();
            // change new now
            self.now.set(head.when);
            if to_wait > 0 {
                Event::Wait { ms: to_wait }
            } else {
                let position = self
                    .queue
                    .borrow()
                    .iter()
                    .position(|it| it.when == head.when && it.order == head.order);
                assert!(position.is_some());
                if let Some(position) = position {
                    let removed = self.queue.borrow_mut().swap_remove(position);
                    assert_eq!(removed.when, head.when);
                    assert_eq!(removed.order, head.order);
                }
                Event::Execute { msg: head.payload }
            }
        } else {
            Event::Halt
        }
    }

    /// Advances the time by the given value and feeds messages to the handler
    #[cfg(not(target_arch = "thumbv6m-none-eabi"))]
    pub fn advance_time_by(&self, time: u32, handler: &dyn Fn(T)) {
        let target = self.now.get() + time;
        let mut elapsed: u32 = 0;
        loop {
            match self.poll() {
                Event::Execute { msg } => {
                    handler(msg);
                }
                Event::Wait { ms } => {
                    elapsed = elapsed + ms;
                    if elapsed > time {
                        self.now.set(target);
                        break;
                    }
                }
                Event::Halt => {
                    break;
                }
            }
        }
    }

    fn peek_head(&self) -> Option<Msg<T>> {
        self.queue
            .borrow()
            .iter()
            .min_by(|lhs, rhs| {
                let by_when = lhs.when.cmp(&rhs.when);
                match by_when {
                    Equal => lhs.order.cmp(&rhs.order),
                    _ => by_when,
                }
            })
            .cloned()
    }

    pub fn schedule(&self, delay: u32, payload: T) {
        let when = self.now.get() + delay;

        let order = self
            .queue
            .borrow()
            .iter()
            .filter(|message| message.when == when)
            .map(|it| it.order)
            .max()
            .unwrap_or(0);

        self.queue.borrow_mut().push(Msg {
            when,
            order,
            payload,
        });

        assert!(self.queue.borrow().len() < 10);
    }

    pub fn remove<F>(&self, mut predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.queue.borrow_mut().retain(|it| !predicate(&it.payload));
    }

    #[cfg(not(target_arch = "thumbv6m-none-eabi"))]
    pub fn queue_len(&self) -> usize {
        self.queue.borrow().len()
    }
}
