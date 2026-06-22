//! XML request builders for CT-e web services.
//!
//! Each builder produces the payload that goes inside `<cteDadosMsg>`; the SOAP
//! envelope is added by the transport layer ([`super::soap`]). Field order is
//! grounded in the official `consStatServCTe` / `consSitCTe` schemas.

use fiscal_core::types::SefazEnvironment;

use super::{CTE_NAMESPACE, CTE_VERSION};

/// Strip a leading `<?xml …?>` declaration so a signed document can be embedded.
fn strip_xml_declaration(xml: &str) -> &str {
    let trimmed = xml.trim_start();
    if let Some(rest) = trimmed.strip_prefix("<?xml") {
        if let Some(idx) = rest.find("?>") {
            return rest[idx + 2..].trim_start();
        }
    }
    trimmed
}

/// Build a CT-e status request XML (`<consStatServCTe>`).
///
/// Unlike MDF-e, CT-e carries a `<cUF>` because the authorizer depends on the
/// issuer state. Schema order: `tpAmb`, `cUF`, `xServ`.
pub fn build_cte_status_request(state_code: &str, environment: SefazEnvironment) -> String {
    let tp_amb = environment.as_str();
    format!(
        "<consStatServCTe xmlns=\"{CTE_NAMESPACE}\" versao=\"{CTE_VERSION}\"><tpAmb>{tp_amb}</tpAmb><cUF>{state_code}</cUF><xServ>STATUS</xServ></consStatServCTe>"
    )
}

/// Build a CT-e consultation request XML (`<consSitCTe>`) for an access key.
///
/// Schema order: `tpAmb`, `xServ`, `chCTe`.
///
/// # Panics
///
/// Panics if `access_key` is not exactly 44 ASCII digits.
pub fn build_cte_consulta_request(access_key: &str, environment: SefazEnvironment) -> String {
    assert!(
        access_key.len() == 44 && access_key.bytes().all(|b| b.is_ascii_digit()),
        "CT-e access key must be exactly 44 digits"
    );
    let tp_amb = environment.as_str();
    format!(
        "<consSitCTe xmlns=\"{CTE_NAMESPACE}\" versao=\"{CTE_VERSION}\"><tpAmb>{tp_amb}</tpAmb><xServ>CONSULTAR</xServ><chCTe>{access_key}</chCTe></consSitCTe>"
    )
}

/// Build the synchronous reception payload for `CTeRecepcaoSincV4`.
///
/// `signed_cte_xml` is the complete signed `<CTe>…</CTe>` (its XML declaration
/// is stripped automatically). The sync service receives the **bare** signed
/// `<CTe>` document — gzip-compressed by [`super::soap::build_envelope_compressed`]
/// — with no batch wrapper.
///
/// # Panics
///
/// Panics if `signed_cte_xml` is empty.
pub fn build_cte_recepcao_sinc_payload(signed_cte_xml: &str) -> String {
    assert!(
        !signed_cte_xml.trim().is_empty(),
        "signed CT-e XML is required for the reception payload"
    );
    strip_xml_declaration(signed_cte_xml).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_request_has_cuf_in_order() {
        let xml = build_cte_status_request("35", SefazEnvironment::Homologation);
        assert!(xml.starts_with(
            "<consStatServCTe xmlns=\"http://www.portalfiscal.inf.br/cte\" versao=\"4.00\">"
        ));
        assert!(xml.contains("<tpAmb>2</tpAmb><cUF>35</cUF><xServ>STATUS</xServ>"));
        assert!(xml.ends_with("</consStatServCTe>"));
    }

    #[test]
    fn consulta_request_embeds_access_key() {
        let key = "35250612345678000190570010000000011000000017";
        let xml = build_cte_consulta_request(key, SefazEnvironment::Production);
        assert!(xml.contains("<tpAmb>1</tpAmb><xServ>CONSULTAR</xServ>"));
        assert!(xml.contains(&format!("<chCTe>{key}</chCTe>")));
    }

    #[test]
    #[should_panic(expected = "44 digits")]
    fn consulta_request_rejects_short_key() {
        build_cte_consulta_request("123", SefazEnvironment::Homologation);
    }

    #[test]
    fn recepcao_payload_is_bare_cte() {
        let signed = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><CTe xmlns=\"http://www.portalfiscal.inf.br/cte\"><infCte Id=\"CTe35...\"/></CTe>";
        let xml = build_cte_recepcao_sinc_payload(signed);
        assert!(xml.starts_with("<CTe xmlns=\"http://www.portalfiscal.inf.br/cte\">"));
        assert!(!xml.contains("<?xml"), "declaration must be stripped");
        assert!(xml.ends_with("</CTe>"));
    }

    #[test]
    #[should_panic(expected = "signed CT-e XML is required")]
    fn recepcao_payload_rejects_empty() {
        build_cte_recepcao_sinc_payload("   ");
    }
}
