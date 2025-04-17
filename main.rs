use serde::Deserialize;
use std::env;
use reqwest::Client;
use serde_json::json;
use tokio;

#[derive(Deserialize)]
struct FrontEntry {
    member: Member,
}

#[derive(Deserialize)]
struct Member {
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting VRChat SPS status updater...");

    // Load configuration
    println!("Loading environment variables...");
    let sps_token = env::var("SPS_API_TOKEN").expect("SPS_API_TOKEN not set");
    println!("SPS_API_TOKEN loaded.");
    let vr_username = env::var("VRCHAT_USERNAME").expect("VRCHAT_USERNAME not set");
    println!("VRCHAT_USERNAME loaded: {}", vr_username);
    let vr_password = env::var("VRCHAT_PASSWORD").expect("VRCHAT_PASSWORD not set");
    println!("VRCHAT_PASSWORD loaded.");
    let sps_base = env::var("SPS_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.apparyllis.com/v1".to_string());
    println!("Using SPS base URL: {}", sps_base);
    let vr_base = env::var("VRCHAT_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.vrchat.cloud/api/1".to_string());
    println!("Using VRChat base URL: {}", vr_base);

    // Build HTTP client
    println!("Building HTTP client...");
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to build HTTP client");
    println!("Client built successfully.");

    // 1. Fetch current fronts from Simply Plural
    let fronts_url = format!("{}/fronters", sps_base);
    println!("Fetching fronts from SPS: {}", fronts_url);
    let fronts_response = client
        .get(&fronts_url)
        .header("Authorization", &sps_token)
        .send()
        .await?;
    println!("Received response (status: {})", fronts_response.status());
    let fronts: Vec<FrontEntry> = fronts_response
        .error_for_status()?
        .json()
        .await?;
    println!("Parsed {} front entries.", fronts.len());
    let front_names: Vec<String> = fronts.into_iter().map(|e| e.member.name).collect();
    println!("Front names: {:?}", front_names);

    // Format status as "F: <fronter1>, <fronter2>, ..."
    let status_desc = if front_names.is_empty() {
        println!("No fronts found.");
        "F: none?".to_string()
    } else {
        let desc = format!("F: {}", front_names.join(", "));
        println!("Formatted statusDescription: {}", desc);
        desc
    };

    // 2. Authenticate with VRChat
    let auth_url = format!("{}/auth/user", vr_base);
    println!("Authenticating with VRChat: {}", auth_url);
    let auth_response = client
        .get(&auth_url)
        .basic_auth(&vr_username, Some(&vr_password))
        .send()
        .await?;
    println!("Authenticated (status: {})", auth_response.status());
    let auth_json: serde_json::Value = auth_response
        .error_for_status()?
        .json()
        .await?;
    let user_id = auth_json["id"].as_str().expect("Missing user ID");
    println!("Retrieved user ID: {}", user_id);

    // 3. Update VRChat status
    let update_url = format!("{}/users/{}", vr_base, user_id);
    println!("Updating VRChat status at: {}", update_url);
    let update_payload = json!({
        "status": "active",
        "statusDescription": status_desc,
    });
    println!("Payload: {}", update_payload);
    let update_response = client
        .put(&update_url)
        .basic_auth(&vr_username, Some(&vr_password))
        .json(&update_payload)
        .send()
        .await?;
    println!("Update response status: {}", update_response.status());

    println!("VRChat status updated successfully.");
    Ok(())
}