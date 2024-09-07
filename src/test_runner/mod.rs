pub mod local;
pub mod remote;

pub trait TestRunner {
    fn run_test(
        &self,
        distro: &str,
        package: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}