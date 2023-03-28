use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server, StatusCode, Uri,
};
use std::{
    convert::Infallible,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn serve_static_file(path: PathBuf) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

async fn serve_req(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();

    if path.starts_with("/static") {
        let path = path.strip_prefix("/static").unwrap();
        let path = PathBuf::from("build/static").join(path);
        let resp = match serve_static_file(path).await {
            Ok(content) => Response::new(content.into()),
            Err(_) => {
                let body = Body::from("File not found");
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body)
                    .unwrap()
            }
        };
        Ok(resp)
    } else {
        match serve_static_file(PathBuf::from("build/index.html")).await {
            Ok(content) => Ok(Response::new(content.into())),
            Err(_) => {
                let body = Body::from("File not found");
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body)
                    .unwrap())
            }
        }
    }
}

async fn run_server(addr: SocketAddr) {
    println!("Listening on http://{}", addr);
    let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
        {
            Ok::<_, hyper::Error>(service_fn(serve_req))
        }
    }));

    if let Err(e) = serve_future.await {
        eprintln!("server error: {}", e);
    }
}

//실제로 비동기로 돌리는 서버
#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    run_server(addr).await;
}
