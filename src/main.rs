use std::time::Instant;
use wgpu_n_body::State;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut state = State::new().await;
    let total_runtime = Instant::now();
    let mut i = 0;
    loop {
        i += 1;
        let start_instant = Instant::now();
        state.tick().await;
        state.render(i).await;
        let runtime = start_instant.elapsed().as_secs_f32();
        println!("Finished iteration #{} in {runtime}s. Total runtime - {:?}", i, total_runtime.elapsed());
    }
}

