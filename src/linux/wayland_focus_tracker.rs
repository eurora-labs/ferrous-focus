use crate::FerrousFocusResult;
use crate::FocusedWindow;

pub fn track_focus<F>(_on_focus: F) -> FerrousFocusResult<()>
where
    F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
{
    todo!()
}
