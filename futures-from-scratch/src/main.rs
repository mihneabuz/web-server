mod reactor;
mod executor;
mod myfutures;

use std::time::Duration;

use futures::join;

use executor::block_on;
use myfutures::Timeout;

fn main() {
    let start = std::time::Instant::now();

    let fut1 = async {
        Timeout::new(Duration::from_millis(1000)).await;
        println!("finished 1 at time: {:.2}.", start.elapsed().as_secs_f32());
    };

    let fut2 = async {
        Timeout::new(Duration::from_millis(2000)).await;
        println!("finished 2 at time: {:.2}.", start.elapsed().as_secs_f32());
    };

    let mainfut = async {
        join! { fut1, fut2 };
    };

    block_on(mainfut);
}
