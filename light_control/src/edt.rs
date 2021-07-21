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

impl<T: Copy> EDT<T> {
    pub fn now(&self) -> u32 { self.now.get() }

    pub fn exit(&self) {
        // clearing the queue ends the loop
        self.queue.replace([None; SIZE]);
    }

    /// Processes all events which are due within the advance_time_by from now
    /// returns the time until the next event, which can be used to sleep
    /// TODO replace with mut Fn like remove in EDT
    pub fn process_events(&self, advance_time_by: u32, handler: &dyn Fn(T)) -> u32 {
        let target_time = self.now.get() + advance_time_by;
        loop {
            if self.queue.borrow().iter().all(|msg| { msg.is_none() }) {
                return 0;
            }

            let head = self.peek_head();

            if head.when > target_time {
                // processed all messages due before target_time
                self.now.set(target_time);
                return head.when - target_time;
            } else {
                let next_when = head.when;

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
                handler(head.payload);
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

        // I cannot comprehend loops anymore I guess
        // for opt in self.queue.borrow_mut().iter_mut() {
        //    if opt.is_none() {
        //        *opt = Some(Msg { when, order, payload });
        //        break;
        //    }
        // }
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
