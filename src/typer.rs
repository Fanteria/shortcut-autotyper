use std::{error::Error, process::Command};

pub trait TypeText {
    fn type_text<T: AsRef<str>>(text: T, delay: usize) -> Result<(), Box<dyn Error>>;
}

pub struct XDoTool {}

impl TypeText for XDoTool {
    fn type_text<T: AsRef<str>>(text: T, delay: usize) -> Result<(), Box<dyn Error>> {
        let mut sys_comand = Command::new("xdotool");
        sys_comand.args(["type", "--delay", &delay.to_string(), text.as_ref()]);
        sys_comand.spawn()?;
        Ok(())
    }
}

pub struct Wtype {}

impl TypeText for Wtype {
    fn type_text<T: AsRef<str>>(text: T, delay: usize) -> Result<(), Box<dyn Error>> {
        let mut sys_comand = Command::new("wtype");
        sys_comand.args(["-d", &delay.to_string(), text.as_ref()]);
        sys_comand.spawn()?;
        Ok(())
    }
}
