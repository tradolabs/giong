use giong_core::application::application::Application;

pub(in crate::presentation) fn handler(application: &Application) -> impl AsyncFnMut() + '_ {
    async || {
        if let Err(error) = application.poll_repositories(None).await {
            eprintln!("Scheduled repository poll failed: {error:#}");
        }
    }
}
