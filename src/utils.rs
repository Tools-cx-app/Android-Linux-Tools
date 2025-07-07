pub fn option_to_str<T: Default>(option: Option<T>) -> T {
    option.unwrap_or_default()
}
