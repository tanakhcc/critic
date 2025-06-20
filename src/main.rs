#[cfg(feature = "ssr")]
mod config;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use critic::app::*;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tracing::{debug, info};
    use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

    let config = match config::Config::try_create().await {
        Ok(x) => x,
        Err(e) => {
            panic!("Error reading config: {e}.");
        },
    };

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let my_crate_filter = EnvFilter::new("critic");
    let subscriber = tracing_subscriber::registry().with(my_crate_filter).with(
        tracing_subscriber::fmt::layer()
            .compact()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_line_number(true)
            .with_filter(config.log_level),
    );
    tracing::subscriber::set_global_default(subscriber).expect("static tracing config");
    debug!("Tracing enabled.");

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    // we want to build our Router:
    // protected (including /api/protected/*server_fn)
    // login layer
    // leptos_routes_with_exclusions (exclude protected and login layer) - this generates all other
    // leptos routes
    let app = Router::new()
        .leptos_routes(&config.leptos_options, routes, {
            let leptos_options = config.leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(config.leptos_options.clone());

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    info!("listening on http://{}", &config.leptos_options.site_addr);
    let listener = tokio::net::TcpListener::bind(&config.leptos_options.site_addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
