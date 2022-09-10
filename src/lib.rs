pub mod config;
pub mod project;
pub mod runner;

mod util {
    use globset::{Glob, GlobSet, GlobSetBuilder};

    pub(crate) fn glob_set_from_iter<I, S>(patterns: I) -> Result<GlobSet, globset::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(Glob::new(pattern.as_ref())?);
        }
        builder.build()
    }
}
