use crate::proto::NotifyV1;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io, path::Path};

macro_rules! swriteln {
    ($dst:expr, $($arg:tt)*) => {{
        use std::fmt::Write;
        writeln!($dst, $($arg)*).expect("Writing to String cannot fail")
    }};
}

impl NotifyV1 {
    pub fn write_to_string(&self) -> String {
        let mut str = String::new();
        swriteln!(str, "{}", Self::VERSION);
        swriteln!(str, "{}: {}", Self::TITLE, self.title);
        swriteln!(str, "{}: {}", Self::PROGRAM, self.program);
        swriteln!(str, "{}: {}", Self::LEVEL, self.level.as_ref());
        swriteln!(str, "{}: {}", Self::BODY_LENGTH, self.body.len());
        swriteln!(str, "\n{}", self.body);
        str
    }
    pub fn write_to_dir(&self, dir: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
        // it is highly recommended to use uuid v4
        let file_name = uuid::Uuid::new_v4().to_string();
        let path = dir.as_ref().join(&file_name);
        let mut file = fs::File::create_new(&path)?;
        file.write_all(self.write_to_string().as_bytes())?;
        Ok(path)
    }
}

#[test]
fn correct() {
    use crate::proto::Level;
    let notify = NotifyV1 {
        level: Level::Info,
        title: "This is just info".to_string(),
        program: "test-test".to_string(),
        body: "something very very long...".to_string(),
    };
    let notify1 = NotifyV1::parse(&notify.write_to_string()).unwrap();
    assert_eq!(notify, notify1);
}

#[test]
#[ignore = "this test writes to disk"]
fn write() {
    use crate::proto::Level;
    let notify = NotifyV1 {
        level: Level::Info,
        title: "This is just info".to_string(),
        program: "test-test".to_string(),
        body: "something very very long...".to_string(),
    };
    notify.write_to_dir(".").unwrap();
}
