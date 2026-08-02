pub use xfina_models as models;

pub mod mutual_funds {
    pub use xfina_mf_cams as cams;
}

pub mod intl_stocks {
    pub use xfina_intl_stocks_ibkr as ibkr;
}

pub mod credit_cards {
    pub use xfina_cc_hdfc as hdfc;
    pub use xfina_cc_icici as icici;
}

pub mod bank_accounts {
    pub use xfina_ba_hdfc as hdfc;
    pub use xfina_ba_icici as icici;
    pub use xfina_ba_sbi as sbi;
    pub use xfina_ba_bob as bob;
    pub use xfina_ba_axis as axis;
}
