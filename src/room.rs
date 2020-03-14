use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct Room<'a> {
    pub name: Cow<'a, str>,
    pub id: u64,
}

impl<'a> std::fmt::Display for Room<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

impl<'a> Room<'a> {
    pub fn remove_hashes(&self) -> &str {
        crate::util::remove_hashes(&self.name)
    }
}

impl<'a> crate::NameAndId for Room<'a> {
    fn name(&self) -> Cow<'_, str> {
        self.name.clone()
    }
    fn id(&self) -> u64 {
        self.id
    }
}
