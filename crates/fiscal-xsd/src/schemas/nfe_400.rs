//! Embedded official NF-e/NFC-e 4.00 schemas (PL_010_V1.30).
//!
//! Root `enviNFe_v4.00.xsd` (the authorization lote envelope) pulls in
//! `leiauteNFe` → {`tiposBasico`, `xmldsig-core`, `DFeTiposBasicos`}. Validates
//! an `<enviNFe>` lote (NF-e model 55 and NFC-e model 65 share this schema).

use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "enviNFe_v4.00.xsd",
        include_bytes!("../../schemas/nfe_pl010/enviNFe_v4.00.xsd"),
    ),
    (
        "leiauteNFe_v4.00.xsd",
        include_bytes!("../../schemas/nfe_pl010/leiauteNFe_v4.00.xsd"),
    ),
    (
        "tiposBasico_v4.00.xsd",
        include_bytes!("../../schemas/nfe_pl010/tiposBasico_v4.00.xsd"),
    ),
    (
        "xmldsig-core-schema_v1.01.xsd",
        include_bytes!("../../schemas/nfe_pl010/xmldsig-core-schema_v1.01.xsd"),
    ),
    (
        "DFeTiposBasicos_v1.00.xsd",
        include_bytes!("../../schemas/nfe_pl010/DFeTiposBasicos_v1.00.xsd"),
    ),
];

static NFE_LOTE: XsdSchema = XsdSchema::new("nfe_lote_v400", FILES, "enviNFe_v4.00.xsd");

/// The NF-e/NFC-e 4.00 lote schema bundle. Validate a signed `<enviNFe>` lote:
///
/// ```no_run
/// # let lote_xml = "";
/// if let Err(erros) = fiscal_xsd::schemas::nfe_lote().validate(lote_xml) {
///     eprintln!("lote NF-e inválido: {erros:?}");
/// }
/// ```
pub fn nfe_lote() -> &'static XsdSchema {
    &NFE_LOTE
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_compiles_and_rejects_empty_lote() {
        let err = super::nfe_lote()
            .validate("<enviNFe xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"4.00\"/>")
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
