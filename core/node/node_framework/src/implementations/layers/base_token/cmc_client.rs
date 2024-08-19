use std::sync::Arc;

use tokio::sync::Mutex;
use zksync_config::configs::ExternalPriceApiClientConfig;
use zksync_external_price_api::cmc_api::CMCPriceAPIClient;

use crate::{
    implementations::resources::price_api_client::PriceAPIClientResource,
    wiring_layer::{WiringError, WiringLayer},
    IntoContext,
};

/// Wiring layer for `CmcApiClient`
///
/// Responsible for inserting a resource with a client to get base token prices from CoinMarketCap to be
/// used by the `BaseTokenRatioPersister`.
#[derive(Debug)]
pub struct CmcClientLayer {
    config: ExternalPriceApiClientConfig,
}

impl CmcClientLayer {
    /// Identifier of used client type.
    /// Can be used to choose the layer for the client based on configuration variables.
    pub const CLIENT_NAME: &'static str = "cmc";
}

#[derive(Debug, IntoContext)]
#[context(crate = crate)]
pub struct Output {
    pub price_api_client: PriceAPIClientResource,
}

impl CmcClientLayer {
    pub fn new(config: ExternalPriceApiClientConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl WiringLayer for CmcClientLayer {
    type Input = ();
    type Output = Output;

    fn layer_name(&self) -> &'static str {
        "cmc_api_client"
    }

    async fn wire(self, _input: Self::Input) -> Result<Self::Output, WiringError> {
        let cmc_client = Arc::new(Mutex::new(CMCPriceAPIClient::new(self.config)));

        Ok(Output {
            price_api_client: cmc_client.into(),
        })
    }
}
