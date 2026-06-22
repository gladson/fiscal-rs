//! Schemas NFS-e **São Paulo (PMSP)** v02 (reforma tributária — "v4 paulistana").
//! Valida o `PedidoEnvioLoteRPS` versão 2 (IM 12 dígitos, IBS/CBS).

use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "PedidoEnvioLoteRPS_v02.xsd",
        include_bytes!("../../schemas/saopaulo_v02/PedidoEnvioLoteRPS_v02.xsd"),
    ),
    (
        "TiposNFe_v02.xsd",
        include_bytes!("../../schemas/saopaulo_v02/TiposNFe_v02.xsd"),
    ),
    (
        "xmldsig-core-schema_v02.xsd",
        include_bytes!("../../schemas/saopaulo_v02/xmldsig-core-schema_v02.xsd"),
    ),
];

static LOTE: XsdSchema = XsdSchema::new("sp_lote_rps_v02", FILES, "PedidoEnvioLoteRPS_v02.xsd");

/// Bundle SP v02 (reforma). Valida um `<PedidoEnvioLoteRPS>` versão 2.
pub fn sp_lote_rps_v2() -> &'static XsdSchema {
    &LOTE
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_compila_e_rejeita_vazio() {
        let err = super::sp_lote_rps_v2()
            .validate("<PedidoEnvioLoteRPS xmlns=\"http://www.prefeitura.sp.gov.br/nfe\"/>")
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
