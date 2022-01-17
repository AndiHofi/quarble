use crate::conf::BreaksConfig;
use crate::data::day_normalizer::we::We;
use crate::data::work_day::WorkDay;
use crate::data::{
    Action, ActiveDay, Day, DayEnd, DayStart, JiraIssue, TimedAction, Work, WorkStart,
};
use crate::parsing::round_mode::RoundMode;
use crate::parsing::time::Time;
use crate::parsing::time_limit::{InvalidTime, TimeRange, TimeResult};
use crate::parsing::time_relative::TimeRelative;
use std::collections::BTreeSet;
use std::num::NonZeroU32;

#[cfg(test)]
mod test;
mod we;

pub struct NormalizedDay {
    pub date: Day,
    pub entries: Vec<Work>,
    pub orig_breaks: BreaksInfo,
    pub final_breaks: BreaksInfo,
}

impl From<&NormalizedDay> for WorkDay {
    fn from(n: &NormalizedDay) -> Self {
        WorkDay {
            date: n.date,
            entries: n.entries.clone(),
            synchronized: false,
        }
    }
}

#[derive(Debug)]
pub struct Normalizer {
    pub resolution: NonZeroU32,
    pub breaks_config: BreaksConfig,
    pub combine_bookings: bool,
    pub add_break: bool,
}

impl Normalizer {
    pub fn create_normalized(&self, current_day: &ActiveDay) -> Result<NormalizedDay, String> {
        let mut actions = current_day.actions().clone();
        let mut active_issue = current_day.active_issue().cloned();

        let mut splits = day_splits(&mut actions, &mut active_issue)?;

        let mut free_standing = handle_free_standing(actions);
        splits.append(&mut free_standing);

        let orig_breaks = calc_breaks(&splits);

        for range in &mut splits {
            round_bookings(range, self.resolution)?;
            if self.combine_bookings {
                combine_bookings(&mut range.work);
            }
        }

        let mut entries = flatten_ranges(splits);

        // when there are only automatic bookings around noon, may punch a hole
        // to add an automatic breaks
        if orig_breaks.break_time == TimeRelative::ZERO
            && self.breaks_config.min_breaks_minutes > 0
            && orig_breaks.work_time.offset_minutes()
                >= self.breaks_config.min_work_time_minutes as i32
        {
            try_insert_break(&self.breaks_config, &mut entries);
        }

        let final_breaks = calc_breaks(&entries);

        Ok(NormalizedDay {
            date: current_day.get_day(),
            entries: entries.into_iter().map(Work::from).collect(),
            orig_breaks,
            final_breaks,
        })
    }
}

fn flatten_ranges(ranges: Vec<FilledRange>) -> Vec<We> {
    ranges.into_iter().flat_map(|r| r.work).collect()
}

fn start_end_spans(actions: &BTreeSet<Action>) -> Result<Vec<TimeRange>, String> {
    let mut result = Vec::new();
    let mut current_start = None;

    for action in actions {
        match action {
            Action::DayStart(DayStart { ts, .. }) => {
                if current_start.is_none() {
                    current_start = Some(*ts);
                }
                // otherwise ignore the DayStart here - likely a forgotten DayEnd before
                // lunch break
            }
            Action::DayEnd(DayEnd { ts }) => {
                if let Some(start) = std::mem::take(&mut current_start) {
                    result.push(TimeRange::new(start, *ts));
                } else {
                    return Err(format!("Unmatched DayEnd: at {}", ts));
                }
            }
            _ => (),
        }
    }

    if let Some(start) = current_start {
        return Err(format!("Missing DayEnd: started at {}", start));
    }

    Ok(result)
}

#[derive(Debug, Eq, PartialEq)]
struct FilledRange {
    range: TimeRange,
    work: Vec<We>,
}

fn within_range(range: TimeRange, action: &Action) -> bool {
    let overlaps = |r: TimeRange, t| matches!(r.check_time_overlaps(t), TimeResult::Valid(_));

    let (start, end) = action.times();
    if let Action::WorkStart(WorkStart { ts, .. }) = action {
        return matches!(
            range.check_time_overlaps(*ts),
            TimeResult::Valid(_) | TimeResult::Invalid(InvalidTime::TooEarly { .. })
        );
    }
    overlaps(range, start) || end.map(|end| overlaps(range, end)).unwrap_or(false)
}

fn unbooked_times(mut range: TimeRange, work: &[We]) -> Vec<TimeRange> {
    let mut result = Vec::new();
    for work in work {
        if range.is_empty() {
            return result;
        }

        let (before, after) = range.split(TimeRange::new(work.start, work.end));
        if !before.is_empty() {
            result.push(before);
        }
        range = after;
    }

    if !range.is_empty() {
        result.push(range);
    }

    result
}

fn fill_gap(
    range: TimeRange,
    active_issue: &Option<JiraIssue>,
    work: &mut Vec<We>,
) -> Vec<TimeRange> {
    let unbooked = unbooked_times(range, work);
    if unbooked.is_empty() || active_issue.is_none() {
        return unbooked;
    };

    let active_issue = active_issue.clone().unwrap();

    for r in unbooked {
        let description = active_issue
            .default_action
            .as_deref()
            .unwrap_or("work")
            .to_string();
        work.push(We {
            start: r.min(),
            end: r.max(),
            id: active_issue.ident.clone(),
            description,
            implicit: true,
        });
    }

    work.sort();

    Vec::new()
}

fn fail_unbooked(unbooked: Vec<TimeRange>) -> Result<(), String> {
    if unbooked.is_empty() {
        Ok(())
    } else {
        let ranges: Vec<_> = unbooked
            .iter()
            .map(|r| format!("{}-{}", r.min(), r.max()))
            .collect();

        Err(format!("Unbooked times: {}", ranges.join(", ")))
    }
}

fn fill_range(
    range: TimeRange,
    active_issue: &mut Option<JiraIssue>,
    actions: &mut BTreeSet<Action>,
) -> Result<FilledRange, String> {
    let overlapping: Vec<_> = actions
        .iter()
        .filter(|action| within_range(range, action))
        .cloned()
        .collect();

    let mut remaining_range = range;
    let mut work = Vec::new();

    for action in overlapping {
        actions.remove(&action);
        match action {
            Action::WorkStart(s) => {
                let unbooked = fill_gap(range.with_max(s.ts), active_issue, &mut work);
                fail_unbooked(unbooked)?;
                remaining_range = range.with_min(s.ts);
                *active_issue = Some(JiraIssue {
                    ident: s.task.ident.clone(),
                    description: s.task.description.clone(),
                    default_action: Some(s.description.clone()),
                });
            }
            Action::WorkEnd(e) => {
                let unbooked = fill_gap(range.with_max(e.ts), active_issue, &mut work);
                fail_unbooked(unbooked)?;
                remaining_range = range.with_min(e.ts);
                if e.task.ident
                    == active_issue
                        .as_ref()
                        .map(|i| i.ident.as_str())
                        .unwrap_or_default()
                {
                    *active_issue = None;
                }
            }
            Action::Work(w) => {
                work.push(We {
                    id: w.task.ident,
                    description: w.description,
                    start: w.start,
                    end: w.end,
                    implicit: false,
                });
            }
            _ => (),
        }
    }

    let unbooked = fill_gap(remaining_range, active_issue, &mut work);
    fail_unbooked(unbooked)?;

    if !work.is_empty() {
        Ok(FilledRange {
            range: TimeRange::new(work.first().unwrap().start, work.last().unwrap().end),
            work,
        })
    } else {
        Ok(FilledRange { range, work })
    }
}

fn day_splits(
    entries: &mut BTreeSet<Action>,
    active_issue: &mut Option<JiraIssue>,
) -> Result<Vec<FilledRange>, String> {
    let time_spans = start_end_spans(entries);

    let mut parts = Vec::new();

    for range in time_spans? {
        let range = fill_range(range, active_issue, entries)?;
        parts.push(range);
    }

    Ok(parts)
}

fn combine_bookings(work: &mut Vec<We>) {
    let orig: Vec<We> = std::mem::take(work);

    for w in orig {
        if let Some(existing) = work.iter_mut().find(|r| r.same_issue(&w)) {
            existing.end = existing.end + w.duration();
        } else {
            work.push(w)
        }
    }

    work.sort();

    compact_booking(work)
}

fn compact_booking(work: &mut Vec<We>) {
    let mut iter = work.iter_mut();

    if let Some(initial) = iter.next() {
        let mut next_start = initial.end;

        for w in iter {
            let duration = w.duration();
            w.start = next_start;
            next_start = w.start + duration;
            w.end = next_start;
        }
    }
}

fn move_to_different_start(work: &mut Vec<We>, new_start: Time) -> Result<(), String> {
    if let Some(orig_start) = work.first().map(|w| w.start) {
        let offset = new_start - orig_start;
        for w in work {
            w.start = w
                .start
                .try_add_relative(offset)
                .ok_or_else(|| "overflow".to_string())?;
            w.end = w
                .end
                .try_add_relative(offset)
                .ok_or_else(|| "overflow".to_string())?;
        }
    }
    Ok(())
}

fn round_bookings(range: &mut FilledRange, resolution: NonZeroU32) -> Result<(), String> {
    let work = &mut range.work;
    if work.is_empty() {
        return Ok(());
    }

    let rounded_start = work
        .first()
        .unwrap()
        .start
        .round(RoundMode::Normal, resolution);
    let mut total_duration = TimeRelative::ZERO;
    let mut total_rounded_duration = TimeRelative::ZERO;
    for w in work.iter_mut() {
        let duration = w.duration();
        total_duration += duration;
        let mut rounded = duration.round(RoundMode::Normal, resolution);
        if rounded.offset_minutes() == 0 {
            rounded = TimeRelative::from_minutes_sat(resolution.get() as i32);
        }
        total_rounded_duration += rounded;

        w.end = w.start + rounded;
    }

    let mut round_error = (total_rounded_duration - total_duration).offset_minutes();

    for w in work.iter_mut().rev() {
        if (round_error.abs() as u32) < resolution.get() {
            break;
        }
        while (round_error.abs() as u32) >= resolution.get()
            && (w.duration().offset_minutes() as u32) > resolution.get()
        {
            let corr = -(resolution.get() as i32 * round_error.signum());
            round_error += corr;
            w.end = w.end + TimeRelative::from_minutes_sat(corr);
        }
    }

    if round_error.abs() > resolution.get() as i32 {
        return Err(format!(
            "Failed to round: remaining error is {}",
            round_error
        ));
    }

    compact_booking(work);
    move_to_different_start(work, rounded_start)?;

    range.range = range
        .range
        .with_min(rounded_start)
        .with_max(work.last().unwrap().end);

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreaksInfo {
    pub work_time: TimeRelative,
    pub break_time: TimeRelative,
    pub breaks: Vec<TimeRange>,
}

fn calc_breaks<T: we::HasRange>(ranges: &[T]) -> BreaksInfo {
    let mut iter = ranges.iter();
    let mut work_time = TimeRelative::ZERO;
    let mut break_time = TimeRelative::ZERO;
    let mut breaks = vec![];
    let mut prev_start = if let Some(first) = iter.next() {
        work_time += first.range().duration();
        first.range().max()
    } else {
        return BreaksInfo {
            work_time,
            break_time,
            breaks,
        };
    };

    for cur in iter {
        let cur_break = TimeRange::new(prev_start, cur.range().min());
        work_time += cur.range().duration();
        break_time += cur_break.duration();
        prev_start = cur.range().max();
        if !cur_break.is_empty() {
            breaks.push(cur_break);
        }
    }

    BreaksInfo {
        work_time,
        break_time,
        breaks,
    }
}

fn try_insert_break(config: &BreaksConfig, entries: &mut Vec<We>) {
    let break_bounds = TimeRange::new(config.default_break.0, config.default_break.1);
    let candidate = entries.iter().enumerate().find(|(_, e)| {
        e.implicit
            && e.duration().offset_minutes() >= config.min_breaks_minutes as i32
            && break_bounds.overlaps(e.range())
    });

    if let Some((index, _)) = candidate {
        let to_split = entries.remove(index);
        let orig_range = to_split.range();
        if orig_range.min() <= break_bounds.min() && orig_range.max() >= break_bounds.max() {
            let (p1, p2) = orig_range.split(break_bounds);
            if !p1.is_empty() {
                entries.insert(
                    index,
                    We {
                        start: p1.min(),
                        end: p1.max(),
                        ..to_split.clone()
                    },
                );
            }
            if !p2.is_empty() {
                entries.insert(
                    index + 1,
                    We {
                        start: p2.min(),
                        end: p2.max(),
                        ..to_split
                    },
                )
            }
        } else if break_bounds.min() < orig_range.min() {
            let range = orig_range.with_min(orig_range.min() + break_bounds.duration());
            if !range.is_empty() {
                entries.insert(
                    index,
                    We {
                        start: range.min(),
                        end: range.max(),
                        ..to_split
                    },
                )
            }
        } else {
            let range = orig_range.with_max(orig_range.max() + (-break_bounds.duration()));
            if !range.is_empty() {
                entries.insert(
                    index,
                    We {
                        start: range.min(),
                        end: range.max(),
                        ..to_split
                    },
                )
            }
        }
    }
}

fn handle_free_standing(entries: BTreeSet<Action>) -> Vec<FilledRange> {
    entries
        .into_iter()
        .filter(|e| matches!(e, Action::Work(_)))
        .map(|e| match e {
            Action::Work(w) => FilledRange {
                range: TimeRange::new(w.start, w.end),
                work: vec![We {
                    id: w.task.ident,
                    description: w.description,
                    start: w.start,
                    end: w.end,
                    implicit: false,
                }],
            },
            _ => panic!("did filter for Work"),
        })
        .collect()
}

fn merge_adjacent_ranges(ranges: &mut Vec<FilledRange>) {
    let orig = std::mem::take(ranges);

    let mut iter = orig.into_iter();
    if let Some(first) = iter.next() {
        let last_end = first.range.max();
        let mut current = first;

        for mut r in iter {
            if r.range.min() > last_end {
                std::mem::swap(&mut current, &mut r);
                ranges.push(r);
            } else {
                current.work.append(&mut r.work);
                current.range = current.range.with_max(r.range.max());
            }
        }

        ranges.push(current);
    }
}
