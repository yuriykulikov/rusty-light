#[cfg(test)]
mod tests {
    use no_std_compat::cell::RefCell;

    use light_control::edt::EDT;

    #[test]
    fn events_are_appended_to_the_queue() {
        let edt: EDT<u32> = EDT::create();

        // when
        edt.schedule(1000, 1);
        edt.schedule(3000, 2);
        edt.schedule(3000, 3);
        edt.schedule(3000, 4);

        // assert that queue has 4 messages
        assert_eq!(
            edt.queue
                .borrow()
                .iter()
                .filter(|it| { it.is_some() })
                .count(),
            4
        );
    }

    #[test]
    fn events_can_be_removed_with_predicate() {
        let edt: EDT<u32> = EDT::create();

        // given
        edt.schedule(1000, 1);
        edt.schedule(3000, 2);
        edt.schedule(3000, 2);
        edt.schedule(3000, 4);

        // when
        edt.remove(|payload| *payload == 2);

        // then 2 messages are removed, 2 retained
        let retained = edt.queue.borrow().iter().filter(|it| it.is_some()).count();
        assert_eq!(retained, 2);
    }

    #[test]
    fn events_are_handled_based_on_the_time() {
        let edt: EDT<u32> = EDT::create();
        edt.schedule(30, 3);
        edt.schedule(20, 2);
        edt.schedule(10, 1);
        edt.schedule(0, 0);

        let events: RefCell<Vec<u32>> = RefCell::new(vec![]);

        // when time advances 25 milliseconds
        edt.advance_time_by(25, &|payload| {
            events.borrow_mut().push(payload);
        });

        // then three messages are handled, according to their due time
        assert_eq!(events.borrow_mut().remove(0), 0);
        assert_eq!(events.borrow_mut().remove(0), 1);
        assert_eq!(events.borrow_mut().remove(0), 2);

        // then one event message remains
        assert_eq!(
            edt.queue
                .borrow()
                .iter()
                .filter(|it| { it.is_some() })
                .count(),
            1
        );
    }

    #[test]
    fn events_are_handled_based_on_the_insertion_order() {
        let edt: EDT<u32> = EDT::create();
        // these 4 occupy first 4 elements in the array
        edt.schedule(10, 0);
        edt.schedule(10, 1);
        edt.schedule(10, 2);
        edt.schedule(5, 3);

        // this creates a "hole" in the array
        edt.remove(|payload| *payload == 1);
        edt.remove(|payload| *payload == 2);

        // events are scheduled in the "hole"
        edt.schedule(10, 4);
        edt.schedule(10, 5);

        let events: RefCell<Vec<u32>> = RefCell::new(vec![]);

        // when time advances 25 milliseconds
        edt.advance_time_by(25, &|payload| {
            events.borrow_mut().push(payload);
        });

        // then three messages are handled, according to their due time
        assert_eq!(events.borrow_mut().remove(0), 3); // this was due in 5
        assert_eq!(events.borrow_mut().remove(0), 0); // this was due in 10, order 0
        assert_eq!(events.borrow_mut().remove(0), 4); // this was due in 10, order 1
        assert_eq!(events.borrow_mut().remove(0), 5); // this was due in 10, order 2

        // then one event message remains
        assert_eq!(edt.queue.borrow().iter().all(|it| { it.is_none() }), true);
    }
}
