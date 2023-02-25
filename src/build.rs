use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // Platforms
        raspi: { all(target_arch="arm", target_os="linux", target_env="gnu") },
    }
}