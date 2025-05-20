use std::{
    collections::HashSet,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering::SeqCst},
    },
};

use tokio::{
    sync::{mpsc, mpsc::Sender, oneshot},
    task,
    task::JoinHandle,
    time::{Duration, sleep},
};

use crate::{
    AiterError, CHANNEL_BUFFER_DEFAULT, TRUNCATE_PROGRESS_MESSAGE,
    api::learn::*,
    content::doc::DocContent,
    db,
    db::mem::*,
    error::AiterResult,
    learn::digest::{DocDigested, DocDigestor},
    utils::text::truncate_format,
};

pub mod digest;
pub mod utils;

pub async fn digest(
    mem_path: &Path,
    options: &DigestOptions,
    mem_write_event_sender: Sender<MemWriteEvent>,
    digest_event_sender: Option<Sender<DigestEvent>>,
) -> AiterResult<DigestResult> {
    doc::reset_not_digested(mem_path).await?;
    doc_frag::reset_not_digested(mem_path).await?;
    doc_part::reset_not_digested(mem_path).await?;
    doc_seg::reset_not_digested(mem_path).await?;

    if options.retry {
        let mut retry_doc_ids: HashSet<String> = HashSet::new();

        retry_doc_ids.extend(doc_part::list_not_digested_doc_ids(mem_path).await?);
        retry_doc_ids.extend(doc_seg::list_not_digested_doc_ids(mem_path).await?);

        if options.deep {
            retry_doc_ids.extend(doc_frag::list_not_digested_doc_ids(mem_path).await?);
        }

        for doc_id in retry_doc_ids {
            doc::set_not_digested(mem_path, &doc_id).await?;
        }
    }

    let total_doc_todo = doc::count_not_digested(mem_path).await? as usize;
    let total_doc_done = Arc::new(AtomicUsize::new(0));

    let progress_sender = if let Some(digest_event_sender) = &digest_event_sender {
        let digest_event_sender = digest_event_sender.clone();
        let total_doc_done = Arc::clone(&total_doc_done);

        let (progress_sender, mut progress_receiver) =
            mpsc::channel::<DigestEvent>(CHANNEL_BUFFER_DEFAULT);

        tokio::spawn(async move {
            while let Some(event) = progress_receiver.recv().await {
                match event {
                    DigestEvent::Progress(message) => {
                        // Show current progress percentage if there are multiple documents to be processed
                        if total_doc_todo > 1 {
                            let done_percentage = total_doc_done.load(SeqCst).min(total_doc_todo)
                                as f64
                                / total_doc_todo as f64
                                * 100.0;

                            let _ = digest_event_sender
                                .send(DigestEvent::Progress(format!(
                                    "{} [{:.2}%]",
                                    &message, done_percentage
                                )))
                                .await;
                        } else {
                            let _ = digest_event_sender
                                .send(DigestEvent::Progress(message))
                                .await;
                        }
                    }
                }
            }
        });

        Some(progress_sender)
    } else {
        None
    };

    let total_part_todo = Arc::new(AtomicUsize::new(0));
    let total_part_done = Arc::new(AtomicUsize::new(0));
    let total_seg_todo = Arc::new(AtomicUsize::new(0));
    let total_seg_done = Arc::new(AtomicUsize::new(0));
    let total_frag_todo = Arc::new(AtomicUsize::new(0));
    let total_frag_done = Arc::new(AtomicUsize::new(0));

    let mut handles: Vec<JoinHandle<AiterResult<()>>> = vec![];
    for i in 0..options.batch.max(1) {
        let mem_path = mem_path.to_path_buf();
        let mem_write_event_sender = mem_write_event_sender.clone();
        let progress_sender = progress_sender.clone();

        let concurrent = options.concurrent;
        let deep = options.deep;

        let total_doc_done = Arc::clone(&total_doc_done);
        let total_part_todo = Arc::clone(&total_part_todo);
        let total_part_done = Arc::clone(&total_part_done);
        let total_seg_todo = Arc::clone(&total_seg_todo);
        let total_seg_done = Arc::clone(&total_seg_done);
        let total_frag_todo = Arc::clone(&total_frag_todo);
        let total_frag_done = Arc::clone(&total_frag_done);

        let handle: JoinHandle<AiterResult<()>> = task::spawn(async move {
            sleep(Duration::from_secs(i.try_into().unwrap_or(0))).await;

            while let Some(the_doc) = doc::get_not_digested(&mem_path).await? {
                let doc_id = the_doc.id;

                let doc_part_todo = AtomicUsize::new(0);
                let doc_part_done = AtomicUsize::new(0);
                let doc_seg_todo = AtomicUsize::new(0);
                let doc_seg_done = AtomicUsize::new(0);
                let doc_frag_todo = AtomicUsize::new(0);
                let doc_frag_done = AtomicUsize::new(0);

                if let Some(progress_sender) = &progress_sender {
                    let _ = progress_sender
                        .send(DigestEvent::Progress(format!("[{}]", &the_doc.source)))
                        .await;
                }

                {
                    let (resp_sender, resp_receiver) = oneshot::channel();
                    mem_write_event_sender
                        .send(MemWriteEvent::SetDocDigestStart {
                            doc_id: doc_id.clone(),
                            resp_sender,
                        })
                        .await?;
                    let _ = resp_receiver.await?;
                }

                let process = async || {
                    let dgst = Arc::new(DocDigestor::new(
                        &mem_path,
                        &doc_id,
                        mem_write_event_sender.clone(),
                        progress_sender.clone(),
                    ));

                    dgst.load_meta().await?;

                    let (seg_todo, seg_done) = dgst.digest_segs(concurrent).await?;
                    doc_seg_todo.fetch_add(seg_todo, SeqCst);
                    doc_seg_done.fetch_add(seg_done, SeqCst);

                    if deep {
                        let (frag_todo, frag_done) = dgst.digest_frags(concurrent).await?;
                        doc_frag_todo.fetch_add(frag_todo, SeqCst);
                        doc_frag_done.fetch_add(frag_done, SeqCst);
                    }

                    let (part_todo, part_done) = dgst.digest_parts(concurrent).await?;
                    doc_part_todo.fetch_add(part_todo, SeqCst);
                    doc_part_done.fetch_add(part_done, SeqCst);

                    dgst.summarize_doc(concurrent).await?;

                    AiterResult::Ok(DocDigested {
                        part_todo: doc_part_todo.load(SeqCst),
                        part_done: doc_part_done.load(SeqCst),
                        seg_todo: doc_seg_todo.load(SeqCst),
                        seg_done: doc_seg_done.load(SeqCst),
                        frag_todo: doc_frag_todo.load(SeqCst),
                        frag_done: doc_frag_done.load(SeqCst),
                    })
                };

                let process_result = process().await;
                let success = process_result.is_ok();

                {
                    let (resp_sender, resp_receiver) = oneshot::channel();
                    mem_write_event_sender
                        .send(MemWriteEvent::SetDocDigestEnd {
                            doc_id: doc_id.clone(),
                            success,
                            resp_sender,
                        })
                        .await?;
                    let _ = resp_receiver.await?;
                }

                match process_result {
                    Ok(DocDigested {
                        part_todo,
                        part_done,
                        seg_todo,
                        seg_done,
                        frag_todo,
                        frag_done,
                    }) => {
                        total_doc_done.fetch_add(1, SeqCst);
                        total_part_todo.fetch_add(part_todo, SeqCst);
                        total_part_done.fetch_add(part_done, SeqCst);
                        total_seg_todo.fetch_add(seg_todo, SeqCst);
                        total_seg_done.fetch_add(seg_done, SeqCst);
                        total_frag_todo.fetch_add(frag_todo, SeqCst);
                        total_frag_done.fetch_add(frag_done, SeqCst);
                    }

                    Err(err) => {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocDigestError {
                                doc_id: doc_id.clone(),
                                error: err.to_string(),
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }
                }
            }

            Ok(())
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Err(err) = handle.await {
            return Err(err.into());
        }
    }

    Ok(DigestResult {
        doc_count: (total_doc_todo, total_doc_done.load(SeqCst)),
        part_count: (total_part_todo.load(SeqCst), total_part_done.load(SeqCst)),
        seg_size: (total_seg_todo.load(SeqCst), total_seg_done.load(SeqCst)),
        frag_size: (total_frag_todo.load(SeqCst), total_frag_done.load(SeqCst)),
    })
}

pub async fn digest_doc(
    mem_path: &Path,
    doc_id: &str,
    options: &DigestOptions,
    mem_write_event_sender: Sender<MemWriteEvent>,
    digest_event_sender: Option<Sender<DigestEvent>>,
) -> AiterResult<DigestResult> {
    if let Some(the_doc) = doc::get(mem_path, doc_id).await? {
        let progress_sender = if let Some(digest_event_sender) = &digest_event_sender {
            let digest_event_sender = digest_event_sender.clone();

            let (progress_sender, mut progress_receiver) =
                mpsc::channel::<DigestEvent>(CHANNEL_BUFFER_DEFAULT);

            tokio::spawn(async move {
                while let Some(event) = progress_receiver.recv().await {
                    match event {
                        DigestEvent::Progress(message) => {
                            let _ = digest_event_sender
                                .send(DigestEvent::Progress(message))
                                .await;
                        }
                    }
                }
            });

            Some(progress_sender)
        } else {
            None
        };

        let mem_path = mem_path.to_path_buf();
        let mem_write_event_sender = mem_write_event_sender.clone();
        let progress_sender = progress_sender.clone();

        let concurrent = options.concurrent;
        let deep = options.deep;

        let doc_part_todo = AtomicUsize::new(0);
        let doc_part_done = AtomicUsize::new(0);
        let doc_seg_todo = AtomicUsize::new(0);
        let doc_seg_done = AtomicUsize::new(0);
        let doc_frag_todo = AtomicUsize::new(0);
        let doc_frag_done = AtomicUsize::new(0);

        if let Some(progress_sender) = &progress_sender {
            let _ = progress_sender
                .send(DigestEvent::Progress(format!("[{}]", &the_doc.source)))
                .await;
        }

        {
            let (resp_sender, resp_receiver) = oneshot::channel();
            mem_write_event_sender
                .send(MemWriteEvent::SetDocDigestStart {
                    doc_id: doc_id.to_string(),
                    resp_sender,
                })
                .await?;
            let _ = resp_receiver.await?;
        }

        let process = async || {
            let dgst = Arc::new(DocDigestor::new(
                &mem_path,
                doc_id,
                mem_write_event_sender.clone(),
                progress_sender.clone(),
            ));

            dgst.load_meta().await?;

            let (seg_todo, seg_done) = dgst.digest_segs(concurrent).await?;
            doc_seg_todo.fetch_add(seg_todo, SeqCst);
            doc_seg_done.fetch_add(seg_done, SeqCst);

            if deep {
                let (frag_todo, frag_done) = dgst.digest_frags(concurrent).await?;
                doc_frag_todo.fetch_add(frag_todo, SeqCst);
                doc_frag_done.fetch_add(frag_done, SeqCst);
            }

            let (part_todo, part_done) = dgst.digest_parts(concurrent).await?;
            doc_part_todo.fetch_add(part_todo, SeqCst);
            doc_part_done.fetch_add(part_done, SeqCst);

            dgst.summarize_doc(concurrent).await?;

            AiterResult::Ok(DocDigested {
                part_todo: doc_part_todo.load(SeqCst),
                part_done: doc_part_done.load(SeqCst),
                seg_todo: doc_seg_todo.load(SeqCst),
                seg_done: doc_seg_done.load(SeqCst),
                frag_todo: doc_frag_todo.load(SeqCst),
                frag_done: doc_frag_done.load(SeqCst),
            })
        };

        let process_result = process().await;
        let success = process_result.is_ok();

        {
            let (resp_sender, resp_receiver) = oneshot::channel();
            mem_write_event_sender
                .send(MemWriteEvent::SetDocDigestEnd {
                    doc_id: doc_id.to_string(),
                    success,
                    resp_sender,
                })
                .await?;
            let _ = resp_receiver.await?;
        }

        match process_result {
            Ok(DocDigested {
                part_todo,
                part_done,
                seg_todo,
                seg_done,
                frag_todo,
                frag_done,
            }) => Ok(DigestResult {
                doc_count: (1, 1),
                part_count: (part_todo, part_done),
                seg_size: (seg_todo, seg_done),
                frag_size: (frag_todo, frag_done),
            }),

            Err(err) => {
                let (resp_sender, resp_receiver) = oneshot::channel();
                mem_write_event_sender
                    .send(MemWriteEvent::SetDocDigestError {
                        doc_id: doc_id.to_string(),
                        error: err.to_string(),
                        resp_sender,
                    })
                    .await?;
                let _ = resp_receiver.await?;

                Err(err)
            }
        }
    } else {
        Err(AiterError::NotExists(format!("Doc '{doc_id}' not exists")))
    }
}

pub async fn read_doc(
    mem_path: &Path,
    source: &str,
    doc_content: &dyn DocContent,
    mem_write_event_sender: Sender<MemWriteEvent>,
    read_event_sender: Option<Sender<ReadEvent>>,
) -> AiterResult<ReadResult> {
    let doc = doc::Doc::new(source, doc_content)?;
    if let Some(same_doc) = doc::get_same(mem_path, &doc).await? {
        return Ok(ReadResult {
            doc_id: same_doc.id,
            doc_exists: true,
        });
    }

    {
        let (resp_sender, resp_receiver) = oneshot::channel();
        mem_write_event_sender
            .send(MemWriteEvent::InsertDoc {
                doc: doc.clone(),
                resp_sender,
            })
            .await?;
        let _ = resp_receiver.await?;
    }

    if let Some(doc) = doc::get(mem_path, &doc.id).await? {
        let doc_context = doc.get_context();

        let tokenizer = db::mem::get_mem_tokenizer(mem_path);

        let parts = doc_content.split(&tokenizer);
        for (part_index, seg_contents) in parts.iter().enumerate() {
            let part = doc_part::DocPart::new(&doc.id, part_index as u64, None);
            {
                let (resp_sender, resp_receiver) = oneshot::channel();
                mem_write_event_sender
                    .send(MemWriteEvent::UpsertDocPart {
                        doc_part: part.clone(),
                        context: doc_context.clone(),
                        resp_sender,
                    })
                    .await?;
                let _ = resp_receiver.await?;
            }

            for (seg_index, seg_content) in seg_contents.iter().enumerate() {
                let seg = doc_seg::DocSeg::new(
                    &doc.id,
                    &part.id,
                    seg_index as u64,
                    seg_content.as_ref(),
                )?;
                {
                    let (resp_sender, resp_receiver) = oneshot::channel();
                    mem_write_event_sender
                        .send(MemWriteEvent::UpsertDocSeg {
                            doc_seg: seg.clone(),
                            context: doc_context.clone(),
                            resp_sender,
                        })
                        .await?;
                    let _ = resp_receiver.await?;
                }

                if let Some(read_event_sender) = &read_event_sender {
                    let _ = read_event_sender
                        .send(ReadEvent::Progress(
                            truncate_format(
                                &seg_content.to_string(),
                                TRUNCATE_PROGRESS_MESSAGE,
                                true,
                            )
                            .replace("\n", " "),
                        ))
                        .await;
                }

                let frag_contents = seg_content.split(&tokenizer);
                for (frag_index, frag_content) in frag_contents.iter().enumerate() {
                    let frag = doc_frag::DocFrag::new(
                        &doc.id,
                        &part.id,
                        &seg.id,
                        frag_index as u64,
                        frag_content.as_ref(),
                    )?;
                    {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::UpsertDocFrag {
                                doc_frag: frag.clone(),
                                context: doc_context.clone(),
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }
                }
            }
        }

        Ok(ReadResult {
            doc_id: doc.id,
            doc_exists: false,
        })
    } else {
        Err(AiterError::NotExists(format!(
            "Doc '{source}' not saved correctly"
        )))
    }
}
