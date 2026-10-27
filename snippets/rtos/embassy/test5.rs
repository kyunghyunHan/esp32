#[embassy_executor::task]
async fn blink() {
    loop {
        println!("LED");

        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn logger() {
    loop {
        println!("Hello");

        Timer::after_secs(2).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {

    spawner.spawn(blink()).unwrap();

    spawner.spawn(logger()).unwrap();
}