use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;

#[cfg(feature = "rustls-tls")]
#[test]
fn rustls_is_available_for_http_and_websocket() {
    reqwest::Client::builder().use_rustls_tls().build().unwrap();

    fn accept_rustls_stream(stream: MaybeTlsStream<TcpStream>) {
        if let MaybeTlsStream::Rustls(_) = stream {}
    }

    let _ = accept_rustls_stream;
}

#[cfg(feature = "native-tls")]
#[test]
fn native_tls_is_available_for_http_and_websocket() {
    reqwest::Client::builder().use_native_tls().build().unwrap();

    fn accept_native_tls_stream(stream: MaybeTlsStream<TcpStream>) {
        if let MaybeTlsStream::NativeTls(_) = stream {}
    }

    let _ = accept_native_tls_stream;
}
