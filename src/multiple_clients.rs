use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

// const MESSAGE: &[u8] = b"ping";
// const MESSAGE: &[u8] = b"hello from client";
const NUM_MESSAGES_PER_CONN: usize = 2;
const NUM_CONNECTIONS: usize = 2;
const START_EPOCH: u64 = 838;
const END_EPOCH: u64 = 840;

fn main() {
    let mut handles = Vec::new();

    for conn_id in 0..NUM_CONNECTIONS {
        let handle = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:12345").expect("Failed to connect");

            let mut all_tvl = Vec::new();
            let mut total_received_bytes = 0;

            let start = Instant::now();

            for msg_id in 0..NUM_MESSAGES_PER_CONN {
                // send request (newline-terminated)
                let request = format!("START {} END {}\n", START_EPOCH, END_EPOCH);
                stream.write_all(request.as_bytes()).unwrap();

                // accumulate until we see a full-batch terminator "END\n"
                let mut accumulator = String::new();
                let mut buf = [0u8; 8192];

                loop {
                    let n = match stream.read(&mut buf) {
                        Ok(0) => break, // server closed
                        Ok(n) => n,
                        Err(_) => break,
                    };
                    total_received_bytes += n;
                    accumulator.push_str(&String::from_utf8_lossy(&buf[..n]));
                    // stop when the server signals end of this batch
                    if accumulator.ends_with("END\n") || accumulator.contains("\nEND\n") {
                        break;
                    }
                }

                // parse all complete lines of this batch
                let mut batch_tvl = Vec::new();
                for line in accumulator.lines() {
                    if line == "END" { continue; }
                    if let Some(rhs) = line.split("=>").nth(1) {
                        if let Some(num_str) = rhs.trim().split_whitespace().last() {
                            if let Ok(val) = num_str.parse::<f64>() {
                                batch_tvl.push(val);
                            }
                        }
                    }
                }
                // println!("accumulator: {}", accumulator);
                // println!("batch_tvl: {:?}", batch_tvl);
                if !batch_tvl.is_empty() {
                    let avg = batch_tvl.iter().sum::<f64>() / batch_tvl.len() as f64;
                    // println!(
                    //     "Conn {} Msg {}: received {} values, avg TVL = {:.2}",
                    //     conn_id, msg_id, batch_tvl.len(), avg
                    // );
                    all_tvl.extend(batch_tvl);
                }
            }

            let duration = start.elapsed();
            (all_tvl, total_received_bytes, duration)
        });

        handles.push(handle);
    }

    // aggregate results
    let mut all_tvl = Vec::new();
    let mut total_bytes = 0usize;
    let mut total_time = Duration::from_secs(0);

    for handle in handles {
        let (vals, bytes, time) = handle.join().unwrap();
        all_tvl.extend(vals);
        total_bytes += bytes;
        total_time += time;
    }

    let avg_tvl = if !all_tvl.is_empty() {
        all_tvl.iter().sum::<f64>() / all_tvl.len() as f64
    } else {
        0.0
    };

    let throughput = if total_time.as_secs_f64() > 0.0 {
        total_bytes as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };

    println!("--- Aggregated ---");
    println!("Total connections: {}", NUM_CONNECTIONS);
    // println!("Total tvl values: {}", all_tvl.len());
    // println!("Average tvl (all): {:.2}", avg_tvl);
    println!("Total throughput: {:.2} bytes/sec", throughput);
    println!("Total time (sum over clients): {:.2?}", total_time);
}