use crate::config::Config;
use crate::config::InputData;
use crate::core::segments::SegmentData;

pub fn collect(_config: &Config, _input: &InputData) -> Option<SegmentData> {
    // 不再显示任何内容
    None
}
