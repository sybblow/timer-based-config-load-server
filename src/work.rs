use std::time::Duration;
use tokio_timer::{Timer, TimerError};
use futures::Future;

type MyError = TimerError;
type BoxFuture<'a> = Box<Future<Item = (), Error = MyError> + 'a>;

pub struct CycleWorker<'a> {
    call_fn: Box<Fn() + 'a>,
    timer: Box<Timer>,
}

impl<'scope> CycleWorker<'scope> {
    pub fn new<F>(f: F) -> Self
        where F: Fn() + 'scope
    {
        CycleWorker {
            call_fn: Box::new(f),
            timer: Box::new(::tokio_timer::wheel()
                .num_slots(8)
                .max_timeout(Duration::from_secs(2))
                .build()),
        }
    }

    pub fn cycle_run(self) -> BoxFuture<'scope> {
        let ft = self.timer
            .sleep(Duration::from_secs(1))
            .and_then(move |_| {
                // try to do work
                (*self.call_fn)();

                println!("try sleep again");
                self.cycle_run()
            });
        Box::new(ft)
    }
}

#[inline]
pub fn scope_run<'scope, F>(f: F) -> BoxFuture<'scope>
    where F: Fn() + 'scope
{
    let cycle_worker = CycleWorker::new(f);

    cycle_worker.cycle_run()
}

// TODO: stupid compiler hack
#[inline]
pub fn await<F, I, E>(ft: F) -> Result<I, E>
    where F: Future<Item = I, Error = E> + Sized
{
    ft.wait()
}

#[cfg(test)]
mod tests {
    #![cfg_attr(feature = "nightly", allow(unused_unsafe))]

    use super::*;

    #[test]
    fn simple() {
        let message = "simple run once";
        let s = &message;
        let do_it = || {
            println!("{}", s);
        };

        let ft = scope_run(do_it);
        match await(ft) {
            Ok(_) => println!("sleep loop finished"),
            Err(_) => println!("sleep loop failed"),
        };

        println!("still have: {}", message);
    }
}
