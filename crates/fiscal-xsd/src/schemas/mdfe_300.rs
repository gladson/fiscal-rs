//! Embedded official MDF-e 3.00 schemas (PL_MDFe_300a).
//!
//! Root `mdfe_v3.00.xsd` pulls in `mdfeTiposBasico` → the four modal schemas +
//! `tiposGeralMDFe` + `xmldsig-core`. Validates a **signed** `<MDFe>` document
//! (the schema requires the enveloped `ds:Signature`).

use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "mdfe_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfe_v3.00.xsd"),
    ),
    (
        "mdfeTiposBasico_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeTiposBasico_v3.00.xsd"),
    ),
    (
        "tiposGeralMDFe_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/tiposGeralMDFe_v3.00.xsd"),
    ),
    (
        "mdfeModalRodoviario_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeModalRodoviario_v3.00.xsd"),
    ),
    (
        "mdfeModalAereo_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeModalAereo_v3.00.xsd"),
    ),
    (
        "mdfeModalAquaviario_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeModalAquaviario_v3.00.xsd"),
    ),
    (
        "mdfeModalFerroviario_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeModalFerroviario_v3.00.xsd"),
    ),
    (
        "xmldsig-core-schema_v1.01.xsd",
        include_bytes!("../../schemas/mdfe_300/xmldsig-core-schema_v1.01.xsd"),
    ),
];

static MDFE: XsdSchema = XsdSchema::new("mdfe_v300", FILES, "mdfe_v3.00.xsd");

/// The MDF-e 3.00 schema bundle. Validate a signed `<MDFe>` document:
///
/// ```no_run
/// # let signed_mdfe_xml = "";
/// if let Err(erros) = fiscal_xsd::schemas::mdfe().validate(signed_mdfe_xml) {
///     eprintln!("MDF-e inválido: {erros:?}");
/// }
/// ```
pub fn mdfe() -> &'static XsdSchema {
    &MDFE
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_compiles_and_rejects_empty_mdfe() {
        // A bare <MDFe/> is missing infMDFe/Signature — it must fail validation,
        // which proves the full include graph compiled (no schema-load error).
        let err = super::mdfe()
            .validate("<MDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\"/>")
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
