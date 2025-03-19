#[derive(Default)]
pub struct FrameworkOptions<T, E> {
    pub commands: Vec<super::command::Command<T, E>>,
}

#[derive(Default)]
pub struct Framework<T, E> {
    options: FrameworkOptions<T, E>,
}

impl serenity::Framework for Framework<T, E> {
    async fn init(&mut self, client: &serenity::Client) {}

    async fn dispatch(&self, ctx: &serenity::all::Context, event: &serenity::all::FullEvent) {
        // we should probably inject our own framework context that contains stuff like the the framework options
        // we will handle stuff like dispatching commands after, after we have you know... written that stuff.
        dbg!(event);
    }
}
