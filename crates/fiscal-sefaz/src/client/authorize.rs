//! Authorization, consultation, and batch submission methods.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders;
use crate::response_parsers::{self, AuthorizationResponse, StatusResponse};
use crate::services::SefazService;

use super::SefazClient;

impl SefazClient {
    // ── Typed convenience methods ────────────────────────────────────────

    /// Check SEFAZ operational status (`NFeStatusServico4`).
    ///
    /// Builds the `<consStatServ>` request, sends it, and parses the
    /// `<retConsStatServ>` response.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] if `uf` is invalid.
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    pub async fn status(
        &self,
        uf: &str,
        environment: SefazEnvironment,
    ) -> Result<StatusResponse, FiscalError> {
        let request_xml = request_builders::build_status_request(uf, environment);
        let raw = self
            .send(SefazService::StatusServico, uf, environment, &request_xml)
            .await?;
        response_parsers::parse_status_response(&raw)
    }

    /// Submit a signed NF-e for authorization (`NFeAutorizacao4`).
    ///
    /// The `signed_xml` must already be signed with `fiscal_crypto::sign_xml`.
    /// Uses synchronous processing (`indSinc=1`) for a single document.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    pub async fn authorize(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_opts(uf, environment, signed_xml, lot_id, false, 55)
            .await
    }

    /// Submit a signed NF-e for authorization with gzip compression.
    ///
    /// Same as [`SefazClient::authorize`] but compresses the XML with gzip and uses
    /// `<nfeDadosMsgZip>` in the SOAP envelope, matching PHP sped-nfe's
    /// `$compactar = true` mode.
    pub async fn authorize_compressed(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_opts(uf, environment, signed_xml, lot_id, true, 55)
            .await
    }

    /// Submit a signed NFC-e (model 65) for authorization.
    ///
    /// Same as [`SefazClient::authorize`] but routes to the NFC-e endpoint.
    pub async fn authorize_nfce(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_opts(uf, environment, signed_xml, lot_id, false, 65)
            .await
    }

    /// Submit a signed NFC-e (model 65) for authorization with gzip compression.
    ///
    /// Same as [`SefazClient::authorize_nfce`] but compresses the XML with gzip.
    pub async fn authorize_nfce_compressed(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_opts(uf, environment, signed_xml, lot_id, true, 65)
            .await
    }

    /// Internal: submit authorization with optional compression and model selection.
    async fn authorize_opts(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
        compress: bool,
        model: u8,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let request_xml =
            request_builders::build_autorizacao_request(signed_xml, lot_id, true, compress);
        let raw = if compress {
            self.send_model_compressed(
                SefazService::Autorizacao,
                uf,
                environment,
                &request_xml,
                model,
            )
            .await?
        } else {
            self.send_model(
                SefazService::Autorizacao,
                uf,
                environment,
                &request_xml,
                model,
            )
            .await?
        };
        response_parsers::parse_autorizacao_response(&raw)
    }

    /// Submit a signed NF-e for authorization through a SEFAZ contingency
    /// authorizer (SVC-AN or SVC-RS).
    ///
    /// Used when the issuer's own SEFAZ authorizer is unavailable. The library
    /// resolves the correct contingency endpoint for `issuer_uf` via
    /// [`get_sefaz_contingency_url`](crate::urls::get_sefaz_contingency_url):
    /// SVC-RS for `{AM, BA, GO, MA, MS, MT, PE, PR}` and SVC-AN for all other
    /// states. Only the **endpoint** changes — the SOAP envelope keeps the
    /// **issuer's** `<cUF>` (e.g. `SP` → `35`), never an SVC pseudo-UF.
    ///
    /// The `signed_xml` must already be signed with `fiscal_crypto::sign_xml`
    /// and carry the contingency emission type (`tpEmis=6` for SVC-AN,
    /// `tpEmis=7` for SVC-RS). Uses synchronous processing (`indSinc=1`).
    ///
    /// NFC-e (model 65) has no SVC contingency and is intentionally not
    /// supported here.
    ///
    /// # Arguments
    ///
    /// * `issuer_uf` — State abbreviation of the issuer (real UF, e.g. `"SP"`).
    /// * `environment` — SEFAZ environment (production or homologation).
    /// * `signed_xml` — The signed NF-e XML.
    /// * `lot_id` — Lot identifier for the submission.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] if `issuer_uf` has no
    /// contingency mapping.
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    pub async fn authorize_contingency(
        &self,
        issuer_uf: &str,
        environment: SefazEnvironment,
        signed_xml: &str,
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let service = SefazService::Autorizacao;
        // Resolve the SVC (SVC-AN / SVC-RS) endpoint for the issuer's UF; the
        // library decides which contingency authorizer applies.
        let url =
            crate::urls::get_sefaz_contingency_url(issuer_uf, environment, service.url_key())?;
        let request_xml =
            request_builders::build_autorizacao_request(signed_xml, lot_id, true, false);
        // The endpoint is the SVC authorizer, but the envelope <cUF> stays the
        // issuer's real state code (SVC keeps the issuer's cUF).
        let raw = self
            .send_to_url(service, &url, issuer_uf, &request_xml)
            .await?;
        response_parsers::parse_autorizacao_response(&raw)
    }

    /// Submit multiple signed NF-e documents as an asynchronous batch
    /// (`NFeAutorizacao4`, `indSinc=0`).
    ///
    /// Matches the PHP `sefazEnviaLote()` method from `Tools.php`.
    /// The batch can contain 1 to 50 NF-e documents. SEFAZ returns a
    /// receipt number (`nRec`) in [`AuthorizationResponse::receipt_number`]
    /// that should be polled via `SefazClient::consult_receipt` to obtain
    /// the individual processing results.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment (production or homologation).
    /// * `signed_xmls` — Slice of signed NF-e XML strings (1 to 50).
    /// * `lot_id` — Lot identifier for the submission batch.
    ///
    /// # Errors
    ///
    /// Returns `FiscalError::InvalidTaxData` if the batch is empty or
    /// exceeds 50 documents.
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    pub async fn authorize_batch(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xmls: &[String],
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_batch_opts(uf, environment, signed_xmls, lot_id, false, 55)
            .await
    }

    /// Submit multiple signed NF-e documents as an asynchronous batch
    /// with gzip compression.
    ///
    /// Same as [`authorize_batch`](SefazClient::authorize_batch) but
    /// compresses the XML with gzip and uses `<nfeDadosMsgZip>` in the
    /// SOAP envelope.
    pub async fn authorize_batch_compressed(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xmls: &[String],
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_batch_opts(uf, environment, signed_xmls, lot_id, true, 55)
            .await
    }

    /// Submit multiple signed NFC-e (model 65) documents as an asynchronous
    /// batch (`NFeAutorizacao4`, `indSinc=0`).
    ///
    /// Same as [`authorize_batch`](SefazClient::authorize_batch) but routes
    /// to the NFC-e endpoint.
    pub async fn authorize_batch_nfce(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xmls: &[String],
        lot_id: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        self.authorize_batch_opts(uf, environment, signed_xmls, lot_id, false, 65)
            .await
    }

    /// Internal: submit batch authorization with optional compression and
    /// model selection.
    async fn authorize_batch_opts(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        signed_xmls: &[String],
        lot_id: &str,
        compress: bool,
        model: u8,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let refs: Vec<&str> = signed_xmls.iter().map(|s| s.as_str()).collect();
        let request_xml = request_builders::build_autorizacao_batch_request(&refs, lot_id, 0)?;
        let raw = if compress {
            self.send_model_compressed(
                SefazService::Autorizacao,
                uf,
                environment,
                &request_xml,
                model,
            )
            .await?
        } else {
            self.send_model(
                SefazService::Autorizacao,
                uf,
                environment,
                &request_xml,
                model,
            )
            .await?
        };
        response_parsers::parse_autorizacao_response(&raw)
    }

    /// Query batch processing result by receipt number (`NFeRetAutorizacao4`).
    ///
    /// Used after asynchronous authorization (`indSinc=0`) to poll for
    /// the result.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    pub async fn consult_receipt(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        receipt: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let request_xml = request_builders::build_consulta_recibo_request(receipt, environment);
        let raw = self
            .send(SefazService::RetAutorizacao, uf, environment, &request_xml)
            .await?;
        response_parsers::parse_autorizacao_response(&raw)
    }

    /// Consult an NF-e by its 44-digit access key (`NFeConsultaProtocolo4`).
    ///
    /// Returns the current status, protocol number, and authorization
    /// timestamp for an existing NF-e.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    pub async fn consult(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
    ) -> Result<AuthorizationResponse, FiscalError> {
        let request_xml = request_builders::build_consulta_request(access_key, environment);
        let raw = self
            .send(
                SefazService::ConsultaProtocolo,
                uf,
                environment,
                &request_xml,
            )
            .await?;
        response_parsers::parse_autorizacao_response(&raw)
    }
}

#[cfg(test)]
mod tests {
    use crate::services::SefazService;
    use crate::soap;
    use crate::urls::get_sefaz_contingency_url;
    use fiscal_core::types::SefazEnvironment;

    // `authorize_contingency` resolves the SVC endpoint via the same
    // `get_sefaz_contingency_url` it calls internally. These tests assert the
    // resolution per issuer UF and that the SOAP envelope keeps the issuer's
    // `<cUF>` (never an SVC pseudo-UF) — a live POST is covered by an
    // `#[ignore]` integration test that needs a certificate.

    fn svc_autorizacao_url(uf: &str, env: SefazEnvironment) -> String {
        get_sefaz_contingency_url(uf, env, SefazService::Autorizacao.url_key()).unwrap()
    }

    #[test]
    fn contingency_sp_resolves_svc_an_autorizacao_url() {
        // SP maps to SVC-AN (sefazvirtual).
        let url = svc_autorizacao_url("SP", SefazEnvironment::Production);
        assert_eq!(
            url,
            "https://www.sefazvirtual.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx"
        );
    }

    #[test]
    fn contingency_ba_resolves_svc_rs_autorizacao_url() {
        // BA maps to SVC-RS (svrs.rs.gov.br).
        let url = svc_autorizacao_url("BA", SefazEnvironment::Production);
        assert_eq!(
            url,
            "https://nfe.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx"
        );
    }

    #[test]
    fn contingency_envelope_keeps_issuer_cuf_not_pseudo_uf() {
        // The envelope must use the ISSUER's cUF (SP = 35), even though the
        // POST goes to the SVC-AN endpoint. This mirrors how
        // `authorize_contingency` calls `send_to_url(.., issuer_uf, ..)`.
        let meta = SefazService::Autorizacao.meta();
        let body = "<enviNFe><idLote>1</idLote></enviNFe>";

        let envelope = soap::build_envelope(body, "SP", &meta).unwrap();
        assert!(
            envelope.contains(body),
            "envelope must preserve the request body"
        );

        // A pseudo-UF must NOT be accepted as the envelope UF.
        assert!(
            soap::build_envelope(body, "SVCAN", &meta).is_err(),
            "SVC pseudo-UF must not derive a cUF"
        );

        // BA issuer → SVC-RS endpoint, but envelope still builds from BA.
        assert!(soap::build_envelope(body, "BA", &meta).is_ok());
    }

    #[test]
    fn contingency_url_unknown_uf_errors() {
        let result =
            get_sefaz_contingency_url("XX", SefazEnvironment::Production, "NfeAutorizacao");
        assert!(result.is_err());
    }

    /// SP in homologation resolves the SVC-AN homolog Autorizacao endpoint,
    /// which differs in host (`hom.` vs `www.`) and stays on the SVC-AN domain.
    #[test]
    fn contingency_sp_resolves_svc_an_autorizacao_url_homologation() {
        let url = svc_autorizacao_url("SP", SefazEnvironment::Homologation);
        assert_eq!(
            url,
            "https://hom.sefazvirtual.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx"
        );
        // Must differ from the production endpoint (different host + path).
        assert_ne!(url, svc_autorizacao_url("SP", SefazEnvironment::Production));
    }

    /// `authorize_contingency` builds its request body via
    /// `build_autorizacao_request(.., true, false)`: synchronous (`indSinc=1`)
    /// and uncompressed (plain XML text, not a gzip stream).
    #[test]
    fn contingency_request_is_sync_and_uncompressed() {
        use crate::request_builders;

        let signed_nfe = "<NFe><infNFe Id=\"NFe41250106157250000116550010000000011000000017\">\
             </infNFe></NFe>";
        // Same arguments `authorize_contingency` passes internally.
        let body = request_builders::build_autorizacao_request(signed_nfe, "1", true, false);

        // indSinc=1 (synchronous), never indSinc=0.
        assert!(
            body.contains("<indSinc>1</indSinc>"),
            "contingency must use synchronous processing: {body}"
        );
        assert!(!body.contains("<indSinc>0</indSinc>"));

        // The body is plain XML text (not gzip): it starts with the <enviNFe>
        // envelope and embeds the signed NF-e verbatim — no gzip magic bytes.
        assert!(body.trim_start().starts_with("<enviNFe"));
        assert!(body.contains(signed_nfe));
        assert!(
            !body.as_bytes().starts_with(&[0x1f, 0x8b]),
            "request body must not be gzip-compressed"
        );
    }

    /// Every SVC-RS state resolves an SVC-RS (`svrs.rs.gov.br`) Autorizacao
    /// URL, and a sample of SVC-AN states resolve an SVC-AN
    /// (`sefazvirtual.fazenda.gov.br`) Autorizacao URL — for the Autorizacao
    /// service specifically.
    #[test]
    fn contingency_uf_to_authorizer_matrix() {
        const SVC_RS_PROD: &str =
            "https://nfe.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx";
        const SVC_AN_PROD: &str =
            "https://www.sefazvirtual.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx";

        // Per urls::get_state_contingency_authorizer, SVC-RS covers:
        // AM, BA, GO, MA, MS, MT, PE, PR. (Note: BA is SVC-RS, not SVC-AN —
        // the source mapping, not the audit's 7-UF list.)
        for uf in ["AM", "BA", "GO", "MA", "MS", "MT", "PE", "PR"] {
            assert_eq!(
                svc_autorizacao_url(uf, SefazEnvironment::Production),
                SVC_RS_PROD,
                "{uf} must resolve the SVC-RS Autorizacao URL"
            );
        }

        // Sample of SVC-AN states (all remaining states route to SVC-AN).
        for uf in ["SP", "RJ", "MG", "RS", "SC"] {
            assert_eq!(
                svc_autorizacao_url(uf, SefazEnvironment::Production),
                SVC_AN_PROD,
                "{uf} must resolve the SVC-AN Autorizacao URL"
            );
        }
    }

    /// `authorize_contingency` with an unmapped UF returns
    /// `FiscalError::InvalidStateCode` — the variant `get_sefaz_contingency_url`
    /// produces for a state with no contingency authorizer.
    #[tokio::test]
    async fn contingency_authorize_invalid_uf_errors() {
        use fiscal_core::FiscalError;

        // Build a client from the shared test certificate so the method runs
        // far enough to hit the URL resolution (which fails before any network
        // call for an unmapped UF).
        let pfx_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../..",
            "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
        );
        let pfx = std::fs::read(pfx_path).expect("test PFX not found");
        let client = super::SefazClient::new(&pfx, "minhasenha").expect("client builds");

        let signed = "<NFe><infNFe Id=\"NFe41250106157250000116550010000000011000000017\">\
             </infNFe></NFe>";
        let err = client
            .authorize_contingency("XX", SefazEnvironment::Production, signed, "1")
            .await
            .expect_err("unmapped UF must error before any network call");
        assert!(
            matches!(err, FiscalError::InvalidStateCode(ref uf) if uf == "XX"),
            "expected InvalidStateCode(\"XX\"), got: {err}"
        );
    }
}
