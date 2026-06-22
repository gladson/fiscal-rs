//! Transporte SOAP do `lotenfe.asmx` (PMSP). Método `EnvioLoteRPS` (produção) e
//! `TesteEnvioLoteRPS` (homologação — valida, não gera nota). Ambos recebem
//! `VersaoSchema` + `MensagemXML` (o `PedidoEnvioLoteRPS` assinado, escapado).

#![cfg(feature = "client")]

use crate::error::{MunError, Result};
use crate::model::{Ambiente, EmitOutput, Status};

const SP_NS: &str = "http://www.prefeitura.sp.gov.br/nfe";

/// Escapa XML para ir como conteúdo-texto do `MensagemXML`.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Método SOAP conforme ambiente (nome da operação no WSDL).
pub fn metodo(amb: Ambiente) -> &'static str {
    match amb {
        Ambiente::Producao => "EnvioLoteRPS",
        Ambiente::Homologacao => "TesteEnvioLoteRPS",
    }
}

/// SOAPAction conforme método (valores exatos do WSDL — `/ws/` + camelCase).
pub fn soap_action(metodo: &str) -> &'static str {
    match metodo {
        "TesteEnvioLoteRPS" => "http://www.prefeitura.sp.gov.br/nfe/ws/testeenvio",
        "EnvioLoteRPS" => "http://www.prefeitura.sp.gov.br/nfe/ws/envioLoteRPS",
        "EnvioRPS" => "http://www.prefeitura.sp.gov.br/nfe/ws/envioRPS",
        "CancelamentoNFe" => "http://www.prefeitura.sp.gov.br/nfe/ws/cancelamentoNFe",
        "ConsultaNFe" => "http://www.prefeitura.sp.gov.br/nfe/ws/consultaNFe",
        _ => "",
    }
}

/// Monta o envelope SOAP 1.1 do `lotenfe.asmx`. O wrapper do body é
/// `{Metodo}Request{ VersaoSchema, MensagemXML }`. `versao` = 1 (legado) ou 2 (reforma).
pub fn soap_envio(metodo: &str, signed_lote: &str, versao: u8) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:nfe=\"{SP_NS}\">\
<soap:Body><nfe:{metodo}Request><nfe:VersaoSchema>{versao}</nfe:VersaoSchema>\
<nfe:MensagemXML>{}</nfe:MensagemXML></nfe:{metodo}Request></soap:Body></soap:Envelope>",
        escape(signed_lote)
    )
}

/// Desescapa entidades XML do RetornoXML embutido.
fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn tag_val(xml: &str, tag: &str) -> Option<String> {
    for open in [format!("<{tag}>"), format!(":{tag}>")] {
        if let Some(i) = xml.find(&open) {
            let rest = &xml[i + open.len()..];
            if let Some(j) = rest.find('<') {
                let v = rest[..j].trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Interpreta o `RetornoEnvioLoteRPS` (que vem escapado dentro do SOAP).
pub fn parse_retorno(http_status: u16, body: &str) -> EmitOutput {
    let inner = unescape(body);
    let sucesso = tag_val(&inner, "Sucesso")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let ok = (200..300).contains(&http_status) && sucesso;

    let numero = tag_val(&inner, "NumeroNFe").or_else(|| tag_val(&inner, "NumeroNota"));
    let cod_verif = tag_val(&inner, "CodigoVerificacao");
    let motivo = if ok {
        None
    } else {
        match (tag_val(&inner, "Codigo"), tag_val(&inner, "Descricao")) {
            (Some(c), Some(d)) => Some(format!("{c}: {d}")),
            (_, Some(d)) => Some(d),
            _ => Some(inner.chars().take(600).collect()),
        }
    };

    EmitOutput {
        status: if ok {
            Status::Autorizado
        } else {
            Status::Rejeitado
        },
        numero_nfse: numero,
        codigo_verificacao: cod_verif,
        protocolo: None,
        data_emissao: tag_val(&inner, "DataEmissaoNFe"),
        xml: if ok { Some(inner.clone()) } else { None },
        motivo,
        // SP não devolve URL pública confiável; o painel orienta consultar pelo
        // número + código de verificação no portal da prefeitura.
        link: None,
        raw: body.to_string(),
    }
}

/// POST SOAP ao `lotenfe.asmx`.
pub async fn post_envio(
    http: &reqwest::Client,
    endpoint: &str,
    metodo: &str,
    envelope: &str,
) -> Result<(u16, String)> {
    let resp = http
        .post(endpoint)
        .header("Content-Type", "text/xml; charset=utf-8")
        .header("SOAPAction", soap_action(metodo))
        .body(envelope.to_string())
        .send()
        .await
        .map_err(|e| MunError::Transporte(format!("{e}")))?;
    let status = resp.status().as_u16();
    let body = resp
        .text()
        .await
        .map_err(|e| MunError::Transporte(format!("read body: {e}")))?;
    Ok((status, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Ambiente, EmitOutput, Status};

    // ── metodo ────────────────────────────────────────────────────────

    #[test]
    fn metodo_producao_returns_envio_lote_rps() {
        assert_eq!(metodo(Ambiente::Producao), "EnvioLoteRPS");
    }

    #[test]
    fn metodo_homologacao_returns_teste_envio_lote_rps() {
        assert_eq!(metodo(Ambiente::Homologacao), "TesteEnvioLoteRPS");
    }

    // ── soap_action ───────────────────────────────────────────────────

    #[test]
    fn soap_action_teste_envio() {
        assert_eq!(
            soap_action("TesteEnvioLoteRPS"),
            "http://www.prefeitura.sp.gov.br/nfe/ws/testeenvio"
        );
    }

    #[test]
    fn soap_action_envio_lote_rps() {
        assert_eq!(
            soap_action("EnvioLoteRPS"),
            "http://www.prefeitura.sp.gov.br/nfe/ws/envioLoteRPS"
        );
    }

    #[test]
    fn soap_action_envio_rps() {
        assert_eq!(
            soap_action("EnvioRPS"),
            "http://www.prefeitura.sp.gov.br/nfe/ws/envioRPS"
        );
    }

    #[test]
    fn soap_action_cancelamento() {
        assert_eq!(
            soap_action("CancelamentoNFe"),
            "http://www.prefeitura.sp.gov.br/nfe/ws/cancelamentoNFe"
        );
    }

    #[test]
    fn soap_action_consulta() {
        assert_eq!(
            soap_action("ConsultaNFe"),
            "http://www.prefeitura.sp.gov.br/nfe/ws/consultaNFe"
        );
    }

    #[test]
    fn soap_action_unknown_returns_empty() {
        assert_eq!(soap_action("MetodoInexistente"), "");
    }

    // ── soap_envio ────────────────────────────────────────────────────

    #[test]
    fn soap_envio_produces_valid_soap_envelope() {
        let envelope = soap_envio("EnvioLoteRPS", "<PedidoEnvioLoteRPS/>", 1);
        assert!(envelope.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(envelope.contains("<soap:Envelope"));
        assert!(envelope.contains("xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\""));
        assert!(envelope.contains("<nfe:EnvioLoteRPSRequest>"));
        assert!(envelope.contains("<nfe:VersaoSchema>1</nfe:VersaoSchema>"));
        assert!(envelope.contains("<nfe:MensagemXML>"));
        assert!(envelope.contains("</nfe:EnvioLoteRPSRequest>"));
    }

    #[test]
    fn soap_envio_escapes_xml_special_chars_in_body() {
        let envelope = soap_envio(
            "TesteEnvioLoteRPS",
            "<Lote><RPS><Valor>100 & 50</Valor></RPS></Lote>",
            2,
        );
        // The & must be escaped to &amp; inside the MensagemXML text content.
        assert!(envelope.contains("100 &amp; 50"));
        assert!(!envelope.contains("<RPS>100 & 50</RPS>"));
        assert!(envelope.contains("<nfe:VersaoSchema>2</nfe:VersaoSchema>"));
    }

    #[test]
    fn soap_envio_uses_correct_method_name_as_wrapper() {
        let envelope = soap_envio("CancelamentoNFe", "<Cancelamento/>", 1);
        assert!(envelope.contains("<nfe:CancelamentoNFeRequest>"));
    }

    // ── escape / unescape roundtrip ────────────────────────────────────

    #[test]
    fn escape_and_unescape_roundtrip() {
        let original = "<Tag attr=\"value\">text &amp; data</Tag>";
        let escaped = super::escape(original);
        // The original already has &amp; — escape will double-encode the & in &amp;
        // This tests that unescape correctly handles what escape produces.
        let plain = "<root>a < b & c > d</root>";
        let escaped = super::escape(plain);
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
        assert!(!escaped.contains('&'));
        let restored = super::unescape(&escaped);
        assert_eq!(restored, plain);
    }

    // ── parse_retorno ─────────────────────────────────────────────────

    #[test]
    fn parse_retorno_autorizado_extrai_campos() {
        let body = "<RetornoEnvioLoteRPS>\
            <Sucesso>true</Sucesso>\
            <NumeroNFe>42</NumeroNFe>\
            <CodigoVerificacao>XYZ-123</CodigoVerificacao>\
            <DataEmissaoNFe>2025-06-01</DataEmissaoNFe>\
            </RetornoEnvioLoteRPS>";
        let out = parse_retorno(200, body);
        assert_eq!(out.status, Status::Autorizado);
        assert_eq!(out.numero_nfse.as_deref(), Some("42"));
        assert_eq!(out.codigo_verificacao.as_deref(), Some("XYZ-123"));
        assert_eq!(out.data_emissao.as_deref(), Some("2025-06-01"));
        assert!(out.xml.is_some()); // autorizado = Some(inner)
    }

    #[test]
    fn parse_retorno_rejeitado_extrai_codigo_e_descricao() {
        let body = "<RetornoEnvioLoteRPS>\
            <Sucesso>false</Sucesso>\
            <Codigo>E500</Codigo>\
            <Descricao>Erro interno do servidor</Descricao>\
            </RetornoEnvioLoteRPS>";
        let out = parse_retorno(200, body);
        assert_eq!(out.status, Status::Rejeitado);
        assert_eq!(out.numero_nfse, None);
        assert_eq!(out.xml, None);
        assert_eq!(
            out.motivo.as_deref(),
            Some("E500: Erro interno do servidor")
        );
    }

    #[test]
    fn parse_retorno_http_error_without_sucesso_marks_rejected() {
        let body = "<html>502 Bad Gateway</html>";
        let out = parse_retorno(502, body);
        assert_eq!(out.status, Status::Rejeitado);
    }

    #[test]
    fn parse_retorno_sucesso_false_but_no_codigo_uses_truncated_body() {
        let body = "<RetornoEnvioLoteRPS>\
            <Sucesso>false</Sucesso>\
            </RetornoEnvioLoteRPS>";
        let out = parse_retorno(200, body);
        assert_eq!(out.status, Status::Rejeitado);
        assert!(out.motivo.is_some());
    }

    // ── tag_val ───────────────────────────────────────────────────────

    #[test]
    fn tag_val_extracts_value_with_ns_prefix() {
        let xml = "<nfe:NumeroNFe>42</nfe:NumeroNFe>";
        assert_eq!(super::tag_val(xml, "NumeroNFe").as_deref(), Some("42"));
    }

    #[test]
    fn tag_val_extracts_value_without_ns_prefix() {
        let xml = "<NumeroNFe>42</NumeroNFe>";
        assert_eq!(super::tag_val(xml, "NumeroNFe").as_deref(), Some("42"));
    }

    #[test]
    fn tag_val_returns_none_for_empty_tag() {
        let xml = "<NumeroNFe></NumeroNFe>";
        assert_eq!(super::tag_val(xml, "NumeroNFe"), None);
    }

    #[test]
    fn tag_val_returns_none_for_missing_tag() {
        assert_eq!(super::tag_val("<other/>", "NumeroNFe"), None);
    }
}
