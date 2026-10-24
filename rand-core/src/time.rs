//! フレームと時間 (本家の watch)。ゲームは 1 秒 30 フレームで、乱数は 1 フレームに 1 つ進む

use std::fmt;
use std::time::Duration;

/// 1 秒のフレーム数
pub const FPS: u64 = 30;

/// フレーム数で表した時間
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameTime(pub u64);

impl GameTime {
    /// 全体のフレーム数
    pub const fn total_frames(self) -> u64 {
        self.0
    }

    pub const fn days(self) -> u64 {
        self.0 / (FPS * 60 * 60 * 24)
    }

    /// 日に満たない時間 (0〜23)
    pub const fn hours(self) -> u64 {
        self.0 / (FPS * 60 * 60) % 24
    }

    /// 時間に満たない分 (0〜59)
    pub const fn minutes(self) -> u64 {
        self.0 / (FPS * 60) % 60
    }

    /// 分に満たない秒 (0〜59)
    pub const fn seconds(self) -> u64 {
        self.0 / FPS % 60
    }

    /// 秒に満たないフレーム (0〜29)
    pub const fn frames(self) -> u64 {
        self.0 % FPS
    }

    /// 実際の時間 (秒に満たないフレームは小数の秒になる)
    pub fn to_duration(self) -> Duration {
        Duration::from_secs(self.0 / FPS) + Duration::from_nanos(self.0 % FPS * 1_000_000_000 / FPS)
    }
}

/// 本家の watch と同じ書き方 (0d 0h 2m 37s 13f)
impl fmt::Display for GameTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}d {}h {}m {}s {}f",
            self.days(),
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.frames()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 本家の watch(4723) は 0d 0h 2m 37s 13f。日をまたぐ大きな値も
    #[test]
    fn matches_watch() {
        assert_eq!(GameTime(4723).to_string(), "0d 0h 2m 37s 13f");
        let t = GameTime(2_592_000 * 3 + 108_000 * 5 + 1800 * 7 + 30 * 9 + 11);
        assert_eq!(t.to_string(), "3d 5h 7m 9s 11f");
        assert_eq!(
            GameTime(4723).to_duration(),
            Duration::from_nanos(157_433_333_333)
        );
    }
}
