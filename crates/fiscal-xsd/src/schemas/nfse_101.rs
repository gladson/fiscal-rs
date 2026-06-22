//! Embedded official NFS-e Nacional 1.01 schemas (RTC, com grupo IBS/CBS).
//!
//! `DPS_v1.01.xsd` (root da DPS) inclui `tiposComplexos` → `tiposSimples` +
//! `xmldsig-core`. Valida uma **DPS assinada** (Declaração de Prestação de
//! Serviço) antes do envio ao SEFIN Nacional.

use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "DPS_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/DPS_v1.01.xsd"),
    ),
    (
        "tiposComplexos_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposComplexos_v1.01.xsd"),
    ),
    (
        "tiposSimples_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposSimples_v1.01.xsd"),
    ),
    (
        "xmldsig-core-schema.xsd",
        include_bytes!("../../schemas/nfse_101/xmldsig-core-schema.xsd"),
    ),
];

static DPS: XsdSchema = XsdSchema::new("dps_v101", FILES, "DPS_v1.01.xsd");

/// O bundle da DPS (NFS-e Nacional 1.01). Valida uma `<DPS>` assinada.
pub fn dps() -> &'static XsdSchema {
    &DPS
}

/// Bundle do pedido de registro de evento (cancelamento etc.).
static FILES_EVT: &[(&str, &[u8])] = &[
    (
        "pedRegEvento_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/pedRegEvento_v1.01.xsd"),
    ),
    (
        "tiposEventos_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposEventos_v1.01.xsd"),
    ),
    (
        "tiposComplexos_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposComplexos_v1.01.xsd"),
    ),
    (
        "tiposSimples_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposSimples_v1.01.xsd"),
    ),
    (
        "xmldsig-core-schema.xsd",
        include_bytes!("../../schemas/nfse_101/xmldsig-core-schema.xsd"),
    ),
];

static PED_EVT: XsdSchema = XsdSchema::new("ped_evt_v101", FILES_EVT, "pedRegEvento_v1.01.xsd");

/// Valida um `<pedRegEvento>` assinado (cancelamento de NFS-e etc.).
pub fn nfse_evento() -> &'static XsdSchema {
    &PED_EVT
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_compiles_and_rejects_empty_dps() {
        let err = super::dps()
            .validate("<DPS xmlns=\"http://www.sped.fazenda.gov.br/nfse\"/>")
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
