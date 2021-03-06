use std::borrow::Cow;

pub trait NameAndId {
    fn name(&self) -> Cow<'_, str>;

    fn id(&self) -> u64;

    fn check_list<'a, I>(&self, mut collection: I) -> anyhow::Result<()>
    where
        I: Iterator<Item = &'a String> + 'a,
    {
        // TODO why is this allocating it?
        let id = self.id().to_string();
        let name = self.name();
        if collection.any(|s| s == &id || s == &name) {
            return Ok(());
        }
        log::debug!("{} ({}) isn't on the list", name, id);
        crate::dont_care()
    }
}
