pub fn option_to_str<T: Default>(option: Option<T>) -> T {
    option.unwrap_or_default()
}

pub mod http {
    use std::io::Read;

    use anyhow::Result;

    pub fn download_file(url: &str) -> Result<u8> {
        let mut res = reqwest::blocking::get(url)?;
        let mut body = String::new();
        res.read_to_string(&mut body)?;
        println!("{}", body);
        Ok(2)
    }
}
