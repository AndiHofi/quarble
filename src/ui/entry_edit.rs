use crate::ui::Message;

pub trait EntryEdit {
    fn show<'a, 'b: 'a>(&'b mut self) -> crate::ui::QElement<'a>;
    fn update_id(&mut self, id: usize);
    fn update(&mut self, msg: Message) -> Option<Message>;
    fn has_focus(&self) -> bool;
}
