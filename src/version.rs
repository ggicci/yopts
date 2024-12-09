use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Display, PartialEq, Eq, EnumString, EnumIter)]
pub enum Version {
    #[strum(serialize = "1.0.0")]
    V1_0_0,
}

impl Version {
    pub fn supported_versions() -> Vec<String> {
        Version::iter().map(|v| v.to_string()).collect()
    }
}

#[cfg(test)]
mod test {
    use super::Version;

    #[test]
    fn test_version() -> anyhow::Result<()> {
        assert_eq!("1.0.0", Version::V1_0_0.to_string());
        assert_eq!(Version::V1_0_0, Version::try_from("1.0.0")?);
        assert!(matches!(
            Version::try_from("1.0"),
            Err(strum::ParseError::VariantNotFound)
        ));
        Ok(())
    }
}
