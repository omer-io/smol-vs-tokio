use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

const MESSAGE: &[u8] = b"ping";
const NUM_MESSAGES: usize = 10000;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:12345").expect("Failed to connect");

    let mut latencies = Vec::new();
    let mut total_sent = 0;

    let start = Instant::now();
    for _ in 0..NUM_MESSAGES {
        let send_time = Instant::now();
        stream.write_all(MESSAGE).unwrap();

        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).unwrap();

        let latency = send_time.elapsed();
        latencies.push(latency);
        total_sent += MESSAGE.len();
    }
    let total_duration = start.elapsed();

    let avg_latency = latencies.iter().map(|d| d.as_nanos()).sum::<u128>() / NUM_MESSAGES as u128;
    let throughput = total_sent as f64 / total_duration.as_secs_f64();

    println!("Average latency: {} ns", avg_latency);
    println!("Throughput: {:.2} bytes/sec", throughput);
}
