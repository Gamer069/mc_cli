use minecraft_msa_auth::MinecraftAuthorizationFlow;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, DeviceAuthorizationUrl, RedirectUrl, Scope, StandardDeviceAuthorizationResponse, TokenResponse, TokenUrl};
use reqwest::Client;
use serde::Deserialize;

const MINECRAFT_PROFILE_URL: &'static str = "https://api.minecraftservices.com/minecraft/profile";
const DEVICE_CODE_URL: &'static str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MSA_AUTHORIZE_URL: &'static str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MSA_TOKEN_URL: &'static str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

#[derive(Deserialize, Clone, Debug)]
pub struct ProfileResponseJson {
    name: String,
    uuid: String,
}

pub fn authenticate(cli: String) -> (String, String, String, String) { // (xid, jwt, json.uuid, json.name)
    let client = BasicClient::new(ClientId::new(cli))
        .set_auth_uri(
            AuthUrl::new(MSA_AUTHORIZE_URL.to_string()).unwrap()
        )
        .set_token_uri(
            TokenUrl::new(MSA_TOKEN_URL.to_string()).unwrap()
        ).set_device_authorization_url(
            DeviceAuthorizationUrl::new(DEVICE_CODE_URL.to_string()).unwrap()
        ).set_redirect_uri(RedirectUrl::new("https://login.microsoftonline.com/common/oauth2/nativeclient".to_string()).unwrap());

    let client_blocking = reqwest::blocking::Client::builder().build().unwrap();

    let details: StandardDeviceAuthorizationResponse = client.exchange_device_code().add_scope(Scope::new("XboxLive.signin offline_access".to_string())).request(&client_blocking).unwrap();
    let uri = details.verification_uri().to_string();
    let code = details.user_code().secret().to_string();
    let _ = webbrowser::open(&uri);

    println!("If your web browser didn't get opened, manually go to {}. Enter the code {}", uri, code);

    let token = client.exchange_device_access_token(&details).request(&client_blocking, std::thread::sleep, None).unwrap();

    println!("microsoft token: {:?}", token);

    let mc_flow = MinecraftAuthorizationFlow::new(Client::new());
    let rt = tokio::runtime::Runtime::new().unwrap();
    println!("access token: {}", token.access_token().secret());
    let mc_tok = rt.block_on(mc_flow.exchange_microsoft_token(token.access_token().secret())).unwrap();
    println!("minecraft tok: {:?}", mc_tok);
    std::process::exit(1);
}
