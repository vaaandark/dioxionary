#[derive(Clone, Copy, PartialEq)]
pub enum AnswerStatus {
    Show,
    Hide,
}

#[derive(Clone, Copy)]
pub enum ExitCode {
    ManualExit,
    OutOfCard,
}

impl AnswerStatus {
    pub fn flip(self) -> Self {
        match self {
            AnswerStatus::Show => AnswerStatus::Hide,
            AnswerStatus::Hide => AnswerStatus::Show,
        }
    }
}

pub fn get_width_and_height(s: &str) -> (usize, usize) {
    let v: Vec<_> = s.split("\n").collect();
    let height = v.len();
    let width = v.into_iter().fold(10usize, |res, x| Ord::max(res, x.len()));
    (height, width)
}
