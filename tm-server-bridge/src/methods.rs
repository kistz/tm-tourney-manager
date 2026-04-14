use tm_server_manager_api_rs::{EventContext, EventRawServerMethod, server_method_response};

use crate::{SPACETIME, TRACKMANIA};

pub fn method_call_received(_: &EventContext, method: &EventRawServerMethod) {
    tracing::error!("{method:#?}");
    let new = method.clone();
    tokio::spawn(async move {
        let response = TRACKMANIA
            .wait()
            .method(
                //SAFETY: Its the same type but rust cant know that.
                unsafe {
                    std::mem::transmute::<
                        tm_server_manager_api_rs::MethodCall,
                        tm_server_types::method::MethodCall,
                    >(new.call)
                },
            )
            .await;

        SPACETIME.read().reducers.server_method_response(
            new.id, //SAFETY: Its the same type but rust cant know that.
            unsafe {
                std::mem::transmute::<
                    tm_server_types::method::MethodResponse,
                    tm_server_manager_api_rs::MethodResponse,
                >(response)
            },
        )
    });
}
