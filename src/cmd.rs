use crate::data::ActiveDay;

pub fn print_active_day(day: Option<ActiveDay>) -> ! {
    if day.is_none() {
        println!("No active day");
        std::process::exit(1)
    }
    let day = day.unwrap();
    eprintln!("Day {}", day.get_day());
    eprintln!(
        "Initial issue: {}",
        day.active_issue()
            .map(|i| i.ident.as_str())
            .unwrap_or("<none>")
    );
    eprintln!("Entries:");
    for entry in day.actions() {
        eprintln!("  {}", entry);
    }

    std::process::exit(0);
}
