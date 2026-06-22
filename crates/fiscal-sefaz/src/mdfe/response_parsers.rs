//! Typed parsers for MDF-e SEFAZ responses.
//!
//! These are self-contained (they do their own SOAP-body stripping and raw-tag
//! extraction via [`fiscal_core::xml_utils`]) so the MDF-e module stays
//! independent of the NF-e response-parser internals.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;
use serde::{Deserialize, Serialize};

pub use crate::response_parsers::StatusResponse;

/// Parsed result of an MDF-e synchronous reception (`retMDFe` / `retEnviMDFe`)
/// response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MdfeAuthorizationResponse {
    /// SEFAZ status code (`cStat`) — the protocol's status when present, else
    /// the envelope status.
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// MDF-e access key (`chMDFe`) echoed in the protocol, when authorized.
    pub access_key: Option<String>,
    /// Protocol number (`nProt`), present when the MDF-e was authorized.
    pub protocol_number: Option<String>,
    /// Timestamp when SEFAZ processed the document (`dhRecbto`).
    pub authorized_at: Option<String>,
    /// Raw `<protMDFe>…</protMDFe>` XML fragment, for storage/attachment to
    /// the authorized MDF-e.
    pub protocol_xml: Option<String>,
}

/// Parsed result of an MDF-e consultation (`retConsSitMDFe`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MdfeConsultaResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// MDF-e access key (`chMDFe`), when the document exists.
    pub access_key: Option<String>,
    /// Protocol number (`nProt`), present when the MDF-e is authorized.
    pub protocol_number: Option<String>,
    /// Raw `<protMDFe>…</protMDFe>` XML fragment, when present.
    pub protocol_xml: Option<String>,
    /// Raw `<procEventoMDFe>…</procEventoMDFe>` fragments (encerramento,
    /// cancelamento, …) linked to this MDF-e, if any.
    pub event_xmls: Vec<String>,
}

/// Strip an outer SOAP `<…:Body>` wrapper (if any) and remove a default `mdfe:`
/// element prefix so plain tag extraction works.
pub(super) fn strip_soap(xml: &str) -> String {
    let body = inner_of(xml, "Body").unwrap_or_else(|| xml.to_string());
    body.replace("<mdfe:", "<").replace("</mdfe:", "</")
}

/// Return the inner content of the first `<local>`/`</local>` pair, matching
/// namespace-prefixed variants (`<x:local>`).
fn inner_of(xml: &str, local: &str) -> Option<String> {
    let start = raw_tag_span(xml, local)?;
    Some(xml[start.content_start..start.content_end].to_string())
}

struct TagSpan {
    /// Byte offset of the opening `<`.
    open_start: usize,
    /// Byte offset just after the opening `>`.
    content_start: usize,
    /// Byte offset of the closing `<` (start of `</local>`).
    content_end: usize,
    /// Byte offset just after the closing `>`.
    close_end: usize,
}

/// Locate the first `<local …>…</local>` (namespace-prefix tolerant).
fn raw_tag_span(xml: &str, local: &str) -> Option<TagSpan> {
    let mut from = 0;
    let open_start;
    let content_start;
    loop {
        let lt = xml[from..].find('<')? + from;
        let gt = xml[lt..].find('>')? + lt;
        let tag = &xml[lt + 1..gt];
        if tag.starts_with('/') || tag.starts_with('?') {
            from = gt + 1;
            continue;
        }
        let name = tag.split_whitespace().next().unwrap_or(tag);
        let local_part = name.split_once(':').map(|(_, l)| l).unwrap_or(name);
        if local_part == local {
            open_start = lt;
            content_start = gt + 1;
            break;
        }
        from = gt + 1;
    }

    // Find the matching closing tag (no nesting of the same local name is
    // expected for the tags we extract here).
    let mut search = content_start;
    loop {
        let close = xml[search..].find("</")? + search;
        let close_gt = xml[close..].find('>')? + close;
        let tag = &xml[close + 2..close_gt];
        let local_part = tag.split_once(':').map(|(_, l)| l).unwrap_or(tag);
        if local_part == local {
            return Some(TagSpan {
                open_start,
                content_start,
                content_end: close,
                close_end: close_gt + 1,
            });
        }
        search = close_gt + 1;
    }
}

/// Extract the full `<local …>…</local>` fragment, inclusive.
fn raw_tag(xml: &str, local: &str) -> Option<String> {
    let span = raw_tag_span(xml, local)?;
    Some(xml[span.open_start..span.close_end].to_string())
}

/// Extract every `<local …>…</local>` fragment, inclusive.
fn raw_tags(xml: &str, local: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut base = 0usize;
    while base < xml.len() {
        let Some(span) = raw_tag_span(&xml[base..], local) else {
            break;
        };
        out.push(xml[base + span.open_start..base + span.close_end].to_string());
        base += span.close_end;
    }
    out
}

/// Parse an MDF-e status response (`retConsStatServMDFe`).
///
/// Identical shape to NF-e status (`cStat`/`xMotivo`/`tMed`), so this reuses
/// the shared [`StatusResponse`] type and the crate's status parser.
///
/// # Errors
///
/// Returns `FiscalError::XmlParsing` if `<cStat>` is missing.
pub fn parse_mdfe_status_response(xml: &str) -> Result<StatusResponse, FiscalError> {
    crate::response_parsers::parse_status_response(xml)
}

/// Parse an MDF-e synchronous reception response.
///
/// SVRS replies with `retMDFe` (sync) wrapping a `<protMDFe>` once authorized;
/// the protocol's `infProt` carries the authoritative `cStat`/`nProt`/`chMDFe`.
/// When no protocol is present (rejection), the envelope-level `cStat`/`xMotivo`
/// are returned.
///
/// # Errors
///
/// Returns `FiscalError::XmlParsing` if no `<cStat>` can be found at all.
pub fn parse_mdfe_authorization_response(
    xml: &str,
) -> Result<MdfeAuthorizationResponse, FiscalError> {
    let body = strip_soap(xml);
    let protocol_xml = raw_tag(&body, "protMDFe");

    // Prefer the protocol's infProt fields when present.
    let (status_code, status_message, access_key, protocol_number, authorized_at) =
        if let Some(prot) = &protocol_xml {
            let inf = inner_of(prot, "infProt").unwrap_or_else(|| prot.clone());
            (
                extract_xml_tag_value(&inf, "cStat"),
                extract_xml_tag_value(&inf, "xMotivo"),
                extract_xml_tag_value(&inf, "chMDFe"),
                extract_xml_tag_value(&inf, "nProt"),
                extract_xml_tag_value(&inf, "dhRecbto"),
            )
        } else {
            (None, None, None, None, None)
        };

    let status_code = status_code
        .or_else(|| extract_xml_tag_value(&body, "cStat"))
        .ok_or_else(|| {
            FiscalError::XmlParsing("missing <cStat> in MDF-e reception response".into())
        })?;
    let status_message = status_message
        .or_else(|| extract_xml_tag_value(&body, "xMotivo"))
        .unwrap_or_default();

    Ok(MdfeAuthorizationResponse {
        status_code,
        status_message,
        access_key,
        protocol_number,
        authorized_at,
        protocol_xml,
    })
}

/// Parse an MDF-e consultation response (`retConsSitMDFe`).
///
/// # Errors
///
/// Returns `FiscalError::XmlParsing` if `<cStat>` is missing.
pub fn parse_mdfe_consulta_response(xml: &str) -> Result<MdfeConsultaResponse, FiscalError> {
    let body = strip_soap(xml);

    let status_code = extract_xml_tag_value(&body, "cStat").ok_or_else(|| {
        FiscalError::XmlParsing("missing <cStat> in MDF-e consulta response".into())
    })?;
    let status_message = extract_xml_tag_value(&body, "xMotivo").unwrap_or_default();

    let protocol_xml = raw_tag(&body, "protMDFe");
    let (access_key, protocol_number) = match &protocol_xml {
        Some(prot) => {
            let inf = inner_of(prot, "infProt").unwrap_or_else(|| prot.clone());
            (
                extract_xml_tag_value(&inf, "chMDFe"),
                extract_xml_tag_value(&inf, "nProt"),
            )
        }
        None => (extract_xml_tag_value(&body, "chMDFe"), None),
    };

    let event_xmls = raw_tags(&body, "procEventoMDFe");

    Ok(MdfeConsultaResponse {
        status_code,
        status_message,
        access_key,
        protocol_number,
        protocol_xml,
        event_xmls,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status() {
        let xml = "<retConsStatServMDFe><cStat>107</cStat>\
            <xMotivo>Servico em Operacao</xMotivo><tMed>1</tMed></retConsStatServMDFe>";
        let r = parse_mdfe_status_response(xml).unwrap();
        assert_eq!(r.status_code, "107");
        assert_eq!(r.average_time.as_deref(), Some("1"));
    }

    #[test]
    fn parses_authorized_reception_with_protocol() {
        let xml = "<retMDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\" versao=\"3.00\">\
            <tpAmb>2</tpAmb><cStat>132</cStat><xMotivo>Lote recebido</xMotivo>\
            <protMDFe versao=\"3.00\"><infProt>\
            <tpAmb>2</tpAmb><verAplic>SVRS</verAplic>\
            <chMDFe>43250612345678000190580010000000011000000017</chMDFe>\
            <dhRecbto>2026-06-05T10:00:00-03:00</dhRecbto>\
            <nProt>143250000000123</nProt><digVal>abc=</digVal>\
            <cStat>100</cStat><xMotivo>Autorizado o uso do MDF-e</xMotivo>\
            </infProt></protMDFe></retMDFe>";
        let r = parse_mdfe_authorization_response(xml).unwrap();
        // Protocol-level status (100) wins over envelope status (132).
        assert_eq!(r.status_code, "100");
        assert_eq!(r.status_message, "Autorizado o uso do MDF-e");
        assert_eq!(
            r.access_key.as_deref(),
            Some("43250612345678000190580010000000011000000017")
        );
        assert_eq!(r.protocol_number.as_deref(), Some("143250000000123"));
        assert_eq!(
            r.authorized_at.as_deref(),
            Some("2026-06-05T10:00:00-03:00")
        );
        assert!(r.protocol_xml.as_deref().unwrap().contains("<infProt>"));
    }

    #[test]
    fn parses_rejected_reception_without_protocol() {
        let xml = "<retMDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\" versao=\"3.00\">\
            <tpAmb>2</tpAmb><cStat>225</cStat>\
            <xMotivo>Rejeicao: Falha no Schema XML</xMotivo></retMDFe>";
        let r = parse_mdfe_authorization_response(xml).unwrap();
        assert_eq!(r.status_code, "225");
        assert!(r.protocol_number.is_none());
        assert!(r.protocol_xml.is_none());
    }

    #[test]
    fn parses_soap_wrapped_reception() {
        let xml = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"><soap:Body>
            <mdfeResultMsg xmlns="http://www.portalfiscal.inf.br/mdfe/wsdl/MDFeRecepcaoSinc">
            <retMDFe xmlns="http://www.portalfiscal.inf.br/mdfe" versao="3.00">
            <cStat>132</cStat><xMotivo>Lote recebido</xMotivo>
            <protMDFe><infProt><chMDFe>43250612345678000190580010000000011000000017</chMDFe>
            <nProt>143250000000123</nProt><cStat>100</cStat>
            <xMotivo>Autorizado</xMotivo></infProt></protMDFe>
            </retMDFe></mdfeResultMsg></soap:Body></soap:Envelope>"#;
        let r = parse_mdfe_authorization_response(xml).unwrap();
        assert_eq!(r.status_code, "100");
        assert_eq!(r.protocol_number.as_deref(), Some("143250000000123"));
    }

    #[test]
    fn parses_consulta_with_event() {
        let xml = "<retConsSitMDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\" versao=\"3.00\">\
            <tpAmb>1</tpAmb><cStat>100</cStat><xMotivo>Autorizado</xMotivo>\
            <protMDFe><infProt>\
            <chMDFe>43250612345678000190580010000000011000000017</chMDFe>\
            <nProt>143250000000123</nProt><cStat>100</cStat></infProt></protMDFe>\
            <procEventoMDFe><eventoMDFe><infEvento><tpEvento>110112</tpEvento>\
            </infEvento></eventoMDFe></procEventoMDFe></retConsSitMDFe>";
        let r = parse_mdfe_consulta_response(xml).unwrap();
        assert_eq!(r.status_code, "100");
        assert_eq!(r.protocol_number.as_deref(), Some("143250000000123"));
        assert_eq!(r.event_xmls.len(), 1);
        assert!(r.event_xmls[0].contains("110112"));
    }

    #[test]
    fn consulta_rejects_missing_cstat() {
        let xml = "<retConsSitMDFe><xMotivo>x</xMotivo></retConsSitMDFe>";
        assert!(matches!(
            parse_mdfe_consulta_response(xml),
            Err(FiscalError::XmlParsing(_))
        ));
    }
}
