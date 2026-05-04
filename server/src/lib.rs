use spin_sdk::http::Router;
use spin_sdk::http::{IntoResponse, Request};
use spin_sdk::http_component;

use crate::routes::{
    change_key, change_password, create_account, delete_account, delete_key, get_key, list_keys,
    login, set_key,
};
use crate::util::pong;

mod encryption;
mod log;
mod rate_limiting;
mod routes;
mod util;

#[http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::new();
    router.get("/ping", pong);

    router.post("/account/login", login);
    router.post("/account", create_account);
    router.put("/account", change_password);
    router.delete("/account", delete_account);

    router.get("/key", get_key);
    router.get("/key/list", list_keys);
    router.post("/key", set_key);
    router.put("/key", change_key);
    router.delete("/key", delete_key);

    Ok(router.handle(req))
}
