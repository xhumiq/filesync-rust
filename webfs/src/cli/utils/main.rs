use chrono::{NaiveDate, Weekday, Datelike};

fn list_weeks_in_range(start_date: NaiveDate, end_date: NaiveDate) -> Vec<(NaiveDate, NaiveDate)> {
    let mut weeks = Vec::new();
    let mut current = start_date;

    while current <= end_date {
        // Find the Saturday of the current week (or end_date if earlier)
        let days_to_saturday = (6 - current.weekday().num_days_from_sunday()) as i64;
        let week_end = current + chrono::Duration::days(days_to_saturday);
        let actual_end = if week_end > end_date { end_date } else { week_end };

        weeks.push((current, actual_end));

        // If we've reached the end date, we're done
        if actual_end >= end_date {
            break;
        }

        // Move to the next Sunday
        current = actual_end + chrono::Duration::days(1);
        // If current is not Sunday, find the next Sunday
        if current.weekday() != Weekday::Sun {
            let days_to_sunday = (7 - current.weekday().num_days_from_sunday()) % 7;
            current = current + chrono::Duration::days(days_to_sunday as i64);
        }
    }

    weeks
}

fn main() {
    // Hard-coded date ranges for testing
    let test_ranges = vec![
        (
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),  // Monday, Jan 1, 2024
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), // Monday, Jan 15, 2024
        ),
        (
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),  // Wednesday, Jan 3, 2024
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(), // Saturday, Jan 20, 2024
        ),
        (
            NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(),  // Sunday, Jan 7, 2024
            NaiveDate::from_ymd_opt(2024, 1, 21).unwrap(), // Sunday, Jan 21, 2024
        ),
    ];

    for (i, (start_date, end_date)) in test_ranges.iter().enumerate() {
        println!("Test Range {}: {} to {}", i + 1, start_date, end_date);
        println!("Start day: {}", start_date.format("%A"));
        println!("End day: {}", end_date.format("%A"));
        println!();

        let weeks = list_weeks_in_range(*start_date, *end_date);

        for (week_num, (week_start, week_end)) in weeks.iter().enumerate() {
            println!("  Week {}: {} ({}) to {} ({})",
                week_num + 1,
                week_start,
                week_start.format("%A"),
                week_end,
                week_end.format("%A")
            );
        }
        println!("---");
        println!();
    }
}