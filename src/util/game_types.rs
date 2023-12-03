use num_enum::TryFromPrimitive;
use rocket::form::{self, FromFormField, ValueField};

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum League {
    Casual,
    Pro,
    Elite,
}

impl<'r> FromFormField<'r> for League {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        let num: u8 = field.value.parse()?;
        let league = match Self::try_from(num) {
            Ok(res) => res,
            _ => Err(form::Error::validation("failed to convert to League enum"))?,
        };
        Ok(league)
    }
}
