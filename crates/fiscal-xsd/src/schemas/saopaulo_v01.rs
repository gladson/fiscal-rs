//! Schemas NFS-e **São Paulo (PMSP)** v01 (layout até 31/12/2025). Valida o
//! `PedidoEnvioLoteRPS` (lote de RPS) antes de transmitir ao `lotenfe.asmx`.
//!
//! Namespace `http://www.prefeitura.sp.gov.br/nfe` (+ `/nfe/tipos`). Importa
//! `TiposNFe_v01.xsd` e `xmldsig-core-schema_v01.xsd` (schemaLocation já no XSD).

use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "PedidoEnvioLoteRPS_v01.xsd",
        include_bytes!("../../schemas/saopaulo_v01/PedidoEnvioLoteRPS_v01.xsd"),
    ),
    (
        "TiposNFe_v01.xsd",
        include_bytes!("../../schemas/saopaulo_v01/TiposNFe_v01.xsd"),
    ),
    (
        "xmldsig-core-schema_v01.xsd",
        include_bytes!("../../schemas/saopaulo_v01/xmldsig-core-schema_v01.xsd"),
    ),
];

static LOTE: XsdSchema = XsdSchema::new("sp_lote_rps_v01", FILES, "PedidoEnvioLoteRPS_v01.xsd");

/// Bundle SP v01. Valida um `<PedidoEnvioLoteRPS>`.
pub fn sp_lote_rps() -> &'static XsdSchema {
    &LOTE
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_compila_e_rejeita_vazio() {
        let err = super::sp_lote_rps()
            .validate("<PedidoEnvioLoteRPS xmlns=\"http://www.prefeitura.sp.gov.br/nfe\"/>")
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
