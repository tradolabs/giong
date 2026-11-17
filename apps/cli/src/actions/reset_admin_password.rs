use giong_core::application::application::Application;

pub(crate) async fn execute(application: &Application) -> anyhow::Result<()> {
    let password = application.reset_admin_password().await?;
    println!("Admin password: {password}");
    Ok(())
}
