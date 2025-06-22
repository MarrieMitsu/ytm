use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Semaphore, broadcast, mpsc};
use tokio::time::{Duration, sleep};
use ytm::config::Config;
use ytm::schema::load_metadata_from_file;
use ytm::service::ServiceHandler;
use ytm::shutdown::Shutdown;
use ytm::vault::Vault;
use ytm::youtube::load_youtube_components;

const MAX_CONNECTIONS: usize = 250;

/// Listener
struct Listener {
    listener: TcpListener,
    vault: Vault,
    limit_connection: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

impl Listener {
    async fn run(&mut self) -> Result<()> {
        log::debug!("Accepting inbound connections");

        let mut http = http1::Builder::new();
        http.keep_alive(true);

        loop {
            let permit = self.limit_connection.clone().acquire_owned().await.unwrap();
            let mut shutdown = Shutdown::new(self.notify_shutdown.subscribe());

            let socket = self.accept().await?;
            let io = TokioIo::new(socket);

            let service = ServiceHandler {
                vault: self.vault.clone(),
            };
            let connection = http.serve_connection(io, service);

            tokio::spawn(async move {
                tokio::select! {
                    res = connection => {
                        if let Err(e) = res {
                            log::error!("failed to serve connection: {:?}", e);
                        }
                    }
                    _ = shutdown.recv() => {}
                }

                drop(permit);
            });
        }
    }

    async fn accept(&mut self) -> Result<TcpStream> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        log::error!("failed reconnecting too many times. {:?}", err);
                        return Err(err.into());
                    }
                }
            };

            log::info!("reconnecting in {}s", backoff);
            sleep(Duration::from_secs(backoff)).await;

            backoff *= 2;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let env = Env::default()
        .filter_or("YTM_LOG_LEVEL", "info")
        .write_style_or("YTM_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let config = Config::parse();

    log::info!("Preparing files and components...");

    let metadata_table = load_metadata_from_file(&config.file)?;
    let youtube = load_youtube_components().await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;

    log::info!("Listening on http://{}", addr);

    let mut server = Listener {
        listener,
        vault: Vault::new(metadata_table, youtube),
        limit_connection: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        shutdown_complete_tx,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(e) = res {
                log::error!("{:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            log::info!("Shutting down, please wait...");
        }
    }

    let Listener {
        notify_shutdown,
        shutdown_complete_tx,
        ..
    } = server;

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    let _ = shutdown_complete_rx.recv().await;

    Ok(())
}
