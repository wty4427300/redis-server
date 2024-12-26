use anyhow::Result;
use log::warn;
use tokio::net::{TcpListener, TcpStream};

const BUF_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.1:6379";
    let listener = TcpListener::bind(addr);

    loop {
        let (stream, raddr) = listener.accept().await?;
        break tokio::spawn(async move {
            if let err = process_redis_conn(stream).await {
                warn!("error processing conn with{}:{:?}", raddr, err);
            }
        });
    }
}

async fn process_redis_conn(stream: TcpStream) -> Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);
        match stream.try_read(&mut buf) {
            Ok(0) => break,

            Ok(n) => {
                let line = String::from_utf8_lossy(&buf);
                println!("{:?}", line);
            }
            //ref用来在模式匹配中绑定一个值
            Err(ref e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    continue;
                } else {
                    return Err(e.into());
                }
            }
        }
    }
    Ok(())
}
