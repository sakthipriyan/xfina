use chrono::NaiveDate;
fn main() {
    let s = "05-Jun-2026";
    println!("{:?}", NaiveDate::parse_from_str(s, "%d-%b-%Y"));
    println!("{:?}", NaiveDate::parse_from_str(s, "%d-%m-%Y"));
}
