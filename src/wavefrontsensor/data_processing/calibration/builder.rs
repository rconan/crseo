use serde::{Deserialize, Serialize};

use crate::{GmtBuilder, SourceBuilder};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationBuilder {
    gmt_builder: Option<GmtBuilder>,
    src_builder: Option<SourceBuilder>,
}
impl CalibrationBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn gmt(mut self, gmt: GmtBuilder) -> Self {
        self.gmt_builder = Some(gmt);
        self
    }
    pub fn source(mut self, source: SourceBuilder) -> Self {
        self.src_builder = Some(source);
        self
    }
}
