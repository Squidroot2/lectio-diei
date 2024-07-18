use std::{env, fs, path::Path};

use lectio_diei::{args::ConfigCommand, commands};

#[test]
fn full_thread() {
    // Set ENV Variables
    let temp_dir_root = env::current_dir().unwrap().join("temp_test");
    let data_dir = temp_dir_root.join("data");
    let config_dir = temp_dir_root.join("config");
    let state_dir = temp_dir_root.join("state");
    env::set_var("XDG_DATA_HOME", data_dir.as_os_str());
    env::set_var("XDG_STATE_HOME", state_dir.as_os_str());
    env::set_var("XDG_CONFIG_HOME", config_dir.as_os_str());

    // Create the directories so that when we remove them, we don't ignore the error
    fs::create_dir_all(&config_dir).unwrap();
    fs::remove_dir_all(&config_dir).unwrap();

    //TODO more of full thread
    test_config_init_no_force(&config_dir);

    // Cleanup
    fs::remove_dir_all(temp_dir_root).unwrap();
}

fn test_config_init_no_force(config_dir: &Path) {
    assert!(commands::handle_config_command(ConfigCommand::Init { force: false }).is_ok());
    let config = config_dir.join(env!("CARGO_PKG_NAME")).join("config.toml");
    assert!(config.is_file());
}
