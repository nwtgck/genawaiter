use crate::{
    ops::{Coroutine, GeneratorState},
    rc::{
        engine::{advance, Airlock, Next},
        Co,
    },
};
use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};

/// This is a generator which stores its state on the heap.
///
/// _See the module-level docs for examples._
pub struct Gen<Y, R, F: Future> {
    airlock: Airlock<Y, R>,
    future: Pin<Box<F>>,
}

impl<Y, R, F: Future> Gen<Y, R, F> {
    /// Creates a new generator from a function.
    ///
    /// The function accepts a [`Co`] object, and returns a future. Every time
    /// the generator is resumed, the future is polled. Each time the future is
    /// polled, it should do one of two things:
    ///
    /// - Call `Co::yield_()`, and then return `Poll::Pending`.
    /// - Drop the `Co`, and then return `Poll::Ready`.
    ///
    /// Typically this exchange will happen in the context of an `async fn`.
    ///
    /// _See the module-level docs for examples._
    pub fn new(start: impl FnOnce(Co<Y, R>) -> F) -> Self {
        let airlock = Rc::new(RefCell::new(Next::Empty));
        let future = {
            let airlock = airlock.clone();
            Box::pin(start(Co { airlock }))
        };
        Self { airlock, future }
    }

    /// Resumes execution of the generator.
    ///
    /// The argument will become the output of the future returned from
    /// [`Co::yield_`](struct.Co.html#method.yield_).
    ///
    /// If the generator yields a value, `Yielded` is returned. Otherwise,
    /// `Completed` is returned.
    ///
    /// _See the module-level docs for examples._
    pub fn resume_with(&mut self, arg: R) -> GeneratorState<Y, F::Output> {
        *self.airlock.borrow_mut() = Next::Resume(arg);
        advance(self.future.as_mut(), &self.airlock)
    }
}

impl<Y, F: Future> Gen<Y, (), F> {
    /// Resumes execution of the generator.
    ///
    /// If the generator yields a value, `Yielded` is returned. Otherwise,
    /// `Completed` is returned.
    ///
    /// _See the module-level docs for examples._
    pub fn resume(&mut self) -> GeneratorState<Y, F::Output> {
        self.resume_with(())
    }
}

impl<Y, R, F: Future> Coroutine for Gen<Y, R, F> {
    type Yield = Y;
    type Resume = R;
    type Return = F::Output;

    fn resume_with(
        mut self: Pin<&mut Self>,
        arg: R,
    ) -> GeneratorState<Self::Yield, Self::Return> {
        Self::resume_with(&mut *self, arg)
    }
}
