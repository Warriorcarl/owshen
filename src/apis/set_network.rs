use crate::{Config, Context, Network, SetNetworkRequest, SetNetworkResponse};

use axum::{extract::Query, Json};
use ethers::providers::{Http, Provider};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

lazy_static! {
    static ref NETWORK_CONFIG_MAP: HashMap<String, (String, String)> = [
        (
            "1337".into(),
            ("http://127.0.0.1:8545".into(), "localhost.json".into())
        ),
        (
            "0x5".into(),
            (
                "https://ethereum-goerli.publicnode.com".into(),
                "goerli.json".into()
            )
        ),
        (
            "11155111".into(),
            (
                "https://sepolia.infura.io/v3/".into(),
                "Sepolia.json".into()
            )
        )
    ]
    .into_iter()
    .collect();
}

pub async fn set_network(
    Query(req): Query<SetNetworkRequest>,
    ctx: Arc<Mutex<Context>>,
    test: bool,
) -> Result<Json<SetNetworkResponse>, eyre::Report> {
    let chain_id = req.chain_id;
    let (provider_url, config_path) = NETWORK_CONFIG_MAP.get(&chain_id).unwrap().clone();
    let app_dir_path = std::env::var("APPDIR").unwrap_or_else(|_| "".to_string());

    let config_path = if test {
        std::fs::read_to_string(&config_path)
    } else {
        std::fs::read_to_string(format!(
            "{}/usr/share/networks/{}",
            app_dir_path, config_path
        ))
    };

    let provider: Arc<Provider<Http>> = Arc::new(Provider::<Http>::try_from(provider_url.clone())?);
    let mut ctx = ctx.lock().await;
    let config = config_path
        .map(|s| {
            let c: Config = serde_json::from_str(&s).expect("Invalid config file!");
            c
        })
        .ok()
        .unwrap();

    ctx.network = Some(Network { provider, config });
    // reset the current coins with last provider
    ctx.coins.clear();
    Ok(Json(SetNetworkResponse {
        success: crate::Empty { ok: true },
    }))
}
