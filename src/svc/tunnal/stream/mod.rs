// #[cfg(unix)]
// use std::os::unix::io::{AsRawFd, RawFd};
// #[cfg(windows)]
// use std::os::windows::io::{AsRawSocket, RawSocket};
use std::io;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use httparse;

enum HttpTunnalState {
    WAIT_HANDSHAKE,
    WAIT_RESPONSE,
    STREAM,
    EXCEPTION(String),
}

[#[derive(Debug)]]
pub struct HttpTunnalStream<IO> {
    pub(crate) head_len: i32,
    pub(crate) raw: Vec<u8>,
    pub(crate) io: IO,
    pub(crate) state: HttpTunnalState,
}

impl<IO> HttpTunnalStream<IO> {
    fn wants_response(&self) -> bool {
        return self.state == HttpTunnalState::WAIT_RESPONSE;
    }

    fn wants_handshake(&self) -> bool {
        return self.state == HttpTunnalState::WAIT_HANDSHAKE;
    }

    fn head(&mut self) {
        let mut headers = [EMPTY_HEADER; 1];
        let mut response = httparse::Response::new(&mut headers[..]);
        match response.parse(&self.buf[0..self.head_buf_size]) {
            Err(e) => {
                self.state = HttpTunnalState::EXCEPTION(format!("{}", e))
            },
            Ok(res) => {
                //判断是200还是407
            },
        };
    }

    // fn poll_read_priv(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
    //     loop {
    //         match &self.state {
    //             HttpTunnalState::WAIT_RESPONSE => {
    //                 // let raw: Vec<u8> = vec![0u8, 1024];
    //                 // let mut buf = ReadBuf::new(raw.as_mut_slice());
    //                 let ret = Pin::new(&mut self.io).poll_read(self.cx, &mut self.buf) {
    //                     Poll::Read(Ok(())) => Ok(buf.filled().len()),
    //                     Poll::Ready(Err(err)) => Err(err),
    //                     Poll::Pending => Err(io::Error::Kind::WouldBlock.into()),
    //                 };
    //             },
    //             HttpTunnalState::STREAM => {
    //                 Pin::new(&mut self.io).poll_read(self.cx, &mut buf)
    //             },
    //             _ => Ok(buf.filled().len())
    //         }
    //     }
    // }
}

struct HttpTunnalStreamRead<IO> {
    buf: ReadBuf,
    stream: HttpTunnalStream<IO>,
}

impl<IO> HttpTunnalStreamRead<IO> 
where
    IO: AsyncRead + AsyncWrite + Unpin
{
    fn new(stream: HttpTunnalStream<IO>) -> Self {
        Self {
            buf: ReadBuf::new(io.raw.as_mut_slice()),
            stream: stream,
        }
    }

    fn handle_response(&mut self, buf: &[u8]) {
        let head_eof = '\r\n\r\n'.as_bytes();

        for i in self.head_len..buf.len() {
            let slice = buf[i .. i + head_eof.len() + 1];

            if head_eof.len() > slice.len() {
                break;
            } else if slice.iter().zip(head_eof).all(|(x, y)| x == y) {
                //TODO
                self.head();
            } else {
            }
            //TODO 判断是否连接成功
            // self.state = HttpTunnalState::STREAM;
            // break;
        }

        // if self.state == HttpTunnalState::STREAM {
        //     //多接收的数据，写入io
        //     if self.head_len < self.buf_len {
        //         self.io.write_all(self.buf[self.head]);
        //         self.buf_len = self.head_len;
        //     }
        // }
    }
}

impl<IO> AsyncRead for HttpTunnalStreamRead<IO>
where
    IO: AsyncRead + Unpin
{
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        match &self.state {
            HttpTunnalState::WAIT_RESPONSE => {
                // let raw: Vec<u8> = vec![0u8, 1024];
                // let mut buf = ReadBuf::new(raw.as_mut_slice());
                let ret = Pin::new(&mut self.stream.io).poll_read(self.cx, &mut self.buf) {
                    Poll::Read(Ok(())) => Ok(buf.filled().len()),
                    Poll::Ready(Err(err)) => Err(err),
                    Poll::Pending => Err(std::io::Error::Kind::WouldBlock.into()),
                };

                match self.handle_response() {
                    Some(n) => {
                        //需要将数据在buf测移动。
                        buf.put_slice(self.buf.filled()[0 .. n + 1]);
                        Ok(n)
                    },
                    None => Err(std::io::Error::Kind::WouldBlock.into())
                }
            },
            HttpTunnalState::STREAM => {
                Pin::new(&mut self.io).poll_read(self.cx, &mut buf)
            },
            _ => Ok(buf.filled().len())
        }
    }
}

struct HttpTunnalStreamWrite<IO> {
    stream: HttpTunnalStream<IO>,
}

impl<IO> AsyncWrite for HttpTunnalStreamWrite<IO>
where
    IO: AsyncWrite + Unpin
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        self.stream.io.poll_write(cx buf)
    }

    fn poll_write_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[io::IoSlice<'_>]) -> Poll<io::Result<usize>> {
        self.stream.io.poll_write_vectored(cx bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.stream.io.is_write_vectored()
    }

    // `poll_shutdown` on a write half shutdowns the stream in the "write" direction.
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.stream.io.poll_shutdown(cx)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // tcp flush is a no-op
        self.stream.io.poll_flush(cx)
    }
}