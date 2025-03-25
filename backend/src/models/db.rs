use serde::Deserialize;

#[derive(Deserialize)]
pub struct WipeParams {
    pub secret: Option<String>,
}
