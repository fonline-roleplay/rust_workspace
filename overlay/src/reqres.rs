use std::{
    thread::{self, JoinHandle},
    sync::Arc,
};
use crossbeam::atomic::AtomicCell;

pub struct Requester<Q, A, E> {
    cells: Arc<Responder<Q, A, E>>,
    handle: JoinHandle<()>,
}
impl<Q: Copy,A, E> Requester<Q, A, E> {
    pub fn new(cells: Arc<Responder<Q, A, E>>, handle: JoinHandle<()>) -> Self {
        Requester{cells, handle}
    }
    pub fn is_free(&self) -> bool {
        self.cells.input.load().is_none()
    }
    pub fn send(&self, input: Q) {
        let old = self.cells.input.swap(Some(input));
        assert!(old.is_none());
        self.handle.thread().unpark();
    }
    pub fn receive(&self) -> Option<(Q, Result<A, E>)> {
        self.cells.output.swap(None)
    }
}

pub struct Responder<Q, A, E> {
    input: AtomicCell<Option<Q>>,
    output: AtomicCell<Option<(Q, Result<A, E>)>>,
}
impl<Q: Copy,A,E> Responder<Q, A,E> {
    pub fn new() -> Self {
        Responder {
            input: AtomicCell::new(None),
            output: AtomicCell::new(None),
        }
    }
    pub fn wait_question(&self) -> Q {
        loop {
            match self.input.load() {
                Some(question) => {
                    return question
                },
                None => {
                    thread::park();
                }
            }
        }
    }
    pub fn set_answer(&self, answer: A) {
        let q = self.input.load().expect("Can't answer with None question");
        self.output.store(Some((q, Ok(answer))));
        self.input.store(None);
    }
    pub fn set_err(&self, err: E) {
        let q = self.input.load().expect("Can't answer with None question");
        self.output.store(Some((q, Err(err))));
        self.input.store(None);
    }
}
