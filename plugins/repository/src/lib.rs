mod plugin;

fn main() -> std::process::ExitCode {
    giong_node::sdk::run::<plugin::Repository>()
}
