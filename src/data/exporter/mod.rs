use crate::data::NormalizedDay;
use std::fmt::Write;

pub struct TimeCockpitExporter;

impl TimeCockpitExporter {
    pub fn export(day: &NormalizedDay) -> String {
        let mut out = String::new();

        for w in &day.entries {
            writeln!(
                out,
                "{}|{}|{}|{}|{}",
                day.date, w.start, w.end, w.task.ident, w.description
            )
            .unwrap();
        }

        out
    }
}

#[cfg(test)]
mod test {
    use crate::data::exporter::TimeCockpitExporter;
    use crate::data::{BreaksInfo, Day, JiraIssue, NormalizedDay, Work};
    use crate::parsing::time::Time;
    use crate::parsing::time_limit::TimeRange;
    use crate::parsing::time_relative::TimeRelative;

    #[test]
    fn test_export() {
        let breaks = BreaksInfo {
            work_time: TimeRelative::from_minutes_sat(300),
            break_time: TimeRelative::from_minutes_sat(45),
            breaks: vec![TimeRange::new(Time::hm(12, 00), Time::hm(12, 45))],
        };
        let d = NormalizedDay {
            date: Day::ymd(2022, 1, 6),
            entries: vec![
                work(845, 900, "I-15", "some meeting+org"),
                work(900, 1200, "ISSUE-12345", "other"),
                work(1245, 1700, "A-51", "the afternoon"),
            ],
            orig_breaks: breaks.clone(),
            final_breaks: breaks,
        };

        let exported = TimeCockpitExporter::export(&d);
        assert_eq!(
            exported,
            r#"2022-01-06|08:45|09:00|I-15|some meeting+org
2022-01-06|09:00|12:00|ISSUE-12345|other
2022-01-06|12:45|17:00|A-51|the afternoon
"#
        )
    }

    fn work(start: u32, end: u32, task: &str, description: &str) -> Work {
        Work {
            start: Time::hm(start / 100, start % 100),
            end: Time::hm(end / 100, end % 100),
            task: JiraIssue::create(task).unwrap(),
            description: description.to_string(),
        }
    }
}
