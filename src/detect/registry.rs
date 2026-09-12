//! The one table that defines what xfina can read.
//!
//! Every surface derives from it: the `Format` enum and its metadata, the
//! parse and probe dispatch, the CLI's accepted `--as` values, the ids in the
//! JSON envelope, and the list the web UI builds its picker from. A new parser
//! is a row here, not an edit in six files.
//!
//! A format's `id` is its Cargo feature name, so the same string identifies it
//! in the manifest, on the command line and on the wire.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::decode::Decoded;
use crate::detect::{Claim, Container};
use crate::error::XfinaError;
use crate::models::account::Account;
use crate::models::request::ParseRequest;
use crate::models::validation::ValidationReport;

/// The kind of financial instrument a format describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    BankAccount,
    CreditCard,
    MutualFunds,
    IntlStocks,
}

impl Category {
    pub const fn as_str(self) -> &'static str {
        match self {
            Category::BankAccount => "bank_account",
            Category::CreditCard => "credit_card",
            Category::MutualFunds => "mutual_funds",
            Category::IntlStocks => "intl_stocks",
        }
    }
}

macro_rules! formats {
    ($(
        $variant:ident {
            id: $id:literal,
            category: $category:ident,
            institution: $institution:literal,
            extension: $extension:literal,
            locked: $locked:literal,
            download_url: $download_url:literal,
            download_path: $download_path:literal,
            containers: [$($container:ident),+ $(,)?],
            priority: $priority:literal,
            parse: $parse:path,
            probe: $probe:path,
        }
    )+) => {
        /// A parser identity: which institution's format, in which category.
        ///
        /// Variants are never feature-gated. They are pure data, and gating
        /// them would push `#[cfg]` into every match in the crate and into the
        /// bindings. Whether a format can actually run in this build is asked
        /// separately, with [`Format::is_enabled`].
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Format {
            $($variant),+
        }

        impl Format {
            /// Every format this crate knows about, enabled or not.
            pub const ALL: &'static [Format] = &[$(Format::$variant),+];

            /// The format's stable id, equal to its Cargo feature name.
            pub const fn id(self) -> &'static str {
                match self { $(Format::$variant => $id),+ }
            }

            pub const fn category(self) -> Category {
                match self { $(Format::$variant => Category::$category),+ }
            }

            /// The institution's full name, as it should appear everywhere.
            pub const fn institution(self) -> &'static str {
                match self { $(Format::$variant => $institution),+ }
            }

            /// The extension the institution actually puts on the file.
            ///
            /// Deliberately not derived from [`Format::containers`]: what a
            /// file is *called* and what it *is* differ. ICICI's card export
            /// is named `.xls` and is really a zip, and several banks can be
            /// read from either container. Callers filtering a folder want the
            /// name; detection wants the container.
            pub const fn extension(self) -> &'static str {
                match self { $(Format::$variant => $extension),+ }
            }

            /// Where the institution hands this statement out.
            ///
            /// The hardest part of using any of this is getting the file in
            /// the first place, and that knowledge lives nowhere else.
            pub const fn download_url(self) -> &'static str {
                match self { $(Format::$variant => $download_url),+ }
            }

            /// How to get to the download once you are signed in.
            pub const fn download_path(self) -> &'static str {
                match self { $(Format::$variant => $download_path),+ }
            }

            /// Whether this format is normally password protected.
            ///
            /// Nothing can be read out of such a file until it is opened, so a
            /// caller can say so before asking for anything.
            pub const fn is_password_protected(self) -> bool {
                match self { $(Format::$variant => $locked),+ }
            }

            /// Containers this format can appear in. A candidate whose
            /// containers exclude the sniffed one is never probed.
            pub const fn containers(self) -> &'static [Container] {
                match self {
                    $(Format::$variant => &[$(Container::$container),+]),+
                }
            }

            /// Tie-break order when two formats claim a file equally strongly.
            /// Values are unique, which makes the tie-break total.
            pub const fn priority(self) -> u16 {
                match self { $(Format::$variant => $priority),+ }
            }

            /// Whether this build was compiled with the format's parser.
            pub const fn is_enabled(self) -> bool {
                #[allow(unreachable_patterns)]
                match self {
                    $(
                        #[cfg(feature = $id)]
                        Format::$variant => true,
                    )+
                    _ => false,
                }
            }

            pub fn from_id(id: &str) -> Option<Format> {
                Format::ALL.iter().copied().find(|f| f.id() == id)
            }
        }

        /// Runs the format's parser and erases its account type.
        #[allow(unused_variables)]
        pub(crate) fn dispatch_parse(
            format: Format,
            decoded: &Decoded<'_>,
            input: &ParseRequest<'_>,
        ) -> Result<(Account, ValidationReport), XfinaError> {
            #[allow(unreachable_patterns)]
            match format {
                $(
                    #[cfg(feature = $id)]
                    Format::$variant => {
                        let result = $parse(decoded, input)?;
                        Ok((Account::from(result.data), result.validation))
                    }
                )+
                other => Err(XfinaError::FormatNotEnabled(other.id())),
            }
        }

        /// Asks the format whether the file is its own.
        #[allow(unused_variables)]
        pub(crate) fn dispatch_probe(format: Format, decoded: &Decoded<'_>) -> Claim {
            #[allow(unreachable_patterns)]
            match format {
                $(
                    #[cfg(feature = $id)]
                    Format::$variant => $probe(decoded),
                )+
                _ => Claim::NO,
            }
        }
    };
}

formats! {
    BankHdfc {
        id: "ba-hdfc",
        category: BankAccount,
        institution: "HDFC Bank",
        extension: "xls",
        locked: false,
        download_url: "https://www.hdfc.bank.in/",
        download_path: "Home \u{2192} Get Statement \u{2192} select account \u{2192} time period, Type: Excel \u{2192} Download",
        containers: [Ole2, Zip],
        priority: 10,
        parse: crate::bank_accounts::hdfc::parse_decoded,
        probe: crate::bank_accounts::hdfc::probe,
    }
    BankIcici {
        id: "ba-icici",
        category: BankAccount,
        institution: "ICICI Bank",
        extension: "xls",
        locked: false,
        download_url: "https://www.icici.bank.in/",
        download_path: "Overview \u{2192} View Statement \u{2192} Select Account \u{2192} Download/Email Statement \u{2192} Statement Period & Format: XLS \u{2192} Download Statement",
        containers: [Ole2, Zip],
        priority: 11,
        parse: crate::bank_accounts::icici::parse_decoded,
        probe: crate::bank_accounts::icici::probe,
    }
    BankSbi {
        id: "ba-sbi",
        category: BankAccount,
        institution: "State Bank of India",
        extension: "pdf",
        locked: true,
        download_url: "https://yonoretail.sbi.bank.in/registration/welcome",
        download_path: "Overview \u{2192} View Accounts \u{2192} Select Account \u{2192} Statements \u{2192} Select Duration, Format: PDF \u{2192} Download",
        containers: [Pdf],
        priority: 12,
        parse: crate::bank_accounts::sbi::parse_decoded,
        probe: crate::bank_accounts::sbi::probe,
    }
    BankBob {
        id: "ba-bob",
        category: BankAccount,
        institution: "Bank of Baroda",
        extension: "xls",
        locked: false,
        download_url: "https://bobibanking.bankofbaroda.bank.in/",
        download_path: "Account \u{2192} Operative Account \u{2192} Generate Account Statement \u{2192} Search Transaction to filter \u{2192} Download XLS",
        containers: [Ole2, Zip],
        priority: 13,
        parse: crate::bank_accounts::bob::parse_decoded,
        probe: crate::bank_accounts::bob::probe,
    }
    BankAxis {
        id: "ba-axis",
        category: BankAccount,
        institution: "Axis Bank",
        extension: "xls",
        locked: false,
        download_url: "https://omni.axisbank.co.in/axisretailbanking/",
        download_path: "Accounts \u{2192} Account Statement \u{2192} choose period \u{2192} Download as XLS",
        containers: [Ole2, Zip],
        priority: 14,
        parse: crate::bank_accounts::axis::parse_decoded,
        probe: crate::bank_accounts::axis::probe,
    }
    CardHdfc {
        id: "cc-hdfc",
        category: CreditCard,
        institution: "HDFC Bank",
        extension: "xls",
        locked: false,
        download_url: "https://www.hdfc.bank.in/",
        download_path: "Home \u{2192} Cards \u{2192} select card \u{2192} Get Statement \u{2192} Past Statement, time period, Format: Excel \u{2192} Download",
        containers: [Ole2, Zip],
        priority: 20,
        parse: crate::credit_cards::hdfc::parse_decoded,
        probe: crate::credit_cards::hdfc::probe,
    }
    CardIcici {
        id: "cc-icici",
        category: CreditCard,
        institution: "ICICI Bank",
        extension: "xls",
        locked: false,
        download_url: "https://www.icici.bank.in/",
        download_path: "Cards \u{2192} Credit Card \u{2192} Select Card \u{2192} Statements \u{2192} Select Past, XLS \u{2192} Download",
        containers: [Zip, Ole2],
        priority: 21,
        parse: crate::credit_cards::icici::parse_decoded,
        probe: crate::credit_cards::icici::probe,
    }
    CardAxis {
        id: "cc-axis",
        category: CreditCard,
        institution: "Axis Bank",
        extension: "xlsx",
        locked: false,
        download_url: "https://omni.axisbank.co.in/axisretailbanking/",
        download_path: "Dashboard \u{2192} Credit Card \u{2192} Get Statement \u{2192} time period, Type: XLS \u{2192} Download Statement",
        containers: [Zip, Ole2],
        priority: 22,
        parse: crate::credit_cards::axis::parse_decoded,
        probe: crate::credit_cards::axis::probe,
    }
    MutualFundsCams {
        id: "mf-cams",
        category: MutualFunds,
        institution: "CAMS",
        extension: "pdf",
        locked: true,
        download_url: "https://www.camsonline.com/Investors/Statements/Consolidated-Account-Statement",
        download_path: "Statement Type \u{2192} Time Period \u{2192} Folio Listing \u{2192} your email and a password you choose \u{2192} Submit; arrives by email as a PDF locked with that password",
        containers: [Pdf],
        priority: 30,
        parse: crate::mutual_funds::cams::parse_decoded,
        probe: crate::mutual_funds::cams::probe,
    }
    EquityIbkr {
        id: "is-ibkr",
        category: IntlStocks,
        institution: "Interactive Brokers",
        extension: "csv",
        locked: false,
        download_url: "https://www.interactivebrokers.co.in/en/home.php",
        download_path: "Performance & Reports \u{2192} Statements \u{2192} Activity Statement \u{2192} Select Period \u{2192} Download CSV",
        containers: [Text],
        priority: 40,
        parse: crate::intl_stocks::ibkr::parse_decoded,
        probe: crate::intl_stocks::ibkr::probe,
    }
}

/// The wire value of a `Format` is always its id, which is also its feature
/// name. Written by hand so the three can never drift apart.
impl Serialize for Format {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for Format {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let id = String::deserialize(d)?;
        Format::from_id(&id)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown format id '{}'", id)))
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

/// A format as described to callers: everything the web UI needs to build a
/// picker, and everything the CLI needs to list its `--as` values.
#[derive(Debug, Clone, Serialize)]
pub struct FormatInfo {
    pub id: &'static str,
    pub category: Category,
    pub institution: &'static str,
    /// The extension the institution puts on the file, for callers filtering
    /// a folder or building a file picker.
    pub extension: &'static str,
    /// Whether the file normally arrives password protected.
    pub password_protected: bool,
    /// Where the institution hands the statement out.
    pub download_url: &'static str,
    /// How to reach the download once signed in.
    pub download_path: &'static str,
    pub containers: &'static [Container],
    /// Whether this build can actually parse the format.
    pub enabled: bool,
}

impl From<Format> for FormatInfo {
    fn from(f: Format) -> Self {
        FormatInfo {
            id: f.id(),
            category: f.category(),
            institution: f.institution(),
            extension: f.extension(),
            password_protected: f.is_password_protected(),
            download_url: f.download_url(),
            download_path: f.download_path(),
            containers: f.containers(),
            enabled: f.is_enabled(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_priority_is_distinct() {
        // The tie-break between two equally strong claims is only total if no
        // two formats share a priority.
        let mut seen: Vec<u16> = Format::ALL.iter().map(|f| f.priority()).collect();
        seen.sort_unstable();
        let before = seen.len();
        seen.dedup();
        assert_eq!(before, seen.len(), "two formats share a priority");
    }

    #[test]
    fn every_id_round_trips() {
        for f in Format::ALL {
            assert_eq!(Format::from_id(f.id()), Some(*f));
        }
        assert_eq!(Format::from_id("not-a-format"), None);
    }

    #[test]
    fn an_id_is_a_cargo_feature_name() {
        // The id is used as #[cfg(feature = ...)] in the generated dispatch, so
        // these strings must stay in step with Cargo.toml.
        let manifest = include_str!("../../Cargo.toml");
        for f in Format::ALL {
            assert!(
                manifest.contains(&format!("\n{} = [", f.id())),
                "format id '{}' has no matching Cargo feature",
                f.id()
            );
        }
    }

    #[test]
    fn the_listing_reads_alphabetically_within_each_category() {
        // Surfaces render this list as-is; sorting it here means none of them
        // has to, and none of them can disagree about the order.
        let listed = crate::detect::formats();
        let keys: Vec<(Category, &str)> =
            listed.iter().map(|f| (f.category, f.institution)).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(
            keys, sorted,
            "formats() must be ordered by category, then institution"
        );
        assert_eq!(
            listed.len(),
            Format::ALL.len(),
            "every format must be listed"
        );
    }

    #[test]
    fn a_declared_extension_is_one_the_container_could_hold() {
        // The two describe different things -- what a file is called versus
        // what it is -- but they cannot contradict each other. ICICI's card
        // export being named .xls while really being a zip is exactly why this
        // is checked rather than derived.
        for f in Format::ALL {
            let plausible = match f.extension() {
                "pdf" => &[Container::Pdf][..],
                "csv" | "txt" => &[Container::Text][..],
                "xls" => &[Container::Ole2, Container::Zip][..],
                "xlsx" => &[Container::Zip][..],
                other => panic!("{} declares an unknown extension '{}'", f, other),
            };
            assert!(
                f.containers().iter().any(|c| plausible.contains(c)),
                "{} is named .{} but reads none of the containers that implies",
                f,
                f.extension()
            );
        }
    }

    #[test]
    fn every_format_says_where_to_get_the_file() {
        // Getting hold of the statement is the hardest part of using any of
        // this, and it is knowledge that lives nowhere else in the codebase.
        for f in Format::ALL {
            let url = f.download_url();
            assert!(
                url.starts_with("https://"),
                "{} must link somewhere over https, got '{}'",
                f,
                url
            );
            assert!(
                !f.download_path().is_empty(),
                "{} says where to go but not how to get there",
                f
            );
        }
    }

    #[test]
    fn only_pdf_formats_arrive_locked() {
        // Password protection is a property of the document format; a
        // spreadsheet or CSV marked locked would be a table entry error.
        for f in Format::ALL.iter().filter(|f| f.is_password_protected()) {
            assert_eq!(
                f.extension(),
                "pdf",
                "{} is marked password protected but is not a PDF",
                f
            );
        }
    }

    #[test]
    fn every_format_declares_a_container() {
        for f in Format::ALL {
            assert!(!f.containers().is_empty(), "{} declares no container", f);
        }
    }
}
