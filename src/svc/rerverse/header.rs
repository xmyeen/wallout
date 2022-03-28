/**
 * 
End-to-end 端到端头部
此类头部字段会转发给 请求/响应 的最终接收目标。
必须保存在由缓存生成的响应头部。
必须被转发。
Hop-by-hop 逐跳首部
此类头部字段只对单次转发有效。会因为转发给缓存/代理服务器而失效。
HTTP 1.1 版本之后，如果要使用Hop-by-hop头部字段则需要提供Connection字段。
除了一下8个字段为逐跳字段，其余均为端到端字段。
Connection
Keep-Alive
Proxy-Authenticate
Proxy-Authenrization
Trailer
TE
Tranfer-Encoding
Upgrade
 * 
 * 
 */
use lazy_static::lazy_static;
// use unicase::Ascii;
use hyper::header::{HeaderMap, HeaderValue};

pub fn is_hop_header(name: &str) -> bool {
    use unicase::Ascii;
    
    // A list of the headers, using `unicase` to help us compare without
    // worrying about the case, and `lazy_static!` to prevent reallocation
    // of the vector.
    lazy_static! {
        static ref HOP_HEADERS: Vec<Ascii<&'static str>> = vec![
            Ascii::new("Connection"),
            Ascii::new("Keep-Alive"),
            Ascii::new("Proxy-Authenticate"),
            Ascii::new("Proxy-Authorization"),
            Ascii::new("Te"),
            Ascii::new("Trailers"),
            Ascii::new("Transfer-Encoding"),
            Ascii::new("Upgrade"),
        ];
    }

    HOP_HEADERS.iter().any(|h| h == &name)
}

/// Returns a clone of the headers without the [hop-by-hop headers].
///
/// [hop-by-hop headers]: http://www.w3.org/Protocols/rfc2616/rfc2616-sec13.html
pub fn remove_hop_headers(headers: &HeaderMap<HeaderValue>) -> HeaderMap<HeaderValue> {
    let mut result = HeaderMap::new();
    for (k, v) in headers.iter() {
        if !is_hop_header(k.as_str()) {
            result.insert(k.clone(), v.clone());
        }
    }
    result
}