mod auth;
mod db;
mod state;

use aide::{
    axum::{
        routing::{get, get_with, post, post_with},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
};
use axum::{Extension, Json};
use cute::c;
use shuttle_runtime::{CustomError, Secrets, SecretStore};
use sqlx::PgPool;
use toml::Table;

use scaffold::{Entry, EntryWithID, Queue, QueueNew};
use auth::types::{AuthBody, AuthPayload, Claims};
use state::MyState;

// async fn hello_world() -> &'static str {
//     "`rsgistry` API\n\
//         \tDocumentation at /api.json"
// }

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[shuttle_runtime::main]
async fn main(
    #[Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:{secrets.PASSWORD}@localhost:19087/postgres"
    )]
    pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let cargo_toml = std::fs::read_to_string("shuttle/Cargo.toml")?
        .parse::<Table>()
        .expect("Unable to read or parse shuttle/Cargo.toml. Is the file missing, or is there a syntax error?");

    let mut api = OpenApi {
        info: Info {
            title: cargo_toml["package"]["name"].as_str().unwrap().to_string(),
            description: Some(cargo_toml["package"]["name"].as_str().unwrap().to_string()),
            ..Info::default()
        },

        ..OpenApi::default()
    };

    let state = MyState { pool, secrets };
    let keys = Entry::get_keys();
    // TODO: document routes
    let router = ApiRouter::new()
        .route("/api.json", get(serve_api))
        // .api_route(
        //     "/",
        //     get_with(hello_world, |o| {
        //         o.id("index")
        //             .description("Returns a hello message")
        //             .response_with::<200, String, _>(|res| {
        //                 res.description("A hello message")
        //                     .example("`rsgistry` API\n\tDocumentation at /api.json")
        //             })
        //     }),
        // )
        .api_route_with(
            "/api/v1/update",
            get_with(db::update::auth_push, |o| 
                o.id("update")
                    .description("Allows an authenticated user to approve an entry from the queue into the registry")
                    .input::<Claims>()
                    .response_with::<200, EntryWithID, _>(|res| {
                        res.description("The Entry as added to the registry")
                    })
                    .response_with::<400, String, _>(|res| {
                        res.description("Unable to add the entry to the registry from the bearer token")
                    })
                    .response_with::<500, String, _>(|res| {
                        res.description("Unable to delete the queued entry. Adding to the registry was successful.")
                    })
            ),
            |r| { r.security_requirement("bearer") }
        )
        .api_route("/api/v1/queue/add", post_with(db::update::enqueue, |o| {
            o.id("queue/add")
                .description("Add an entry to the registry")
                .input::<QueueNew>()
                .response_with::<201, Queue, _>(|res| {
                    res.description("The Queue item as added to the queue")
                })
        }))
        .api_route("/api/v1/queue/peek", get(db::fetch::queue_peek))
        .api_route("/api/v1/queue/fetch", get(db::fetch::queue_fetch))
        .api_route(
            &format!(
                "/api/v1/fetch/{}",
                // List comps look better than maps.
                c![format!(":{}", keys[x]), for x in 0..keys.len()].join("/")
            ),
            get_with(db::fetch::retrieve, |o| {
                Entry::transform_fetch_route( // Basically add the unique params to the spec
                o.id("fetch")
                    .description("Retrieve a unique entry from the registry"))
                    .parameter::<String, _>("name", |f| {f})
            }),
        )
        .api_route("/login", post_with(auth::login, |o| {
            o.id("login")
                .description("Use a client ID and secret to receive a JWT for approving a queue item")
                .input::<AuthPayload>()
                .response_range_with::<4, String, _>(|res| {
                    res.description("Error with authentication")
                })
                .response_with::<500, String, _>(|res| {
                    res.description("Server error")
                })
                .response_with::<200, AuthBody, _>(|res| {
                    res.description("A Bearer token to approve a queue item into the registry")
                })
        }));

    #[cfg(debug_assertions)]
    {
        let router = router.route("/api/v1/debug/update", post(db::update::push));
        Ok(router
            .with_state(state)
            .finish_api(&mut api)
            .layer(Extension(api))
            .into())
    }

    #[cfg(not(debug_assertions))]
    {
        mod assets {
            use aide::axum::{routing::get, ApiRouter};
            use axum::{
                body::{Body, Bytes},
                http::header,
                response::{Html, Response},
            };

            /* Bundle the frontend assets into the binary */
            const INDEX_HTML: &str = include_str!("../../assets/index.html");
            const APP_WASM: &[u8] = include_bytes!("../../assets/rsgistry-leptos_bg.wasm");
            const APP_JS: &str = include_str!("../../assets/rsgistry-leptos.js");

            pub fn inject_ui(router: ApiRouter<()>) -> ApiRouter<()> {
                router
                    .route("/", get(landing))
                    .route("/rsgistry-leptos_bg.wasm", get(get_wasm))
                    .route("/rsgistry-leptos.js", get(get_js))
                    .fallback(landing)
            }

            /* Methods to get frontend assets */
            pub async fn landing() -> Html<&'static str> {
                Html(INDEX_HTML)
            }

            pub async fn get_wasm() -> Response<Body> {
                let bytes = Bytes::from_static(APP_WASM);
                let body: Body = bytes.into();

                Response::builder()
                    .header(header::CONTENT_TYPE, "application/wasm")
                    .body(body)
                    .unwrap()
            }

            pub async fn get_js() -> Response<Body> {
                let bytes = Bytes::from_static(APP_JS.as_bytes());
                let body: Body = bytes.into();

                Response::builder()
                    .header(header::CONTENT_TYPE, "application/javascript;charset=utf-8")
                    .body(body)
                    .unwrap()
            }
        }
        use assets::inject_ui;
        Ok(inject_ui(router
            .with_state(state))
            .finish_api(&mut api)
            .layer(Extension(api))
            .into())
    }
}
