//! Critic README TODO

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use axum::Router;
    use axum_login::{
        login_required,
        tower_sessions::{Expiry, MemoryStore, SessionManagerLayer},
        AuthManagerLayerBuilder,
    };
    use critic::app::*;
    use critic::shared::auth::GitlabOauthBackend;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use time::Duration;
    use tracing::{debug, info};
    use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

    let config = match critic::server::config::Config::try_create().await {
        Ok(x) => x,
        Err(e) => {
            panic!("Error reading config: {e}.");
        }
    };
    let config_arc = Arc::new(config);

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    match sqlx::migrate!().run(&config_arc.db).await {
        Ok(_) => {}
        Err(e) => {
            panic!("Error migrating database: {e}");
        }
    }

    let my_crate_filter = EnvFilter::new("critic");
    let subscriber = tracing_subscriber::registry().with(my_crate_filter).with(
        tracing_subscriber::fmt::layer()
            .compact()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_line_number(true)
            .with_filter(config_arc.log_level),
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
    let config_capsule = config_arc.clone();
    let app_core = Router::new()
        .leptos_routes_with_context(
            &config_arc.leptos_options,
            routes,
            move || {
                provide_context::<Arc<critic::server::config::Config>>(config_capsule.clone());
            },
            {
                let leptos_options = config_arc.leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(config_arc.leptos_options.clone());

    // create the auth layer on top of our application core
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(axum_login::tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));
    let backend = GitlabOauthBackend::new(config_arc.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = app_core
        .route_layer(login_required!(GitlabOauthBackend, login_url = "/login"))
        .merge(critic::shared::auth::backend::auth_router())
        .layer(auth_layer);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    info!(
        "listening on http://{}",
        &config_arc.leptos_options.site_addr
    );
    let listener = tokio::net::TcpListener::bind(&config_arc.leptos_options.site_addr)
        .await
        .unwrap();
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
