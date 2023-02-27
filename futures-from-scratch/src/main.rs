mod reactor;
mod executor;
mod myfutures;

use futures::join;

use executor::block_on;
use myfutures::Task;

fn main() {
    let start = std::time::Instant::now();

    let fut1 = async {
        let val = Task::new(1, 1).await;
        println!("Got {} at time: {:.2}.", val, start.elapsed().as_secs_f32());
    };

    let fut2 = async {
        let val = Task::new(2, 2).await;
        println!("Got {} at time: {:.2}.", val, start.elapsed().as_secs_f32());
    };

    let mainfut = async {
        join! { fut1, fut2 };
    };

    block_on(mainfut);
}
