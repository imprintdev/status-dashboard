use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::{Digest, Sha256};
use crate::models::check_result::CheckStatus;
use super::{CheckError, CheckOutput, Checker, ConfigError};

type HmacSha256 = Hmac<Sha256>;

// AWS Cost Explorer is a global service, always signed against us-east-1
const CE_REGION: &str = "us-east-1";
const CE_SERVICE: &str = "ce";
const CE_HOST: &str = "ce.us-east-1.amazonaws.com";
const CE_TARGET: &str = "AWSInsightsIndexService.GetCostAndUsage";

pub struct AwsBillingChecker {
    access_key_id: String,
    secret_access_key: String,
    threshold_usd: f64,
    degraded_pct: f64,
}

impl AwsBillingChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let access_key_id = config["access_key_id"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("aws_billing requires 'access_key_id'".into()))?
            .to_string();
        let secret_access_key = config["secret_access_key"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("aws_billing requires 'secret_access_key'".into()))?
            .to_string();
        let threshold_usd = config["threshold_usd"].as_f64().unwrap_or(100.0);
        let degraded_pct = config["degraded_pct"].as_f64().unwrap_or(0.8);
        Ok(Self { access_key_id, secret_access_key, threshold_usd, degraded_pct })
    }
}

#[async_trait]
impl Checker for AwsBillingChecker {
    async fn check(&self) -> Result<CheckOutput, CheckError> {
        match self.fetch_mtd_cost().await {
            Err(e) => Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: None,
                detail: None,
                error_message: Some(e),
            }),
            Ok(cost_usd) => {
                let status = if cost_usd >= self.threshold_usd {
                    CheckStatus::Down
                } else if cost_usd >= self.threshold_usd * self.degraded_pct {
                    CheckStatus::Degraded
                } else {
                    CheckStatus::Up
                };
                Ok(CheckOutput {
                    status,
                    response_ms: None,
                    detail: Some(serde_json::json!({
                        "cost_usd": cost_usd,
                        "threshold_usd": self.threshold_usd
                    })),
                    error_message: None,
                })
            }
        }
    }
}

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn derive_signing_key(secret: &str, date: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{secret}").as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, CE_REGION.as_bytes());
    let k_service = hmac_sha256(&k_region, CE_SERVICE.as_bytes());
    hmac_sha256(&k_service, b"aws4_request")
}

impl AwsBillingChecker {
    async fn fetch_mtd_cost(&self) -> Result<f64, String> {
        let now = chrono::Utc::now();
        let datetime_str = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_str = now.format("%Y%m%d").to_string();
        let month_start = now.format("%Y-%m-01").to_string();
        let today = now.format("%Y-%m-%d").to_string();

        let payload = serde_json::json!({
            "TimePeriod": { "Start": month_start, "End": today },
            "Granularity": "MONTHLY",
            "Metrics": ["BlendedCost"]
        })
        .to_string();

        let payload_hash = sha256_hex(payload.as_bytes());

        // Canonical headers must be sorted alphabetically by header name
        let canonical_headers = format!(
            "content-type:application/x-amz-json-1.1\nhost:{CE_HOST}\nx-amz-date:{datetime_str}\nx-amz-target:{CE_TARGET}\n"
        );
        let signed_headers = "content-type;host;x-amz-date;x-amz-target";

        let canonical_request = format!(
            "POST\n/\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );

        let credential_scope =
            format!("{date_str}/{CE_REGION}/{CE_SERVICE}/aws4_request");
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{datetime_str}\n{credential_scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );

        let signing_key = derive_signing_key(&self.secret_access_key, &date_str);
        let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes()));

        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
            self.access_key_id
        );

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("https://{CE_HOST}/"))
            .header("Content-Type", "application/x-amz-json-1.1")
            .header("X-Amz-Target", CE_TARGET)
            .header("X-Amz-Date", &datetime_str)
            .header("Authorization", &authorization)
            .body(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("AWS API {status}: {body}"));
        }

        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        json["ResultsByTime"][0]["Total"]["BlendedCost"]["Amount"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| "Could not parse BlendedCost from AWS response".to_string())
    }
}
