use std::convert::Infallible;

use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};

async fn proxy_service(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let req_url = _req.uri();
    let proxy_url = &format!(
        "http://www.maofly.com/{}?{}",
        req_url.path(),
        (match req_url.query() {
            Some(v) => v,
            None => "",
        })
    );

    let mut req = Request::builder().method(_req.method()).uri(proxy_url);

    for (k, v) in _req.headers() {
        if k != "HOST" && k != "LOCATION" {
            req = req.header(k, v);
        }
    }
    let body = _req.into_body();
    let request = req.body(body).unwrap();

    let result = Client::new().request(request).await.unwrap();

    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse()?;
    let make_service = make_service_fn(|_conn: &AddrStream| async {
        Ok::<_, Infallible>(service_fn(proxy_service))
    });
    let server = Server::bind(&addr).serve(make_service);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}
