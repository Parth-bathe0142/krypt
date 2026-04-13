use spin_sdk::http::Router;
use spin_sdk::http::{IntoResponse, Request};
use spin_sdk::http_component;

use crate::routes::{change_password, create_account, delete_account};
use crate::util::pong;

mod routes;
mod util;
mod models;

#[http_component]
fn handle_wakey_server(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::new();
    router.get("/ping", pong);
    
    router.post("/account", create_account);
    router.put("/account", change_password);
    router.delete("/account", delete_account);

    Ok(router.handle(req))
}
