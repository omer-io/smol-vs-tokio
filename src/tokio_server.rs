use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn main() {

    let rt = tokio::runtime::Builder::new_current_thread()
    .enable_io()
    .build()
    .unwrap();

    rt.block_on(async move {
    let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();
    println!("Tokio server running...");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = [0u8; 4];
            loop {
                if socket.read_exact(&mut buf).await.is_err() {
                    break;
                }
                if socket.write_all(&buf).await.is_err() {
                    break;
                }
            }
        });
    }
    })
}
