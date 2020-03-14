use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct User<'a> {
    pub name: Cow<'a, str>,
    pub id: u64,
}

impl<'a> std::fmt::Display for User<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

impl<'a> crate::NameAndId for User<'a> {
    fn name(&self) -> Cow<'_, str> {
        self.name.clone()
    }
    fn id(&self) -> u64 {
        self.id
    }
}
