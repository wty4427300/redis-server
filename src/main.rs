use anyhow::Result;
use std::io;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

const BUF_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:6379";
    let listener = TcpListener::bind(addr).await?;

    loop {
        // 等待新的连接
        let (stream, raddr) = listener.accept().await?;
        info!("accepted connection from {}", raddr);
        // 启动一个新的任务来处理连接
        tokio::spawn(async move {
            if let Err(err) = process_redis_conn(stream).await {
                warn!("error processing conn with {}: {:?}", raddr, err);
            }
        });
    }
}

//Cow<T>借用+copy,读取时借用，修改时copy
async fn process_redis_conn(mut stream: TcpStream) -> Result<()> {
    loop {
        //tcp stream readable
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,

            Ok(n) => {
                info!("read {} bytes", n);
                let line = String::from_utf8_lossy(&buf);
                info!("{:?}", line);
                stream.write_all(b"+OK\r\n").await?
            }
            //ref用来在模式匹配中绑定一个值
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => continue,
                _ => return Err(e.into()),
            },
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_process_redis_conn() -> Result<()> {
        // 连接到 Redis 服务器
        let addr = "localhost:6379";
        let mut stream = TcpStream::connect(addr).await?;
        println!("Connected to {}", addr);

        // 发送一段文字
        let message = "PING\r\n";
        stream.write_all(message.as_bytes()).await?;
        println!("Sent: {}", message.trim());

        // 读取响应
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).await?;
        let response = String::from_utf8_lossy(&buf[..n]);
        println!("Received: {}", response.trim());

        Ok(())
    }
}
