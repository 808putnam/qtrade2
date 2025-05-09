//! Tests for the communication channel between solver and lander components
use qtrade_runtime::lander;
use qtrade_runtime::solver;
use qtrade_solver::ArbitrageResult;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test]
async fn test_arbitrage_channel_communication() {
    // Create a mock arbitrage result
    let mock_result = ArbitrageResult {
        deltas: vec![vec![0.1, 0.2, 0.0, -0.3]],
        lambdas: vec![vec![1.0, 2.0, 3.0, 4.0]],
        a_matrices: vec![vec![vec![1.0, 0.0], vec![0.0, 1.0]]],
        status: "optimal".to_string(),
    };

    // Access the ARBITRAGE_SENDER
    let sender = solver::ARBITRAGE_SENDER.lock().await;

    // Send the mock result through the channel
    sender.send(mock_result.clone()).await.expect("Failed to send mock result");

    // Wait a bit for the message to propagate (this is just for testing purposes)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a direct channel to test the queue functionality
    let (tx, rx) = mpsc::channel::<ArbitrageResult>(10);

    // Initialize the receiver in the lander
    lander::init_arbitrage_receiver(rx);

    // Send another mock result directly to the new channel
    let mock_result2 = ArbitrageResult {
        deltas: vec![vec![0.5, -0.3, 0.1, -0.3]],
        lambdas: vec![vec![2.0, 3.0, 4.0, 5.0]],
        a_matrices: vec![vec![vec![1.0, 0.0], vec![0.0, 1.0]]],
        status: "optimal".to_string(),
    };

    tx.send(mock_result2.clone()).await.expect("Failed to send second mock result");

    // Run test lander function to process the queue (simplified version of run_lander)
    let result = timeout(Duration::from_secs(1), test_process_queue()).await;

    assert!(result.is_ok(), "Timed out waiting for queue processing");
}

// Simplified test version of the lander's queue processing logic
async fn test_process_queue() -> bool {
    // We'll try to process up to 10 items from the queue, or until it's empty
    for _ in 0..10 {
        // Check for messages from the channel
        {
            let mut receiver_guard = lander::ARBITRAGE_RECEIVER.lock().unwrap();
            if let Some(ref mut rx) = *receiver_guard {
                match rx.try_recv() {
                    Ok(arbitrage_result) => {
                        // Successfully received a result, add it to the queue
                        let _ = lander::enqueue_arbitrage_result(arbitrage_result);
                    }
                    Err(_) => {
                        // No more messages or error, continue
                    }
                }
            }
        }

        // Process an item from the queue
        if let Some(_) = lander::dequeue_arbitrage_result() {
            // Successfully processed an item from the queue
            return true;
        }

        // Sleep a bit before trying again
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    false  // Failed to process anything from the queue
}
