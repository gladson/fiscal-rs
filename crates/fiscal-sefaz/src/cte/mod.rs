//! CT-e (Conhecimento de Transporte Eletrônico, model 57) SEFAZ transmission —
//! leiaute **4.00**.
//!
//! Mirrors the self-contained shape of the [`crate::mdfe`] module:
//!
//! - `CteService` — SOAP metadata (method, operation, version) per service.
//! - [`urls`] — endpoint resolution (SVRS, SVSP→SP, and own MG/MS/MT/PR/SP).
//! - [`request_builders`] — `consStatServCTe` / `consSitCTe` / sync payload.
//! - [`response_parsers`] — typed results for status, sync reception, consulta.
//! - [`soap`] — `<cteDadosMsg>` SOAP 1.2 envelope (plain + gzip for sync).
//!
//! The async client methods live on [`crate::client::SefazClient`]
//! (`cte_status`, `cte_consult`, `cte_authorize`, `cte_recepcao_evento`).
//!
//! Service WSDL metadata grounded in `nfephp-org/sped-cte`
//! (`storage/wscte_4.00_mod57.xml`): the synchronous authorization service is
//! named `CteRecepcao` (method `cteRecepcao`, operation `CTeRecepcaoSincV4`) and
//! gzip-compresses the signed `<CTe>`; status/consulta are sent uncompressed.

pub mod events;
pub mod request_builders;
pub mod response_parsers;
pub mod urls;

pub(crate) mod soap;

pub use events::{CteCorrecao, build_cte_cancelamento, build_cte_cce, build_cte_desacordo};

pub use response_parsers::{
    CteAuthorizationResponse, CteConsultaResponse, parse_cte_authorization_response,
    parse_cte_consulta_response, parse_cte_status_response,
};
pub use urls::get_cte_url;

/// CT-e XML namespace (`xmlns`) — shared by every payload root.
pub const CTE_NAMESPACE: &str = "http://www.portalfiscal.inf.br/cte";

/// CT-e layout version sent in the `versao` attribute / `<versaoDados>`.
pub const CTE_VERSION: &str = "4.00";

/// SOAP metadata for a single CT-e web service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CteServiceMeta {
    /// SOAP method name / `SoapAction` suffix (e.g. `"cteRecepcao"`).
    pub method: &'static str,
    /// WSDL operation identifier used to build the namespace and resolve the
    /// URL (e.g. `"CTeRecepcaoSincV4"`).
    pub operation: &'static str,
    /// Schema version sent in the payload `versao` attribute (`"4.00"`).
    pub version: &'static str,
}

/// CT-e SEFAZ web service operations (leiaute 4.00).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CteService {
    /// `CTeStatusServicoV4` — check SEFAZ operational status.
    StatusServico,
    /// `CTeConsultaV4` — consult a CT-e by its 44-digit access key.
    Consulta,
    /// `CTeRecepcaoSincV4` — synchronous (gzip) reception/authorization.
    RecepcaoSinc,
    /// `CTeRecepcaoEventoV4` — submit events (cancelamento, CCe, …).
    RecepcaoEvento,
    /// `CTeRecepcaoGTVe` — recepção da GTV-e (modelo 64), `<gtveDadosMsg>`.
    RecepcaoGTVe,
}

impl CteService {
    /// Return the SOAP metadata for this service.
    ///
    /// # Examples
    ///
    /// ```
    /// use fiscal_sefaz::cte::CteService;
    ///
    /// let meta = CteService::RecepcaoSinc.meta();
    /// assert_eq!(meta.method, "cteRecepcao");
    /// assert_eq!(meta.operation, "CTeRecepcaoSincV4");
    /// assert_eq!(meta.version, "4.00");
    /// ```
    pub fn meta(self) -> CteServiceMeta {
        match self {
            Self::StatusServico => CteServiceMeta {
                method: "cteStatusServicoCT",
                operation: "CTeStatusServicoV4",
                version: CTE_VERSION,
            },
            Self::Consulta => CteServiceMeta {
                method: "cteConsultaCT",
                operation: "CTeConsultaV4",
                version: CTE_VERSION,
            },
            Self::RecepcaoSinc => CteServiceMeta {
                method: "cteRecepcao",
                operation: "CTeRecepcaoSincV4",
                version: CTE_VERSION,
            },
            Self::RecepcaoEvento => CteServiceMeta {
                method: "cteRecepcaoEvento",
                operation: "CTeRecepcaoEventoV4",
                version: CTE_VERSION,
            },
            Self::RecepcaoGTVe => CteServiceMeta {
                method: "cteRecepcaoGTVe",
                operation: "CTeRecepcaoGTVe",
                version: CTE_VERSION,
            },
        }
    }

    /// Service key used as the lookup key in [`urls::get_cte_url`].
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
            CteService::StatusServico,
            CteService::Consulta,
            CteService::RecepcaoSinc,
            CteService::RecepcaoEvento,
        ] {
            let meta = svc.meta();
            assert!(!meta.method.is_empty(), "{svc:?} empty method");
            assert!(!meta.operation.is_empty(), "{svc:?} empty operation");
            assert_eq!(meta.version, "4.00", "{svc:?} must be leiaute 4.00");
        }
    }

    #[test]
    fn sync_reception_metadata() {
        // Grounded in sped-cte wscte_4.00_mod57.xml.
        assert_eq!(CteService::RecepcaoSinc.meta().method, "cteRecepcao");
        assert_eq!(
            CteService::RecepcaoSinc.meta().operation,
            "CTeRecepcaoSincV4"
        );
    }

    #[test]
    fn status_and_consulta_methods_carry_ct_suffix() {
        assert_eq!(
            CteService::StatusServico.meta().method,
            "cteStatusServicoCT"
        );
        assert_eq!(CteService::Consulta.meta().method, "cteConsultaCT");
    }
}
