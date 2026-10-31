use crate::error::Error;
use serde_json::Value;
use wasmtime::component::{Linker, ResourceTable};
use wasmtime::{Config, Engine};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

wasmtime::component::bindgen!({
    path: "wit", world: "provider",
    imports: { default: async },
    exports: { default: async },
});

pub(crate) struct Host {
    table: ResourceTable,
    wasi: WasiCtx,
    client: reqwest::Client,
    inputs: Option<Value>,
}

impl Host {
    pub(crate) fn new(client: reqwest::Client, inputs: Option<Value>) -> Self {
        Self { table: ResourceTable::new(), wasi: WasiCtxBuilder::new().build(), client, inputs }
    }
}

impl WasiView for Host {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView { ctx: &mut self.wasi, table: &mut self.table }
    }
}

impl giong::provider::host::Host for Host {
    async fn secret(&mut self, name: String) -> Option<String> {
        self.inputs.as_ref()?.get(&name)?.as_str().map(str::to_owned)
    }

    async fn http(
        &mut self,
        request: giong::provider::host::HttpRequest,
    ) -> Result<giong::provider::host::HttpResponse, String> {
        let result = async {
            let mut outgoing = self.client.request(
                request.method.parse::<reqwest::Method>().map_err(Error::unexpected)?,
                &request.url,
            );
            for (name, value) in request.headers {
                outgoing = outgoing.header(name, value);
            }
            let response = outgoing.body(request.body).send().await.map_err(Error::unexpected)?;
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(name, value)| {
                    Ok((name.to_string(), value.to_str().map_err(Error::unexpected)?.to_owned()))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let body = response.bytes().await.map_err(Error::unexpected)?.to_vec();
            Ok::<_, Error>(giong::provider::host::HttpResponse { status, headers, body })
        }
        .await;
        result.map_err(|error| error.to_string())
    }
}

pub(crate) fn setup() -> Result<(Engine, Linker<Host>), Error> {
    let mut config = Config::new();
    config.wasm_component_model(true).consume_fuel(true);
    let engine = Engine::new(&config).map_err(Error::unexpected)?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker).map_err(Error::unexpected)?;
    Provider::add_to_linker::<_, wasmtime::component::HasSelf<_>>(
        &mut linker,
        |host: &mut Host| host,
    )
    .map_err(Error::unexpected)?;
    Ok((engine, linker))
}
