// Copyright (c) 2023 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0
//

use clap::Parser;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, Server};
use std::net::SocketAddr;
use std::sync::Arc;

mod cdh;
mod router;
mod ttrpc_proto;
mod utils;

use cdh::{CDHClient, CDH_ROOT};
use router::Router;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

pub const TTRPC_TIMEOUT: i64 = 50 * 1000 * 1000 * 1000;
const DEFAULT_BIND: &str = "127.0.0.1:8006";
const CDH_ADDR: &str = "unix:///run/confidential-containers/cdh.sock";

/// API Server arguments info.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Bind address for API Server
    #[arg(default_value_t = DEFAULT_BIND.to_string(), short, long = "bind")]
    bind: String,

    /// Listen address of CDH TTRPC Service
    #[arg(default_value_t = CDH_ADDR.to_string(), short, long = "cdh_addr")]
    cdh_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Starting API server on {}", args.bind);

    let address: SocketAddr = args.bind.parse().expect("Failed to parse the address");

    let mut router = Router::new();

    router.register_route(
        CDH_ROOT,
        Box::new(CDHClient::new(&args.cdh_addr, vec![Method::GET])?),
    );

    let router = Arc::new(tokio::sync::Mutex::new(router));

    let api_service = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        let local_router = router.clone();

        async move {
            Ok::<_, GenericError>(service_fn(move |req| {
                let local_router = local_router.clone();
                async move { local_router.lock().await.route(remote_addr, req).await }
            }))
        }
    });

    let server = Server::bind(&address).serve(api_service);

    println!("API Server listening on http://{}", args.bind);

    if let Err(e) = server.await {
        eprintln!("API server error: {}", e);
    }

    Ok(())
}
