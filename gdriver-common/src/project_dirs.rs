use lazy_static::lazy_static;
lazy_static!(
    pub static ref PROJECT_DIRS: directories::ProjectDirs = directories::ProjectDirs::from("com", "OMGeeky", "gdriver2").expect(
        "Getting the Project dir needs to work (on all platforms) otherwise nothing will work as expected. \
         This is where all files will be stored, so there is not much use for this app without it.",
    );
);
