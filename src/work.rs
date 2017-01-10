use std::time::Duration;
use std::io;

use futures::stream::Stream;
use tokio_core::reactor::{Core, Interval};

type MyError = io::Error;
type MyBoxStream<'a> = Box<Stream<Item = (), Error = MyError> + 'a>;

// TODO: return BoxStream, and select other io task outside
pub fn interval_run<'scope, F>(f: F) -> io::Result<()>
    where F: Fn() + 'scope
{
    let mut l = Core::new()?;
    let dur = Duration::from_secs(1);

    let interval = Interval::new(dur, &l.handle())?;
    let task = interval.for_each(|()| {
        f();
        println!("try sleep again");
        Ok(())
    });
    l.run(task)?;

    Ok(())
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

        match interval_run(do_it) {
            Ok(_) => println!("sleep loop finished"),
            Err(_) => println!("sleep loop failed"),
        };

        println!("still have: {}", message);
    }
}
