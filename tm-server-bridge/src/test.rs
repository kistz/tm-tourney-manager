#[test]
fn test_test() {
    use testcontainers::{
        GenericImage,
        core::{IntoContainerPort, WaitFor},
        runners::SyncRunner,
    };

    /* let container = GenericImage::new("clockworklabs/spacetime", "latest")
    .with_exposed_port(3000.tcp())
    .with_wait_for(WaitFor::message_on_stdout(
        "Starting SpacetimeDB listening on 0.0.0.0:3000",
    ))
    .start(); */

    let container = GenericImage::new("evoesports/trackmania", "latest")
        .with_exposed_port(2350.tcp())
        .with_exposed_port(2350.udp())
        .with_exposed_port(5000.tcp())
        .with_wait_for(WaitFor::message_on_stdout(
            "Listening for xml-rpc commands on port 5000.",
        ))
        .start();

    println!("{container:?}")
}

/* struct SpacetimeDB {}

#[cfg(test)]
impl Image for SpacetimeDB {
    fn name(&self) -> &str {
        "clockworklabs/spacetime"
    }

    fn tag(&self) -> &str {
        "latest"
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        todo!()
    }

    fn exec_before_ready(
        &self,
        cs: ContainerState,
    ) -> Result<Vec<testcontainers::core::ExecCommand>> {
        Ok(cs.host())
    }
} */
