//! Typed parsers for CT-e SEFAZ responses.
//!
//! Self-contained (own SOAP-body stripping and raw-tag extraction) so the CT-e
//! module stays independent of the NF-e response-parser internals.

use fiscal_core::FiscalError;
use fiscal_core::xml_utils::extract_xml_tag_value;
use serde::{Deserialize, Serialize};

pub use crate::response_parsers::StatusResponse;

/// Parsed result of a CT-e synchronous reception (`retCTe`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CteAuthorizationResponse {
    /// SEFAZ status code (`cStat`) — the protocol's status when present, else
    /// the envelope status.
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// CT-e access key (`chCTe`) echoed in the protocol, when authorized.
    pub access_key: Option<String>,
    /// Protocol number (`nProt`), present when the CT-e was authorized.
    pub protocol_number: Option<String>,
    /// Timestamp when SEFAZ processed the document (`dhRecbto`).
    pub authorized_at: Option<String>,
    /// Raw `<protCTe>…</protCTe>` XML fragment, for storage/attachment to the
    /// authorized CT-e.
    pub protocol_xml: Option<String>,
}

/// Parsed result of a CT-e consultation (`retConsSitCTe`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CteConsultaResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// CT-e access key (`chCTe`), when the document exists.
    pub access_key: Option<String>,
    /// Protocol number (`nProt`), present when the CT-e is authorized.
    pub protocol_number: Option<String>,
    /// Raw `<protCTe>…</protCTe>` XML fragment, when present.
    pub protocol_xml: Option<String>,
    /// Raw `<procEventoCTe>…</procEventoCTe>` fragments linked to this CT-e.
    pub event_xmls: Vec<String>,
}

/// Parse a CT-e synchronous reception response (`retCTe`).
///
/// SEFAZ replies with `retCTe` wrapping a `<protCTe>` once authorized; the
/// protocol's `infProt` carries the authoritative `cStat`/`nProt`/`chCTe`. When
/// no protocol is present (rejection), the envelope-level `cStat`/`xMotivo` are
/// returned.
///
/// # Errors
///
/// Returns `FiscalError::XmlParsing` if no `<cStat>` can be found at all.
pub fn parse_cte_authorization_response(
    xml: &str,
) -> Result<CteAuthorizationResponse, FiscalError> {
    let body = strip_soap(xml);
    let protocol_xml = raw_tag(&body, "protCTe");

    let (status_code, status_message, access_key, protocol_number, authorized_at) =
        if let Some(prot) = &protocol_xml {
            let inf = inner_of(prot, "infProt").unwrap_or_else(|| prot.clone());
            (
                extract_xml_tag_value(&inf, "cStat"),
                extract_xml_tag_value(&inf, "xMotivo"),
                extract_xml_tag_value(&inf, "chCTe"),
                extract_xml_tag_value(&inf, "nProt"),
                extract_xml_tag_value(&inf, "dhRecbto"),
            )
        } else {
            (None, None, None, None, None)
        };

    let status_code = status_code
        .or_else(|| extract_xml_tag_value(&body, "cStat"))
        .ok_or_else(|| {
            FiscalError::XmlParsing("missing <cStat> in CT-e reception response".into())
        })?;
    let status_message = status_message
        .or_else(|| extract_xml_tag_value(&body, "xMotivo"))
        .unwrap_or_default();

    Ok(CteAuthorizationResponse {
        status_code,
        status_message,
        access_key,
        protocol_number,
        authorized_at,
        protocol_xml,
    })
}

/// Parse a CT-e consultation response (`retConsSitCTe`).
///
/// # Errors
///
/// Returns `FiscalError::XmlParsing` if `<cStat>` is missing.
pub fn parse_cte_consulta_response(xml: &str) -> Result<CteConsultaResponse, FiscalError> {
    let body = strip_soap(xml);

    let status_code = extract_xml_tag_value(&body, "cStat").ok_or_else(|| {
        FiscalError::XmlParsing("missing <cStat> in CT-e consulta response".into())
    })?;
    let status_message = extract_xml_tag_value(&body, "xMotivo").unwrap_or_default();

    let protocol_xml = raw_tag(&body, "protCTe");
    let (access_key, protocol_number) = if let Some(prot) = &protocol_xml {
        let inf = inner_of(prot, "infProt").unwrap_or_else(|| prot.clone());
        (
            extract_xml_tag_value(&inf, "chCTe"),
            extract_xml_tag_value(&inf, "nProt"),
        )
    } else {
        (None, None)
    };

    Ok(CteConsultaResponse {
        status_code,
        status_message,
        access_key,
        protocol_number,
        protocol_xml,
        event_xmls: raw_tags(&body, "procEventoCTe"),
    })
}

/// Parse a CT-e status response (`retConsStatServCTe`). Same shape as NF-e
/// status (`cStat`/`xMotivo`/`tMed`).
///
/// # Errors
///
/// Returns `FiscalError::XmlParsing` if `<cStat>` is missing.
pub fn parse_cte_status_response(xml: &str) -> Result<StatusResponse, FiscalError> {
    let body = strip_soap(xml);
    let status_code = extract_xml_tag_value(&body, "cStat")
        .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in CT-e status response".into()))?;
    Ok(StatusResponse {
        status_code,
        status_message: extract_xml_tag_value(&body, "xMotivo").unwrap_or_default(),
        average_time: extract_xml_tag_value(&body, "tMed"),
    })
}

// ── self-contained XML helpers ───────────────────────────────────────────────

/// Strip an outer SOAP `<…:Body>` wrapper and remove a default `cte:` prefix.
fn strip_soap(xml: &str) -> String {
    let body = inner_of(xml, "Body").unwrap_or_else(|| xml.to_string());
    body.replace("<cte:", "<").replace("</cte:", "</")
}

struct TagSpan {
    open_start: usize,
    content_start: usize,
    content_end: usize,
    close_end: usize,
}

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

fn inner_of(xml: &str, local: &str) -> Option<String> {
    let span = raw_tag_span(xml, local)?;
    Some(xml[span.content_start..span.content_end].to_string())
}

fn raw_tag(xml: &str, local: &str) -> Option<String> {
    let span = raw_tag_span(xml, local)?;
    Some(xml[span.open_start..span.close_end].to_string())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_authorized_protocol() {
        let xml = r#"<soap:Body><cteResultMsg><retCTe versao="4.00"><tpAmb>2</tpAmb>
            <protCTe versao="4.00"><infProt><tpAmb>2</tpAmb><cStat>100</cStat>
            <xMotivo>Autorizado o uso do CT-e</xMotivo>
            <chCTe>35250612345678000190570010000000011000000017</chCTe>
            <nProt>135250000000999</nProt><dhRecbto>2026-06-05T10:00:00-03:00</dhRecbto>
            </infProt></protCTe></retCTe></cteResultMsg></soap:Body>"#;
        let r = parse_cte_authorization_response(xml).unwrap();
        assert_eq!(r.status_code, "100");
        assert_eq!(
            r.access_key.as_deref(),
            Some("35250612345678000190570010000000011000000017")
        );
        assert_eq!(r.protocol_number.as_deref(), Some("135250000000999"));
        assert!(r.protocol_xml.unwrap().contains("<infProt>"));
    }

    #[test]
    fn parses_rejection_without_protocol() {
        let xml = r#"<retCTe versao="4.00"><tpAmb>2</tpAmb><cStat>403</cStat>
            <xMotivo>Rejeicao: chave invalida</xMotivo></retCTe>"#;
        let r = parse_cte_authorization_response(xml).unwrap();
        assert_eq!(r.status_code, "403");
        assert!(r.access_key.is_none());
        assert!(r.protocol_xml.is_none());
    }

    #[test]
    fn status_response_parses() {
        let xml = r#"<retConsStatServCTe><tpAmb>2</tpAmb><cStat>107</cStat>
            <xMotivo>Servico em Operacao</xMotivo><tMed>1</tMed></retConsStatServCTe>"#;
        let r = parse_cte_status_response(xml).unwrap();
        assert_eq!(r.status_code, "107");
        assert_eq!(r.average_time.as_deref(), Some("1"));
    }
}
