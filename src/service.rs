use std::pin::Pin;

use anyhow::Result;
use askama::Template;
use chrono::{DateTime, Utc};
use http_body_util::Full;
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
    service::Service,
};

use crate::{
    LOCAL_WIDGET_API_PATH,
    schema::{Metadata, MetadataFilter, Order, Pagination},
    vault::Vault,
};

type Body = Full<Bytes>;

const PAGE_LIMITS: [usize; 10] = [5, 10, 15, 20, 25, 50, 100, 250, 500, 1000];
static CSS: &[u8] = include_bytes!("../assets/style.css");
static ALPINE_JS: &[u8] = include_bytes!("../assets/alpine.js");
static CHART_JS: &[u8] = include_bytes!("../assets/chart.js");

fn full<T: Into<Bytes>>(chunk: T) -> Body {
    Full::new(chunk.into())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    pagination: &'a Pagination,
    page_limits: &'a Vec<usize>,
    orders: &'a Vec<(String, String)>,
    filter: &'a MetadataFilter,
    total_count_raw: usize,
    total_count: usize,
    watch_timeline: &'a Vec<DateTime<Utc>>,
    data: &'a Vec<Metadata>,
}

#[derive(Debug)]
pub struct ServiceHandler {
    pub vault: Vault,
}

impl ServiceHandler {
    pub fn run(&self, req: Request<Incoming>) -> Result<Response<Body>> {
        let payload = (req.method(), req.uri().path());
        let mut state = self.vault.state.lock().unwrap();

        log::debug!("{} {}", payload.0, payload.1);

        match payload {
            // index.html
            (&Method::GET, "/") => {
                let query = req.uri().query().unwrap_or("");

                let filter = serde_urlencoded::from_str(query).unwrap();
                let (pagination, data) = state.metadata_table.get_collection(&filter);

                let html = IndexTemplate {
                    pagination: &pagination,
                    page_limits: &PAGE_LIMITS.to_vec(),
                    orders: &Order::collect_key_label_pair(),
                    filter: &filter,
                    total_count_raw: state.metadata_table.total_count_raw(),
                    total_count: state.metadata_table.total_count(),
                    watch_timeline: &state.metadata_table.watch_timeline(),
                    data: &data,
                };
                let res = Response::new(full(html.render().unwrap()));

                Ok(res)
            }
            (&Method::GET, "/style.css") => {
                let res = Response::new(full(Bytes::from_static(CSS)));

                Ok(res)
            }
            (&Method::GET, "/alpine.js") => {
                let res = Response::new(full(Bytes::from_static(ALPINE_JS)));

                Ok(res)
            }
            (&Method::GET, "/chart.js") => {
                let res = Response::new(full(Bytes::from_static(CHART_JS)));

                Ok(res)
            }
            (&Method::GET, "/iframe_api") => {
                let res = Response::new(full(state.youtube.iframe_api_script.clone()));

                Ok(res)
            }
            (&Method::GET, LOCAL_WIDGET_API_PATH) => {
                let res = Response::new(full(state.youtube.widgetapi_script.clone()));

                Ok(res)
            }
            // 404
            _ => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full(""))
                .unwrap()),
        }
    }
}

/// hyper service trait implementation
impl Service<Request<Incoming>> for ServiceHandler {
    type Response = Response<Body>;
    type Error = anyhow::Error;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let res = self.run(req);

        Box::pin(async { res })
    }
}
