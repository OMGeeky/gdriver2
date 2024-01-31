pub mod drive_id {
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};

    lazy_static! {
        pub static ref ROOT_ID: DriveId = DriveId(String::from("root"));
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct DriveId(pub String);

    impl<T> From<T> for DriveId
    where
        T: Into<String>,
    {
        fn from(s: T) -> Self {
            DriveId(s.into())
        }
    }
    impl AsRef<str> for DriveId {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }
}
