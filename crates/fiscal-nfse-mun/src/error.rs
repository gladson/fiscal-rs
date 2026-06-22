//! Erros do crate de NFS-e municipal.

use std::fmt;

#[derive(Debug)]
pub enum MunError {
    /// Município não suportado (sem provedor registrado).
    MunicipioNaoSuportado(String),
    /// Operação ainda não implementada para o provedor.
    NaoImplementado(&'static str),
    /// Dado de entrada inválido.
    Validacao(String),
    /// Falha ao montar/serializar XML.
    Xml(String),
    /// Falha de assinatura.
    Assinatura(String),
    /// Falha de transporte (rede/SOAP/REST).
    Transporte(String),
}

impl fmt::Display for MunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MunError::MunicipioNaoSuportado(m) => write!(f, "município não suportado: {m}"),
            MunError::NaoImplementado(o) => write!(f, "não implementado: {o}"),
            MunError::Validacao(m) => write!(f, "validação: {m}"),
            MunError::Xml(m) => write!(f, "xml: {m}"),
            MunError::Assinatura(m) => write!(f, "assinatura: {m}"),
            MunError::Transporte(m) => write!(f, "transporte: {m}"),
        }
    }
}

impl std::error::Error for MunError {}

pub type Result<T> = std::result::Result<T, MunError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Display for each variant ──────────────────────────────────────

    #[test]
    fn display_municipio_nao_suportado() {
        let e = MunError::MunicipioNaoSuportado("3304557".into());
        assert_eq!(format!("{e}"), "município não suportado: 3304557");
    }

    #[test]
    fn display_nao_implementado() {
        let e = MunError::NaoImplementado("cancelar");
        assert_eq!(format!("{e}"), "não implementado: cancelar");
    }

    #[test]
    fn display_validacao() {
        let e = MunError::Validacao("CNPJ inválido".into());
        assert_eq!(format!("{e}"), "validação: CNPJ inválido");
    }

    #[test]
    fn display_xml() {
        let e = MunError::Xml("tag não fechada".into());
        assert_eq!(format!("{e}"), "xml: tag não fechada");
    }

    #[test]
    fn display_assinatura() {
        let e = MunError::Assinatura("chave não carregada".into());
        assert_eq!(format!("{e}"), "assinatura: chave não carregada");
    }

    #[test]
    fn display_transporte() {
        let e = MunError::Transporte("connection refused".into());
        assert_eq!(format!("{e}"), "transporte: connection refused");
    }

    // ── Error trait ───────────────────────────────────────────────────

    #[test]
    fn mun_error_implements_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(MunError::Validacao("teste".into()));
        // source() returns None (no inner error).
        assert!(e.source().is_none());
    }

    // ── Debug ─────────────────────────────────────────────────────────

    #[test]
    fn debug_contains_variant_and_message() {
        let e = MunError::Xml("parse error at line 1".into());
        let debug = format!("{e:?}");
        assert!(debug.contains("Xml"));
        assert!(debug.contains("parse error at line 1"));
    }

    // ── Result type alias ─────────────────────────────────────────────

    #[test]
    fn result_ok_works() {
        let r: Result<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn result_err_works() {
        let r: Result<i32> = Err(MunError::Transporte("timeout".into()));
        assert!(r.is_err());
    }
}
