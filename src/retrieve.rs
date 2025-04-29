pub mod doc;
pub mod skill;

#[derive(strum::Display, Clone)]
pub enum RetrieveMethod {
    Fts,
    Vec,
}
