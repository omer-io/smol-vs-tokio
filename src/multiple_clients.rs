use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

const MESSAGE: &[u8] = b"ping";
const NUM_MESSAGES_PER_CONN: usize = 10_000;
const NUM_CONNECTIONS: usize = 10;

fn main() {
    let mut handles = Vec::new();

    for conn_id in 0..NUM_CONNECTIONS {
        let handle = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:12345").expect("Failed to connect");

            let mut latencies = Vec::with_capacity(NUM_MESSAGES_PER_CONN);
            let mut total_sent = 0;

            let start = Instant::now();
            for _ in 0..NUM_MESSAGES_PER_CONN {
                let send_time = Instant::now();
                stream.write_all(MESSAGE).unwrap();

                let mut buf = [0u8; 4];
                stream.read_exact(&mut buf).unwrap();

                let latency = send_time.elapsed();
                latencies.push(latency);
                total_sent += MESSAGE.len();
            }
            let duration = start.elapsed();

            let avg_latency = latencies.iter().map(|d| d.as_nanos()).sum::<u128>() / NUM_MESSAGES_PER_CONN as u128;
            let throughput = total_sent as f64 / duration.as_secs_f64();

            println!(
                "Connection {}: Avg latency = {} ns, Throughput = {:.2} bytes/sec",
                conn_id, avg_latency, throughput
            );

            (latencies, total_sent, duration)
        });

        handles.push(handle);
    }

    // Wait for all threads and aggregate stats
    let mut all_latencies = Vec::new();
    let mut total_bytes = 0;
    let mut total_time = Duration::from_secs(0);

    for handle in handles {
        let (latencies, bytes, time) = handle.join().unwrap();
        all_latencies.extend(latencies);
        total_bytes += bytes;
        total_time += time;
    }

    let avg_latency = all_latencies.iter().map(|d| d.as_nanos()).sum::<u128>() / all_latencies.len() as u128;
    let throughput = total_bytes as f64 / total_time.as_secs_f64();

    println!("--- Aggregated ---");
    println!("Total connections: {}", NUM_CONNECTIONS);
    println!("Average latency (all): {} ns", avg_latency);
    println!("Total throughput: {:.2} bytes/sec", throughput);
}
