#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    English,
    Chinese,
}

pub struct ModeDecision;

impl ModeDecision {
    pub fn for_process(
        exe_name: &str,
        is_english: bool,
        eng_list: &[&str],
        chn_list: &[&str],
    ) -> Option<InputMode> {
        if eng_list.iter().any(|name| name.eq_ignore_ascii_case(exe_name)) {
            if !is_english {
                return Some(InputMode::English);
            }
        }
        if chn_list.iter().any(|name| name.eq_ignore_ascii_case(exe_name)) {
            if is_english {
                return Some(InputMode::Chinese);
            }
        }
        None
    }
}
