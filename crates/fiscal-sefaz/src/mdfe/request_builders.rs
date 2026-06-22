//! XML request builders for MDF-e web services.
//!
//! Each builder produces the payload that goes inside `<mdfeDadosMsg>`; the
//! SOAP envelope is added by the transport layer ([`super::soap`]).

use fiscal_core::types::SefazEnvironment;

use super::{MDFE_NAMESPACE, MDFE_VERSION};

/// Strip a leading `<?xml …?>` declaration (and any leading whitespace) so a
/// signed document can be embedded inside an envelope.
fn strip_xml_declaration(xml: &str) -> &str {
    let trimmed = xml.trim_start();
    if let Some(rest) = trimmed.strip_prefix("<?xml") {
        if let Some(idx) = rest.find("?>") {
            return rest[idx + 2..].trim_start();
        }
    }
    trimmed
}

/// Build an MDF-e status request XML (`<consStatServMDFe>`).
///
/// Unlike NF-e's `<consStatServ>`, the MDF-e status payload carries **no**
/// `<cUF>` — only `tpAmb` and the fixed `xServ=STATUS` — because SVRS is the
/// single authorizer.
pub fn build_mdfe_status_request(environment: SefazEnvironment) -> String {
    let tp_amb = environment.as_str();
    format!(
        "<consStatServMDFe xmlns=\"{MDFE_NAMESPACE}\" versao=\"{MDFE_VERSION}\"><tpAmb>{tp_amb}</tpAmb><xServ>STATUS</xServ></consStatServMDFe>"
    )
}

/// Build an MDF-e consultation request XML (`<consSitMDFe>`) for an access key.
///
/// # Panics
///
/// Panics if `access_key` is not exactly 44 ASCII digits.
pub fn build_mdfe_consulta_request(access_key: &str, environment: SefazEnvironment) -> String {
    assert!(
        access_key.len() == 44 && access_key.bytes().all(|b| b.is_ascii_digit()),
        "MDF-e access key must be exactly 44 digits"
    );
    let tp_amb = environment.as_str();
    format!(
        "<consSitMDFe xmlns=\"{MDFE_NAMESPACE}\" versao=\"{MDFE_VERSION}\"><tpAmb>{tp_amb}</tpAmb><xServ>CONSULTAR</xServ><chMDFe>{access_key}</chMDFe></consSitMDFe>"
    )
}

/// Build the synchronous reception payload (`<enviMDFe>`) wrapping a single
/// signed MDF-e document.
///
/// `signed_mdfe_xml` is the complete signed `<MDFe>…</MDFe>` (its XML
/// declaration is stripped automatically).
///
/// The **synchronous** reception service (`MDFeRecepcaoSinc`) receives the bare
/// signed `<MDFe>` document — gzip-compressed by
/// [`super::soap::build_envelope_compressed`] — **not** the `<enviMDFe>`/`idLote`
/// batch wrapper. That wrapper belongs to the *asynchronous* lote service
/// (`MDFeRecepcao`); sending it to the sync endpoint yields SEFAZ cStat 215
/// ("enviMDFe element is not declared"). `lot_id` is therefore unused here and
/// kept only for signature stability / future async use.
///
/// # Panics
///
/// Panics if `signed_mdfe_xml` is empty.
pub fn build_mdfe_recepcao_sinc_payload(signed_mdfe_xml: &str, _lot_id: &str) -> String {
    assert!(
        !signed_mdfe_xml.trim().is_empty(),
        "signed MDF-e XML is required for the reception payload"
    );
    strip_xml_declaration(signed_mdfe_xml).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_request_has_no_cuf() {
        let xml = build_mdfe_status_request(SefazEnvironment::Homologation);
        assert!(xml.starts_with(
            "<consStatServMDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\" versao=\"3.00\">"
        ));
        assert!(xml.contains("<tpAmb>2</tpAmb>"));
        assert!(xml.contains("<xServ>STATUS</xServ>"));
        assert!(!xml.contains("<cUF>"), "MDF-e status carries no cUF");
        assert!(xml.ends_with("</consStatServMDFe>"));
    }

    #[test]
    fn status_request_production_tp_amb() {
        let xml = build_mdfe_status_request(SefazEnvironment::Production);
        assert!(xml.contains("<tpAmb>1</tpAmb>"));
    }

    #[test]
    fn consulta_request_embeds_access_key() {
        let key = "43250612345678000190580010000000011000000017";
        let xml = build_mdfe_consulta_request(key, SefazEnvironment::Production);
        assert!(xml.contains("<xServ>CONSULTAR</xServ>"));
        assert!(xml.contains(&format!("<chMDFe>{key}</chMDFe>")));
        assert!(xml.contains("<tpAmb>1</tpAmb>"));
    }

    #[test]
    #[should_panic(expected = "44 digits")]
    fn consulta_request_rejects_short_key() {
        build_mdfe_consulta_request("123", SefazEnvironment::Homologation);
    }

    #[test]
    fn recepcao_payload_is_bare_mdfe_not_envimdfe() {
        let signed = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><MDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\"><infMDFe Id=\"MDFe43...\"/></MDFe>";
        let xml = build_mdfe_recepcao_sinc_payload(signed, "1");
        // Sync service wants the bare signed <MDFe>, never <enviMDFe> (cStat 215).
        assert!(xml.starts_with("<MDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\">"));
        assert!(!xml.contains("<?xml"), "declaration must be stripped");
        assert!(!xml.contains("enviMDFe"), "no enviMDFe wrapper for sync");
        assert!(!xml.contains("idLote"), "no idLote for sync");
        assert!(xml.ends_with("</MDFe>"));
    }

    #[test]
    #[should_panic(expected = "signed MDF-e XML is required")]
    fn recepcao_payload_rejects_empty() {
        build_mdfe_recepcao_sinc_payload("   ", "1");
    }
}
