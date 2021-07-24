use no_std_compat::cell::{Cell, RefCell};
use no_std_compat::cmp::Ordering::Equal;

const SIZE: usize = 25;

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct Msg<T: Sized> {
    pub when: u32,
    pub order: u32,
    pub payload: T,
}

pub struct EDT<T> {
    now: Cell<u32>,
    // TODO private, currently public because of integration tests
    pub queue: RefCell<[Option<Msg<T>>; SIZE]>,
}

impl<T: Copy> EDT<T> {
    pub fn create() -> EDT<T> {
        EDT {
            now: Cell::new(0),
            queue: RefCell::new([None; SIZE]),
        }
    }
}

pub enum Event<T> {
    Execute { msg: T },
    Wait { ms: u32 },
    Halt,
}

impl<T: Copy> EDT<T> {
    pub fn now(&self) -> u32 { self.now.get() }

    pub fn exit(&self) {
        // clearing the queue ends the loop
        self.queue.replace([None; SIZE]);
    }

    pub fn poll(&self) -> Event<T> {
        if self.queue.borrow().iter().all(|msg| { msg.is_none() }) {
            return Event::Halt;
        }

        let head = self.peek_head();
        let to_wait = head.when - self.now.get();

        return if to_wait > 0 {
            // processed all messages due before target_time
            self.now.set(head.when);
            Event::Wait { ms: to_wait }
        } else {
            let next_when = head.when;

            // WTF is that
            self.queue.borrow_mut()
                .iter_mut()
                .filter(|it| {
                    it.is_some()
                        && it.unwrap().when == head.when
                        && it.unwrap().order == head.order
                })
                .take(1)
                .for_each(|opt| { *opt = None; });

            self.now.set(next_when);
            Event::Execute { msg: head.payload }
        };
    }

    /// Advances the time by the given value and feeds messages to the handler
    pub fn advance_time_by(&self, time: u32,  handler: &dyn Fn(T)) {
        let target = self.now.get() + time;
        let mut elapsed: u32 = 0;
        loop {
            match self.poll() {
                Event::Execute { msg } => { handler(msg); }
                Event::Wait { ms } => {
                    elapsed = elapsed + ms;
                    if elapsed > time {
                        self.now.set(target);
                        break;
                    }
                }
                Event::Halt => { break; }
            }
        }
    }

    fn peek_head(&self) -> Msg<T> {
        self.queue.borrow()
            .iter()
            .filter_map(|&opt| { opt })
            .min_by(|lhs, rhs| {
                let by_when = lhs.when.cmp(&rhs.when);
                if by_when != Equal { by_when } else { lhs.order.cmp(&rhs.order) }
            })
            .unwrap()
    }

    pub fn schedule(&self, delay: u32, payload: T) {
        let when = self.now.get() + delay;

        let order = self.queue.borrow()
            .iter()
            .filter(|opt| { opt.is_some() && opt.unwrap().when == when })
            .map(|it| { it.unwrap().order })
            .max()
            .unwrap_or(0);

        // TODO this is a very strange way to do that, especially take(1)
        self.queue.borrow_mut()
            .iter_mut()
            .filter(|it| { it.is_none() })
            .take(1)
            .for_each(|opt| {
                *opt = Some(Msg { when, order, payload });
            })
    }

    pub fn remove<F>(&self, mut predicate: F) where F: FnMut(&T) -> bool {
        self.queue.borrow_mut()
            .iter_mut()
            .filter(|it| it.is_some() && predicate(&it.unwrap().payload))
            .for_each(|opt| {
                // return to pool
                *opt = None;
            });
    }
}
