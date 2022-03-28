use tokio::io::{ReadBuf};
use std::net::TcpStream;

struct Priv {
    raw: Vec<u8>,
    buf: ReadBuf,
}

impl Priv {
    pub fn new() -> Self {
        Self {

        }
    }
}

fn main() {
    // let mut raw: Vec<u8> = Vec::with_capacity(1024);
    let mut raw = vec![0u8; 1024];
    println!("#0 raw_len({}) slice({})", raw.len(), raw.as_slice().len());

    let mut buf = ReadBuf::new(raw.as_mut_slice());
    println!("#1 {:?} remaining:{}", buf, buf.remaining());

    buf.put_slice(&[1u8,2u8,3u8,4u8,5u8,6u8]);
    println!("#2 {:?}", buf);

    let v1 = vec!["1","2","3","4","5","6","7"];
    let s1 = v1.as_slice();
    let v2 = vec!["1","2","3"];
    let s2 = v2.as_slice();

    if s1[0 .. 0 + s2.len() + 1 ].iter().zip(s2)
        .all(|(x, y)| std::cmp::Ordering::Equal == x.cmp(y)) {
        // Some(std::cmp::Ordering::Equal) => {
            //TODO
            println!("Eq");
        // },
        // _ => {
        //     //TODO
        //     println!("Neq");
        // },
    };
}