use crate::models::deposit::{Transaction, TransactionType};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Reorders same-day transactions so their printed balances form one
/// continuous chain.
///
/// Some banks — ICICI in particular — emit a day's rows in an order that does
/// not match the balance column. The amounts and the balances are each correct,
/// but the rows are shuffled within the day, so a strict running-balance check
/// fails on rows that are not themselves wrong.
///
/// Every row carries the balance it left behind, so it also implies the balance
/// it started from (`printed - delta`). That makes each row an edge between two
/// balances, and the day's true order is the walk using every edge exactly once
/// — an Eulerian path, found here in O(n) with Hierholzer's algorithm instead of
/// by trying permutations.
///
/// A day is rewritten only when a walk consumes every one of its rows, so the
/// outcome is either a day that chains exactly or the statement's own order left
/// untouched. Rows never move across dates.
///
/// Returns whether any day was reordered.
pub fn reorder_same_day_transactions(transactions: &mut [Transaction]) -> bool {
    let mut reordered = false;
    let mut start = 0;
    // The balance the next day has to start from, once a day has been settled.
    let mut running: Option<Decimal> = None;

    while start < transactions.len() {
        let date = transactions[start].value_date;
        let mut end = start + 1;
        while end < transactions.len() && transactions[end].value_date == date {
            end += 1;
        }

        let group = &mut transactions[start..end];

        // The first day has no established balance to chain from, so every row
        // is a candidate head; later days must start where the previous ended.
        let candidates: Vec<Decimal> = match running {
            Some(balance) => vec![balance],
            None => group.iter().map(opening_balance_of).collect(),
        };

        if !chains(group, candidates[0]) {
            for candidate in candidates {
                if let Some(order) = eulerian_order(group, candidate) {
                    apply(group, &order);
                    reordered = true;
                    break;
                }
            }
        }

        running = group.last().map(|t| t.current_balance);
        start = end;
    }

    reordered
}

/// What the transaction did to the balance: positive for a credit.
fn delta(txn: &Transaction) -> Decimal {
    match txn.r#type {
        TransactionType::Credit => txn.amount,
        TransactionType::Debit => -txn.amount,
    }
}

/// The balance this transaction must have started from.
fn opening_balance_of(txn: &Transaction) -> Decimal {
    txn.current_balance - delta(txn)
}

/// Whether the rows already chain, in the order given, from `opening`.
fn chains(group: &[Transaction], opening: Decimal) -> bool {
    let mut running = opening;
    for txn in group {
        running += delta(txn);
        if running != txn.current_balance {
            return false;
        }
    }
    true
}

/// Walks every row exactly once, starting from `opening`, following each row
/// from the balance it starts at to the balance it prints.
///
/// Hierholzer's algorithm: walk greedily until stuck, and splice in the detours
/// found along the way. Returns `None` when no such walk exists — a day whose
/// rows genuinely do not add up, rather than one that is merely shuffled.
fn eulerian_order(group: &[Transaction], opening: Decimal) -> Option<Vec<usize>> {
    let mut unused: HashMap<Decimal, Vec<usize>> = HashMap::new();
    for (idx, txn) in group.iter().enumerate() {
        unused.entry(opening_balance_of(txn)).or_default().push(idx);
    }
    // Popped from the back, so reversing keeps the statement's own order the
    // first thing tried whenever several rows start from the same balance.
    for indices in unused.values_mut() {
        indices.reverse();
    }

    // Each entry is a balance reached, and the row that reached it.
    let mut stack: Vec<(Decimal, Option<usize>)> = vec![(opening, None)];
    let mut order = Vec::with_capacity(group.len());

    while let Some(&(balance, arrived_by)) = stack.last() {
        match unused.get_mut(&balance).and_then(|indices| indices.pop()) {
            Some(idx) => stack.push((group[idx].current_balance, Some(idx))),
            None => {
                if let Some(idx) = arrived_by {
                    order.push(idx);
                }
                stack.pop();
            }
        }
    }

    order.reverse();
    (order.len() == group.len()).then_some(order)
}

fn apply(group: &mut [Transaction], order: &[usize]) {
    let reordered: Vec<Transaction> = order.iter().map(|&idx| group[idx].clone()).collect();
    group.clone_from_slice(&reordered);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn txn(day: u32, credit: bool, amount: i64, balance: i64) -> Transaction {
        Transaction {
            value_date: NaiveDate::from_ymd_opt(2026, 7, day),
            r#type: if credit {
                TransactionType::Credit
            } else {
                TransactionType::Debit
            },
            amount: Decimal::from(amount),
            current_balance: Decimal::from(balance),
            ..Default::default()
        }
    }

    /// Shape of the report: the amount and balance of each row, in order.
    fn shape(txns: &[Transaction]) -> Vec<(i64, i64)> {
        txns.iter()
            .map(|t| {
                (
                    delta(t).try_into().unwrap(),
                    t.current_balance.try_into().unwrap(),
                )
            })
            .collect()
    }

    #[test]
    fn reorders_a_shuffled_day() {
        // The rows from sakthipriyan/xfina#53. Only one walk uses them all:
        // 10,000 is the only balance no row prints, so it is the day's opening,
        // and from there each row's start balance picks out the next.
        let mut txns = vec![
            txn(1, true, 10_000, 20_000),
            txn(1, false, 25_000, 45_000),
            txn(1, true, 50_000, 70_000),
            txn(1, false, 5_000, 40_000),
        ];

        assert!(reorder_same_day_transactions(&mut txns));
        assert_eq!(
            shape(&txns),
            vec![
                (10_000, 20_000),
                (50_000, 70_000),
                (-25_000, 45_000),
                (-5_000, 40_000)
            ]
        );
    }

    #[test]
    fn leaves_a_day_that_already_chains() {
        let mut txns = vec![
            txn(1, true, 10_000, 20_000),
            txn(1, false, 5_000, 15_000),
            txn(2, false, 3_000, 12_000),
        ];
        let before = shape(&txns);

        assert!(!reorder_same_day_transactions(&mut txns));
        assert_eq!(shape(&txns), before);
    }

    #[test]
    fn leaves_a_day_that_does_not_add_up() {
        // The second row prints a balance no ordering can produce.
        let mut txns = vec![txn(1, true, 10_000, 20_000), txn(1, false, 5_000, 99_999)];
        let before = shape(&txns);

        assert!(!reorder_same_day_transactions(&mut txns));
        assert_eq!(shape(&txns), before);
    }

    #[test]
    fn never_moves_rows_across_dates() {
        // Read as one block these chain; per day, neither day does.
        let mut txns = vec![
            txn(1, true, 100, 1_100),
            txn(2, false, 400, 700),
            txn(1, true, 200, 900),
        ];
        let before = shape(&txns);

        assert!(!reorder_same_day_transactions(&mut txns));
        assert_eq!(shape(&txns), before);
    }

    #[test]
    fn follows_the_detour_rather_than_the_first_row_out() {
        // Two rows leave 1,000; taking the -100 first strands the other two.
        let mut txns = vec![
            txn(1, false, 100, 900),
            txn(1, true, 500, 1_500),
            txn(1, false, 500, 1_000),
        ];

        assert!(reorder_same_day_transactions(&mut txns));
        assert_eq!(shape(&txns), vec![(500, 1_500), (-500, 1_000), (-100, 900)]);
    }
}
