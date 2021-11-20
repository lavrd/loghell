use log::{debug, error, info};
use std::str::{from_utf8, Utf8Error};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

// TODO: Waiting while all loops will be stopped by channels!

enum ProcessDataEvent {
    Close,
    Error(Utf8Error),
}

pub struct Daemon {
    socket_addr: String,
    tx: watch::Sender<bool>,
    rx: watch::Receiver<bool>,
}

impl Daemon {
    pub fn new(socket_addr: String) -> Self {
        let (tx, mut rx) = watch::channel(true);
        Daemon {
            socket_addr,
            tx,
            rx,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.socket_addr).await?;
        let local_addr = listener.local_addr()?;
        info!("socket start at {}", local_addr);

        loop {
            let socket = listener.accept().await?;
            // tokio::spawn(process_socket(socket.0, self.rx.clone()));
            self.fff().await?;
        }
    }

    async fn fff(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::spawn(async move {
            println!(">>>");
            self.rx.changed().await.unwrap();
            println!(">>>");
        });
        Ok(())
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        println!("drop used");
        self.tx.send(true).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!(">>>")
    }
}

async fn process_socket(mut socket: TcpStream, mut rx: watch::Receiver<bool>) {
    let socket_addr = socket.local_addr().unwrap();
    debug!("new client from {}", socket_addr);

    // let data = poll_fn(|cx| {
    //     let mut buf = [0; 1024];
    //     let mut buf = ReadBuf::new(&mut buf);
    //
    //     match socket.poll_peek(cx, &mut buf) {
    //         Poll::Pending => Poll::Pending,
    //         Poll::Ready(Ok((buf))) => Poll::Ready(Ok(buf)),
    //         Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
    //     }
    // }).await.unwrap();
    //
    // let (tx, rx) = tokio::sync::oneshot::channel();
    // tx.send(());
    //

    tokio::select! {
        _ = async {
            loop {
                let mut buf = [0; 1024];
                let n = socket.read(&mut buf).await.unwrap();
                match process_data(n, &buf[0..n], socket_addr.to_string().as_str()) {
                    None => debug!("lll"),
                    Some(ProcessDataEvent::Close) => {
                        debug!("123");
                        return
                    },
                    Some(ProcessDataEvent::Error(e)) => error!("failed to process data : {}", e)
                };
            }
        } => {}
        _ = rx.changed() => {
            println!("terminating accept loop");
            return
        }
    }

    // loop {
    //     let n = match socket.read(&mut buf).await {
    //         Ok(n) if n == 0 => {
    //             debug!("connection with {} socket closed", socket_addr);
    //             return;
    //         }
    //         Ok(n) if n == 5 && buf[0..n] == [255, 244, 255, 253, 6] => {
    //             debug!("telnet connection with {} socket closed", socket_addr);
    //             return;
    //         }
    //         Ok(n) => {
    //             // We use n-2 to remove /r/n.
    //             let data = match from_utf8(&buf[0..n - 2]) {
    //                 Ok(data) => data,
    //                 Err(e) => {
    //                     error!("failed to convert incoming data to string : {}", e);
    //                     return;
    //                 }
    //             };
    //             debug!("new data from {} : {:?}", socket_addr, data);
    //             n
    //         }
    //         Err(e) => {
    //             error!("failed to read from socket; err : {:?}", e);
    //             0
    //         }
    //     };
    //
    //     if let Err(e) = socket.write_all(&buf[0..n]).await {
    //         error!("failed to write to socket; err : {:?}", e);
    //         return;
    //     }
    // };
}

fn process_data(n: usize, buf: &[u8], socket_addr: &str) -> Option<ProcessDataEvent> {
    match n {
        n if n == 0 => {
            debug!("connection with {} socket closed", socket_addr);
            Some(ProcessDataEvent::Close)
        }
        // n if n == 5 && buf[0..n] == [255, 244, 255, 253, 6] => {
        //     debug!("telnet connection with {} socket closed", socket_addr);
        //     None
        // }
        n => {
            // We use n-2 to remove /r/n.
            let data = match from_utf8(&buf[0..n - 2]) {
                Ok(data) => data,
                Err(e) => {
                    error!("failed to convert incoming data to string : {}", e);
                    return Some(ProcessDataEvent::Error(e));
                }
            };
            debug!("new data from {} : {:?}", socket_addr, data);
            None
        }
    }
}
