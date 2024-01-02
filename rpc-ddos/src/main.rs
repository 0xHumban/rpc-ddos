use ethers::{providers::{Middleware, Provider,Http}};
use eyre::Result;
use std::{sync::{Arc, Mutex},time::SystemTime};

#[allow(unreachable_code)]
async fn spam_get_block(provider: Provider<Http>, max_threads: i32) -> Result<()>{
    // get last block
    let block_number = provider.get_block_number().await?;
    println!("\nLast block: {}", block_number);

    // get current timestamp, to calculate the average requests number per second
    let total_now = SystemTime::now();

    // Create a channel for communication between threads
    let (sender, _receiver) = std::sync::mpsc::channel();
    let sender = Arc::new(Mutex::new(sender));

    // var share between thread to count requests number
    let total_requests = Arc::new(Mutex::new(0.0));

    // start all threads
    for _ in 0..max_threads {
        // Wrap in an Arc and Mutex to share it between threads
        let sender = Arc::clone(&sender);
        let total_requests_clone = Arc::clone(&total_requests);
        let provider_clone = provider.clone();

        // start async task with tokio::spawn
        tokio::spawn(async move {
            let mut old = block_number;

            loop {
                // get latest block
                let block_number = match provider_clone.get_block_number().await {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("Error getting block number: {:?}", e);
                        continue;
                    }
                };

                // if latest block is a new block
                if old != block_number {
                    println!(
                        "\n|{:?}| New block: {}",
                        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(),
                        old
                    );
                    old = block_number;
                }

                // increment total requests number
                {
                    let mut total_requests_guard = total_requests_clone.lock().unwrap();
                    *total_requests_guard += 1.0;
                }
                sender.lock().unwrap().send(block_number).unwrap();
            }
        });
    }


    // Main thread to show average request number per second
    loop {

        let total_requests_guard = total_requests.lock().unwrap();
        let total_requests_value = *total_requests_guard; // get total requests number

        // every 1000 requests, show average number per second
        if total_requests_value % 1000.0 < 1.0 {
            match total_now.elapsed() {
                Ok(elapsed) => {
                    println!("\n\nTotal requests: {}", total_requests_value);
                    let moy = total_requests_value as f32 / elapsed.as_secs_f32();
                    println!("\nRequests per second: {}", moy);
                }
                Err(e) => {
                    println!("Error: {e:?}")
                }
            }
        }

    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()>{
    let _provider_url = "https://mainnet.infura.io/v3/4846c560765c41629f20dc5a8430d478";

    println!("Enter your HTTP rpc url: ");

    // get user provider
    let mut provider_url: String = String::new();
    std::io::stdin().read_line(&mut provider_url).expect("Enter correct url");

    println!("Enter threads number (250 by default): ");

    // get threads numbers
    let mut user_thread: String = String::new();
    std::io::stdin().read_line(&mut user_thread).expect("Enter correct number");

    let max_threads: i32 = match user_thread.trim().parse() {
        Ok(number) => number,
        Err(e) => {
            eprintln!("Enter correct number, error: {}", e);
            250
        }
    };


    // create provider instance
    let provider = Provider::<Http>::try_from(provider_url)
        .expect("could not instance http provider");

    // start requests
    let _ = spam_get_block(provider, max_threads).await;


    Ok(())
}
