use std::{num::NonZero, sync::Arc};

use actix_cors::Cors;
use actix_web::{
    dev::Service,
    error::ErrorUnauthorized,
    http::header::AUTHORIZATION,
    web::{scope, Data},
    App, HttpServer,
};
use aiter::{
    api,
    api::learn::DigestOptions,
    error::*,
    utils::crypto::sha256,
    web,
    web::{AppConfig, AppState, NotifyDigestEvent},
    CHANNEL_BUFFER_DEFAULT,
};
use colored::Colorize;
use tokio::sync::{mpsc, Semaphore};

#[derive(clap::Args)]
pub struct ServeCommand {
    #[arg(
        short = 'a',
        long = "address",
        default_value = "127.0.0.1",
        help = "Listening address, change it to 0.0.0.0 to allow remote access, default value is 127.0.0.1"
    )]
    address: String,

    #[arg(
        short = 'b',
        long = "base",
        help = "The base URI for depolyment, for example, setting it to /aiter means deploying to https://<domain>/aiter"
    )]
    base: Option<String>,

    #[arg(
        short = 'P',
        long = "pass",
        default_value = "",
        help = "If pass is set, all requests need to set the SHA-256 encrypted pass to the Bearer token in the AUTHORIZATION header"
    )]
    pass: String,

    #[arg(
        short = 'p',
        long = "port",
        default_value = "6868",
        help = "Listening port, default value is 6868"
    )]
    port: u16,

    #[arg(
        short = 'w',
        long = "workers",
        default_value = "1",
        help = "Multi threads, it is automatically set to the number of CPUs if the value is set to 0, default value is 1"
    )]
    workers: usize,

    #[arg(
        long = "digest-batch",
        default_value = "2",
        help = "Documents digested simultaneously, default value is 2"
    )]
    option_digest_batch: usize,

    #[arg(
        long = "digest-concurrent",
        default_value = "8",
        help = "Concurrency of digesting, default value is 8"
    )]
    option_digest_concurrent: usize,

    #[arg(
        long = "digest-deep",
        help = "Deeply digest and understand the content"
    )]
    option_digest_deep: bool,

    #[arg(long = "skip-digest", help = "Skip digesting when learning")]
    option_skip_digest: bool,
}

impl ServeCommand {
    pub async fn exec(&self) {
        if let Err(err) = self.start_web_server().await {
            println!("{}", err.to_string().red());
        }
    }

    async fn start_web_server(&self) -> AiterResult<()> {
        if let Some(base) = &self.base {
            let mut guard = web::WEB_BASE.write().await;
            *guard = base.to_string();
        }

        let workers = if self.workers > 0 {
            self.workers
        } else {
            std::thread::available_parallelism()
                .unwrap_or(NonZero::new(1).unwrap())
                .get()
        };

        let sha256_pass: Option<String> = if self.pass.is_empty() {
            None
        } else {
            Some(sha256(self.pass.as_bytes()))
        };

        let app_config = AppConfig {
            digest_batch: self.option_digest_batch.max(1),
            digest_concurrent: self.option_digest_concurrent.max(1),
            digest_deep: self.option_digest_deep,
            skip_digest: self.option_skip_digest,
        };

        // Reset all terminated digesting tasks
        let _ = reset_terminated_digesting_tasks().await;

        let server = HttpServer::new(move || {
            let (notify_digest_event_sender, notify_digest_event_receiver) =
                mpsc::channel::<NotifyDigestEvent>(CHANNEL_BUFFER_DEFAULT);

            spawn_digest_queue(app_config.clone(), notify_digest_event_receiver);

            // Spawn to process not digested documents
            spawn_process_not_digested(app_config.clone(), notify_digest_event_sender.clone());

            App::new()
                .app_data(Data::new(AppState {
                    notify_digest_event_sender: notify_digest_event_sender.clone(),
                }))
                .wrap(Cors::permissive())
                .service(
                    scope("/api")
                        .wrap_fn({
                            let sha256_pass = sha256_pass.clone();
                            move |req, srv| {
                                if let Some(sha256_pass) = &sha256_pass {
                                    if let Some(header) = req.headers().get(AUTHORIZATION) {
                                        if let Ok(header_str) = header.to_str() {
                                            let header_parts: Vec<&str> =
                                                header_str.split_whitespace().collect();
                                            if header_parts.len() == 2
                                                && header_parts[0] == "Bearer"
                                                && header_parts[1].to_lowercase() == *sha256_pass
                                            {
                                                return srv.call(req);
                                            }
                                        }
                                    }

                                    Box::pin(async {
                                        Err(ErrorUnauthorized("Bearer token not authenticated"))
                                    })
                                } else {
                                    srv.call(req)
                                }
                            }
                        })
                        .service(web::api::version)
                        .service(
                            scope("/ai")
                                .service(web::api::ai::add)
                                .service(web::api::ai::delete)
                                .service(web::api::ai::list)
                                .service(web::api::ai::rename),
                        )
                        .service(
                            scope("/chat")
                                .service(web::api::chat::index)
                                .service(web::api::chat::clear)
                                .service(web::api::chat::delete)
                                .service(web::api::chat::history),
                        )
                        .service(
                            scope("/doc")
                                .service(web::api::doc::count_part)
                                .service(web::api::doc::delete)
                                .service(web::api::doc::learn)
                                .service(web::api::doc::list)
                                .service(web::api::doc::list_by_ids)
                                .service(web::api::doc::list_digesting_ids)
                                .service(web::api::doc::get_part),
                        )
                        .service(
                            scope("/llm")
                                .service(web::api::llm::active)
                                .service(web::api::llm::config)
                                .service(web::api::llm::delete)
                                .service(web::api::llm::edit)
                                .service(web::api::llm::list)
                                .service(web::api::llm::list_actived_names)
                                .service(web::api::llm::test_chat),
                        )
                        .service(
                            scope("/mem")
                                .service(web::api::mem::stats)
                                .service(web::api::mem::vacuum),
                        )
                        .service(
                            scope("/skill")
                                .service(web::api::skill::add)
                                .service(web::api::skill::adds)
                                .service(web::api::skill::delete)
                                .service(web::api::skill::list),
                        )
                        .service(
                            scope("/tool")
                                .service(web::api::tool::delete_by_toolset)
                                .service(web::api::tool::get)
                                .service(web::api::tool::import)
                                .service(web::api::tool::list_by_ids)
                                .service(web::api::tool::list_toolsets)
                                .service(web::api::tool::query_by_toolset),
                        ),
                )
                .route("/{path:.*}", actix_web::web::get().to(web::serve_webui))
        })
        .workers(workers)
        .bind((self.address.clone(), self.port))?
        .run();

        println!(
            "Aiter is running on http://{}:{}",
            self.address.bold().green(),
            self.port.to_string().bold().green()
        );

        server.await.map_err(AiterError::from)
    }
}

async fn process_not_digested(
    ai: Option<&str>,
    notify_digest_event_sender: mpsc::Sender<NotifyDigestEvent>,
) -> AiterResult<()> {
    let docs = api::mem::doc::list_not_digested(ai).await?;

    for doc in docs {
        notify_digest_event_sender
            .send(NotifyDigestEvent {
                ai: ai.map(|s| s.to_string()),
                doc_id: doc.id,
            })
            .await?;
    }

    Ok(())
}

async fn reset_terminated_digesting_tasks() -> AiterResult<()> {
    for ai in api::ai::list().await? {
        api::mem::doc::reset_not_digested_but_started(Some(&ai.name)).await?;
    }
    api::mem::doc::reset_not_digested_but_started(None).await?;

    Ok(())
}

fn spawn_digest_queue(
    app_config: AppConfig,
    mut notify_digest_event_receiver: mpsc::Receiver<NotifyDigestEvent>,
) {
    let semaphore = Arc::new(Semaphore::new(app_config.digest_batch));

    tokio::spawn(async move {
        while let Some(event) = notify_digest_event_receiver.recv().await {
            if app_config.skip_digest {
                continue;
            }

            let mem_write_event_sender = api::mem::spawn_mem_write(event.ai.as_deref())
                .await
                .expect("Spawn mem write error");

            if let Ok(permit) = semaphore.clone().acquire_owned().await {
                let app_config = app_config.clone();
                let mem_write_event_sender = mem_write_event_sender.clone();

                tokio::spawn(async move {
                    let options = DigestOptions::default()
                        .with_concurrent(app_config.digest_concurrent)
                        .with_deep(app_config.digest_deep);

                    let _ = api::learn::digest_doc(
                        event.ai.as_deref(),
                        &event.doc_id,
                        &options,
                        mem_write_event_sender,
                        None,
                    )
                    .await;

                    drop(permit);
                });
            }
        }
    });
}

fn spawn_process_not_digested(
    app_config: AppConfig,
    notify_digest_event_sender: mpsc::Sender<NotifyDigestEvent>,
) {
    if app_config.skip_digest {
        return;
    }

    tokio::spawn(async move {
        let _ = process_not_digested(None, notify_digest_event_sender.clone()).await;

        if let Ok(ai_list) = api::ai::list().await {
            for ai in ai_list {
                let _ =
                    process_not_digested(Some(&ai.name), notify_digest_event_sender.clone()).await;
            }
        }
    });
}
