#[cfg(not(feature = "nocloudflare"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}

#[cfg(all(feature = "nocloudflare", not(feature = "cloudflare")))]
#[tokio::main]
async fn main() {
    use audio_recorder::app::*;
    use axum::{
        Router,
        http::{HeaderName, HeaderValue},
    };
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    // Fix: configure any_spawner for server-side Leptos code
    any_spawner::Executor::init_futures_executor().expect("Failed to init any_spawner executor");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // build our application with a route
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler_with_context(
            move || {
                // if you want to add custom headers to the static file handler response,
                // you can do that by providing `ResponseOptions` via context
                let opts = use_context::<leptos_axum::ResponseOptions>().unwrap_or_default();
                opts.insert_header(
                    HeaderName::from_static("cross-origin-opener-policy"),
                    HeaderValue::from_static("same-origin"),
                );
                opts.insert_header(
                    HeaderName::from_static("cross-origin-embedder-policy"),
                    HeaderValue::from_static("require-corp"),
                );
                provide_context(opts);
            },
            shell,
        ))
        .with_state(leptos_options);
    leptos::logging::log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
