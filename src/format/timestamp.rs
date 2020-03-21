use std::convert::*;

pub trait Timestamp {
    /// NOTE: this assumes only positive timestamps
    fn as_secs(&self) -> u64;

    fn as_timestamp(&self) -> String {
        let time = self.as_secs();
        let hours = time / (60 * 60);
        let minutes = (time / 60) % 60;
        let seconds = time % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }

    fn as_readable_time(&self) -> String {
        const TABLE: [(&str, u64); 4] = [
            ("days", 86400),
            ("hours", 3600),
            ("minutes", 60),
            ("seconds", 1),
        ];

        fn pluralize(s: &str, n: u64) -> String {
            format!("{} {}", n, if n > 1 { s } else { &s[..s.len() - 1] })
        }

        let mut time = vec![];
        let mut secs = self.as_secs();
        for (name, d) in &TABLE {
            let div = secs / d;
            if div > 0 {
                time.push(pluralize(name, div));
                secs -= d * div;
            }
        }

        let len = time.len();
        if len > 1 {
            if len > 2 {
                for segment in time.iter_mut().take(len - 2) {
                    segment.push(',')
                }
            }
            time.insert(len - 1, "and".into())
        }
        time.join(" ")
    }
}

impl Timestamp for std::time::Duration {
    fn as_secs(&self) -> u64 {
        self.as_secs()
    }
}

impl Timestamp for time::Duration {
    fn as_secs(&self) -> u64 {
        self.whole_seconds() as u64
    }
}
