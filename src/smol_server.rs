use async_net::TcpListener;
use smol::{block_on, spawn};
use futures::prelude::*;

fn main() {
    block_on(async {
        let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();
        println!("Smol server running...");

        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            spawn(async move {
                let mut buf = [0u8; 4];
                loop {
                    if stream.read_exact(&mut buf).await.is_err() {
                        break;
                    }
                    if stream.write_all(&buf).await.is_err() {
                        break;
                    }
                }
            }).detach();
        }
    });
}
