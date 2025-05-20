use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    path::Path,
    time::Instant,
};

use probminhash::jaccard::compute_probminhash_jaccard;
use tokio::task::JoinHandle;

use crate::{
    RETRIEVE_FRAG_SURROUND, RETRIEVE_FTS_LIMIT, RETRIEVE_VEC_LIMIT, Tokenizer, content, db,
    error::AiterResult, retrieve::RetrieveMethod, utils::text::minhash,
};

pub async fn retrieve_doc_implicit(
    method: &RetrieveMethod,
    mem_path: &Path,
    question: &str,
    related_queries: &[String],
    _deep: bool,
) -> AiterResult<Vec<String>> {
    let mut content_tuples: RetrievedContents = vec![];

    let instant = Instant::now();

    let signature_dims = db::mem::get_mem_signature_dims(mem_path);
    let tokenizer = db::mem::get_mem_tokenizer(mem_path);

    let similarity_sig = minhash(question, signature_dims, &tokenizer)?;

    let all_questions: HashSet<String> = std::iter::once(question.to_string())
        .chain(related_queries.iter().cloned())
        .collect();

    let mut handles: Vec<JoinHandle<AiterResult<RetrievedContents>>> = vec![];

    for q in all_questions {
        let method = method.clone();
        let mem_path = mem_path.to_path_buf();
        let similarity_sig = similarity_sig.clone();

        handles.push(tokio::spawn(async move {
            single_retrieve_doc_implicit(
                &method,
                &mem_path,
                &q,
                signature_dims,
                tokenizer,
                &similarity_sig,
            )
            .await
        }));
    }

    for handle in handles {
        content_tuples.extend(handle.await??);
    }

    content_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    let limit = match method {
        RetrieveMethod::Fts => RETRIEVE_FTS_LIMIT,
        RetrieveMethod::Vec => RETRIEVE_VEC_LIMIT,
    };

    let contents = content_tuples
        .into_iter()
        .take(limit)
        .map(|(c, _)| c)
        .collect::<Vec<_>>();

    log::debug!(
        "[{}] Retrieved Doc implicits [{:?}]: {:?}",
        method,
        instant.elapsed(),
        &contents.iter().map(|c| c.to_string()).collect::<Vec<_>>()
    );

    Ok(contents)
}

pub async fn retrieve_doc_frag(
    method: &RetrieveMethod,
    mem_path: &Path,
    question: &str,
    related_queries: &[String],
    deep: bool,
) -> AiterResult<Vec<String>> {
    let mut content_tuples: RetrievedContents = vec![];

    let instant = Instant::now();

    let signature_dims = db::mem::get_mem_signature_dims(mem_path);
    let tokenizer = db::mem::get_mem_tokenizer(mem_path);

    let similarity_sig = minhash(question, signature_dims, &tokenizer)?;

    let all_questions: HashSet<String> = std::iter::once(question.to_string())
        .chain(related_queries.iter().cloned())
        .collect();

    let mut handles: Vec<JoinHandle<AiterResult<RetrievedContents>>> = vec![];

    for q in all_questions {
        let method = method.clone();
        let mem_path = mem_path.to_path_buf();
        let similarity_sig = similarity_sig.clone();

        handles.push(tokio::spawn(async move {
            single_retrieve_doc_frag(
                &method,
                &mem_path,
                &q,
                signature_dims,
                tokenizer,
                &similarity_sig,
                deep,
            )
            .await
        }));
    }

    for handle in handles {
        content_tuples.extend(handle.await??);
    }

    content_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    let limit = match method {
        RetrieveMethod::Fts => RETRIEVE_FTS_LIMIT,
        RetrieveMethod::Vec => RETRIEVE_VEC_LIMIT,
    };

    let contents = content_tuples
        .into_iter()
        .take(limit)
        .map(|(c, _)| c)
        .collect::<Vec<_>>();

    log::debug!(
        "[{}] Retrieved Doc frags [{:?}]: {:?}",
        method,
        instant.elapsed(),
        &contents.iter().map(|c| c.to_string()).collect::<Vec<_>>()
    );

    Ok(contents)
}

pub async fn retrieve_doc_knl(
    method: &RetrieveMethod,
    mem_path: &Path,
    question: &str,
    related_queries: &[String],
    deep: bool,
) -> AiterResult<Vec<String>> {
    let mut content_tuples: RetrievedContents = vec![];

    let instant = Instant::now();

    let signature_dims = db::mem::get_mem_signature_dims(mem_path);
    let tokenizer = db::mem::get_mem_tokenizer(mem_path);

    let similarity_sig = minhash(question, signature_dims, &tokenizer)?;

    let all_questions: HashSet<String> = std::iter::once(question.to_string())
        .chain(related_queries.iter().cloned())
        .collect();

    let mut handles: Vec<JoinHandle<AiterResult<RetrievedContents>>> = vec![];

    for q in all_questions {
        let method = method.clone();
        let mem_path = mem_path.to_path_buf();
        let similarity_sig = similarity_sig.clone();

        handles.push(tokio::spawn(async move {
            single_retrieve_doc_knl(
                &method,
                &mem_path,
                &q,
                signature_dims,
                tokenizer,
                &similarity_sig,
                deep,
            )
            .await
        }));
    }

    for handle in handles {
        content_tuples.extend(handle.await??);
    }

    content_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    let limit = match method {
        RetrieveMethod::Fts => RETRIEVE_FTS_LIMIT,
        RetrieveMethod::Vec => RETRIEVE_VEC_LIMIT,
    };

    let contents = content_tuples
        .into_iter()
        .take(limit)
        .map(|(c, _)| c)
        .collect::<Vec<_>>();

    log::debug!(
        "[{}] Retrieved KNLs [{:?}]: {:?}",
        method,
        instant.elapsed(),
        &contents.iter().map(|c| c.to_string()).collect::<Vec<_>>()
    );

    Ok(contents)
}

type RetrievedContents = Vec<(String, f64)>;

async fn single_retrieve_doc_implicit(
    method: &RetrieveMethod,
    mem_path: &Path,
    question: &str,
    signature_dims: usize,
    tokenizer: Tokenizer,
    similarity_sig: &[f32],
) -> AiterResult<RetrievedContents> {
    let mut result: RetrievedContents = vec![];

    let doc_implicits = match method {
        RetrieveMethod::Fts => {
            let mut hits = db::mem::doc_implicit::query_by_search(
                mem_path,
                question,
                RETRIEVE_FTS_LIMIT as u64,
                false,
            )
            .await?;
            if hits.is_empty() {
                hits = db::mem::doc_implicit::query_by_search(
                    mem_path,
                    question,
                    RETRIEVE_FTS_LIMIT as u64,
                    true,
                )
                .await?;
            }

            hits
        }
        RetrieveMethod::Vec => {
            let question_sig = minhash(question, signature_dims, &tokenizer)?;
            db::mem::doc_implicit::query_by_signature(
                mem_path,
                &question_sig,
                RETRIEVE_VEC_LIMIT as u64,
            )
            .await?
        }
    };

    let mut docs: HashMap<String, Option<db::mem::doc::DocEntity>> = HashMap::new();
    for doc_implicit in doc_implicits {
        let doc = if let Some(doc) = docs.get(&doc_implicit.doc_id) {
            doc.clone()
        } else {
            let doc = db::mem::doc::get(mem_path, &doc_implicit.doc_id)
                .await
                .unwrap_or(None);
            docs.insert(doc_implicit.doc_id.clone(), doc.clone());
            doc
        };

        let content_with_context = if let Some(doc) = doc {
            format!("**{}** {}", &doc.get_context(), &doc_implicit.content)
        } else {
            doc_implicit.content
        };
        let content_sig = minhash(&content_with_context, signature_dims, &tokenizer)?;
        let similarity = compute_probminhash_jaccard(similarity_sig, &content_sig);

        result.push((content_with_context, similarity));
    }

    Ok(result)
}

async fn single_retrieve_doc_frag(
    method: &RetrieveMethod,
    mem_path: &Path,
    question: &str,
    signature_dims: usize,
    tokenizer: Tokenizer,
    similarity_sig: &[f32],
    deep: bool,
) -> AiterResult<RetrievedContents> {
    let mut result: RetrievedContents = vec![];

    let doc_frags = match method {
        RetrieveMethod::Fts => {
            let mut hits = db::mem::doc_frag::query_by_search(
                mem_path,
                question,
                RETRIEVE_FTS_LIMIT as u64,
                false,
            )
            .await?;
            if hits.is_empty() {
                hits = db::mem::doc_frag::query_by_search(
                    mem_path,
                    question,
                    RETRIEVE_FTS_LIMIT as u64,
                    true,
                )
                .await?;
            }

            hits
        }
        RetrieveMethod::Vec => {
            let question_sig = minhash(question, signature_dims, &tokenizer)?;
            db::mem::doc_frag::query_by_signature(
                mem_path,
                &question_sig,
                RETRIEVE_VEC_LIMIT as u64,
            )
            .await?
        }
    };

    let mut docs: HashMap<String, Option<db::mem::doc::DocEntity>> = HashMap::new();
    for doc_frag in doc_frags {
        let doc = if let Some(doc) = docs.get(&doc_frag.doc_id) {
            doc.clone()
        } else {
            let doc = db::mem::doc::get(mem_path, &doc_frag.doc_id)
                .await
                .unwrap_or(None);
            docs.insert(doc_frag.doc_id.clone(), doc.clone());
            doc
        };

        let frag_content =
            content::frag::decode_content(&doc_frag.content, &doc_frag.content_type)?.to_string();

        let content_with_context = if let Some(doc) = &doc {
            format!("**{}** {}", &doc.get_context(), &frag_content)
        } else {
            frag_content.clone()
        };
        let content_sig = minhash(&content_with_context, signature_dims, &tokenizer)?;
        let similarity = compute_probminhash_jaccard(similarity_sig, &content_sig);

        let mut surround = vec![];
        surround.push(frag_content);

        let frag_surround = if deep {
            2 * RETRIEVE_FRAG_SURROUND as u64
        } else {
            RETRIEVE_FRAG_SURROUND as u64
        };

        if frag_surround > 0 {
            // Pre surround
            for i in 1..=frag_surround {
                if let Some(prev_frag_index) = doc_frag.index.checked_sub(i) {
                    if let Some(prev_frag) = db::mem::doc_frag::get_by_index(
                        mem_path,
                        &doc_frag.doc_id,
                        &doc_frag.seg_id,
                        prev_frag_index,
                    )
                    .await?
                    {
                        let frag_content = content::frag::decode_content(
                            &prev_frag.content,
                            &prev_frag.content_type,
                        )?
                        .to_string();
                        surround.insert(0, frag_content)
                    }
                }
            }

            // Next surround
            for i in 1..=frag_surround {
                let next_frag_index = doc_frag.index + i;
                if let Some(next_frag) = db::mem::doc_frag::get_by_index(
                    mem_path,
                    &doc_frag.doc_id,
                    &doc_frag.seg_id,
                    next_frag_index,
                )
                .await?
                {
                    let frag_content =
                        content::frag::decode_content(&next_frag.content, &next_frag.content_type)?
                            .to_string();
                    surround.push(frag_content)
                }
            }
        }

        let surround_content = surround.join(" ");
        let surround_content_with_context = if let Some(doc) = &doc {
            format!("**{}** {}", &doc.get_context(), &surround_content)
        } else {
            surround_content
        };

        result.push((surround_content_with_context, similarity));
    }

    Ok(result)
}

async fn single_retrieve_doc_knl(
    method: &RetrieveMethod,
    mem_path: &Path,
    question: &str,
    signature_dims: usize,
    tokenizer: Tokenizer,
    similarity_sig: &[f32],
    deep: bool,
) -> AiterResult<RetrievedContents> {
    let mut result: RetrievedContents = vec![];

    let doc_knls = match method {
        RetrieveMethod::Fts => {
            let mut hits = db::mem::doc_knl::query_by_search(
                mem_path,
                question,
                RETRIEVE_FTS_LIMIT as u64,
                false,
            )
            .await?;
            if hits.is_empty() {
                hits = db::mem::doc_knl::query_by_search(
                    mem_path,
                    question,
                    RETRIEVE_FTS_LIMIT as u64,
                    true,
                )
                .await?;
            }

            hits
        }
        RetrieveMethod::Vec => {
            let question_sig = minhash(question, signature_dims, &tokenizer)?;
            db::mem::doc_knl::query_by_signature(mem_path, &question_sig, RETRIEVE_VEC_LIMIT as u64)
                .await?
        }
    };

    let mut docs: HashMap<String, Option<db::mem::doc::DocEntity>> = HashMap::new();
    for doc_knl in doc_knls {
        let doc = if let Some(doc) = docs.get(&doc_knl.doc_id) {
            doc.clone()
        } else {
            let doc = db::mem::doc::get(mem_path, &doc_knl.doc_id)
                .await
                .unwrap_or(None);
            docs.insert(doc_knl.doc_id.clone(), doc.clone());
            doc
        };

        let mut content: Option<String> = None;

        if let Some(implicit_id) = &doc_knl.doc_ref.get("implicit_id") {
            if let Some(implicit) = db::mem::doc_implicit::get(mem_path, implicit_id).await? {
                content = Some(implicit.content);
            }
        } else if let Some(frag_id) = &doc_knl.doc_ref.get("frag_id") {
            if let Some(frag) = db::mem::doc_frag::get(mem_path, frag_id).await? {
                let frag_content =
                    content::frag::decode_content(&frag.content, &frag.content_type)?;
                content = Some(frag_content.to_string());
            }
        } else if let Some(seg_id) = &doc_knl.doc_ref.get("seg_id") {
            if let Some(seg) = db::mem::doc_seg::get(mem_path, seg_id).await? {
                if deep {
                    let seg_content =
                        content::seg::decode_content(&seg.content, &seg.content_type)?;
                    content = Some(seg_content.to_string());
                } else {
                    content = seg.summary;
                }
            }
        }

        if let Some(content) = content {
            let content_with_context = if let Some(doc) = &doc {
                format!(
                    "**{}** {} {}",
                    &doc.get_context(),
                    &doc_knl.trigger,
                    &content
                )
            } else {
                format!("{} {}", &doc_knl.trigger, &content)
            };
            let content_sig = minhash(&content_with_context, signature_dims, &tokenizer)?;
            let similarity = compute_probminhash_jaccard(similarity_sig, &content_sig);

            result.push((content_with_context, similarity));
        }
    }

    Ok(result)
}
