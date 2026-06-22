//! MDF-e (Manifesto Eletrônico de Documentos Fiscais, model 58) SEFAZ
//! transmission — leiaute **3.00**.
//!
//! The NF-e/NFC-e plumbing in the rest of this crate is hard-wired to the
//! `nfe` portal namespace and the `nfeDadosMsg` SOAP body element, so MDF-e
//! gets its own self-contained module that mirrors the same shape:
//!
//! - `MdfeService` — SOAP metadata (operation, method, version) per service.
//! - [`urls`] — endpoint resolution (every UF authorizes MDF-e through **SVRS**).
//! - [`request_builders`] — `consStatServMDFe` / `consSitMDFe` / `enviMDFe`.
//! - [`response_parsers`] — typed results for status, sync reception, consulta.
//! - [`soap`] — `<mdfeDadosMsg>` SOAP 1.2 envelope (plain + gzip for sync).
//!
//! The async client methods live on [`crate::client::SefazClient`]
//! (`mdfe_status`, `mdfe_consult`, `mdfe_authorize`).

pub mod events;
pub mod request_builders;
pub mod response_parsers;
pub mod urls;

pub(crate) mod soap;

pub use events::{
    MdfeEventResponse, build_mdfe_cancelamento, build_mdfe_encerramento,
    build_mdfe_inclusao_condutor, build_mdfe_inclusao_dfe, parse_mdfe_event_response,
};
pub use response_parsers::{
    MdfeAuthorizationResponse, MdfeConsultaResponse, parse_mdfe_authorization_response,
    parse_mdfe_consulta_response, parse_mdfe_status_response,
};
pub use urls::get_mdfe_url;

/// MDF-e XML namespace (`xmlns`) — shared by every payload root.
pub const MDFE_NAMESPACE: &str = "http://www.portalfiscal.inf.br/mdfe";

/// MDF-e layout version sent in the `versao` attribute / `<versaoDados>`.
pub const MDFE_VERSION: &str = "3.00";

/// SOAP metadata for a single MDF-e web service.
///
/// Mirrors [`crate::services::ServiceMeta`] but is kept separate because the
/// MDF-e WSDL operations and SOAP methods differ from NF-e (and are notably
/// **not** symmetric — see [`MdfeService::meta`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct MdfeServiceMeta {
    /// SOAP method name / `SoapAction` suffix (e.g. `"mdfeStatusServicoMDF"`).
    pub method: &'static str,
    /// WSDL operation identifier used to build the namespace
    /// (e.g. `"MDFeStatusServico"`).
    pub operation: &'static str,
    /// Schema version sent in the payload `versao` attribute (`"3.00"`).
    pub version: &'static str,
}

/// MDF-e SEFAZ web service operations (leiaute 3.00, SVRS).
///
/// Each variant maps to one WSDL endpoint and carries fixed SOAP metadata
/// retrievable via [`meta()`](MdfeService::meta).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MdfeService {
    /// `MDFeStatusServico` — check SEFAZ operational status.
    StatusServico,
    /// `MDFeConsulta` — consult an MDF-e by its 44-digit access key.
    Consulta,
    /// `MDFeRecepcaoSinc` — synchronous (gzip) reception/authorization.
    RecepcaoSinc,
    /// `MDFeRecepcao` — asynchronous batch reception (legacy).
    Recepcao,
    /// `MDFeRetRecepcao` — query an asynchronous batch result by receipt.
    RetRecepcao,
    /// `MDFeRecepcaoEvento` — submit events (encerramento, cancelamento, …).
    RecepcaoEvento,
    /// `MDFeConsNaoEnc` — list the emitter's MDF-e that are still open
    /// (não encerrados).
    ConsNaoEnc,
}

impl MdfeService {
    /// Return the SOAP metadata for this service.
    ///
    /// # Examples
    ///
    /// ```
    /// use fiscal_sefaz::mdfe::MdfeService;
    ///
    /// let meta = MdfeService::StatusServico.meta();
    /// assert_eq!(meta.method, "mdfeStatusServicoMDF");
    /// assert_eq!(meta.operation, "MDFeStatusServico");
    /// assert_eq!(meta.version, "3.00");
    ///
    /// // Reception is asymmetric: the *sync* service's SOAP method is the
    /// // bare `mdfeRecepcao`, while the async batch uses `mdfeRecepcaoLote`.
    /// assert_eq!(MdfeService::RecepcaoSinc.meta().method, "mdfeRecepcao");
    /// assert_eq!(MdfeService::Recepcao.meta().method, "mdfeRecepcaoLote");
    /// ```
    pub fn meta(self) -> MdfeServiceMeta {
        match self {
            Self::StatusServico => MdfeServiceMeta {
                method: "mdfeStatusServicoMDF",
                operation: "MDFeStatusServico",
                version: MDFE_VERSION,
            },
            Self::Consulta => MdfeServiceMeta {
                method: "mdfeConsultaMDF",
                operation: "MDFeConsulta",
                version: MDFE_VERSION,
            },
            Self::RecepcaoSinc => MdfeServiceMeta {
                method: "mdfeRecepcao",
                operation: "MDFeRecepcaoSinc",
                version: MDFE_VERSION,
            },
            Self::Recepcao => MdfeServiceMeta {
                method: "mdfeRecepcaoLote",
                operation: "MDFeRecepcao",
                version: MDFE_VERSION,
            },
            Self::RetRecepcao => MdfeServiceMeta {
                method: "mdfeRetRecepcao",
                operation: "MDFeRetRecepcao",
                version: MDFE_VERSION,
            },
            Self::RecepcaoEvento => MdfeServiceMeta {
                method: "mdfeRecepcaoEvento",
                operation: "MDFeRecepcaoEvento",
                version: MDFE_VERSION,
            },
            Self::ConsNaoEnc => MdfeServiceMeta {
                method: "mdfeConsNaoEnc",
                operation: "MDFeConsNaoEnc",
                version: MDFE_VERSION,
            },
        }
    }

    /// Service name used as the lookup key in [`urls::get_mdfe_url`].
    ///
    /// Matches the WSDL operation identifier returned by [`meta`](Self::meta).
    pub fn url_key(self) -> &'static str {
        self.meta().operation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_services_have_non_empty_meta() {
        for svc in [
            MdfeService::StatusServico,
            MdfeService::Consulta,
            MdfeService::RecepcaoSinc,
            MdfeService::Recepcao,
            MdfeService::RetRecepcao,
            MdfeService::RecepcaoEvento,
            MdfeService::ConsNaoEnc,
        ] {
            let meta = svc.meta();
            assert!(!meta.method.is_empty(), "{svc:?} empty method");
            assert!(!meta.operation.is_empty(), "{svc:?} empty operation");
            assert_eq!(meta.version, "3.00", "{svc:?} must be leiaute 3.00");
        }
    }

    #[test]
    fn reception_methods_are_asymmetric() {
        // Guards against the natural-but-wrong assumption that the sync
        // service mirrors NF-e's `…Sinc` suffix. ACBr source: sync = bare
        // `mdfeRecepcao`, async lote = `mdfeRecepcaoLote`.
        assert_eq!(MdfeService::RecepcaoSinc.meta().method, "mdfeRecepcao");
        assert_eq!(MdfeService::Recepcao.meta().method, "mdfeRecepcaoLote");
    }

    #[test]
    fn url_key_equals_operation() {
        assert_eq!(MdfeService::StatusServico.url_key(), "MDFeStatusServico");
        assert_eq!(MdfeService::RecepcaoSinc.url_key(), "MDFeRecepcaoSinc");
        assert_eq!(MdfeService::ConsNaoEnc.url_key(), "MDFeConsNaoEnc");
    }

    #[test]
    fn status_and_consulta_methods_carry_mdf_suffix() {
        // Note the trailing `MDF` (not `MDFe`) — easy to typo.
        assert_eq!(
            MdfeService::StatusServico.meta().method,
            "mdfeStatusServicoMDF"
        );
        assert_eq!(MdfeService::Consulta.meta().method, "mdfeConsultaMDF");
    }
}
