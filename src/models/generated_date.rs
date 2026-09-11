use chrono::{DateTime, TimeZone, Utc};

use crate::models::account::Account;
use crate::models::request::ParseRequest;

/// Read and write access to an account's generated date, so the fallback
/// policy exists once rather than four times.
pub trait HasGeneratedDate {
    fn generated_date(&self) -> Option<DateTime<Utc>>;
    /// Records a date that did not come from the statement body.
    fn set_estimated_generated_date(&mut self, at: DateTime<Utc>);
}

/// Fills in the statement's generated date from the file's modification time,
/// as a last resort.
///
/// The chain is: what the institution printed, then what the filename says,
/// then this. The first two are applied by the parsers, which are the only
/// things that know where in a statement the date lives; the file's timestamp
/// is not statement knowledge, so it lands here instead of in ten parsers.
///
/// A download timestamp is strictly the weakest evidence -- it says when the
/// file reached the disk, not when the institution produced it -- so it is
/// only ever used when nothing else is available, and always marks the date
/// as an estimate.
pub(crate) fn apply_modified_timestamp_fallback(account: &mut Account, input: &ParseRequest<'_>) {
    if account.generated_date().is_some() {
        return;
    }
    let Some(seconds) = input.modified_timestamp else {
        return;
    };
    let Some(at) = Utc.timestamp_opt(seconds, 0).single() else {
        return;
    };
    account.set_estimated_generated_date(at);
}

/// Each account reaches its date through its own `xfina` extension, which is
/// optional and differently typed per model; the bodies are otherwise the same.
macro_rules! impl_has_generated_date {
    ($($ty:ty => $ext:ty),+ $(,)?) => {$(
        impl HasGeneratedDate for $ty {
            fn generated_date(&self) -> Option<DateTime<Utc>> {
                self.xfina.as_ref().and_then(|x| x.generated_date)
            }

            fn set_estimated_generated_date(&mut self, at: DateTime<Utc>) {
                let ext = self.xfina.get_or_insert_with(<$ext>::default);
                ext.generated_date = Some(at);
                ext.generated_date_derived = Some(true);
            }
        }
    )+};
}

impl_has_generated_date!(
    crate::models::DepositAccount => crate::models::XfinaDepositAccount,
    crate::models::CreditCardAccount => crate::models::XfinaCreditCardAccount,
    crate::models::MutualFundsAccount => crate::models::XfinaMutualFundsAccount,
    crate::models::EquityAccount => crate::models::XfinaEquityAccount,
);

impl HasGeneratedDate for Account {
    fn generated_date(&self) -> Option<DateTime<Utc>> {
        match self {
            Account::Deposit(a) => a.generated_date(),
            Account::CreditCard(a) => a.generated_date(),
            Account::MutualFunds(a) => a.generated_date(),
            Account::Equity(a) => a.generated_date(),
        }
    }

    fn set_estimated_generated_date(&mut self, at: DateTime<Utc>) {
        match self {
            Account::Deposit(a) => a.set_estimated_generated_date(at),
            Account::CreditCard(a) => a.set_estimated_generated_date(at),
            Account::MutualFunds(a) => a.set_estimated_generated_date(at),
            Account::Equity(a) => a.set_estimated_generated_date(at),
        }
    }
}
