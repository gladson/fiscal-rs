//! ICMS monofasico combustiveis data (`IcmsMonoData`).
//!
//! Optional per-item group carrying the fields required by the monophasic fuel
//! taxation CSTs (02, 15, 53, 61). When present on an `InvoiceItemData`, the
//! typed builder routes the item to the corresponding `<ICMS02>`/`<ICMS15>`/
//! `<ICMS53>`/`<ICMS61>` group instead of the generic CST mapping.
//!
//! The exact field set per CST mirrors sped-nfe `TraitTagDetICMS` cases
//! `02`/`15`/`53`/`61`.

use serde::{Deserialize, Serialize};

use crate::newtypes::{Cents, Rate};

/// Monophasic-fuel ICMS data (CST 02/15/53/61).
///
/// A single optional group covering all four monophasic CSTs. Only the fields
/// relevant to the item's `icms_cst` are emitted:
///
/// | CST | Fields used |
/// |-----|-------------|
/// | 02  | `q_bc_mono`, `ad_rem_icms`, `v_icms_mono` |
/// | 15  | 02 fields + `q_bc_mono_reten`, `ad_rem_icms_reten`, `v_icms_mono_reten`, `p_red_ad_rem`, `mot_red_ad_rem` |
/// | 53  | `q_bc_mono`, `ad_rem_icms`, `v_icms_mono_op`, `p_dif`, `v_icms_mono_dif`, `v_icms_mono` |
/// | 61  | `q_bc_mono_ret`, `ad_rem_icms_ret`, `v_icms_mono_ret` |
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct IcmsMonoData {
    /// Monophasic calculation base quantity (`qBCMono`). CST 02/15/53.
    pub q_bc_mono: Option<i64>,
    /// Monophasic ad-rem ICMS rate (`adRemICMS`). CST 02/15/53.
    pub ad_rem_icms: Option<Rate>,
    /// Monophasic ICMS value (`vICMSMono`). CST 02/15/53.
    pub v_icms_mono: Option<Cents>,
    /// Retained monophasic calculation base quantity (`qBCMonoReten`). CST 15.
    pub q_bc_mono_reten: Option<i64>,
    /// Retained monophasic ad-rem ICMS rate (`adRemICMSReten`). CST 15.
    pub ad_rem_icms_reten: Option<Rate>,
    /// Retained monophasic ICMS value (`vICMSMonoReten`). CST 15.
    pub v_icms_mono_reten: Option<Cents>,
    /// Ad-rem reduction rate (`pRedAdRem`). CST 15. Optional.
    pub p_red_ad_rem: Option<Rate>,
    /// Ad-rem reduction reason (`motRedAdRem`). CST 15. Required when `p_red_ad_rem` is set.
    pub mot_red_ad_rem: Option<String>,
    /// Monophasic ICMS value before deferral (`vICMSMonoOp`). CST 53.
    pub v_icms_mono_op: Option<Cents>,
    /// Deferral percentage (`pDif`). CST 53.
    pub p_dif: Option<Rate>,
    /// Deferred monophasic ICMS value (`vICMSMonoDif`). CST 53.
    pub v_icms_mono_dif: Option<Cents>,
    /// Previously-collected monophasic calculation base quantity (`qBCMonoRet`). CST 61.
    pub q_bc_mono_ret: Option<i64>,
    /// Previously-collected monophasic ad-rem ICMS rate (`adRemICMSRet`). CST 61.
    pub ad_rem_icms_ret: Option<Rate>,
    /// Previously-collected monophasic ICMS value (`vICMSMonoRet`). CST 61.
    pub v_icms_mono_ret: Option<Cents>,
}

impl IcmsMonoData {
    /// Create an empty `IcmsMonoData`. Populate the fields relevant to the
    /// item's CST via the chainable setters.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the monophasic calculation base quantity (`qBCMono`).
    pub fn q_bc_mono(mut self, v: i64) -> Self {
        self.q_bc_mono = Some(v);
        self
    }
    /// Set the monophasic ad-rem ICMS rate (`adRemICMS`).
    pub fn ad_rem_icms(mut self, v: Rate) -> Self {
        self.ad_rem_icms = Some(v);
        self
    }
    /// Set the monophasic ICMS value (`vICMSMono`).
    pub fn v_icms_mono(mut self, v: Cents) -> Self {
        self.v_icms_mono = Some(v);
        self
    }
    /// Set the retained monophasic calculation base quantity (`qBCMonoReten`).
    pub fn q_bc_mono_reten(mut self, v: i64) -> Self {
        self.q_bc_mono_reten = Some(v);
        self
    }
    /// Set the retained monophasic ad-rem ICMS rate (`adRemICMSReten`).
    pub fn ad_rem_icms_reten(mut self, v: Rate) -> Self {
        self.ad_rem_icms_reten = Some(v);
        self
    }
    /// Set the retained monophasic ICMS value (`vICMSMonoReten`).
    pub fn v_icms_mono_reten(mut self, v: Cents) -> Self {
        self.v_icms_mono_reten = Some(v);
        self
    }
    /// Set the ad-rem reduction rate (`pRedAdRem`).
    pub fn p_red_ad_rem(mut self, v: Rate) -> Self {
        self.p_red_ad_rem = Some(v);
        self
    }
    /// Set the ad-rem reduction reason (`motRedAdRem`).
    pub fn mot_red_ad_rem(mut self, v: impl Into<String>) -> Self {
        self.mot_red_ad_rem = Some(v.into());
        self
    }
    /// Set the monophasic ICMS value before deferral (`vICMSMonoOp`).
    pub fn v_icms_mono_op(mut self, v: Cents) -> Self {
        self.v_icms_mono_op = Some(v);
        self
    }
    /// Set the deferral percentage (`pDif`).
    pub fn p_dif(mut self, v: Rate) -> Self {
        self.p_dif = Some(v);
        self
    }
    /// Set the deferred monophasic ICMS value (`vICMSMonoDif`).
    pub fn v_icms_mono_dif(mut self, v: Cents) -> Self {
        self.v_icms_mono_dif = Some(v);
        self
    }
    /// Set the previously-collected monophasic calculation base quantity (`qBCMonoRet`).
    pub fn q_bc_mono_ret(mut self, v: i64) -> Self {
        self.q_bc_mono_ret = Some(v);
        self
    }
    /// Set the previously-collected monophasic ad-rem ICMS rate (`adRemICMSRet`).
    pub fn ad_rem_icms_ret(mut self, v: Rate) -> Self {
        self.ad_rem_icms_ret = Some(v);
        self
    }
    /// Set the previously-collected monophasic ICMS value (`vICMSMonoRet`).
    pub fn v_icms_mono_ret(mut self, v: Cents) -> Self {
        self.v_icms_mono_ret = Some(v);
        self
    }
}
