use roboplc::locking::RwLock;
use roboplc::{DataDeliveryPolicy, DeliveryPolicy};
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

const BUF_COUNT: u32 = 20;

#[derive(Clone, Debug)]
pub enum WorkerMessage {
    Frame(Vec<u8>),
    Terminate,
}

impl DataDeliveryPolicy for WorkerMessage {
    fn delivery_policy(&self) -> DeliveryPolicy { DeliveryPolicy::Latest }
}
#[derive(Clone)]
pub struct ServerState {
    pub ws_path: String,
}

#[derive(Debug, Default, Clone)]
pub struct CameraConfig {
    pub interval: (u32, u32),
    pub width: u16,
    pub height: u16,
    pub fourcc: [u8; 4],
    pub buf_size: u32,
    pub dev_idx: u8,
}
#[derive(Default, Debug, Clone)]
pub struct Variables {
    pub camera_config: CameraConfig,
    pub ngrok_auth_token: String,
    pub ngrok_domain: String,
    pub server_address: String,
    pub telegram_config: TelegramConfig,
    pub is_ngrok_started: Arc<RwLock<bool>>,
    pub ngrok_shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

#[derive(Debug, Default, Clone)]
pub struct TelegramConfig {
    pub token: String,
    pub admin_user_id: i64,
    pub allowed_user_ids: Vec<i64>,
}
pub fn init_config_by_env(args: Vec<(String, String)>) -> Variables {
    let mut hashmap = std::collections::HashMap::new();
    args.iter().for_each(|(k, v)| {
        hashmap.insert(k.clone(), v.clone());
    });

    let admin_user_id = hashmap
        .get("TELEGRAM_ADMIN_USER_ID")
        .expect("TELEGRAM_ADMIN_USER_ID is not set")
        .parse()
        .unwrap();
    let allowed_user_ids_config = hashmap
        .get("TELEGRAM_ALLOWED_USER_IDS")
        .expect("TELEGRAM_ALLOWED_USER_IDS is not set");
    let mut allowed_user_ids: Vec<i64> = allowed_user_ids_config.split(',').map(|x| x.parse().unwrap()).collect();
    allowed_user_ids.extend(vec![admin_user_id]);

    let variables = Variables {
        camera_config: CameraConfig {
            interval: (1, 30),
            width: hashmap.get("CAMERA_WIDTH").unwrap_or(&"640".to_string()).parse().unwrap(),
            height: hashmap.get("CAMERA_HEIGHT").unwrap_or(&"480".to_string()).parse().unwrap(),
            fourcc: *b"MJPG",
            buf_size: BUF_COUNT,
            dev_idx: hashmap.get("CAMERA_DEV_IDX").unwrap_or(&"0".to_string()).parse().unwrap(),
        },
        ngrok_auth_token: hashmap
            .get("NGROK_AUTH_TOKEN")
            .expect("NGROK_AUTH_TOKEN is not set")
            .to_string(),
        ngrok_domain: hashmap.get("NGROK_DOMAIN").expect("NGROK_DOMAIN is not set").to_string(),
        server_address: hashmap
            .get("SERVER_ADDRESS")
            .unwrap_or(&"localhost:8080".to_string())
            .to_string(),
        telegram_config: TelegramConfig {
            token: hashmap.get("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN is not set").to_string(),
            admin_user_id: admin_user_id,
            allowed_user_ids: vec![admin_user_id, 427348891],
        },
        is_ngrok_started: Arc::new(RwLock::new(false)),
        ngrok_shutdown_tx: Arc::new(Mutex::new(None)),
    };

    variables
}
