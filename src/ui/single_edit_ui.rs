use crate::data::Action;
use crate::ui::stay_active::StayActive;
use crate::ui::Message;

pub trait SingleEditUi<T>
where
    T: Into<Action>,
{
    fn as_text(&self, orig: &T) -> String;

    fn set_orig(&mut self, orig: T);

    fn try_build(&self) -> Option<T>;

    fn entry_to_edit(&mut self, orig: T) {
        self.set_orig(orig);
    }

    fn on_submit_message(
        result: Option<T>,
        original: &mut Option<T>,
        stay_active: StayActive,
    ) -> Option<Message>
    where
        T: Into<Action>,
    {
        let Some(action) = result else {
            return None;
        };

        if let Some(orig) = std::mem::take(original) {
            Some(Message::ModifyAction {
                stay_active,
                orig: Box::new(orig.into()),
                update: Box::new(action.into()),
            })
        } else {
            Some(Message::StoreAction(stay_active, action.into()))
        }
    }

    fn update_input(&mut self, input: String) -> Option<Message>;

    #[cfg(test)]
    fn parse_input(&mut self, input: &str) {
        self.update_input(input.to_string());
    }

    #[cfg(test)]
    fn convert_input(&mut self, input: &str) -> Option<T> {
        self.update_input(input.to_string());
        self.try_build()
    }
}
