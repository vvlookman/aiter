use std::{cmp::Ordering, collections::HashSet, path::Path, time::Instant};

use probminhash::jaccard::compute_probminhash_jaccard;
use tokio::task::JoinHandle;

use crate::{
    db, error::AiterResult, retrieve::RetrieveMethod, utils::text::minhash, Tokenizer,
    RETRIEVE_FTS_LIMIT, RETRIEVE_VEC_LIMIT,
};

pub async fn retrieve_skill(
    method: &RetrieveMethod,
    mem_path: &Path,
    question: &str,
    related_questions: &[String],
    _deep: bool,
) -> AiterResult<Vec<db::mem::skill::SkillEntity>> {
    let mut skill_tuples: RetrievedSkills = vec![];

    let instant = Instant::now();

    let signature_dims = db::mem::get_mem_signature_dims(mem_path);
    let tokenizer = db::mem::get_mem_tokenizer(mem_path);

    let similarity_sig = minhash(question, signature_dims, &tokenizer)?;

    let all_questions: HashSet<String> = std::iter::once(question.to_string())
        .chain(related_questions.iter().cloned())
        .collect();

    let mut handles: Vec<JoinHandle<AiterResult<RetrievedSkills>>> = vec![];

    for q in all_questions {
        let method = method.clone();
        let mem_path = mem_path.to_path_buf();
        let similarity_sig = similarity_sig.clone();

        handles.push(tokio::spawn(async move {
            single_retrieve_skill(
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
        let result = handle.await??;
        for r in result {
            if skill_tuples.iter().any(|s| s.0.id == r.0.id) {
                continue;
            }

            skill_tuples.push(r);
        }
    }

    skill_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    let limit = match method {
        RetrieveMethod::Fts => RETRIEVE_FTS_LIMIT,
        RetrieveMethod::Vec => RETRIEVE_VEC_LIMIT,
    };

    let skills = skill_tuples
        .into_iter()
        .take(limit)
        .map(|(c, _)| c)
        .collect::<Vec<_>>();

    log::debug!(
        "[{}] Retrieved Skills [{:?}]: {:?}",
        method,
        instant.elapsed(),
        &skills.iter().map(|s| s.trigger.clone()).collect::<Vec<_>>()
    );

    Ok(skills)
}

type RetrievedSkills = Vec<(db::mem::skill::SkillEntity, f64)>;

async fn single_retrieve_skill(
    method: &RetrieveMethod,
    mem_path: &Path,
    question: &str,
    signature_dims: usize,
    tokenizer: Tokenizer,
    similarity_sig: &[f32],
) -> AiterResult<RetrievedSkills> {
    let mut result: RetrievedSkills = vec![];

    let skills = match method {
        RetrieveMethod::Fts => {
            let mut hits = db::mem::skill::query_by_search(
                mem_path,
                question,
                RETRIEVE_FTS_LIMIT as u64,
                false,
            )
            .await?;
            if hits.is_empty() {
                hits = db::mem::skill::query_by_search(
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
            db::mem::skill::query_by_signature(mem_path, &question_sig, RETRIEVE_VEC_LIMIT as u64)
                .await?
        }
    };

    for skill in skills {
        let trigger_sig = minhash(&skill.trigger, signature_dims, &tokenizer)?;
        let similarity = compute_probminhash_jaccard(similarity_sig, &trigger_sig);

        result.push((skill, similarity));
    }

    Ok(result)
}
