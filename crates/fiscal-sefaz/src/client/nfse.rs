//! NFS-e Nacional (SEFIN Nacional) client — REST, leiaute 1.01.
//!
//! Diferente de tudo: é **REST** (não SOAP). O emitente assina a DPS, comprime
//! (gzip + base64) e faz `POST /nfse`. O SEFIN devolve a NFS-e (chave de 50
//! dígitos) síncrono. mTLS pelo certificado do emitente (já no `self.http`).

use base64::Engine as _;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use super::SefazClient;

/// Resultado da recepção da DPS no SEFIN Nacional.
#[derive(Debug, Clone)]
pub struct NfseResponse {
    /// HTTP status da resposta.
    pub http_status: u16,
    /// `chaveAcesso` da NFS-e (50 dígitos), quando autorizada.
    pub chave_acesso: Option<String>,
    /// XML da NFS-e (descomprimido), quando autorizada.
    pub nfse_xml: Option<String>,
    /// Corpo bruto da resposta (JSON) — para erros/auditoria.
    pub raw: String,
}

impl NfseResponse {
    /// `true` quando a NFS-e foi autorizada (HTTP 2xx + chave presente).
    pub fn is_authorized(&self) -> bool {
        (200..300).contains(&self.http_status) && self.chave_acesso.is_some()
    }
}

/// Base URL do SEFIN Nacional por ambiente.
fn sefin_base(env: SefazEnvironment) -> &'static str {
    match env {
        SefazEnvironment::Production => "https://sefin.nfse.gov.br/SefinNacional",
        // Homologação = produção restrita.
        _ => "https://sefin.producaorestrita.nfse.gov.br/SefinNacional",
    }
}

/// Base URL do ADN (Ambiente de Dados Nacional) — parâmetros municipais,
/// distribuição de DF-e etc.
fn adn_base(env: SefazEnvironment) -> &'static str {
    match env {
        SefazEnvironment::Production => "https://adn.nfse.gov.br",
        _ => "https://adn.producaorestrita.nfse.gov.br",
    }
}

/// Extrai o valor de uma chave string num JSON simples (`"chave":"valor"`).
fn json_str(body: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\"");
    let i = body.find(&pat)? + pat.len();
    let rest = &body[i..];
    let colon = rest.find(':')?;
    let after = &rest[colon + 1..];
    let q1 = after.find('"')?;
    let after2 = &after[q1 + 1..];
    let q2 = after2.find('"')?;
    Some(after2[..q2].to_string())
}

impl SefazClient {
    /// Envia uma DPS assinada ao SEFIN Nacional (`POST /nfse`).
    ///
    /// `signed_dps_xml` é a `<DPS>` já assinada. É comprimida (gzip) e
    /// Base64-encoded no corpo JSON `{"dpsXmlGZipB64": "..."}`. Retorna a chave
    /// de acesso + o XML da NFS-e quando autorizada.
    ///
    /// # Errors
    ///
    /// [`FiscalError::XmlGeneration`] se a compressão falhar;
    /// [`FiscalError::Network`] em falha de transporte.
    pub async fn nfse_recepcao(
        &self,
        signed_dps_xml: &str,
        environment: SefazEnvironment,
    ) -> Result<NfseResponse, FiscalError> {
        let payload = with_utf8_prolog(signed_dps_xml);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(payload.as_bytes())
            .map_err(|e| FiscalError::XmlGeneration(format!("gzip DPS: {e}")))?;
        let gz = encoder
            .finish()
            .map_err(|e| FiscalError::XmlGeneration(format!("gzip DPS: {e}")))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(gz);
        let body = format!("{{\"dpsXmlGZipB64\":\"{b64}\"}}");

        let url = format!("{}/nfse", sefin_base(environment));
        let response = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;
        let http_status = response.status().as_u16();
        let raw = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("read body: {e}")))?;

        let chave_acesso = json_str(&raw, "chaveAcesso");
        let nfse_xml = json_str(&raw, "nfseXmlGZipB64").and_then(|b| decode_gzip_b64(&b));

        Ok(NfseResponse {
            http_status,
            chave_acesso,
            nfse_xml,
            raw,
        })
    }

    /// Envia uma DPS assinada a uma **URL arbitrária** (para municípios que usam o
    /// layout nacional num endpoint próprio — ex.: Simpliss/Santana de Parnaíba,
    /// `/v2/nfsen`). Mesmo corpo do `nfse_recepcao` (`{"dpsXmlGZipB64": "..."}`).
    ///
    /// # Errors
    ///
    /// [`FiscalError::XmlGeneration`] na compressão; [`FiscalError::Network`] no transporte.
    pub async fn nfse_recepcao_url(
        &self,
        signed_dps_xml: &str,
        url: &str,
    ) -> Result<NfseResponse, FiscalError> {
        let payload = with_utf8_prolog(signed_dps_xml);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(payload.as_bytes())
            .map_err(|e| FiscalError::XmlGeneration(format!("gzip DPS: {e}")))?;
        let gz = encoder
            .finish()
            .map_err(|e| FiscalError::XmlGeneration(format!("gzip DPS: {e}")))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(gz);
        let body = format!("{{\"dpsXmlGZipB64\":\"{b64}\"}}");

        let response = self
            .http
            .post(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;
        let http_status = response.status().as_u16();
        let raw = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("read body: {e}")))?;
        let chave_acesso = json_str(&raw, "chaveAcesso");
        let nfse_xml = json_str(&raw, "nfseXmlGZipB64").and_then(|b| decode_gzip_b64(&b));
        Ok(NfseResponse {
            http_status,
            chave_acesso,
            nfse_xml,
            raw,
        })
    }

    /// Registra um evento (cancelamento etc.) no SEFIN Nacional.
    ///
    /// `POST /nfse/{chNFSe}/eventos` com o `<pedRegEvento>` assinado, gzip +
    /// Base64 no corpo `{"pedidoRegistroEventoXmlGZipB64": "..."}`. Retorna o
    /// XML do evento processado (`procEventoNFSe`) quando registrado.
    ///
    /// # Errors
    ///
    /// [`FiscalError::XmlGeneration`] se a compressão falhar;
    /// [`FiscalError::Network`] em falha de transporte.
    pub async fn nfse_evento(
        &self,
        ch_nfse: &str,
        signed_evento_xml: &str,
        environment: SefazEnvironment,
    ) -> Result<NfseResponse, FiscalError> {
        let payload = with_utf8_prolog(signed_evento_xml);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(payload.as_bytes())
            .map_err(|e| FiscalError::XmlGeneration(format!("gzip evento: {e}")))?;
        let gz = encoder
            .finish()
            .map_err(|e| FiscalError::XmlGeneration(format!("gzip evento: {e}")))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(gz);
        let body = format!("{{\"pedidoRegistroEventoXmlGZipB64\":\"{b64}\"}}");

        let url = format!("{}/nfse/{}/eventos", sefin_base(environment), ch_nfse);
        let response = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;
        let http_status = response.status().as_u16();
        let raw = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("read body: {e}")))?;

        let chave_acesso = json_str(&raw, "chaveAcesso");
        let nfse_xml = json_str(&raw, "eventoXmlGZipB64")
            .or_else(|| json_str(&raw, "nfseXmlGZipB64"))
            .and_then(|b| decode_gzip_b64(&b));

        Ok(NfseResponse {
            http_status,
            chave_acesso,
            nfse_xml,
            raw,
        })
    }

    /// GET genérico numa URL absoluta usando o mTLS do tenant. Para diagnóstico
    /// (ex.: baixar WSDL de webservice municipal que exige certificado).
    /// Retorna `(http_status, body)`.
    pub async fn https_get(&self, url: &str) -> Result<(u16, String), FiscalError> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("read body: {e}")))?;
        Ok((status, body))
    }

    /// Consulta uma NFS-e pela chave de acesso (`GET /nfse/{chNFSe}`).
    ///
    /// Retorna o XML da NFS-e (descomprimido) quando encontrada.
    ///
    /// # Errors
    ///
    /// [`FiscalError::Network`] em falha de transporte.
    pub async fn nfse_consulta(
        &self,
        ch_nfse: &str,
        environment: SefazEnvironment,
    ) -> Result<NfseResponse, FiscalError> {
        let url = format!("{}/nfse/{}", sefin_base(environment), ch_nfse);
        let response = self
            .http
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;
        let http_status = response.status().as_u16();
        let raw = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("read body: {e}")))?;

        let chave_acesso = json_str(&raw, "chaveAcesso").or_else(|| Some(ch_nfse.to_string()));
        let nfse_xml = json_str(&raw, "nfseXmlGZipB64").and_then(|b| decode_gzip_b64(&b));

        Ok(NfseResponse {
            http_status,
            chave_acesso,
            nfse_xml,
            raw,
        })
    }
}

impl SefazClient {
    /// GET genérico no ADN (parâmetros municipais etc.) com mTLS. `path` é o
    /// caminho absoluto começando com `/` (ex.: `/parametros_municipais/3550308/convenio`).
    /// Retorna `(http_status, body)`.
    pub async fn adn_get(
        &self,
        path: &str,
        environment: SefazEnvironment,
    ) -> Result<(u16, String), FiscalError> {
        let url = format!("{}{}", adn_base(environment), path);
        let response = self
            .http
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("read body: {e}")))?;
        Ok((status, body))
    }
}

/// Garante a declaração XML `<?xml ... encoding="UTF-8"?>` no início do
/// documento — o SEFIN Nacional rejeita (E1229) XML sem o prólogo UTF-8.
fn with_utf8_prolog(xml: &str) -> String {
    if xml.trim_start().starts_with("<?xml") {
        xml.to_string()
    } else {
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>{xml}")
    }
}

/// Decodifica Base64 + gunzip (NFS-e devolvida pelo SEFIN).
fn decode_gzip_b64(b64: &str) -> Option<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
    let mut d = GzDecoder::new(&bytes[..]);
    let mut out = String::new();
    d.read_to_string(&mut out).ok()?;
    Some(out)
}
