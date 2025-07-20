//! Critic README TODO

#![recursion_limit = "256"]

#[cfg(feature = "ssr")]
async fn shutdown_signal(
    handle: axum_server::Handle,
    mut watcher: tokio::sync::watch::Receiver<critic_server::signal_handler::InShutdown>,
) {
    tokio::select! {
        _ = watcher.changed() => {
            tracing::debug!("Shutting down web server now.");
            handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn run_web_server(
    config: std::sync::Arc<critic_server::config::Config>,
    watcher: tokio::sync::watch::Receiver<critic_server::signal_handler::InShutdown>,
    shutdown_tx: tokio::sync::watch::Sender<critic_server::signal_handler::InShutdown>,
) {
    // Generate the list of routes in your Leptos App

    use axum::{Extension, Router};
    use axum_login::{
        login_required,
        tower_sessions::{Expiry, MemoryStore, SessionManagerLayer},
        AuthManagerLayerBuilder,
    };
    use critic::app::*;
    use critic_server::{
        auth::GitlabOauthBackend, signal_handler::InShutdown, upload::upload_router,
    };
    use critic_shared::urls::{STATIC_BASE_URL, UPLOAD_BASE_URL};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use time::Duration;
    let routes = generate_route_list(critic::app::App);

    // we want to build our Router:
    // protected (including /api/protected/*server_fn)
    // login layer
    // leptos_routes_with_exclusions (exclude protected and login layer) - this generates all other
    // leptos routes
    let config_capsule = config.clone();
    let app_core = Router::new()
        .leptos_routes_with_context(
            &config.leptos_options,
            routes,
            move || {
                provide_context::<std::sync::Arc<critic_server::config::Config>>(
                    config_capsule.clone(),
                );
            },
            {
                let leptos_options = config.leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(config.leptos_options.clone());

    // create the auth layer on top of our application core
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(axum_login::tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));
    let backend = GitlabOauthBackend::new(config.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let static_router = match critic_server::static_files::image_dir_router(&config.data_directory)
    {
        Ok(x) => x,
        Err(e) => {
            tracing::error!("Cannot recover when data directory layout is wrong: {e}.");
            tracing::error!("Shutting down NOW");
            shutdown_tx.send_replace(InShutdown::Yes);
            return;
        }
    };
    let app = app_core
        .nest(UPLOAD_BASE_URL, upload_router())
        .route_layer(login_required!(GitlabOauthBackend, login_url = "/login"))
        .merge(critic_server::auth::backend::auth_router())
        .layer(auth_layer)
        .nest(STATIC_BASE_URL, static_router)
        .layer(Extension(config.clone()));

    let shutdown_handle = axum_server::Handle::new();
    let shutdown_future = shutdown_signal(shutdown_handle.clone(), watcher.clone());

    // serve the main app on HTTP
    let web_server_future = axum_server::bind(config.leptos_options.site_addr)
        .handle(shutdown_handle.clone())
        .serve(app.clone().into_make_service());
    tracing::info!("listening on http://{}", &config.leptos_options.site_addr);
    // wait until either some other component shuts down or the webserver shuts down
    tokio::select! {
        r = web_server_future => {
            if let Err(e) = r {
                tracing::error!("Failure while executing http server: {e}. SHUTTING DOWN NOW.");
                shutdown_tx.send_replace(InShutdown::Yes);
            }
        }
        () = shutdown_future => {
        }
    };
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use critic_server::{minification::run_minification, signal_handler::InShutdown};
    use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

    let config = match critic_server::config::Config::try_create().await {
        Ok(x) => x,
        Err(e) => {
            panic!("Error reading config: {e}.");
        }
    };
    let config_arc = Arc::new(config);

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    critic_server::db::migrate(&config_arc.db).await;

    let my_crate_filter = EnvFilter::new("critic");
    let subscriber = tracing_subscriber::registry().with(my_crate_filter).with(
        tracing_subscriber::fmt::layer()
            .compact()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_line_number(true)
            .with_filter(config_arc.log_level),
    );
    tracing::subscriber::set_global_default(subscriber).expect("static tracing config");
    tracing::debug!("Tracing enabled.");

    // cancellation channel
    let (tx, rx) = tokio::sync::watch::channel(InShutdown::No);
    // start the Signal handler
    let signal_handle = tokio::spawn(critic_server::signal_handler::signal_handler(
        rx,
        tx.clone(),
    ));
    let web_server = tokio::spawn(run_web_server(
        config_arc.clone(),
        tx.subscribe(),
        tx.clone(),
    ));
    let minification_service = tokio::task::spawn(run_minification(config_arc, tx.subscribe()));

    // Join the different services
    let (signal_res, web_res, minification_res) =
        tokio::join!(signal_handle, web_server, minification_service);
    match signal_res {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            tracing::error!("Error running the signal handler: {e}");
        }
        Err(e) => {
            tracing::error!("Error joining the signal handler task: {e}");
        }
    };
    if let Err(e) = web_res {
        tracing::error!("Error joining the web server task: {e}");
    };
    if let Err(e) = minification_res {
        tracing::error!("Error joining the minificaiton service: {e}");
    };
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
