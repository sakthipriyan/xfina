use crate::models::schema::Schema;
use crate::models::{CreditCardAccount, DepositAccount, EquityAccount, MutualFundsAccount};

/// Implemented by the four top-level account models.
///
/// Every account renders through the same code path, so a new schema — or a
/// change to an existing one — is a single edit rather than four.
pub trait AccountModel: serde::Serialize {
    /// Fields that must stay `YYYY-MM-DD` rather than becoming a full
    /// timestamp when rendering [`Schema::Rebit`].
    fn date_only_paths(&self) -> &[String];

    fn to_json(&self, schema: Schema) -> serde_json::Value {
        crate::models::serializer::render(self, schema, self.date_only_paths())
    }
}

/// `impl AccountModel` for an account whose `xfina` extension carries
/// `date_only_paths`. All four are structurally identical.
macro_rules! impl_account_model {
    ($($ty:ty),+ $(,)?) => {$(
        impl AccountModel for $ty {
            fn date_only_paths(&self) -> &[String] {
                self.xfina
                    .as_ref()
                    .and_then(|x| x.date_only_paths.as_deref())
                    .unwrap_or(&[])
            }
        }
    )+};
}

impl_account_model!(
    DepositAccount,
    CreditCardAccount,
    MutualFundsAccount,
    EquityAccount,
);

/// A parsed account, whichever of the four models the file turned out to be.
///
/// This is what lets one entry point return every format: a caller matches on
/// the variant, or renders it through [`Account::to_json`] without caring.
///
/// The variants differ in size, which is fine here and not worth a box:
/// exactly one `Account` exists per parse, built once and moved into the
/// returned `Statement`. Boxing would trade that for a heap allocation on
/// every parse to save copying a few hundred bytes once.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Account {
    Deposit(DepositAccount),
    CreditCard(CreditCardAccount),
    MutualFunds(MutualFundsAccount),
    Equity(EquityAccount),
}

impl Account {
    pub fn to_json(&self, schema: Schema) -> serde_json::Value {
        match self {
            Account::Deposit(a) => a.to_json(schema),
            Account::CreditCard(a) => a.to_json(schema),
            Account::MutualFunds(a) => a.to_json(schema),
            Account::Equity(a) => a.to_json(schema),
        }
    }
}

macro_rules! impl_account_from {
    ($($variant:ident => $ty:ty),+ $(,)?) => {$(
        impl From<$ty> for Account {
            fn from(a: $ty) -> Self {
                Account::$variant(a)
            }
        }
    )+};
}

impl_account_from!(
    Deposit => DepositAccount,
    CreditCard => CreditCardAccount,
    MutualFunds => MutualFundsAccount,
    Equity => EquityAccount,
);
