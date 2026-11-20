mod helpers;
mod plugin;
mod types;

fn main() -> std::process::ExitCode {
    giong_node::sdk::run::<plugin::Flow>()
}
