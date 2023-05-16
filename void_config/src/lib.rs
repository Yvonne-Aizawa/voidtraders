use ini::Ini;

fn get_config() -> Ini {
    Ini::load_from_file("./config/config.ini").unwrap()
}
pub fn get_config_key(section: &str, key: &str) -> String {
    let conf = get_config();
    let section = conf.section(Some(section)).unwrap();
    let value = section.get(key).unwrap();
    value.to_string()
}
pub fn set_config_key(section: &str, key: &str, value: &str) {
    let mut conf = get_config();
    let mut section = conf.with_section(Some(section));
    section.set(key, value);
    conf.write_to_file("./config/config.ini").unwrap();
}
