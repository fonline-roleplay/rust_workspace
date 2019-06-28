use crossbeam::atomic::AtomicCell;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

#[derive(Copy, Clone)]
enum Input<T> {
    Req(T),
    Empty,
    Finish,
}
impl<T> Input<T> {
    fn is_empty(&self) -> bool {
        match self {
            Empty => true,
            _ => false,
        }
    }
}
use self::Input::*;

pub struct Requester<Q, A, E> {
    cells: Arc<Responder<Q, A, E>>,
    handle: JoinHandle<()>,
}
impl<Q: Copy, A, E> Requester<Q, A, E> {
    pub fn new(cells: Arc<Responder<Q, A, E>>, handle: JoinHandle<()>) -> Self {
        Requester { cells, handle }
    }
    pub fn is_free(&self) -> bool {
        self.cells.input.load().is_empty()
    }
    pub fn send(&self, input: Q) {
        let old = self.cells.input.swap(Req(input));
        assert!(old.is_empty());
        self.handle.thread().unpark();
    }
    pub fn receive(&self) -> Option<(Q, Result<A, E>)> {
        self.cells.output.swap(None)
    }
    pub fn finish(self) -> JoinHandle<()> {
        self.cells.input.swap(Finish);
        self.handle.thread().unpark();
        self.handle
    }
}

pub struct Responder<Q, A, E> {
    input: AtomicCell<Input<Q>>,
    output: AtomicCell<Option<(Q, Result<A, E>)>>,
}
impl<Q: Copy, A, E> Responder<Q, A, E> {
    pub fn new() -> Self {
        Responder {
            input: AtomicCell::new(Empty),
            output: AtomicCell::new(None),
        }
    }
    pub fn wait_question(&self) -> Result<Q, ()> {
        loop {
            match self.input.load() {
                Req(question) => {
                    return Ok(question);
                }
                Empty => {
                    thread::park();
                }
                Finish => {
                    return Err(());
                }
            }
        }
    }
    pub fn set_answer(&self, answer: A) -> Result<(), ()> {
        if let Req(q) = self.input.load() {
            self.output.store(Some((q, Ok(answer))));
            self.input.store(Empty);
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn set_err(&self, err: E) -> Result<(), ()> {
        if let Req(q) = self.input.load() {
            self.output.store(Some((q, Err(err))));
            self.input.store(Empty);
            Ok(())
        } else {
            Err(())
        }
    }
}
