use std::{sync::Arc, thread::JoinHandle};
use std::fmt::Debug;

use ratatui::widgets::WidgetRef;

use crate::ui::{Screen, UIError, UpdateAction};

pub type TaskAfter<T> = Box<dyn FnOnce(Option<T>) -> Option<Box<dyn Screen>>>;

pub struct TaskScreen<T: Debug> {
	screen: Box<dyn Screen>,
	task: Option<JoinHandle<T>>,
	after: Option<TaskAfter<T>>,
}
impl<T: Debug> Debug for TaskScreen<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("TaskScreen")
			.field("screen", &"Box<dyn Screen>")
			.field("task", &self.task.as_ref().map(|t| format!("JoinHandle<{:?}>", t.thread().name())))
			.field("after", &"Box<dyn FnOnce(Option<T>) -> Option<Box<dyn Screen>>")
			.finish()
	}
}
impl<T: Debug> TaskScreen<T> {
	pub fn new(screen: Box<dyn Screen>, task: JoinHandle<T>, after: impl FnOnce(Option<T>) -> Option<Box<dyn Screen>> + 'static) -> Self {
		Self {
			screen,
			task: Some(task),
			after: Some(Box::new(after)),
		}
	}
}
impl<T: Debug> WidgetRef for TaskScreen<T> {
	fn render_ref(&self,area: ratatui::prelude::Rect,buf: &mut ratatui::prelude::Buffer) {
		self.screen.render_ref(area, buf);
	}
}
impl<T: Debug> Screen for TaskScreen<T> {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, crate::ui::UIError> {
		if let crate::ui::event::Event::Tick = event {
			// check if the task is done
			let done = self.task.as_ref().map(|t| t.is_finished()).unwrap_or(false);
			if done {
				// if done, join the thread and get result
				let task = self.task.take().expect("Task should be Some if done");
				let res = task.join();
				let (res, err) = match res {
					Ok(res) => (Some(res), None),
					Err(e) => (None, Some(UIError::Runtime { src: e })),
				};
				// call the after function with the result
				let res = match self.after.take() {
					Some(after) => { after(res)},
					None => { None },
				};
				match (res, err) {
					(Some(screen), None) => { // good ending
						self.screen = screen;
						return Ok(UpdateAction::Redraw.one());
					},
					(None, Some(err)) => { // bad ending
						return Ok(vec![
							UpdateAction::Pop,
							UpdateAction::ErrorPopUp(Box::new(err)),
						])
					},
					(Some(screen), Some(err)) => { // weird ending
						self.screen = screen;
						return Ok(UpdateAction::ErrorPopUp(Box::new(err)).one());
					},
					(None, None) => {
						return Ok(UpdateAction::Pop.one());
					},
				}
			}
		}
		// pass the event to the screen
		self.screen.handle_event(event, state)
	}
}