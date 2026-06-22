//! Schemas ABRASF 2.03 (extraídos do WSDL DSF/Sorocaba). Valida o payload
//! `GerarNfseEnvio` (RPS → NFS-e síncrono) antes de transmitir.
//!
//! `nfse.xsd` (namespace `http://www.abrasf.org.br/nfse.xsd`) contém todos os
//! tipos e elementos globais (GerarNfseEnvio, EnviarLoteRpsEnvio, CompNfse, ...)
//! e importa `xmldsig`.

use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "nfse.xsd",
        include_bytes!("../../schemas/abrasf_203/nfse.xsd"),
    ),
    (
        "xmldsig-core-schema.xsd",
        include_bytes!("../../schemas/abrasf_203/xmldsig-core-schema.xsd"),
    ),
];

static GERAR_NFSE: XsdSchema = XsdSchema::new("abrasf203_gerarnfse", FILES, "nfse.xsd");

/// Bundle ABRASF 2.03. Valida `<GerarNfseEnvio>` (e demais elementos do nfse.xsd).
pub fn abrasf_gerar_nfse() -> &'static XsdSchema {
    &GERAR_NFSE
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_compila_e_rejeita_vazio() {
        let err = super::abrasf_gerar_nfse()
            .validate("<GerarNfseEnvio xmlns=\"http://www.abrasf.org.br/nfse.xsd\"/>")
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
