//! Embedded official BP-e (model 63) 1.00 schemas (PL_BPe_100b).
//!
//! Root `bpe_v1.00.xsd` pulls in `bpeTiposBasico` → `tiposGeralBPe` +
//! `xmldsig-core`. Validates a **signed** `<BPe>` document.

use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "bpe_v1.00.xsd",
        include_bytes!("../../schemas/bpe_100/bpe_v1.00.xsd"),
    ),
    (
        "bpeTiposBasico_v1.00.xsd",
        include_bytes!("../../schemas/bpe_100/bpeTiposBasico_v1.00.xsd"),
    ),
    (
        "tiposGeralBPe_v1.00.xsd",
        include_bytes!("../../schemas/bpe_100/tiposGeralBPe_v1.00.xsd"),
    ),
    (
        "xmldsig-core-schema_v1.01.xsd",
        include_bytes!("../../schemas/bpe_100/xmldsig-core-schema_v1.01.xsd"),
    ),
];

static BPE: XsdSchema = XsdSchema::new("bpe_v100", FILES, "bpe_v1.00.xsd");

/// The BP-e (model 63) 1.00 schema bundle. Validate a signed `<BPe>`.
pub fn bpe() -> &'static XsdSchema {
    &BPE
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_compiles_and_rejects_empty_bpe() {
        let err = super::bpe()
            .validate("<BPe xmlns=\"http://www.portalfiscal.inf.br/bpe\"/>")
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
