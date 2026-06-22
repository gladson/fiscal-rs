//! MDF-e event request builders and response parser (leiaute 3.00).
//!
//! Unlike NF-e — where events are wrapped in an `<envEvento>` batch with an
//! `<idLote>` — the `MDFeRecepcaoEvento` service receives a **bare
//! `<eventoMDFe>`** element. The `<infEvento>` is signed in place (the
//! `<Signature>` becomes a direct child of `<eventoMDFe>`), which the client
//! handles via [`crate::client::SefazClient`].
//!
//! Supported events:
//! - **Encerramento** (`110112`) — mandatory at the end of the trip; without it
//!   the MDF-e stays open.
//! - **Cancelamento** (`110111`).
//! - **Inclusão de Condutor** (`110114`).
//! - **Inclusão de DF-e** (`110115`).

use fiscal_core::types::SefazEnvironment;
use serde::{Deserialize, Serialize};

use super::{MDFE_NAMESPACE, MDFE_VERSION};

/// `tpEvento` — Encerramento.
pub const EV_ENCERRAMENTO: u32 = 110112;
/// `tpEvento` — Cancelamento.
pub const EV_CANCELAMENTO: u32 = 110111;
/// `tpEvento` — Inclusão de Condutor.
pub const EV_INCLUSAO_CONDUTOR: u32 = 110114;
/// `tpEvento` — Inclusão de DF-e.
pub const EV_INCLUSAO_DFE: u32 = 110115;

/// Render the issuer tax-id tag (`<CNPJ>` for 14 digits, `<CPF>` for 11).
fn tax_id_tag(tax_id: &str) -> String {
    let digits: String = tax_id.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 11 {
        format!("<CPF>{digits}</CPF>")
    } else {
        format!("<CNPJ>{digits}</CNPJ>")
    }
}

/// Current timestamp with the Brazilian `-03:00` offset, in
/// `AAAA-MM-DDThh:mm:ss-03:00` form (required by `TDateTimeUTC`).
fn now_brt() -> String {
    chrono::Utc::now()
        .with_timezone(&chrono::FixedOffset::west_opt(3 * 3600).unwrap())
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

/// Assemble a complete (unsigned) `<eventoMDFe>` from a pre-rendered
/// `detEvento` inner element (e.g. `<evEncMDFe>…</evEncMDFe>`).
///
/// `c_orgao` is the autorizadora UF code; when `None` it is derived from the
/// first two digits of `ch_mdfe`. The `Id` is `ID{tpEvento}{chMDFe}{seq:02}`.
fn build_evento(
    ch_mdfe: &str,
    tp_evento: u32,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
    c_orgao: Option<&str>,
    det_inner: &str,
) -> String {
    assert!(
        ch_mdfe.len() == 44 && ch_mdfe.bytes().all(|b| b.is_ascii_digit()),
        "MDF-e access key must be exactly 44 digits"
    );
    let id = format!("ID{tp_evento}{ch_mdfe}{seq:02}");
    let c_orgao = c_orgao.unwrap_or(&ch_mdfe[..2]);
    let tp_amb = environment.as_str();
    let tax_tag = tax_id_tag(tax_id);
    let dh_evento = now_brt();

    format!(
        "<eventoMDFe xmlns=\"{MDFE_NAMESPACE}\" versao=\"{MDFE_VERSION}\">\
<infEvento Id=\"{id}\">\
<cOrgao>{c_orgao}</cOrgao>\
<tpAmb>{tp_amb}</tpAmb>\
{tax_tag}\
<chMDFe>{ch_mdfe}</chMDFe>\
<dhEvento>{dh_evento}</dhEvento>\
<tpEvento>{tp_evento}</tpEvento>\
<nSeqEvento>{seq}</nSeqEvento>\
<detEvento versaoEvento=\"{MDFE_VERSION}\">{det_inner}</detEvento>\
</infEvento></eventoMDFe>"
    )
}

/// Build an **Encerramento** (`110112`) event for an authorized MDF-e.
///
/// `dt_enc` is the closing date in `AAAA-MM-DD`; `c_uf`/`c_mun` are the UF and
/// IBGE municipality codes where the trip ended. This event is **mandatory** —
/// an MDF-e that is never closed stays open at SEFAZ.
#[allow(clippy::too_many_arguments)]
pub fn build_mdfe_encerramento(
    ch_mdfe: &str,
    protocol: &str,
    dt_enc: &str,
    c_uf: &str,
    c_mun: &str,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
) -> String {
    let det = format!(
        "<evEncMDFe><descEvento>Encerramento</descEvento><nProt>{protocol}</nProt><dtEnc>{dt_enc}</dtEnc><cUF>{c_uf}</cUF><cMun>{c_mun}</cMun></evEncMDFe>"
    );
    build_evento(
        ch_mdfe,
        EV_ENCERRAMENTO,
        seq,
        tax_id,
        environment,
        None,
        &det,
    )
}

/// Build a **Cancelamento** (`110111`) event for an authorized MDF-e.
///
/// # Panics
///
/// Panics if `justification` is shorter than 15 or longer than 255 characters
/// (SEFAZ `xJust` constraint).
pub fn build_mdfe_cancelamento(
    ch_mdfe: &str,
    protocol: &str,
    justification: &str,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
) -> String {
    let len = justification.chars().count();
    assert!(
        (15..=255).contains(&len),
        "cancellation justification (xJust) must be 15–255 chars, got {len}"
    );
    let det = format!(
        "<evCancMDFe><descEvento>Cancelamento</descEvento><nProt>{protocol}</nProt><xJust>{justification}</xJust></evCancMDFe>"
    );
    build_evento(
        ch_mdfe,
        EV_CANCELAMENTO,
        seq,
        tax_id,
        environment,
        None,
        &det,
    )
}

/// Build an **Inclusão de Condutor** (`110114`) event.
pub fn build_mdfe_inclusao_condutor(
    ch_mdfe: &str,
    driver_name: &str,
    driver_cpf: &str,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
) -> String {
    let cpf: String = driver_cpf.chars().filter(|c| c.is_ascii_digit()).collect();
    let det = format!(
        "<evIncCondutorMDFe><descEvento>Inclusao Condutor</descEvento><condutor><xNome>{driver_name}</xNome><CPF>{cpf}</CPF></condutor></evIncCondutorMDFe>"
    );
    build_evento(
        ch_mdfe,
        EV_INCLUSAO_CONDUTOR,
        seq,
        tax_id,
        environment,
        None,
        &det,
    )
}

/// A discharge group for an **Inclusão de DF-e** event: a municipality plus the
/// NF-e access keys discharged there.
#[derive(Debug, Clone)]
pub struct IncDfeDischarge<'a> {
    /// IBGE code of the discharge municipality (`cMunDescarga`).
    pub c_mun_descarga: &'a str,
    /// Name of the discharge municipality (`xMunDescarga`).
    pub x_mun_descarga: &'a str,
    /// NF-e access keys (`chNFe`) discharged in this municipality.
    pub nfe_keys: &'a [&'a str],
}

/// Build an **Inclusão de DF-e** (`110115`) event, adding documents to an
/// already-authorized MDF-e.
///
/// `c_mun_carrega`/`x_mun_carrega` are the loading municipality; each
/// [`IncDfeDischarge`] lists NF-e keys per discharge municipality. `chNFe` is a
/// direct child of `infDoc` (there is no `infNFe` wrapper).
///
/// # Panics
///
/// Panics if `discharges` is empty.
#[allow(clippy::too_many_arguments)]
pub fn build_mdfe_inclusao_dfe(
    ch_mdfe: &str,
    protocol: &str,
    c_mun_carrega: &str,
    x_mun_carrega: &str,
    discharges: &[IncDfeDischarge<'_>],
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
) -> String {
    assert!(
        !discharges.is_empty(),
        "Inclusão de DF-e requires at least one discharge municipality"
    );
    let mut inf_docs = String::new();
    for d in discharges {
        inf_docs.push_str(&format!(
            "<infDoc><cMunDescarga>{}</cMunDescarga><xMunDescarga>{}</xMunDescarga>",
            d.c_mun_descarga, d.x_mun_descarga
        ));
        for key in d.nfe_keys {
            inf_docs.push_str(&format!("<chNFe>{key}</chNFe>"));
        }
        inf_docs.push_str("</infDoc>");
    }
    let det = format!(
        "<evIncDFeMDFe><descEvento>Inclusao DF-e</descEvento><nProt>{protocol}</nProt><cMunCarrega>{c_mun_carrega}</cMunCarrega><xMunCarrega>{x_mun_carrega}</xMunCarrega>{inf_docs}</evIncDFeMDFe>"
    );
    build_evento(
        ch_mdfe,
        EV_INCLUSAO_DFE,
        seq,
        tax_id,
        environment,
        None,
        &det,
    )
}

/// Parsed result of an MDF-e event (`retEventoMDFe`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MdfeEventResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Event type (`tpEvento`), echoed back when registered.
    pub event_type: Option<String>,
    /// MDF-e access key (`chMDFe`), echoed back when registered.
    pub access_key: Option<String>,
    /// Protocol number (`nProt`), present once the event is registered.
    pub protocol_number: Option<String>,
    /// Timestamp the event was registered (`dhRegEvento`).
    pub registered_at: Option<String>,
    /// Signed `<eventoMDFe>` XML that was sent, for archival. Populated by the
    /// client; empty when produced directly by the parser.
    pub signed_event_xml: String,
    /// Full raw SEFAZ response, for archival. Populated by the client; empty
    /// when produced directly by the parser.
    pub raw_response: String,
}

impl MdfeEventResponse {
    /// Whether the event was accepted (`cStat` 135 = registered and linked, or
    /// 136 = registered but not linked). Both are SEFAZ success outcomes.
    pub fn is_registered(&self) -> bool {
        matches!(self.status_code.as_str(), "135" | "136")
    }
}

/// Parse an MDF-e event response (`retEventoMDFe`).
///
/// # Errors
///
/// Returns `FiscalError::XmlParsing` if `<cStat>` is missing.
pub fn parse_mdfe_event_response(xml: &str) -> Result<MdfeEventResponse, fiscal_core::FiscalError> {
    use fiscal_core::xml_utils::extract_xml_tag_value;

    // Reuse the namespace-tolerant SOAP strip from the sibling parser module.
    let body = super::response_parsers::strip_soap(xml);

    let status_code = extract_xml_tag_value(&body, "cStat").ok_or_else(|| {
        fiscal_core::FiscalError::XmlParsing("missing <cStat> in MDF-e event response".into())
    })?;
    let status_message = extract_xml_tag_value(&body, "xMotivo").unwrap_or_default();

    Ok(MdfeEventResponse {
        status_code,
        status_message,
        event_type: extract_xml_tag_value(&body, "tpEvento"),
        access_key: extract_xml_tag_value(&body, "chMDFe"),
        protocol_number: extract_xml_tag_value(&body, "nProt"),
        registered_at: extract_xml_tag_value(&body, "dhRegEvento"),
        signed_event_xml: String::new(),
        raw_response: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "43250612345678000190580010000000011000000017";

    #[test]
    fn encerramento_structure_and_id() {
        let xml = build_mdfe_encerramento(
            KEY,
            "143250000000123",
            "2026-06-05",
            "43",
            "4314902",
            1,
            "12345678000190",
            SefazEnvironment::Production,
        );
        assert!(xml.starts_with(
            "<eventoMDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\" versao=\"3.00\">"
        ));
        assert!(xml.contains(&format!("<infEvento Id=\"ID110112{KEY}01\">")));
        // cOrgao derived from chMDFe[..2] = 43.
        assert!(xml.contains("<cOrgao>43</cOrgao>"));
        assert!(xml.contains("<CNPJ>12345678000190</CNPJ>"));
        assert!(xml.contains("<detEvento versaoEvento=\"3.00\">"));
        assert!(xml.contains(
            "<evEncMDFe><descEvento>Encerramento</descEvento><nProt>143250000000123</nProt><dtEnc>2026-06-05</dtEnc><cUF>43</cUF><cMun>4314902</cMun></evEncMDFe>"
        ));
        assert!(xml.ends_with("</infEvento></eventoMDFe>"));
        assert!(
            !xml.contains("<envEvento"),
            "MDF-e events have no envEvento"
        );
    }

    #[test]
    fn cancelamento_carries_justification() {
        let xml = build_mdfe_cancelamento(
            KEY,
            "143250000000123",
            "Cancelamento por erro de digitacao no manifesto",
            2,
            "12345678000190",
            SefazEnvironment::Homologation,
        );
        assert!(xml.contains("<tpEvento>110111</tpEvento>"));
        assert!(xml.contains("<nSeqEvento>2</nSeqEvento>"));
        assert!(xml.contains("Id=\"ID110111"));
        assert!(xml.contains("<evCancMDFe><descEvento>Cancelamento</descEvento>"));
        assert!(xml.contains("<tpAmb>2</tpAmb>"));
    }

    #[test]
    #[should_panic(expected = "15–255")]
    fn cancelamento_rejects_short_justification() {
        build_mdfe_cancelamento(
            KEY,
            "1",
            "curta",
            1,
            "12345678000190",
            SefazEnvironment::Homologation,
        );
    }

    #[test]
    fn inclusao_condutor_structure() {
        let xml = build_mdfe_inclusao_condutor(
            KEY,
            "Jose da Silva",
            "123.456.789-09",
            1,
            "12345678000190",
            SefazEnvironment::Production,
        );
        assert!(xml.contains("<tpEvento>110114</tpEvento>"));
        assert!(xml.contains(
            "<evIncCondutorMDFe><descEvento>Inclusao Condutor</descEvento><condutor><xNome>Jose da Silva</xNome><CPF>12345678909</CPF></condutor></evIncCondutorMDFe>"
        ));
    }

    #[test]
    fn inclusao_dfe_nests_docs_without_infnfe_wrapper() {
        let discharges = [IncDfeDischarge {
            c_mun_descarga: "3550308",
            x_mun_descarga: "Sao Paulo",
            nfe_keys: &[
                "35250612345678000190550010000000011000000028",
                "35250612345678000190550010000000021000000039",
            ],
        }];
        let xml = build_mdfe_inclusao_dfe(
            KEY,
            "143250000000123",
            "4314902",
            "Porto Alegre",
            &discharges,
            1,
            "12345678000190",
            SefazEnvironment::Production,
        );
        assert!(xml.contains("<tpEvento>110115</tpEvento>"));
        assert!(xml.contains("<descEvento>Inclusao DF-e</descEvento>"));
        assert!(xml.contains("<cMunCarrega>4314902</cMunCarrega>"));
        assert!(xml.contains(
            "<infDoc><cMunDescarga>3550308</cMunDescarga><xMunDescarga>Sao Paulo</xMunDescarga><chNFe>35250612345678000190550010000000011000000028</chNFe><chNFe>35250612345678000190550010000000021000000039</chNFe></infDoc>"
        ));
        assert!(
            !xml.contains("<infNFe>"),
            "chNFe is a direct child of infDoc"
        );
    }

    #[test]
    fn parses_registered_event() {
        let xml = "<retEventoMDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\" versao=\"3.00\">\
            <infEvento Id=\"ID...\"><tpAmb>1</tpAmb><verAplic>SVRS</verAplic>\
            <cOrgao>43</cOrgao><cStat>135</cStat>\
            <xMotivo>Evento registrado e vinculado ao MDF-e</xMotivo>\
            <chMDFe>43250612345678000190580010000000011000000017</chMDFe>\
            <tpEvento>110112</tpEvento><xEvento>Encerramento</xEvento>\
            <nSeqEvento>1</nSeqEvento><dhRegEvento>2026-06-05T10:00:00-03:00</dhRegEvento>\
            <nProt>143250000999888</nProt></infEvento></retEventoMDFe>";
        let r = parse_mdfe_event_response(xml).unwrap();
        assert_eq!(r.status_code, "135");
        assert!(r.is_registered());
        assert_eq!(r.event_type.as_deref(), Some("110112"));
        assert_eq!(r.protocol_number.as_deref(), Some("143250000999888"));
        assert_eq!(
            r.registered_at.as_deref(),
            Some("2026-06-05T10:00:00-03:00")
        );
    }

    #[test]
    fn event_136_is_also_registered() {
        let xml = "<retEventoMDFe><infEvento><cStat>136</cStat>\
            <xMotivo>Evento registrado, mas nao vinculado ao MDF-e</xMotivo></infEvento></retEventoMDFe>";
        let r = parse_mdfe_event_response(xml).unwrap();
        assert!(r.is_registered());
    }

    #[test]
    fn event_rejected_not_registered() {
        let xml = "<retEventoMDFe><infEvento><cStat>573</cStat>\
            <xMotivo>Rejeicao: Duplicidade de evento</xMotivo></infEvento></retEventoMDFe>";
        let r = parse_mdfe_event_response(xml).unwrap();
        assert!(!r.is_registered());
        assert_eq!(r.status_code, "573");
    }
}
